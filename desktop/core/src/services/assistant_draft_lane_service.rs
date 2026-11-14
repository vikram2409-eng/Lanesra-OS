//! AI Agent Platform v2, Phase 7: describe a process - the Admin Assistant
//! drafts a lane (WP-0715, REQ-LANE-022).
//!
//! One model call, through the catalog prompt `assistant/draft-lane`: the
//! model replies with a lane definition file, the reply is normalized to a
//! first draft (format version `1.0.0`, lane version 1, status `draft`) and
//! stored through `process_lane_service::import_lane_definition`, which
//! parses and validates the definition (ADR-0024 §3, the schema's
//! kind-conditional rules mirrored in `validate_definition`) and stores the
//! version as a draft.
//!
//! There is no publish path in this module, by design (WP-0715 AC-2): the
//! assistant cannot publish a lane. Publishing stays a separate admin
//! action (`process_lane_service::publish_lane`), and a draft the assistant
//! made is published the same way as any other draft - by a person, from
//! the blueprint view (WP-0712) or the lane commands. A model reply that
//! claims any other status is normalized back to `draft` before import,
//! and `import_lane_definition` itself hardcodes the stored version's
//! status to `draft`, so the result is a draft even if the model
//! disobeys the prompt.
//!
//! The draft the model may produce is deliberately narrow: form steps and,
//! where the description states something to verify, check steps (the
//! prompt says so, and the schema validation enforces the shape). Record
//! and approval steps need governed writes that do not exist yet, and
//! agent steps need a named agent, so a person adds those kinds when
//! refining the draft.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::chat::ChatMessage;
use crate::models::process_lane::ProcessLane;
use crate::services::{ai_service, process_lane_service};

fn classified_validation(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, code, message)
}

/// Draft a lane from a plain-language description of a business process.
///
/// `description` is the process in the person's own words;
/// `object_key` is the object the lane runs on. The result is always a
/// draft, always valid against the lane schema, and never published.
/// Administrator-only: drafting creates a lane, and lane creation is an
/// admin action (`import_lane_definition` enforces the same gate).
///
/// Errors are `AppError::Validation` with code `validation.arguments`
/// when the description or object is missing, when the model does not
/// reply with a lane definition file, or when the reply fails schema
/// validation - in every refusal case nothing is stored.
pub async fn draft_lane(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor_user_id: Option<&str>,
    description: &str,
    object_key: &str,
) -> AppResult<ProcessLane> {
    super::user_service::require_admin(conn, actor_user_id)?;
    let description = description.trim();
    if description.is_empty() {
        return Err(classified_validation(
            "validation.arguments",
            "describe the process to draft - an empty description drafts nothing",
        ));
    }
    let object_key = object_key.trim();
    if object_key.is_empty() {
        return Err(classified_validation(
            "validation.arguments",
            "a lane needs the object it runs on",
        ));
    }

    let (system_prompt, _) = crate::prompts::render_active(
        "assistant/draft-lane",
        &[("description", description), ("object_key", object_key)],
    )
    .map_err(AppError::Validation)?;
    let history = vec![ChatMessage {
        id: String::new(),
        conversation_id: String::new(),
        role: "user".into(),
        content: Some(description.to_string()),
        tool_calls: None,
        tool_call_id: None,
        created_at: String::new(),
    }];
    let (outcome, _usage) = ai_service::complete_with_tools(
        conn,
        workspace_id,
        master_key,
        &system_prompt,
        &[],
        &history,
        ai_service::CallOptions {
            tools_off: true,
            ..Default::default()
        },
    )
    .await?;
    let text = match outcome {
        ai_service::CompletionOutcome::Text(text) => text,
        ai_service::CompletionOutcome::ToolCalls { .. } => {
            return Err(classified_validation(
                "validation.arguments",
                "the assistant replied with tool calls, not a lane definition",
            ));
        }
    };

    let definition_json = normalize_to_draft(&text)?;
    process_lane_service::import_lane_definition(
        conn,
        workspace_id,
        &definition_json,
        actor_user_id,
    )
}

/// Normalize a model reply to a first draft, refusing anything that is
/// not a lane definition file. The stored definition always says
/// format version `1.0.0`, lane version 1 and status `draft`, whatever
/// the model wrote - the draft invariant is enforced here, not trusted
/// to the reply.
fn normalize_to_draft(text: &str) -> AppResult<String> {
    let mut value: serde_json::Value = serde_json::from_str(text.trim()).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("the assistant's reply is not a JSON lane definition: {e}"),
        )
    })?;
    let lane = value
        .get_mut("lane")
        .and_then(|l| l.as_object_mut())
        .ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                "the assistant's reply is not a lane definition file - it has no 'lane'",
            )
        })?;
    lane.insert(
        "status".to_string(),
        serde_json::Value::String("draft".to_string()),
    );
    lane.insert("version".to_string(), serde_json::Value::Number(1.into()));
    if let Some(file) = value.as_object_mut() {
        file.insert(
            "format_version".to_string(),
            serde_json::Value::String("1.0.0".to_string()),
        );
    }
    serde_json::to_string(&value).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("the assistant's draft could not be stored: {e}"),
        )
    })
}
