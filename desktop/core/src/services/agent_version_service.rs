//! AI Agent Platform v2, Phase 1: the Draft -> Test -> Published ->
//! Deprecated -> Disabled lifecycle for `ai_agent_versions` - see
//! migration `0059_ai_agent_versioning.sql`'s own doc comment for why the
//! table exists and `ai_agent_repo::create_initial_version` for how every
//! agent already has a `v1` Published version before this service is ever
//! called. This file owns the state machine (the table itself enforces
//! nothing - same "service owns the state machine" shape
//! `ai_agent_pipeline`'s topology validation already established).
//!
//! Content edits (`update_draft`) are only ever allowed while a version is
//! `draft`/`test` - once `published` a version is a permanent, immutable
//! historical snapshot (what actually ran, forever), matching the doc
//! comment on `AiAgentVersion` itself. Publishing a version auto-
//! deprecates whatever version was previously Published for the same
//! agent, so "the current Published version" is always unambiguous.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::domain::{AppError, AppResult};
use crate::models::ai_agent::{AiAgentVersion, AiAgentVersionInput};
use crate::repositories::{ai_agent_repo, ai_agent_version_repo, ai_eval_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn get_agent_in_workspace(conn: &Connection, agent_id: &str, workspace_id: &str) -> AppResult<crate::models::ai_agent::AiAgentDefinition> {
    let agent = ai_agent_repo::get(conn, agent_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if agent.workspace_id != workspace_id {
        return Err(AppError::NotFound("Agent".into()));
    }
    Ok(agent)
}

fn get_version_for_agent(conn: &Connection, agent_id: &str, version_id: &str) -> AppResult<AiAgentVersion> {
    let version = ai_agent_version_repo::get(conn, version_id)?.ok_or_else(|| AppError::NotFound("Agent version".into()))?;
    if version.agent_id != agent_id {
        return Err(AppError::NotFound("Agent version".into()));
    }
    Ok(version)
}

fn validate_input(conn: &Connection, workspace_id: &str, input: &AiAgentVersionInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Agent name is required".into()));
    }
    if input.system_prompt.trim().is_empty() {
        return Err(AppError::Validation("Agent instructions are required".into()));
    }
    let mut connector_names: Option<Vec<String>> = None;
    for name in &input.action_names {
        match super::chat_service::tool_source(name) {
            Some("connector_read" | "connector_write") => {
                if connector_names.is_none() {
                    connector_names = Some(super::connector_tool_service::agent_tool_names(conn, workspace_id)?);
                }
                if !connector_names.as_ref().unwrap().contains(name) {
                    return Err(AppError::Validation(format!("'{name}' isn't currently available as an agent tool")));
                }
            }
            Some(_) => {}
            None => return Err(AppError::Validation(format!("'{name}' isn't a known action"))),
        }
    }
    if let Some(schema) = &input.output_schema {
        if !schema.is_object() {
            return Err(AppError::Validation("Output schema must be a JSON Schema object".into()));
        }
    }
    Ok(())
}

/// A new Draft version, seeded by the caller's input (typically a copy of
/// the current Published version's content, edited) - version numbers are
/// monotonically increasing per agent and never reused, even across a
/// Draft that's later disabled without ever publishing.
pub fn create_draft(conn: &Connection, agent_id: &str, workspace_id: &str, input: &AiAgentVersionInput, actor_user_id: Option<&str>) -> AppResult<AiAgentVersion> {
    require_admin(conn, actor_user_id)?;
    get_agent_in_workspace(conn, agent_id, workspace_id)?;
    validate_input(conn, workspace_id, input)?;
    let version_number = ai_agent_version_repo::next_version_number(conn, agent_id)?;
    Ok(ai_agent_version_repo::create_draft(conn, agent_id, version_number, input, actor_user_id)?)
}

pub fn list_versions(conn: &Connection, agent_id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AiAgentVersion>> {
    require_admin(conn, actor_user_id)?;
    get_agent_in_workspace(conn, agent_id, workspace_id)?;
    Ok(ai_agent_version_repo::list_by_agent(conn, agent_id)?)
}

pub fn get_version(conn: &Connection, agent_id: &str, workspace_id: &str, version_id: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentVersion> {
    require_admin(conn, actor_user_id)?;
    get_agent_in_workspace(conn, agent_id, workspace_id)?;
    get_version_for_agent(conn, agent_id, version_id)
}

/// A full content overwrite of a Draft/Test version - rejected once the
/// version has ever been Published (immutability - see this module's own
/// doc comment).
pub fn update_draft(conn: &Connection, agent_id: &str, workspace_id: &str, version_id: &str, input: &AiAgentVersionInput, actor_user_id: Option<&str>) -> AppResult<AiAgentVersion> {
    require_admin(conn, actor_user_id)?;
    get_agent_in_workspace(conn, agent_id, workspace_id)?;
    let version = get_version_for_agent(conn, agent_id, version_id)?;
    if version.status != "draft" && version.status != "test" {
        return Err(AppError::Validation(format!("Version {} is '{}' and can no longer be edited - it's an immutable historical snapshot once Published", version.version_number, version.status)));
    }
    validate_input(conn, workspace_id, input)?;
    Ok(ai_agent_version_repo::update_content(conn, version_id, input)?)
}

/// Every legal `(from, to)` status transition - anything not listed here
/// is rejected. Kept as one flat table rather than a match-per-status so
/// the whole lifecycle is visible at a glance.
const ALLOWED_TRANSITIONS: &[(&str, &str)] = &[
    ("draft", "test"),
    ("test", "draft"),
    ("test", "published"),
    ("draft", "disabled"),
    ("test", "disabled"),
    ("published", "deprecated"),
    ("deprecated", "disabled"),
    ("deprecated", "published"), // re-publishing a rolled-back version (e.g. undoing an accidental deprecate).
];

/// Moves `version_id` to `new_status`, validating the transition is legal
/// for its current status. Publishing auto-deprecates whatever version was
/// previously Published for this agent and repoints `ai_agents.
/// current_version_id` - so "the current Published version" is always
/// exactly one row, never two.
pub fn transition_status(conn: &Connection, agent_id: &str, workspace_id: &str, version_id: &str, new_status: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentVersion> {
    require_admin(conn, actor_user_id)?;
    let agent = get_agent_in_workspace(conn, agent_id, workspace_id)?;
    let version = get_version_for_agent(conn, agent_id, version_id)?;

    if !ALLOWED_TRANSITIONS.contains(&(version.status.as_str(), new_status)) {
        return Err(AppError::Validation(format!("Can't move version {} from '{}' to '{new_status}'", version.version_number, version.status)));
    }

    if new_status == "published" {
        // AI Agent Platform v2, Phase 6a (AI-AC-12): a policy_compliance
        // Eval Suite targeting this agent whose most recent run still has
        // a failure blocks publish outright - "zero critical policy
        // violations" from issue #171's own acceptance criteria. A suite
        // with no run yet, or any other evaluator type, never blocks -
        // opt-in governance, the same shape Phase 2's Policy Engine
        // itself uses (no policy configured, no gating).
        for suite in ai_eval_repo::list_suites(conn, workspace_id)? {
            if suite.target_type != "agent" || suite.target_id != agent_id || suite.evaluator_type != "policy_compliance" {
                continue;
            }
            if let Some(last_run) = ai_eval_repo::list_runs_for_suite(conn, &suite.id, 1)?.into_iter().next() {
                if last_run.failed_count > 0 {
                    return Err(AppError::Validation(format!(
                        "Can't publish: Eval Suite '{}' (policy_compliance) has a failing run - resolve the policy violation and re-run before publishing.",
                        suite.name
                    )));
                }
            }
        }
        if let Some(current_id) = &agent.current_version_id {
            if current_id != version_id {
                if let Some(current) = ai_agent_version_repo::get(conn, current_id)? {
                    if current.status == "published" {
                        ai_agent_version_repo::set_status(conn, current_id, "deprecated", None)?;
                    }
                }
            }
        }
        ai_agent_version_repo::set_current_version(conn, agent_id, version_id)?;
        return Ok(ai_agent_version_repo::set_status(conn, version_id, "published", Some(&now_iso()))?);
    }

    Ok(ai_agent_version_repo::set_status(conn, version_id, new_status, None)?)
}
