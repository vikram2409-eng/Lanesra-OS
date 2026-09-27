//! Voice-First Mode, PR 1: the VoiceActionPlanner (spec §9/§10) - turns one
//! recognized transcript into a typed, versioned `VoiceActionPlanBody`.
//! Speech is never executed directly (spec §10): this module only ever
//! *proposes* a plan; `voice_execution_service` is the sole thing allowed
//! to actually write data, and only after risk/confirmation clears it.
//!
//! Metadata-driven, not phrase-driven (spec §9): intent classification is a
//! small, fixed set of trigger verbs (this is a structural parser, not a
//! canned-sentence table), but *which objects, statuses and fields exist*
//! is always read live from `custom_object_service`/status constants/
//! `custom_field_service::list_definitions` - a brand-new custom object or
//! status is voice-addressable the moment it exists, with zero new code
//! (VOICE-AC-06). Scoped honestly for PR 1 (documented, not hidden): the
//! UPDATE intent supports each core object's own status/stage field - the
//! one field every spec example and every acceptance criterion (VOICE-AC-07)
//! actually exercises - plus any custom field by label; broader arbitrary
//! built-in field coverage is real follow-on work for PR 2/3, not silently
//! implied here.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::company::COMPANY_STATUSES;
use crate::models::contact::CONTACT_STATUSES;
use crate::models::contract::CONTRACT_STATUSES;
use crate::models::custom_object::CUSTOM_RECORD_STATUSES;
use crate::models::opportunity::OPPORTUNITY_STAGES;
use crate::models::order::ORDER_STATUSES;
use crate::models::quote::QUOTE_STATUSES;
use crate::models::task::TASK_STATUSES;
use crate::models::voice::{PendingCreate, ResolutionCandidate, VoiceActionPlanBody, VoicePlanStep};
use crate::services::{ai_agent_service, ai_orchestration_service, custom_field_service, custom_object_service, voice_entity_resolver};
use voice_entity_resolver::ResolutionOutcome;

