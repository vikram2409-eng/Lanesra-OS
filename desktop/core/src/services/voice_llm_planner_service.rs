//! Voice-First Mode: an optional, workspace-wide LLM-backed conversational
//! fallback for a transcript the deterministic `voice_planner_service::
//! plan()` doesn't recognize (`PlanOutcome::Unsupported`). This module's
//! only job is *normalization*: turn a free-form utterance into one of
//! the same canonical command phrasings `plan()` already recognizes (or a
//! clarifying question), then hand that rewritten text straight back to
//! `plan()` itself - never resolve a record, classify risk, or execute
//! anything here. That's what makes this safe to add: every existing
//! guarantee (real entity resolution via `voice_entity_resolver`,
//! risk-gated confirmation via `voice_risk_service`, the same
//! entity-service write path via `voice_execution_service`) still applies
//! completely unchanged to whatever this produces, because it's the exact
//! same code path a manually-phrased command already goes through - an
//! LLM never gets a second, less-scrutinized way to act.
//!
//! Called only when `voice_llm_service`'s settings say a workspace has
//! opted in, and only for a user whose Voice policy already grants
//! `use_agents` (see `voice_execution_service::submit_command`'s own call
//! site) - the same trust tier as letting Voice run an AI Agent, since
//! this is exactly that: an LLM interpreting speech before anything acts
//! on it.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::chat::ChatMessage;
use crate::repositories::voice_repo;
use crate::services::ai_gateway_service::resolve_tier;
use crate::services::ai_service::{self, CompletionOutcome};
use crate::services::voice_planner_service::{self, PlanOutcome};

fn system_prompt(catalog_summary: &str) -> String {
    format!(
        "You help normalize spoken commands for a business app's Voice Mode. You never take any action yourself - you only rewrite text.\n\n\
        Objects and their statuses in this workspace:\n{catalog_summary}\n\n\
        Rewrite the user's utterance into EXACTLY ONE of these canonical forms, filling in the real object/record name and value from what they said:\n\
        - \"open <record name>\" (navigate to it)\n\
        - \"summarize <record name>\" (read-only summary)\n\
        - \"mark <record name> as <status>\" (change its status/stage)\n\
        - \"create a task <description>\" (optionally \"... due <day>\")\n\
        - \"add a <object> <name>\" (create a new record)\n\n\
        Respond with EXACTLY ONE line, one of:\n\
        REWRITE: <the rewritten command, nothing else>\n\
        CLARIFY: <a short question to ask the user instead>\n\n\
        If you cannot confidently rewrite the request into one of those forms - it's ambiguous, missing required info, or genuinely unsupported - respond with CLARIFY, never a guess."
    )
}

enum LlmOutcome {
    Rewrite(String),
    Clarify(String),
}

fn parse_response(text: &str) -> Option<LlmOutcome> {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("REWRITE:") {
        let rewritten = rest.trim();
        return if rewritten.is_empty() { None } else { Some(LlmOutcome::Rewrite(rewritten.to_string())) };
    }
    if let Some(rest) = trimmed.strip_prefix("CLARIFY:") {
        let question = rest.trim();
        return if question.is_empty() { None } else { Some(LlmOutcome::Clarify(question.to_string())) };
    }
    None
}

/// `None` means "didn't run, or couldn't help" - the caller
/// (`voice_execution_service::submit_command`) keeps its original
/// `Unsupported` outcome unchanged in that case. Never surfaces an error
/// of its own: a missing/misconfigured provider, a network failure, or a
/// malformed model response all just fail open to the same honest
/// "didn't understand" the deterministic planner already gives - this
/// fallback either helps or gets out of the way, it never turns a clean
/// failure into a confusing one.
#[allow(clippy::too_many_arguments)]
pub async fn try_plan(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    transcript: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    conversation_reference: Option<(&str, &str)>,
) -> AppResult<Option<PlanOutcome>> {
    let Some(settings) = voice_repo::get_llm_settings(conn, workspace_id)? else { return Ok(None) };
    if !settings.enabled {
        return Ok(None);
    }

    let Ok(resolved) = resolve_tier(conn, workspace_id, master_key, settings.provider_id.as_deref()) else { return Ok(None) };

    let catalog = voice_planner_service::full_catalog(conn, workspace_id)?;
    let prompt = system_prompt(&voice_planner_service::catalog_summary(&catalog));
    let history = vec![ChatMessage {
        id: String::new(),
        conversation_id: String::new(),
        role: "user".into(),
        content: Some(transcript.to_string()),
        tool_calls: None,
        tool_call_id: None,
        created_at: String::new(),
    }];

    let Ok((outcome, _usage)) = ai_service::dispatch_with_tools(&resolved.provider, resolved.base_url.as_deref(), &resolved.model, &resolved.api_key, &prompt, &[], &history).await else {
        return Ok(None);
    };
    let CompletionOutcome::Text(text) = outcome else { return Ok(None) };

    match parse_response(&text) {
        Some(LlmOutcome::Clarify(question)) => Ok(Some(PlanOutcome::NeedsClarification { question, candidates: vec![] })),
        Some(LlmOutcome::Rewrite(rewritten)) => {
            // Hands off to the exact same deterministic pipeline a
            // manually-typed command goes through - real entity
            // resolution, real risk classification, nothing skipped.
            match voice_planner_service::plan(conn, workspace_id, &rewritten, context_object_key, context_record_id, conversation_reference, None)? {
                // The rewrite didn't actually land on a recognized shape -
                // surface the caller's original Unsupported reason, not a
                // second, confusing "the AI tried and also failed" one.
                PlanOutcome::Unsupported { .. } => Ok(None),
                other => Ok(Some(other)),
            }
        }
        None => Ok(None),
    }
}
