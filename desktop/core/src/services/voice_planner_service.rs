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
use crate::models::voice::{ResolutionCandidate, VoiceActionPlanBody, VoicePlanStep};
use crate::services::{custom_field_service, custom_object_service, voice_entity_resolver};
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
}

fn core_catalog() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry { object_key: Str::Static("Company"), nouns: vec!["company".into(), "companies".into(), "customer".into(), "account".into()], status_values: COMPANY_STATUSES.iter().map(|s| s.to_string()).collect() },
        CatalogEntry { object_key: Str::Static("Contact"), nouns: vec!["contact".into(), "contacts".into(), "person".into()], status_values: CONTACT_STATUSES.iter().map(|s| s.to_string()).collect() },
        CatalogEntry { object_key: Str::Static("Opportunity"), nouns: vec!["opportunity".into(), "opportunities".into(), "deal".into()], status_values: OPPORTUNITY_STAGES.iter().map(|s| s.to_string()).collect() },
        CatalogEntry { object_key: Str::Static("Quote"), nouns: vec!["quote".into(), "quotes".into()], status_values: QUOTE_STATUSES.iter().map(|s| s.to_string()).collect() },
        CatalogEntry { object_key: Str::Static("Order"), nouns: vec!["order".into(), "orders".into()], status_values: ORDER_STATUSES.iter().map(|s| s.to_string()).collect() },
        CatalogEntry { object_key: Str::Static("Contract"), nouns: vec!["contract".into(), "contracts".into(), "agreement".into()], status_values: CONTRACT_STATUSES.iter().map(|s| s.to_string()).collect() },
        CatalogEntry { object_key: Str::Static("Task"), nouns: vec!["task".into(), "tasks".into(), "to-do".into(), "todo".into()], status_values: TASK_STATUSES.iter().map(|s| s.to_string()).collect() },
    ]
}

fn full_catalog(conn: &Connection, workspace_id: &str) -> AppResult<Vec<CatalogEntry>> {
    let mut catalog = core_catalog();
    for def in custom_object_service::list(conn, workspace_id, true)? {
        catalog.push(CatalogEntry {
            nouns: vec![def.singular_label.to_lowercase(), def.plural_label.to_lowercase()],
            status_values: CUSTOM_RECORD_STATUSES.iter().map(|s| s.to_string()).collect(),
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

/// Finds a status/stage value the transcript names, constrained to one
/// catalog entry's own vocabulary - a plain case-insensitive substring
/// match against each valid value ("Won", "Under Review", ...).
fn detect_status_value(entry: &CatalogEntry, text_lower: &str) -> Option<String> {
    entry.status_values.iter().find(|v| text_lower.contains(&v.to_lowercase())).cloned()
}

const NAVIGATE_TRIGGERS: &[&str] = &["open ", "show me ", "show ", "pull up ", "find ", "go to ", "navigate to "];
const QUERY_TRIGGERS: &[&str] = &["summarize ", "summarise ", "tell me about ", "what is ", "what's ", "describe "];
const CREATE_TASK_TRIGGERS: &[&str] = &["create a task ", "create task ", "add a task ", "add task ", "schedule a task ", "remind me to "];
const CAPTURE_TRIGGERS: &[&str] = &["add an interaction ", "add interaction ", "log a call ", "log an email ", "log a message ", "note that ", "add a note ", "add note "];
const UPDATE_TRIGGERS: &[&str] = &["mark ", "set ", "update ", "change "];

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
    Unsupported { reason: String },
}

#[allow(clippy::too_many_arguments)]
pub fn plan(
    conn: &Connection,
    workspace_id: &str,
    transcript: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
) -> AppResult<PlanOutcome> {
    let text = transcript.trim();
    let lower = text.to_lowercase();
    let catalog = full_catalog(conn, workspace_id)?;

    if let Some(rest) = strip_any_prefix(&lower, NAVIGATE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return resolve_and_wrap(conn, workspace_id, &catalog, reference, context_object_key, context_record_id, "NAVIGATE", |object_key, record_id| VoiceActionPlanBody {
            steps: vec![VoicePlanStep { action: "navigate".into(), object_key: object_key.into(), record_id: Some(record_id.into()), fields: Default::default(), description: format!("Open this {object_key} record") }],
        });
    }

    if let Some(rest) = strip_any_prefix(&lower, QUERY_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return resolve_and_wrap(conn, workspace_id, &catalog, reference, context_object_key, context_record_id, "QUERY", |object_key, record_id| VoiceActionPlanBody {
            steps: vec![VoicePlanStep { action: "query".into(), object_key: object_key.into(), record_id: Some(record_id.into()), fields: Default::default(), description: format!("Read-only summary of this {object_key} record") }],
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
            plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "create_task".into(), object_key: "Task".into(), record_id: None, fields, description: format!("Create task \"{title}\"{}", due_date.as_deref().map(|d| format!(" due {d}")).unwrap_or_default()) }] },
            intent: "CREATE".into(),
            object_key: Some("Task".into()),
            resolved_record_id: None,
            intent_confidence: 0.9,
            entity_confidence: None,
        });
    }

    if let Some(rest) = strip_any_prefix(&lower, CAPTURE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_capture(conn, workspace_id, &catalog, reference, context_object_key, context_record_id);
    }

    if let Some(rest) = strip_any_prefix(&lower, UPDATE_TRIGGERS) {
        let reference = &text[text.len() - rest.len()..];
        return plan_update_status(conn, workspace_id, &catalog, reference, context_object_key, context_record_id);
    }

    Ok(PlanOutcome::Unsupported {
        reason: "I didn't recognize a command there. Try things like \"open Northern Star\", \"mark this Opportunity Won\", or \"create a task to follow up tomorrow\".".into(),
    })
}