// A tiny helper module so a catalog entry can hold either a `&'static str`
// (core objects) or an owned `String` (custom object keys) behind one
// field type without duplicating the whole struct - built fresh per call
// from `custom_object_service::list` plus the fixed core objects, never a
// hardcoded list of "every object this build knew about at compile time".
mod str_or_owned {
    #[derive(Clone)]
    pub enum Str {
        Static(&'static str),
        Owned(String),
    }
    impl Str {
        pub fn as_str(&self) -> &str {
            match self {
                Str::Static(s) => s,
                Str::Owned(s) => s.as_str(),
            }
        }
    }
}
use str_or_owned::Str;

struct CatalogEntry {
    object_key: Str,
    nouns: Vec<String>,
    status_values: Vec<String>,
    /// Voice-First Mode, PR 3 (industry voice vocabulary packs): a Custom
    /// Object's own select-type custom fields, each carried as (field_key,
    /// real configured options) - e.g. Property Management's Unit carries
    /// `("unit_stage", ["Vacant", "Reserved", "Occupied", ...])`. Always
    /// empty for a core built-in object (its status/stage field is one of
    /// the fixed `*_STATUSES`/`*_STAGES` consts above, not a custom field).
    /// This is what lets "mark Unit 200 as Occupied" resolve against an
    /// installed Industry App's *own* vocabulary the same honest,
    /// metadata-driven way built-in status values already do - never a
    /// second, hardcoded per-package phrase table.
    extra_status_fields: Vec<(String, Vec<String>)>,
}

fn core_catalog() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry { object_key: Str::Static("Company"), nouns: vec!["company".into(), "companies".into(), "customer".into(), "account".into()], status_values: COMPANY_STATUSES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
        CatalogEntry { object_key: Str::Static("Contact"), nouns: vec!["contact".into(), "contacts".into(), "person".into()], status_values: CONTACT_STATUSES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
        CatalogEntry { object_key: Str::Static("Opportunity"), nouns: vec!["opportunity".into(), "opportunities".into(), "deal".into()], status_values: OPPORTUNITY_STAGES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
        CatalogEntry { object_key: Str::Static("Quote"), nouns: vec!["quote".into(), "quotes".into()], status_values: QUOTE_STATUSES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
        CatalogEntry { object_key: Str::Static("Order"), nouns: vec!["order".into(), "orders".into()], status_values: ORDER_STATUSES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
        CatalogEntry { object_key: Str::Static("Contract"), nouns: vec!["contract".into(), "contracts".into(), "agreement".into()], status_values: CONTRACT_STATUSES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
        CatalogEntry { object_key: Str::Static("Task"), nouns: vec!["task".into(), "tasks".into(), "to-do".into(), "todo".into()], status_values: TASK_STATUSES.iter().map(|s| s.to_string()).collect(), extra_status_fields: vec![] },
    ]
}

fn full_catalog(conn: &Connection, workspace_id: &str) -> AppResult<Vec<CatalogEntry>> {
    let mut catalog = core_catalog();
    for def in custom_object_service::list(conn, workspace_id, true)? {
        // Every select-type custom field on this object is a candidate
        // "status-like" vocabulary - an Industry App (or a hand-built
        // Custom Object) typically names exactly one such field per
        // object ("Occupancy Status", "Claim Status", ...), but there's no
        // separate "this is THE status field" marker anywhere in the data
        // model (custom objects don't participate in Status Transition
        // rules the way core objects do), so every select field is offered
        // and `detect_status_value` below resolves which one a spoken
        // value actually belongs to.
        let extra_status_fields = custom_field_service::list_definitions(conn, workspace_id, &def.key, true)?
            .into_iter()
            .filter(|f| f.field_type == "select" && !f.options.is_empty())
            .map(|f| (f.key, f.options))
            .collect();
        catalog.push(CatalogEntry {
            nouns: vec![def.singular_label.to_lowercase(), def.plural_label.to_lowercase()],
            status_values: CUSTOM_RECORD_STATUSES.iter().map(|s| s.to_string()).collect(),
            extra_status_fields,
            object_key: Str::Owned(def.key),
        });
    }
    Ok(catalog)
}

/// Finds the object the transcript names explicitly (by singular/plural
/// noun), if any - session context supplies the object type otherwise.
fn detect_object_key<'a>(catalog: &'a [CatalogEntry], text_lower: &str) -> Option<&'a CatalogEntry> {
    catalog.iter().find(|e| e.nouns.iter().any(|n| text_lower.contains(n.as_str())))
}

/// The longest (most specific) value in `options` that appears in
/// `text_lower`, or none - never the first, since a plain substring check
/// alone would let a shorter value hiding inside a longer one win by
/// accident ("Active" is itself a substring of "Inactive"; a shorter,
/// unrelated status could just as easily sit inside a longer real one on
/// some other object's own vocabulary). Longest-match is a real
/// disambiguation rule here, not a first-match convenience the way
/// `detect_object_key` above's noun check still legitimately is (an
/// object's nouns are never substrings of each other).
fn longest_matching_value(options: &[String], text_lower: &str) -> Option<String> {
    options.iter().filter(|v| text_lower.contains(&v.to_lowercase())).max_by_key(|v| v.len()).cloned()
}

/// Finds a status/stage value the transcript names, constrained to one
/// catalog entry's own vocabulary - a case-insensitive substring match
/// against each valid value ("Won", "Under Review", ...), preferring the
/// longest match (see `longest_matching_value`). Returns the matched value
/// alongside which field it belongs to: `"status"` for every core object's
/// fixed vocabulary and every Custom Object's own generic Active/Inactive/
/// Archived column, or a specific custom field key when the value only
/// belongs to one of that object's own industry-vocabulary select fields
/// (checked in field-definition order; the first field with a match wins -
/// cross-field ambiguity between two different industry fields sharing a
/// value is a real edge case this doesn't resolve, unlike the
/// same-list case above).
fn detect_status_value(entry: &CatalogEntry, text_lower: &str) -> Option<(String, String)> {
    if let Some(v) = longest_matching_value(&entry.status_values, text_lower) {
        return Some((v, "status".to_string()));
    }
    for (field_key, options) in &entry.extra_status_fields {
        if let Some(v) = longest_matching_value(options, text_lower) {
            return Some((v, field_key.clone()));
        }
    }
    None
}

const NAVIGATE_TRIGGERS: &[&str] = &["open ", "show me ", "show ", "pull up ", "find ", "go to ", "navigate to "];
const QUERY_TRIGGERS: &[&str] = &["summarize ", "summarise ", "tell me about ", "what is ", "what's ", "describe "];
const CREATE_TASK_TRIGGERS: &[&str] = &["create a task ", "create task ", "add a task ", "add task ", "schedule a task ", "remind me to "];
/// Voice-First Mode, PR 2 (part 3): generic CREATE for anything that isn't
/// a Task (which `CREATE_TASK_TRIGGERS` above already owns, checked first
/// in `plan()` so "create a task ..." never falls through to here).
const CREATE_TRIGGERS: &[&str] = &["create a ", "create an ", "add a ", "add an ", "new "];
/// The core objects a guided voice CREATE does *not* support yet, named
/// honestly rather than silently failing to resolve: each one's own real
/// `Input` struct requires a related record (a Company, for Contact/
/// Opportunity/Quote/Order/Contract) that a single linear "ask the next
/// missing field" loop doesn't resolve - that's real follow-on work (voice
/// lookup-type fields), not implied by this pass. Company has no such
/// relationship (only `name` is required), so it's the one core object
/// supported here alongside every Custom Object, which by construction
/// only ever requires `primary_name` (VOICE-AC-06: no new code needed for
/// an admin-defined object).
const CREATE_UNSUPPORTED_CORE_OBJECTS: &[&str] = &["Contact", "Opportunity", "Quote", "Order", "Contract", "Task"];
const CAPTURE_TRIGGERS: &[&str] = &["add an interaction ", "add interaction ", "log a call ", "log an email ", "log a message ", "note that ", "add a note ", "add note "];
const UPDATE_TRIGGERS: &[&str] = &["mark ", "set ", "update ", "change "];
/// Voice-First Mode, PR 2: RUN_AGENT (spec v0.4 "Voice & AI Agents") - the
/// same design principle as every other intent: Voice is a new *caller*
/// into the existing chat/orchestration entry points
/// (`chat_service::send_agent_message`/`ai_orchestration_service::run_manual`),
/// never a second agent-runtime. "delegate to " is deliberately generic
/// (no "agent"/"pipeline" noun required) since a spoken name usually makes
/// the target obvious on its own.
const RUN_AGENT_TRIGGERS: &[&str] = &["ask agent ", "ask the agent ", "ask ", "run agent ", "delegate to agent ", "delegate to "];
const RUN_PIPELINE_TRIGGERS: &[&str] = &["run pipeline ", "run the pipeline ", "trigger pipeline "];

fn strip_any_prefix<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    for p in prefixes {
        if let Some(rest) = text.strip_prefix(p) {
            return Some(rest.trim());
        }
    }
    None
}

/// A tiny date-phrase vocabulary (spec's own examples never go beyond
/// "today"/"tomorrow"/a weekday name) - resolved against the *server's*
/// current date, not parsed from free-form natural language date math.
fn resolve_date_phrase(word: &str) -> Option<String> {
    use chrono::{Datelike, Duration, Utc, Weekday};
    let today = Utc::now().date_naive();
    let lower = word.to_lowercase();
    let date = match lower.as_str() {
        "today" => today,
        "tomorrow" => today + Duration::days(1),
        _ => {
            let target = match lower.as_str() {
                "monday" => Weekday::Mon,
                "tuesday" => Weekday::Tue,
                "wednesday" => Weekday::Wed,
                "thursday" => Weekday::Thu,
                "friday" => Weekday::Fri,
                "saturday" => Weekday::Sat,
                "sunday" => Weekday::Sun,
                _ => return None,
            };
            let mut d = today + Duration::days(1);
            while d.weekday() != target {
                d += Duration::days(1);
            }
            d
        }
    };
    Some(date.format("%Y-%m-%d").to_string())
}

fn extract_due_date(text: &str) -> (Option<String>, String) {
    let words = ["today", "tomorrow", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
    let lower = text.to_lowercase();
    for w in words {
        if let Some(pos) = lower.find(w) {
            if let Some(date) = resolve_date_phrase(w) {
                let mut cleaned = text.to_string();
                cleaned.replace_range(pos..pos + w.len(), "");
                let cleaned = cleaned.replace("  ", " ").trim().trim_start_matches("for").trim_start_matches("on").trim().to_string();
                return (Some(date), cleaned);
            }
        }
    }
    (None, text.to_string())
}

/// One outcome of `plan()` - either a ready-to-risk-assess plan, a
/// clarification the caller must ask before planning can continue, or an
/// honest "didn't understand" (never a guess, per spec §20).
#[derive(Debug)]
pub enum PlanOutcome {
    Ready { plan: VoiceActionPlanBody, intent: String, object_key: Option<String>, resolved_record_id: Option<String>, intent_confidence: f64, entity_confidence: Option<f64> },
    NeedsClarification { question: String, candidates: Vec<ResolutionCandidate> },
    /// Voice-First Mode, PR 2 (part 3): a guided CREATE still has at least
    /// one required field left to ask about - distinct from
    /// `NeedsClarification` (which always offers a fixed list of existing
    /// records to pick from): here the caller expects a free-text/spoken
    /// answer to `question`, and `pending` is what `voice_execution_service`
    /// persists so the *next* transcript is read as that answer rather than
    /// re-parsed as a brand-new command.
    NeedsMoreInfo { question: String, pending: PendingCreate },
    Unsupported { reason: String },
}

#[allow(clippy::too_many_arguments)]
pub fn plan(
    conn: &Connection,
    workspace_id: &str,
    transcript: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    conversation_reference: Option<(&str, &str)>,
    pending_create: Option<&PendingCreate>,
) -> AppResult<PlanOutcome> {
    let text = transcript.trim();
    let lower = text.to_lowercase();

    // A guided CREATE already in progress takes this transcript as the
    // answer to whichever field it last asked about - never re-parsed
    // against NAVIGATE/UPDATE/... triggers below, the same way a plain
    // English conversation doesn't restart from scratch mid-sentence.
    if let Some(pending) = pending_create {
        return continue_guided_create(conn, workspace_id, text, pending);
    }

    let catalog = full_catalog(conn, workspace_id)?;

    if let Some(rest) = strip_any_prefix(&lower, NAVIGATE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return resolve_and_wrap(conn, workspace_id, &catalog, reference, context_object_key, context_record_id, conversation_reference, "NAVIGATE", |object_key, record_id| VoiceActionPlanBody {
            steps: vec![VoicePlanStep { action: "navigate".into(), object_key: object_key.into(), record_id: Some(record_id.into()), fields: Default::default(), description: format!("Open this {object_key} record"), brief_description: format!("Opening {object_key}"), detail_note: None }],
        });
    }

    if let Some(rest) = strip_any_prefix(&lower, QUERY_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return resolve_and_wrap(conn, workspace_id, &catalog, reference, context_object_key, context_record_id, conversation_reference, "QUERY", |object_key, record_id| VoiceActionPlanBody {
            steps: vec![VoicePlanStep { action: "query".into(), object_key: object_key.into(), record_id: Some(record_id.into()), fields: Default::default(), description: format!("Read-only summary of this {object_key} record"), brief_description: "Summary".into(), detail_note: None }],
        });
    }

    if let Some(rest) = strip_any_prefix(&lower, CREATE_TASK_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        let (due_date, title) = extract_due_date(reference);
        if title.is_empty() {
            return Ok(PlanOutcome::Unsupported { reason: "I didn't catch what the task should say - try again with a short description.".into() });
        }
        let mut fields = std::collections::HashMap::new();
        fields.insert("title".to_string(), title.clone());
        if let Some(d) = &due_date {
            fields.insert("due_date".to_string(), d.clone());
        }
        return Ok(PlanOutcome::Ready {
            plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "create_task".into(), object_key: "Task".into(), record_id: None, fields, description: format!("Create task \"{title}\"{}", due_date.as_deref().map(|d| format!(" due {d}")).unwrap_or_default()), brief_description: format!("New task: {title}"), detail_note: due_date.as_deref().map(|d| format!("No other fields are set on this task besides its title and its due date of {d}.")) }] },
            intent: "CREATE".into(),
            object_key: Some("Task".into()),
            resolved_record_id: None,
            intent_confidence: 0.9,
            entity_confidence: None,
        });
    }

