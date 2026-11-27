//! AI Agent Platform v2, Phase 7: revise a lane by description - the Admin
//! Assistant edits a lane (WP-0723, REQ-LANE-022).
//!
//! One model call, through the catalog prompt `assistant/edit-lane`: the
//! model receives the lane's current definition plus the change
//! description, and replies with the revised definition file. The reply
//! is normalized to a draft version of the SAME lane (same id, same
//! object, status `draft`, format version `1.0.0`) and stored through
//! the lane lifecycle: `process_lane_service::create_version` when the
//! latest version is published - a new draft version, the published
//! version untouched - or `update_draft_version` when the latest version
//! is already a draft (one draft at a time: a second draft version is
//! refused by the lifecycle, so revising a draft revises that draft in
//! place, and the admin still publishes it by hand).
//!
//! There is no publish path in this module, by design (WP-0723 AC-3):
//! the assistant cannot publish a lane, and a model reply that claims
//! any other status is normalized back to `draft` before storage, the
//! same way the draft service normalizes. The revision is validated
//! before storage against the lane schema (the lifecycle's own
//! `parse_definition`) and the Assistant's authoring boundaries - the
//! adopted Action Type catalog and the named-agent rule, in
//! `assistant_draft_lane_service::validate_assistant_definition` - and
//! nothing is stored on any refusal.
//!
//! What "validates against the schema + publish checks" means here, and
//! what it deliberately does not: the schema check is the hard gate, and
//! the publish check that applies at authoring time is P-12 (every write
//! names an adopted Action Type) - the same check the draft service
//! runs. The full publish report is NOT run on an edit: a draft is
//! allowed to be unpublishable (the builder saves drafts that fail
//! publish checks all the time, and publish is the separate admin step),
//! so refusing an edit because the draft it revises fails some other
//! check - a branch the author is about to add, a provisional type the
//! edit is about to remove - would make the Assistant unable to fix
//! exactly the drafts it exists to fix.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::chat::ChatMessage;
use crate::models::process_lane::{parse_definition, ProcessLaneVersion};
use crate::repositories::process_lane_repo;
use crate::services::{ai_service, assistant_draft_lane_service, process_lane_service};

fn classified_validation(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, code, message)
}

/// Revise an existing lane from a plain-language change description.
///
/// `lane_key` is the lane to revise; `description` is the change in the
/// person's own words. The result is always a draft version of that same
/// lane - a new version when the latest is published, the revised draft
/// when the latest already is one - and never published. The published
/// version is untouched until the admin publishes the draft.
/// Administrator-only: editing creates or rewrites a lane version, and
/// lane versions are an admin action (the lifecycle functions enforce
/// the same gate).
///
/// Errors are `AppError::Validation` with code `validation.arguments`
/// when the description is empty, the lane key is unknown, the model
/// does not reply with a definition of this lane, or the reply fails
/// schema validation or the Assistant's authoring boundaries - in every
/// refusal case nothing is stored.
pub async fn edit_lane(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor_user_id: Option<&str>,
    lane_key: &str,
    description: &str,
) -> AppResult<ProcessLaneVersion> {
    super::user_service::require_admin(conn, actor_user_id)?;
    let lane_key = lane_key.trim();
    if lane_key.is_empty() {
        return Err(classified_validation(
            "validation.arguments",
            "a lane key is needed to edit a lane",
        ));
    }
    let description = description.trim();
    if description.is_empty() {
        return Err(classified_validation(
            "validation.arguments",
            "describe the change to make - an empty description edits nothing",
        ));
    }
    let lane =
        process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?.ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                format!("no lane with key '{lane_key}' - check the lane key"),
            )
        })?;
    let latest = process_lane_repo::latest_version(conn, &lane.id)?.ok_or_else(|| {
        classified_validation(
            "validation.arguments",
            format!("lane '{lane_key}' has no version to revise"),
        )
    })?;
    // One draft at a time: revising a draft revises that draft in
    // place; revising a published lane mints the next version as a
    // draft, and the published version stays exactly as it was.
    let (target_version, revising_draft) = if latest.status == "draft" {
        (latest.version, true)
    } else {
        (latest.version + 1, false)
    };

    let (system_prompt, _) = crate::prompts::render_active(
        "assistant/edit-lane",
        &[
            ("lane_key", lane_key),
            ("current_definition", &latest.definition_json),
            ("description", description),
        ],
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

    let definition_json =
        normalize_to_draft_version(&text, &lane.lane_key, &lane.object_key, target_version)?;
    let file = parse_definition(&definition_json)?;
    assistant_draft_lane_service::validate_assistant_definition(conn, workspace_id, &file)?;
    if revising_draft {
        process_lane_service::update_draft_version(
            conn,
            workspace_id,
            lane_key,
            &definition_json,
            actor_user_id,
        )
    } else {
        process_lane_service::create_version(
            conn,
            workspace_id,
            lane_key,
            &definition_json,
            actor_user_id,
        )
    }
}

/// Normalize a model reply to a draft version of this lane, refusing
/// anything that is not a definition of the lane being revised. The
/// stored definition always says format version `1.0.0`, the lane's own
/// id and object, the target version and status `draft`, whatever the
/// model wrote - the draft invariant is enforced here, not trusted to
/// the reply.
fn normalize_to_draft_version(
    text: &str,
    lane_key: &str,
    object_key: &str,
    target_version: i64,
) -> AppResult<String> {
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
    match lane.get("id").and_then(|id| id.as_str()) {
        Some(id) if id == lane_key => {}
        Some(id) => {
            return Err(classified_validation(
                "validation.arguments",
                format!(
                    "the assistant's reply is not a revision of lane '{lane_key}' - it names '{id}'"
                ),
            ));
        }
        None => {
            return Err(classified_validation(
                "validation.arguments",
                "the assistant's reply is not a lane definition file - its lane has no 'id'",
            ));
        }
    }
    lane.insert(
        "object_key".to_string(),
        serde_json::Value::String(object_key.to_string()),
    );
    lane.insert(
        "status".to_string(),
        serde_json::Value::String("draft".to_string()),
    );
    lane.insert(
        "version".to_string(),
        serde_json::Value::Number(target_version.into()),
    );
    if let Some(file) = value.as_object_mut() {
        file.insert(
            "format_version".to_string(),
            serde_json::Value::String("1.0.0".to_string()),
        );
    }
    serde_json::to_string(&value).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("the assistant's revision could not be stored: {e}"),
        )
    })
}