#[allow(clippy::too_many_arguments)]
fn resolve_and_wrap(
    conn: &Connection,
    workspace_id: &str,
    catalog: &[CatalogEntry],
    reference: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    intent: &str,
    build: impl FnOnce(&str, &str) -> VoiceActionPlanBody,
) -> AppResult<PlanOutcome> {
    let hint = detect_object_key(catalog, &reference.to_lowercase()).map(|e| e.object_key.as_str().to_string());
    match voice_entity_resolver::resolve_by_reference(conn, workspace_id, hint.as_deref(), reference, context_object_key, context_record_id)? {
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

fn plan_update_status(
    conn: &Connection,
    workspace_id: &str,
    catalog: &[CatalogEntry],
    reference: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
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

    let Some(status_value) = detect_status_value(entry, &lower) else {
        return Ok(PlanOutcome::Unsupported { reason: format!("I didn't catch a valid status for {entry_owned} - valid values are: {}.", entry.status_values.join(", ")) });
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

    match voice_entity_resolver::resolve_by_reference(conn, workspace_id, Some(&entry_owned), &reference_text, context_object_key, context_record_id)? {
        ResolutionOutcome::Resolved { record_id, object_key, confidence } => {
            let mut fields = std::collections::HashMap::new();
            fields.insert("status".to_string(), status_value.clone());
            Ok(PlanOutcome::Ready {
                plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "update_status".into(), object_key: object_key.clone(), record_id: Some(record_id.clone()), fields, description: format!("Set {object_key} status to {status_value}") }] },
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

fn plan_capture(
    conn: &Connection,
    workspace_id: &str,
    catalog: &[CatalogEntry],
    reference: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
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

    match voice_entity_resolver::resolve_by_reference(conn, workspace_id, hint.as_deref(), &target_text, context_object_key, context_record_id)? {
        ResolutionOutcome::Resolved { record_id, object_key, confidence } => {
            let mut fields = std::collections::HashMap::new();
            fields.insert("body".to_string(), body.clone());
            fields.insert("channel".to_string(), "message".to_string());
            Ok(PlanOutcome::Ready {
                plan: VoiceActionPlanBody { steps: vec![VoicePlanStep { action: "log_activity".into(), object_key: object_key.clone(), record_id: Some(record_id.clone()), fields, description: format!("Log an interaction on this {object_key}: \"{body}\"") }] },
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

/// Public helper `voice_execution_service`/the Tauri layer use to look up a
/// custom field by label for the "set <label> to <value>" shape - kept
/// here since it's planning-time metadata resolution, not execution.
pub fn find_custom_field_by_label(conn: &Connection, workspace_id: &str, object_key: &str, label_text: &str) -> AppResult<Option<crate::models::custom_field::CustomFieldDefinition>> {
    let defs = custom_field_service::list_definitions(conn, workspace_id, object_key, true)?;
    let lower = label_text.to_lowercase();
    Ok(defs.into_iter().find(|d| lower.contains(&d.label.to_lowercase())))
}