    // CAPTURE's own triggers ("add an interaction ", "add a note ", ...)
    // are checked before the generic CREATE_TRIGGERS below - both "add an "
    // and "add a " are deliberately broad (any object noun can follow), so
    // checking CREATE first would shadow every CAPTURE phrase that also
    // starts with "add a"/"add an" (the more specific phrase must win, same
    // reasoning as CREATE_TASK_TRIGGERS being checked ahead of this generic
    // CREATE_TRIGGERS above).
    if let Some(rest) = strip_any_prefix(&lower, CAPTURE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_capture(conn, workspace_id, &catalog, reference, context_object_key, context_record_id, conversation_reference);
    }

    if let Some(rest) = strip_any_prefix(&lower, CREATE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_create_record(conn, workspace_id, &catalog, reference);
    }

    if let Some(rest) = strip_any_prefix(&lower, UPDATE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_update_status(conn, workspace_id, &catalog, reference, context_object_key, context_record_id, conversation_reference);
    }

    if let Some(rest) = strip_any_prefix(&lower, RUN_PIPELINE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_run_pipeline(conn, workspace_id, reference);
    }

    if let Some(rest) = strip_any_prefix(&lower, RUN_AGENT_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_run_agent(conn, workspace_id, reference);
    }

    Ok(PlanOutcome::Unsupported {
        reason: "I didn't recognize a command there. Try things like \"open Northern Star\", \"mark this Opportunity Won\", \"create a task to follow up tomorrow\", or \"ask the Sales Coach agent to summarize this deal\".".into(),
    })
}

