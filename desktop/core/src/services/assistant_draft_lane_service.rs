//! AI Agent Platform v2, Phase 7: describe a process - the Admin Assistant
//! drafts a lane (WP-0715, extended to a full lane by WP-0723, REQ-LANE-022).
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
//! The draft the model may produce is a full lane (WP-0723): form and
//! check steps as before, plus record steps, approval steps (writes plus
//! approved/rejected branches) and agent steps. Two boundaries hold it
//! to what the model can author without guessing, enforced here against
//! the live workspace data - not by trusting the prompt - in
//! `validate_assistant_definition` (shared with the edit service):
//! every write's Action Type must be an active adopted Action Type on
//! the lane's object (the P-12 rule, read from the same catalog the
//! publish checks read), and every agent step must name its agent. A
//! write the description asks for but no adopted Action Type covers is
//! refused with the named reason, never guessed: writes beyond the
//! adopted set need the WP-0705 follow-up (Claude's track). The prompt
//! itself only authors the one adopted write whose inputs the model can
//! write without guessing parameter keys - `<object>.set_status` with
//! `{"status": ...}` - so in practice a draft's writes are set_status
//! writes; the service check stays the catalog check, so an adopted
//! reassign_owner or set_fields write the model does produce is
//! accepted rather than refused on a hardcoded list.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::chat::ChatMessage;
use crate::models::process_lane::{parse_definition, LaneDefinitionFile, ProcessLane};
use crate::services::{action_type_service, ai_service, process_lane_service};

fn classified_validation(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, code, message)
}

/// The Assistant's authoring boundaries (WP-0723 AC-1), checked against
/// the live workspace before anything is stored: every write names an
/// active adopted Action Type on the lane's object, and every agent step
/// (and every step whose assistance is not a person doing the work)
/// names its agent. Refusals name the step and the reason, and store
/// nothing. This is the P-12 rule, not a hardcoded list: the catalog is
/// the same `action_type_definitions` table the publish checks read, so
/// adopting a type (builder, WP-0705 follow-up) is what unblocks a
/// write, and no agent is ever looked up by existence - the lane schema
/// names agents loosely (an id or a name), and no publish check
/// validates an agent's existence today, so an existence gate here
/// would refuse drafts the builder accepts.
pub fn validate_assistant_definition(
    conn: &Connection,
    workspace_id: &str,
    file: &LaneDefinitionFile,
) -> AppResult<()> {
    for stage in &file.lane.stages {
        for step in &stage.steps {
            if let Some(writes) = &step.writes {
                for write in writes {
                    let adopted = action_type_service::get_by_name(
                        conn,
                        workspace_id,
                        &file.lane.object_key,
                        &write.action_type,
                    )?
                    .map(|d| d.is_active)
                    .unwrap_or(false);
                    if !adopted {
                        return Err(classified_validation(
                            "validation.arguments",
                            format!(
                                "step '{}' uses Action Type '{}' - not in the adopted catalog for '{}' (adopt it first, or leave the write out of the draft; writes beyond the adopted set need the WP-0705 follow-up)",
                                step.id, write.action_type, file.lane.object_key
                            ),
                        ));
                    }
                }
            }
            let agent = step
                .assistance
                .as_ref()
                .and_then(|a| a.agent.as_deref())
                .map(str::trim)
                .filter(|a| !a.is_empty());
            if step.kind == "agent" && agent.is_none() {
                return Err(classified_validation(
                    "validation.arguments",
                    format!(
                        "step '{}' is an agent step but names no agent - name the agent in the description",
                        step.id
                    ),
                ));
            }
            if let Some(assistance) = &step.assistance {
                if assistance.autonomy != "person" && agent.is_none() {
                    return Err(classified_validation(
                        "validation.arguments",
                        format!(
                            "step '{}' needs an agent for autonomy '{}' - name the agent in the description",
                            step.id, assistance.autonomy
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Draft a lane from a plain-language description of a business process.
///
/// `description` is the process in the person's own words;
/// `object_key` is the object the lane runs on. The result is always a
/// draft, always valid against the lane schema, and never published.
/// Administrator-only: drafting creates a lane, and lane creation is an
/// admin action (`import_lane_definition` enforces the same gate).
///
/// The draft is a full lane (WP-0723): form, check, record, approval
/// and agent steps where the description states them. Before anything is
/// stored, `validate_assistant_definition` checks the reply against the
/// live adopted Action Type catalog and the named-agent boundary, so a
/// write no adopted type covers - or an agent step with no named agent -
/// is refused with the named reason and stores nothing.
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
    let file = parse_definition(&definition_json)?;
    validate_assistant_definition(conn, workspace_id, &file)?;
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
