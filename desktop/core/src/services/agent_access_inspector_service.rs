//! Agent Access Governance (issue #245): the Agent Access Inspector - see
//! `models::agent_access_inspection`'s own doc comment for what this
//! traces and why. Administrator-only, same visibility as the rest of an
//! agent's configuration (`ai_agent_service::set_memory` etc.).

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::access_role::Capability;
use crate::models::agent_access_inspection::AgentAccessInspection;
use crate::repositories::{ai_agent_policy_repo, ai_agent_repo};

use super::{access_service, chat_service, policy_engine_service, user_service};

#[allow(clippy::too_many_arguments)]
pub fn inspect(
    conn: &Connection,
    workspace_id: &str,
    agent_id: &str,
    tool_name: &str,
    object_key: Option<&str>,
    record_id: Option<&str>,
    simulate_as_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<AgentAccessInspection> {
    user_service::require_admin(conn, actor_user_id)?;
    let agent = ai_agent_repo::get(conn, agent_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;

    let tool_in_action_list = agent.action_names.iter().any(|n| n == tool_name);
    let source = chat_service::tool_source(tool_name);
    let policy_decision = policy_engine_service::evaluate(conn, workspace_id, Some(agent_id), tool_name, source)?;
    let policy_source = if ai_agent_policy_repo::get(conn, workspace_id, Some(agent_id))?.is_some() {
        "agent-specific policy".to_string()
    } else if ai_agent_policy_repo::get(conn, workspace_id, None)?.is_some() {
        "workspace default policy".to_string()
    } else {
        "no policy configured (defaults to Allow)".to_string()
    };

    let record_access_enforced = policy_engine_service::resolve_policy(conn, workspace_id, Some(agent_id))?.map(|p| p.enforce_record_access).unwrap_or(false);
    let acting_as_user_id = if record_access_enforced {
        agent.acts_as_user_id.clone().or_else(|| simulate_as_user_id.map(String::from))
    } else {
        None
    };

    let record_access = match (&acting_as_user_id, object_key) {
        (Some(uid), Some(key)) => match tool_name {
            "create_record" => Some(access_service::explain_access(conn, uid, key, Capability::Create, None)?),
            "update_record" => Some(access_service::explain_access(conn, uid, key, Capability::Update, record_id)?),
            "archive_record" => Some(access_service::explain_access(conn, uid, key, Capability::Delete, record_id)?),
            _ => None,
        },
        _ => None,
    };

    Ok(AgentAccessInspection {
        agent_id: agent.id.clone(),
        agent_name: agent.name.clone(),
        tool_name: tool_name.to_string(),
        tool_in_action_list,
        policy_decision,
        policy_source,
        record_access_enforced,
        acting_as_user_id,
        record_access,
    })
}