/// Splits "<name> to|with|that|: <message>" into (name, message) - the same
/// permissive "first separator keyword wins" shape `plan_capture` already
/// uses for "add an interaction to X that Y happened", so a spoken agent/
/// pipeline name doesn't need any special quoting convention. `" with "`
/// covers the Pipeline phrasing ("run pipeline Lead Triage with this lead");
/// the others cover the Agent phrasing ("ask the Sales Coach agent to ...").
fn split_name_and_message(reference: &str) -> (String, String) {
    let lower = reference.to_lowercase();
    let split_at = [" to ", " with ", " that ", ": ", " saying "].iter().find_map(|kw| lower.find(kw).map(|i| (i, kw.len())));
    match split_at {
        Some((idx, kw_len)) => (reference[..idx].trim().to_string(), reference[idx + kw_len..].trim().to_string()),
        None => (String::new(), String::new()),
    }
}

/// RUN_AGENT (spec v0.4): resolves a spoken agent name against this
/// workspace's real `ai_agent_service::list` - never a fixed roster - and
/// hands the rest of the sentence to it verbatim as a chat turn. Voice
/// never talks to an LLM provider itself; it only calls
/// `chat_service::send_agent_message`, the exact same function the AI
/// Agent Foundry chat panel calls (see `voice_execution_service`'s own
/// design-principle doc comment).
fn plan_run_agent(conn: &Connection, workspace_id: &str, reference: &str) -> AppResult<PlanOutcome> {
    let (name_part, message) = split_name_and_message(reference);
    if name_part.is_empty() || message.is_empty() {
        return Ok(PlanOutcome::Unsupported { reason: "Try \"ask <agent name> to <what you want>\", e.g. \"ask the Sales Coach agent to summarize this deal\".".into() });
    }
    let name_lower = name_part.to_lowercase();
    let name_lower = name_lower.trim_start_matches("the ").trim_end_matches(" agent").trim();
    let agents = ai_agent_service::list(conn, workspace_id, true)?;
    let matches: Vec<_> = agents.into_iter().filter(|a| { let n = a.name.to_lowercase(); n.contains(name_lower) || name_lower.contains(n.as_str()) }).collect();
    match matches.len() {
        0 => Ok(PlanOutcome::Unsupported { reason: format!("I couldn't find an AI Agent named \"{name_part}\" - check the name under AI Agent Foundry.") }),
        1 => {
            let agent = &matches[0];
            let mut fields = std::collections::HashMap::new();
            fields.insert("message".to_string(), message.clone());
            Ok(PlanOutcome::Ready {
                plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "run_agent".into(), object_key: "AiAgent".into(), record_id: Some(agent.id.clone()), fields, description: format!("Ask {} agent: \"{}\"", agent.name, message), brief_description: format!("Asking {}", agent.name), detail_note: Some("The agent's reply is spoken once it responds, after you confirm.".into()) }] },
                intent: "RUN_AGENT".into(),
                object_key: Some("AiAgent".into()),
                resolved_record_id: Some(agent.id.clone()),
                intent_confidence: 0.85,
                entity_confidence: Some(1.0),
            })
        }
        _ => Ok(PlanOutcome::NeedsClarification {
            question: format!("More than one Agent matches \"{name_part}\" - which one did you mean?"),
            candidates: matches.into_iter().map(|a| ResolutionCandidate { record_id: a.id, label: a.name }).collect(),
        }),
    }
}

