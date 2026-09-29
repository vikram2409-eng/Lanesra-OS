//! AI Agent Platform v2, Phase 2: the risk classification
//! `policy_engine_service`'s Tool-Call Firewall evaluates every tool call
//! against. `default_risk_for` is a pure, no-DB classification of every
//! tool name this codebase's own catalogs (`chat_service::record_tools`/
//! `admin_tools`, plus a connector-derived name's `tool_source` prefix) can
//! ever produce - a handful of explicit, deliberately-chosen overrides for
//! the genuinely sensitive names, falling back to a name-prefix convention
//! (`list_*`/`get_*`/`*search_*` -> Read, `create_*`/`set_*` -> Write) for
//! everything else, so a *future* tool this file has never heard of still
//! gets a sane default instead of silently falling through as unclassified.
//! `ai_tool_registry` (migration `0061_agent_policy_engine.sql`) layers a
//! per-workspace, admin-editable override on top of that default - see
//! `effective_risk`.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::ai_tool_registry::{AiToolRegistryOverride, AiToolRegistryOverrideInput, RiskLevel};
use crate::repositories::ai_tool_registry_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// The genuinely sensitive names that don't fit the prefix convention
/// below, or would be misclassified by it (`create_record`/`update_record`
/// are ordinary business-data writes, not a workspace-configuration
/// change, so they're pulled down from the `create_*`/`update_*`
/// default rather than up).
fn hardcoded_override(tool_name: &str) -> Option<RiskLevel> {
    match tool_name {
        "create_record" | "update_record" => Some(RiskLevel::LowWrite),
        "archive_record" => Some(RiskLevel::Destructive),
        "create_user" | "create_api_client" | "create_connection" | "update_workspace_profile" | "run_ai_agent" | "run_ai_agent_pipeline" => {
            Some(RiskLevel::Privileged)
        }
        _ => None,
    }
}

/// `source` is `tool_source`'s own classification (`chat_service.rs`) -
/// `Some("connector_read")`/`Some("connector_write")` for a
/// connector-derived tool name, `Some("record")`/`Some("admin")` (or
/// `None`, for a Foundry-only name like `update_memory` this function is
/// never actually asked to classify - see `policy_engine_service`'s own
/// doc comment) for a native one.
pub fn default_risk_for(tool_name: &str, source: Option<&str>) -> RiskLevel {
    if let Some(risk) = hardcoded_override(tool_name) {
        return risk;
    }
    match source {
        Some("connector_read") => RiskLevel::Read,
        Some("connector_write") => RiskLevel::ExternalAction,
        _ => {
            if tool_name.starts_with("list_") || tool_name.starts_with("get_") || tool_name.contains("search_") {
                RiskLevel::Read
            } else {
                // create_*/set_*/update_* and anything this convention has
                // never seen before - a workspace-configuration change is
                // the safe middle default, neither silently trusted nor
                // over-alarmed.
                RiskLevel::Write
            }
        }
    }
}

/// The default above, unless this workspace has deliberately overridden
/// this exact tool name.
pub fn effective_risk(conn: &Connection, workspace_id: &str, tool_name: &str, source: Option<&str>) -> AppResult<RiskLevel> {
    if let Some(over) = ai_tool_registry_repo::get_by_tool_name(conn, workspace_id, tool_name)? {
        return Ok(over.risk_level);
    }
    Ok(default_risk_for(tool_name, source))
}

pub fn list_overrides(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AiToolRegistryOverride>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_tool_registry_repo::list(conn, workspace_id)?)
}

pub fn set_override(conn: &Connection, workspace_id: &str, input: &AiToolRegistryOverrideInput, actor_user_id: Option<&str>) -> AppResult<AiToolRegistryOverride> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_tool_registry_repo::set(conn, workspace_id, input, actor_user_id)?)
}

/// Reverts a tool back to its built-in default classification.
pub fn clear_override(conn: &Connection, workspace_id: &str, tool_name: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_tool_registry_repo::clear(conn, workspace_id, tool_name)?)
}