/// RUN_AGENT's Pipeline counterpart - same shape, resolved against
/// `ai_orchestration_service::list_pipelines` and executed through
/// `ai_orchestration_service::run_manual`, never a second orchestration
/// engine for Voice.
fn plan_run_pipeline(conn: &Connection, workspace_id: &str, reference: &str) -> AppResult<PlanOutcome> {
    let (name_part, message) = split_name_and_message(reference);
    if name_part.is_empty() || message.is_empty() {
        return Ok(PlanOutcome::Unsupported { reason: "Try \"run pipeline <name> with <input>\", e.g. \"run pipeline Lead Triage with this new lead\".".into() });
    }
    let name_lower = name_part.to_lowercase();
    let name_lower = name_lower.trim_start_matches("the ").trim_end_matches(" pipeline").trim();
    let pipelines = ai_orchestration_service::list_pipelines(conn, workspace_id, true)?;
    let matches: Vec<_> = pipelines.into_iter().filter(|p| { let n = p.name.to_lowercase(); n.contains(name_lower) || name_lower.contains(n.as_str()) }).collect();
    match matches.len() {
        0 => Ok(PlanOutcome::Unsupported { reason: format!("I couldn't find a Pipeline named \"{name_part}\" - check the name under Orchestration.") }),
        1 => {
            let pipeline = &matches[0];
            let mut fields = std::collections::HashMap::new();
            fields.insert("message".to_string(), message.clone());
            Ok(PlanOutcome::Ready {
                plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "run_pipeline".into(), object_key: "AiAgentPipeline".into(), record_id: Some(pipeline.id.clone()), fields, description: format!("Run {} pipeline: \"{}\"", pipeline.name, message), brief_description: format!("Running {} pipeline", pipeline.name), detail_note: Some("The pipeline's result is spoken once it finishes, after you confirm.".into()) }] },
                intent: "RUN_AGENT".into(),
                object_key: Some("AiAgentPipeline".into()),
                resolved_record_id: Some(pipeline.id.clone()),
                intent_confidence: 0.85,
                entity_confidence: Some(1.0),
            })
        }
        _ => Ok(PlanOutcome::NeedsClarification {
            question: format!("More than one Pipeline matches \"{name_part}\" - which one did you mean?"),
            candidates: matches.into_iter().map(|p| ResolutionCandidate { record_id: p.id, label: p.name }).collect(),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_and_wrap(
    conn: &Connection,
    workspace_id: &str,
    catalog: &[CatalogEntry],
    reference: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    conversation_reference: Option<(&str, &str)>,
    intent: &str,
    build: impl FnOnce(&str, &str) -> VoiceActionPlanBody,
) -> AppResult<PlanOutcome> {
    let hint = detect_object_key(catalog, &reference.to_lowercase()).map(|e| e.object_key.as_str().to_string());
    match voice_entity_resolver::resolve_by_reference(conn, workspace_id, hint.as_deref(), reference, context_object_key, context_record_id, conversation_reference)? {
        ResolutionOutcome::Resolved { record_id, object_key, confidence } => Ok(PlanOutcome::Ready {
            plan: build(&object_key, &record_id),
            intent: intent.to_string(),
            object_key: Some(object_key),
            resolved_record_id: Some(record_id),
            intent_confidence: 0.85,
            entity_confidence: Some(confidence),
        }),
        ResolutionOutcome::NeedsClarification { candidates } => Ok(PlanOutcome::NeedsClarification { question: "I found more than one record that could match - which one did you mean?".into(), candidates }),
        ResolutionOutcome::NotFound => Ok(PlanOutcome::Unsupported { reason: format!("I couldn't find a record matching \"{reference}\".") }),
    }
}

#[allow(clippy::too_many_arguments)]
fn plan_update_status(
    conn: &Connection,
    workspace_id: &str,
    catalog: &[CatalogEntry],
    reference: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    conversation_reference: Option<(&str, &str)>,
) -> AppResult<PlanOutcome> {
    let lower = reference.to_lowercase();

    // The object type is either named explicitly, or - far more common in
    // practice ("mark this Won") - implied by whatever record the user is
    // currently looking at (spec §7's context-aware voice).
    let entry = match detect_object_key(catalog, &lower) {
        Some(e) => e,
        None => match context_object_key.and_then(|ck| catalog.iter().find(|e| e.object_key.as_str().eq_ignore_ascii_case(ck))) {
            Some(e) => e,
            None => return Ok(PlanOutcome::Unsupported { reason: "I couldn't tell which kind of record to update - try naming it, e.g. \"mark this Opportunity Won\".".into() }),
        },
    };
    let entry_owned = entry.object_key.as_str().to_string();

    let Some((status_value, status_field_key)) = detect_status_value(entry, &lower) else {
        let mut all_values = entry.status_values.clone();
        for (_, options) in &entry.extra_status_fields {
            all_values.extend(options.iter().cloned());
        }
        return Ok(PlanOutcome::Unsupported { reason: format!("I didn't catch a valid status for {entry_owned} - valid values are: {}.", all_values.join(", ")) });
    };

    // Whatever's left after removing the object noun and the status value
    // itself is the record reference ("Northern Star CRM implementation").
    let mut remainder = lower.clone();
    for n in &entry.nouns {
        remainder = remainder.replace(n.as_str(), " ");
    }
    remainder = remainder.replace(&status_value.to_lowercase(), " ");
    // Trim stray connective words ("mark X as Won" -> just "X") - the
    // status word itself is already stripped above.
    let reference_text = remainder.split_whitespace().filter(|w| !["as", "to", "the"].contains(w)).collect::<Vec<_>>().join(" ");
    let reference_text = if reference_text.trim().is_empty() { reference.to_string() } else { reference_text };

    match voice_entity_resolver::resolve_by_reference(conn, workspace_id, Some(&entry_owned), &reference_text, context_object_key, context_record_id, conversation_reference)? {
        ResolutionOutcome::Resolved { record_id, object_key, confidence } => {
            let mut fields = std::collections::HashMap::new();
            fields.insert("status".to_string(), status_value.clone());
            // Only meaningful for a Custom Object ("status" itself when
            // the value matched the generic Active/Inactive/Archived
            // column) - voice_execution_service's core-object arms ignore
            // this marker entirely, since each already writes to its own
            // hardcoded field.
            let human_field = if status_field_key == "status" { "status".to_string() } else { status_field_key.replace('_', " ") };
            fields.insert("status_field_key".to_string(), status_field_key);
            Ok(PlanOutcome::Ready {
                plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "update_status".into(), object_key: object_key.clone(), record_id: Some(record_id.clone()), fields, description: format!("Set {object_key} status to {status_value}"), brief_description: format!("Status: {status_value}"), detail_note: Some(format!("This only changes the {human_field} field on this {object_key} record - nothing else is affected.")) }] },
                intent: "UPDATE".into(),
                object_key: Some(object_key),
                resolved_record_id: Some(record_id),
                intent_confidence: 0.85,
                entity_confidence: Some(confidence),
            })
        }
        ResolutionOutcome::NeedsClarification { candidates } => Ok(PlanOutcome::NeedsClarification { question: format!("Which {entry_owned} did you mean?"), candidates }),
        ResolutionOutcome::NotFound => Ok(PlanOutcome::Unsupported { reason: format!("I couldn't find a {entry_owned} matching \"{reference_text}\".") }),
    }
}

#[allow(clippy::too_many_arguments)]
fn plan_capture(
    conn: &Connection,
    workspace_id: &str,
    catalog: &[CatalogEntry],
    reference: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    conversation_reference: Option<(&str, &str)>,
) -> AppResult<PlanOutcome> {
    // "Add an interaction to Contact ID 113609 that renewal email is
    // sent" - split on the first "that"/"saying"/"noting" into (who it's
    // about) and (what to log); fall back to session context entirely if
    // no target is named (spec §7).
    let lower = reference.to_lowercase();
    let split_at = ["that ", "saying ", "noting ", "noting that "].iter().find_map(|kw| lower.find(kw).map(|i| (i, kw.len())));

    let (target_part, body) = match split_at {
        Some((idx, kw_len)) => (reference[..idx].trim().to_string(), reference[idx + kw_len..].trim().to_string()),
        None => (String::new(), reference.to_string()),
    };
    if body.is_empty() {
        return Ok(PlanOutcome::Unsupported { reason: "I didn't catch what to note - try \"add an interaction to <record> that <what happened>\".".into() });
    }

    let hint = detect_object_key(catalog, &target_part.to_lowercase()).map(|e| e.object_key.as_str().to_string());
    let target_text = if target_part.is_empty() { "this".to_string() } else { target_part.replace("to ", "").replace("on ", "").replace("for ", "").trim().to_string() };

    match voice_entity_resolver::resolve_by_reference(conn, workspace_id, hint.as_deref(), &target_text, context_object_key, context_record_id, conversation_reference)? {
        ResolutionOutcome::Resolved { record_id, object_key, confidence } => {
            let mut fields = std::collections::HashMap::new();
            fields.insert("body".to_string(), body.clone());
            fields.insert("channel".to_string(), "message".to_string());
            Ok(PlanOutcome::Ready {
                plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "log_activity".into(), object_key: object_key.clone(), record_id: Some(record_id.clone()), fields, description: format!("Log an interaction on this {object_key}: \"{body}\""), brief_description: format!("Logging a note on this {object_key}"), detail_note: Some("This only adds a new interaction entry - it doesn't change any of this record's own fields.".into()) }] },
                intent: "CAPTURE".into(),
                object_key: Some(object_key),
                resolved_record_id: Some(record_id),
                intent_confidence: 0.85,
                entity_confidence: Some(confidence),
            })
        }
        ResolutionOutcome::NeedsClarification { candidates } => Ok(PlanOutcome::NeedsClarification { question: "Which record should this interaction be logged against?".into(), candidates }),
        ResolutionOutcome::NotFound => Ok(PlanOutcome::Unsupported { reason: format!("I couldn't find a record matching \"{target_text}\".") }),
    }
}

/// Voice-First Mode, PR 2 (part 3): "create a <object> <name>" / "add a
/// <object> ..." - the object noun is detected the same way every other
/// intent detects it (`detect_object_key`), then stripped from the
/// reference so whatever's left (if anything) is taken as the record's
/// name/title, exactly the same "everything after the noun is the value"
/// shape `CREATE_TASK_TRIGGERS` already uses for a Task's title.
fn plan_create_record(conn: &Connection, workspace_id: &str, catalog: &[CatalogEntry], reference: &str) -> AppResult<PlanOutcome> {
    let lower = reference.to_lowercase();
    let Some(entry) = detect_object_key(catalog, &lower) else {
        return Ok(PlanOutcome::Unsupported { reason: "I couldn't tell what kind of record to create - try naming it, e.g. \"create a company Acme Corp\" or \"create a Unit\".".into() });
    };
    let object_key = entry.object_key.as_str().to_string();
    if CREATE_UNSUPPORTED_CORE_OBJECTS.contains(&object_key.as_str()) {
        return Ok(PlanOutcome::Unsupported {
            reason: format!("Voice can't create a {object_key} yet - {object_key} records need a related Company chosen first, which voice doesn't support in this release. Try \"create a company\" or a custom object instead."),
        });
    }

    // Strip the first matching noun (case-insensitively) from the
    // *original*-case reference, so a given name keeps its real casing
    // ("Acme Corp", not "acme corp") - `detect_object_key` above only
    // needed the lowercased copy to find which noun matched.
    let mut remainder_orig = reference.to_string();
    let remainder_lower = lower.clone();
    if let Some(pos) = entry.nouns.iter().find_map(|n| remainder_lower.find(n.as_str()).map(|p| (p, n.len()))) {
        let (pos, len) = pos;
        remainder_orig = format!("{}{}", &remainder_orig[..pos], &remainder_orig[pos + len..]);
    }
    let given_name = remainder_orig.split_whitespace().collect::<Vec<_>>().join(" ");

    let (name_key, _) = create_name_field(conn, workspace_id, &object_key)?;
    let mut fields = std::collections::HashMap::new();
    if !given_name.is_empty() {
        fields.insert(name_key, given_name);
    }
    build_guided_create_outcome(conn, workspace_id, &object_key, fields)
}

/// The one field every guided create always starts by asking about if it
/// wasn't already given in the initial utterance - `name` for the one core
/// object supported here (Company), `primary_name` for any Custom Object
/// (the field every custom record has unconditionally, per
/// `CustomRecordInput`/VOICE-AC-06 - no per-object configuration needed).
fn create_name_field(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<(String, String)> {
    if object_key == "Company" {
        return Ok(("name".to_string(), "company name".to_string()));
    }
    let label = custom_object_service::list(conn, workspace_id, true)?
        .into_iter()
        .find(|d| d.key == object_key)
        .map(|d| d.singular_label)
        .unwrap_or_else(|| object_key.to_string());
    Ok(("primary_name".to_string(), format!("{} name", label.to_lowercase())))
}

/// The single loop every turn of a guided create runs through: is the
/// name-like field filled in yet, then is every `required` custom field on
/// this object filled in yet (in their own `sort_order`, matching the order
/// the record form itself would present them) - the first one still
/// missing is what gets asked about next. Once nothing is left, builds the
/// final `create_record` plan step. Reused identically for both the very
/// first utterance and every follow-up answer (`continue_guided_create`
/// below), so there is exactly one place that decides "what's still
/// missing" - never two copies to keep in sync.
fn build_guided_create_outcome(conn: &Connection, workspace_id: &str, object_key: &str, fields: std::collections::HashMap<String, String>) -> AppResult<PlanOutcome> {
    let (name_key, name_label) = create_name_field(conn, workspace_id, object_key)?;
    if fields.get(&name_key).map(|v| v.trim().is_empty()).unwrap_or(true) {
        let pending = PendingCreate { object_key: object_key.to_string(), fields, asking_key: name_key, asking_label: name_label.clone(), asking_type: "text".to_string() };
        return Ok(PlanOutcome::NeedsMoreInfo { question: format!("What should the {name_label} be?"), pending });
    }

    let mut defs = custom_field_service::list_definitions(conn, workspace_id, object_key, true)?;
    defs.sort_by_key(|d| d.sort_order);
    for def in defs.iter().filter(|d| d.required) {
        if fields.get(&def.key).map(|v| v.trim().is_empty()).unwrap_or(true) {
            let hint = match def.field_type.as_str() {
                "select" if !def.options.is_empty() => format!(" (one of: {})", def.options.join(", ")),
                "boolean" => " (yes or no)".to_string(),
                _ => String::new(),
            };
            let pending = PendingCreate { object_key: object_key.to_string(), fields, asking_key: def.key.clone(), asking_label: def.label.clone(), asking_type: def.field_type.clone() };
            return Ok(PlanOutcome::NeedsMoreInfo { question: format!("What's the {}{hint}?", def.label), pending });
        }
    }

    let display = fields.get(&name_key).cloned().unwrap_or_default();
    let extra_fields: Vec<String> = fields.iter().filter(|(k, _)| *k != &name_key).map(|(k, v)| format!("{}: {v}", k.replace('_', " "))).collect();
    let detail_note = if extra_fields.is_empty() { None } else { Some(format!("Also sets {}.", extra_fields.join(", "))) };
    Ok(PlanOutcome::Ready {
        plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "create_record".into(), object_key: object_key.to_string(), record_id: None, fields, description: format!("Create {object_key} \"{display}\""), brief_description: format!("New {object_key}: {display}"), detail_note }] },
        intent: "CREATE".into(),
        object_key: Some(object_key.to_string()),
        resolved_record_id: None,
        intent_confidence: 0.9,
        entity_confidence: None,
    })
}

/// Best-effort normalization of a spoken/typed answer against its field's
/// own type, mirroring the vocabulary this module already uses elsewhere
/// (`detect_status_value`'s case-insensitive option match,
/// `resolve_date_phrase`'s today/tomorrow/weekday words) - never a hard
/// requirement: a value this can't normalize still flows through as typed,
/// and the real save-time validation in
/// `custom_field_service::set_entity_values` (run at execution, not here)
/// is the actual enforcement backstop, exactly like every other field this
/// codebase validates - this function only tries to make the common
/// spoken forms ("yes", "tomorrow", an option's own wording) land cleanly.
fn normalize_answer(conn: &Connection, workspace_id: &str, object_key: &str, field_key: &str, field_type: &str, raw: &str) -> AppResult<String> {
    match field_type {
        "select" => {
            let defs = custom_field_service::list_definitions(conn, workspace_id, object_key, true)?;
            if let Some(def) = defs.iter().find(|d| d.key == field_key) {
                let lower = raw.to_lowercase();
                if let Some(opt) = def.options.iter().find(|o| o.to_lowercase() == lower || lower.contains(&o.to_lowercase())) {
                    return Ok(opt.clone());
                }
            }
            Ok(raw.to_string())
        }
        "boolean" => {
            let lower = raw.to_lowercase();
            if ["yes", "true", "yeah", "yep", "sure"].contains(&lower.as_str()) {
                Ok("true".to_string())
            } else if ["no", "false", "nope", "nah"].contains(&lower.as_str()) {
                Ok("false".to_string())
            } else {
                Ok(raw.to_string())
            }
        }
        "date" => Ok(resolve_date_phrase(raw).unwrap_or_else(|| raw.to_string())),
        _ => Ok(raw.to_string()),
    }
}

/// One turn of an already-in-progress guided create: `answer_text` is
/// read as the answer to `pending.asking_key`, never re-parsed against any
/// intent trigger (spec's own "guided, one field at a time" shape - see
/// this module's top-level `plan()` doc comment). A plain cancel word
/// abandons the create outright (returned as `Unsupported`, which is what
/// makes `voice_execution_service::submit_command` clear the pending state
/// - see its own doc comment) rather than forcing the user to answer a
/// question about a record they no longer want.
fn continue_guided_create(conn: &Connection, workspace_id: &str, answer_text: &str, pending: &PendingCreate) -> AppResult<PlanOutcome> {
    let raw = answer_text.trim();
    let lower = raw.to_lowercase();
    if ["cancel", "never mind", "nevermind", "stop", "forget it"].contains(&lower.as_str()) {
        return Ok(PlanOutcome::Unsupported { reason: format!("Okay, cancelled creating that {}.", pending.object_key) });
    }
    if raw.is_empty() {
        return Ok(PlanOutcome::NeedsMoreInfo { question: format!("I still need the {} - what should it be?", pending.asking_label), pending: pending.clone() });
    }

    let value = normalize_answer(conn, workspace_id, &pending.object_key, &pending.asking_key, &pending.asking_type, raw)?;
    let mut fields = pending.fields.clone();
    fields.insert(pending.asking_key.clone(), value);
    build_guided_create_outcome(conn, workspace_id, &pending.object_key, fields)
}

/// Public helper `voice_execution_service`/the Tauri layer use to look up a
/// custom field by label for the "set <label> to <value>" shape - kept
/// here since it's planning-time metadata resolution, not execution.
pub fn find_custom_field_by_label(conn: &Connection, workspace_id: &str, object_key: &str, label_text: &str) -> AppResult<Option<crate::models::custom_field::CustomFieldDefinition>> {
    let defs = custom_field_service::list_definitions(conn, workspace_id, object_key, true)?;
    let lower = label_text.to_lowercase();
    Ok(defs.into_iter().find(|d| lower.contains(&d.label.to_lowercase())))
}
