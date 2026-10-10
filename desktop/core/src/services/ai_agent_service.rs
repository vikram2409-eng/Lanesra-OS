//! AI & Agentic Layer, Phase 6: CRUD for the AI Agent Foundry's Agents and
//! Skills - Administrator-only to create/update/deactivate, the same
//! admin-builder shape `custom_object_service.rs` already established.
//! Running an agent (the tool-calling loop, memory, skill lookup,
//! delegation) lives in `chat_service.rs` instead, which already owns the
//! tool catalogs and the provider-call plumbing this needs - see that
//! module's own doc comment. `chat_service::tool_source`/
//! `agent_requires_admin` are the one piece of shared vocabulary between
//! the two files.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ai::{AiAgentModelRouting, AiTokenUsageSummary};
use crate::models::ai_agent::{AiAgentDefinition, AiAgentInput, AiSkill, AiSkillInput};
use crate::repositories::{ai_agent_repo, ai_provider_repo, ai_token_usage_repo, audit_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_action_names(conn: &Connection, workspace_id: &str, action_names: &[String]) -> AppResult<()> {
    // Connector-derived names classify by prefix alone (see
    // `chat_service::tool_source`'s own doc comment), but that doesn't
    // confirm the connector/action they name still exists and is
    // currently enabled for agent use - check those against the real,
    // workspace-scoped catalog here rather than let a stale/bogus name
    // through to be silently dropped later by `chat_service::agent_tools`.
    let mut connector_names: Option<Vec<String>> = None;
    let mut mcp_names: Option<Vec<String>> = None;
    for name in action_names {
        match super::chat_service::tool_source(name) {
            Some("connector_read" | "connector_write") => {
                if connector_names.is_none() {
                    connector_names = Some(super::connector_tool_service::agent_tool_names(conn, workspace_id)?);
                }
                if !connector_names.as_ref().unwrap().contains(name) {
                    return Err(AppError::Validation(format!("'{name}' isn't currently available as an agent tool")));
                }
            }
            Some("mcp_read" | "mcp_write") => {
                if mcp_names.is_none() {
                    mcp_names = Some(super::mcp_client_service::agent_tool_names(conn, workspace_id)?);
                }
                if !mcp_names.as_ref().unwrap().contains(name) {
                    return Err(AppError::Validation(format!("'{name}' isn't currently available as an agent tool")));
                }
            }
            Some(_) => {}
            None => return Err(AppError::Validation(format!("'{name}' isn't a known action"))),
        }
    }
    Ok(())
}

fn validate_agent_input(conn: &Connection, workspace_id: &str, id: Option<&str>, input: &AiAgentInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Agent name is required".into()));
    }
    if input.system_prompt.trim().is_empty() {
        return Err(AppError::Validation("Agent instructions are required".into()));
    }
    validate_action_names(conn, workspace_id, &input.action_names)?;
    // A direct self-reference is a trivial, pointless cycle worth
    // rejecting outright at save time - anything indirect is left to the
    // runtime delegation-depth guard, same reasoning as
    // migration 0038's own comment on why this table has no cycle check.
    if let Some(id) = id {
        if input.delegate_agent_ids.iter().any(|d| d == id) {
            return Err(AppError::Validation("An agent can't delegate to itself".into()));
        }
    }
    for delegate_id in &input.delegate_agent_ids {
        let delegate = ai_agent_repo::get(conn, delegate_id)?.ok_or_else(|| AppError::Validation("Selected delegate agent does not exist".into()))?;
        if delegate.workspace_id != workspace_id || !delegate.is_active {
            return Err(AppError::Validation("Selected delegate agent does not exist".into()));
        }
    }
    for skill_id in &input.skill_ids {
        let skill = ai_agent_repo::get_skill(conn, skill_id)?.ok_or_else(|| AppError::Validation("Selected skill does not exist".into()))?;
        if skill.workspace_id != workspace_id || !skill.is_active {
            return Err(AppError::Validation("Selected skill does not exist".into()));
        }
    }
    Ok(())
}

/// FND-01 System Graph: an agent DELEGATES_TO each agent in its own
/// `delegate_agent_ids` - the one already-reliable cross-agent reference
/// this v1 slice syncs (tool/skill/model-routing references are a
/// documented future gap, not modeled yet).
fn sync_graph_node(conn: &Connection, workspace_id: &str, agent: &AiAgentDefinition) -> AppResult<()> {
    let edges: Vec<_> = agent
        .delegate_agent_ids
        .iter()
        .map(|target_id| crate::models::system_graph::SystemEdgeTarget { edge_type: "delegates_to".into(), to_node_type: "ai_agent".into(), to_component_id: target_id.clone() })
        .collect();
    super::system_graph_service::sync_node(conn, workspace_id, "ai_agent", &agent.id, &agent.name, "{}", &edges)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &AiAgentInput, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_agent_input(conn, workspace_id, None, input)?;
    let id = crate::domain::ids::new_uuid();
    let created = ai_agent_repo::create(conn, &id, workspace_id, input, actor_user_id)?;
    // AI Agent Platform v2, Phase 1: gives this agent its v1 Published
    // version so it's never left versionless - see
    // `ai_agent_repo::create_initial_version`'s own doc comment. This sets
    // `current_version_id` on the row `created` was already fetched from,
    // so it's re-fetched afterward - otherwise every freshly created
    // agent would be handed back to its caller (and to the frontend that
    // just rendered it) with a stale `current_version_id: None`.
    ai_agent_repo::create_initial_version(conn, &created, actor_user_id)?;
    // Phase 7c: makes this agent a Solution-addable component the moment
    // it's created, same "every component-creating service function tags
    // itself to 'local'" convention `custom_object_service::create`/
    // `business_rule_service::create_rule` already follow - a package
    // install's own retag pass corrects this afterward for an agent that
    // came from a package instead of an admin's own hand.
    super::solution_component_service::tag_local(conn, workspace_id, "ai_agent", &created.id, actor_user_id)?;
    let hydrated = ai_agent_repo::get(conn, &created.id)?.expect("just created");
    sync_graph_node(conn, workspace_id, &hydrated)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("ai_agent"), Some(&created.id), &format!("Created AI agent '{}'", created.name), None)?;
    Ok(hydrated)
}

pub fn update(conn: &Connection, id: &str, workspace_id: &str, input: &AiAgentInput, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    validate_agent_input(conn, workspace_id, Some(id), input)?;
    let updated = ai_agent_repo::update(conn, id, input, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &updated)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("ai_agent"), Some(id), &format!("Updated AI agent '{}'", updated.name), None)?;
    Ok(updated)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<AiAgentDefinition>> {
    Ok(ai_agent_repo::get(conn, id)?)
}

/// Any authenticated user can list active agents - the same "the form
/// needs these to render" reasoning `business_rule_service::list_rules`
/// already documents. Whether a given agent is actually usable by this
/// caller is computed by the caller from `agent_requires_admin`, not
/// filtered here - the frontend hides what it can't use, and
/// `send_agent_message` gates for real regardless (defense in depth).
pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<AiAgentDefinition>> {
    Ok(ai_agent_repo::list(conn, workspace_id, active_only)?)
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    Ok(ai_agent_repo::set_active(conn, id, is_active, actor_user_id)?)
}

/// An admin directly editing an agent's memory from its edit form -
/// gated, unlike the agent's own `update_memory` tool call (dispatched
/// straight through `chat_service`/`ai_agent_repo::update_memory`, no
/// admin check - the agent revising its own memory during an ordinary,
/// possibly non-admin-gated conversation must keep working regardless of
/// who it's chatting with).
pub fn set_memory(conn: &Connection, id: &str, memory_md: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    ai_agent_repo::update_memory(conn, id, memory_md, actor_user_id.unwrap_or("admin"))?;
    Ok(ai_agent_repo::get(conn, id)?.expect("just updated"))
}

/// Phase 7c: an agent's operational-boundary statement - its own admin
/// action, same "not part of the main create/update form payload" shape
/// `set_memory` above already established.
pub fn set_guardrails(conn: &Connection, id: &str, guardrails_md: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    ai_agent_repo::set_guardrails(conn, id, guardrails_md)?;
    Ok(ai_agent_repo::get(conn, id)?.expect("just updated"))
}

/// Phase 7b: most-recent-first history of this agent's `memory_md`
/// changes - read-only, Administrator only (same visibility as the rest
/// of an agent's configuration).
pub fn list_memory_history(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<crate::models::ai_agent::AiAgentMemorySnapshot>> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    Ok(ai_agent_repo::list_memory_history(conn, id)?)
}

fn validate_routing(conn: &Connection, workspace_id: &str, routing: &AiAgentModelRouting) -> AppResult<()> {
    for provider_id in [&routing.primary_provider_id, &routing.fallback_provider_id, &routing.local_fallback_provider_id].into_iter().flatten() {
        let provider = ai_provider_repo::get(conn, provider_id)?.ok_or_else(|| AppError::Validation("Selected AI provider does not exist".into()))?;
        if provider.workspace_id != workspace_id {
            return Err(AppError::Validation("Selected AI provider does not exist".into()));
        }
    }
    for class in &routing.force_air_gapped_for {
        if !super::dlp_service::CLASSES.contains(&class.as_str()) {
            return Err(AppError::Validation(format!("'{class}' isn't a recognized sensitive-data class")));
        }
    }
    if let Some(t) = routing.temperature {
        if !(0.0..=2.0).contains(&t) {
            return Err(AppError::Validation("Temperature must be between 0 and 2".into()));
        }
    }
    Ok(())
}

/// Phase 7a: an agent's Gateway routing policy - its own admin action,
/// not part of the main create/update form payload, the same "own action,
/// not folded into the main form" shape `set_memory` above already
/// established. `routing: None` clears the policy entirely, returning
/// this agent to the plain pre-7a workspace-default behavior - see
/// `ai_agent_repo::set_routing`'s own "full overwrite" doc comment.
pub fn set_model_routing(conn: &Connection, id: &str, workspace_id: &str, routing: Option<AiAgentModelRouting>, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if let Some(r) = &routing {
        validate_routing(conn, workspace_id, r)?;
    }
    ai_agent_repo::set_routing(conn, id, routing.as_ref())?;
    Ok(ai_agent_repo::get(conn, id)?.expect("just updated"))
}

/// Agent Access Governance (issue #245): an agent's own bound identity
/// for record-write access-control purposes - its own separate admin
/// action, the same shape `set_model_routing` above already established.
/// `None` clears it (today's unscoped behavior for this agent). A `Some`
/// value must be an active user of this workspace - the same bar a
/// manual owner reassignment holds itself to.
pub fn set_acts_as(conn: &Connection, id: &str, workspace_id: &str, acts_as_user_id: Option<String>, actor_user_id: Option<&str>) -> AppResult<AiAgentDefinition> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    if let Some(user_id) = &acts_as_user_id {
        let users = super::user_service::list(conn, workspace_id)?;
        let user = users.iter().find(|u| &u.id == user_id).ok_or_else(|| AppError::Validation("Selected user does not exist".into()))?;
        if !user.is_active {
            return Err(AppError::Validation("Selected user is not active".into()));
        }
    }
    ai_agent_repo::set_acts_as(conn, id, acts_as_user_id.as_deref())?;
    Ok(ai_agent_repo::get(conn, id)?.expect("just updated"))
}

/// The Agent tier's real usage-vs-budget snapshot - shown alongside this
/// agent's Model Routing settings, same "any authenticated user can see
/// the numbers" reasoning `ai_service::token_usage_today` already
/// documents for the System tier.
pub fn token_usage_today(conn: &Connection, id: &str, workspace_id: &str) -> AppResult<AiTokenUsageSummary> {
    let agent = ai_agent_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
    let (input_tokens, output_tokens) = ai_token_usage_repo::today_totals_for_agent(conn, workspace_id, id)?;
    let daily_token_budget = agent.model_routing.and_then(|r| r.daily_token_budget);
    Ok(AiTokenUsageSummary { today_input_tokens: input_tokens, today_output_tokens: output_tokens, daily_token_budget })
}

// --- Skills -----------------------------------------------------------

fn validate_skill_input(input: &AiSkillInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Skill name is required".into()));
    }
    if input.description.trim().is_empty() {
        return Err(AppError::Validation("Skill description is required - it's what the model sees before deciding to use it".into()));
    }
    if input.instructions_md.trim().is_empty() {
        return Err(AppError::Validation("Skill instructions are required".into()));
    }
    Ok(())
}

pub fn create_skill(conn: &Connection, workspace_id: &str, input: &AiSkillInput, actor_user_id: Option<&str>) -> AppResult<AiSkill> {
    require_admin(conn, actor_user_id)?;
    validate_skill_input(input)?;
    let id = crate::domain::ids::new_uuid();
    let created = ai_agent_repo::create_skill(conn, &id, workspace_id, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "ai_skill", &created.id, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("ai_skill"), Some(&created.id), &format!("Created AI skill '{}'", created.name), None)?;
    Ok(created)
}

pub fn update_skill(conn: &Connection, id: &str, input: &AiSkillInput, actor_user_id: Option<&str>) -> AppResult<AiSkill> {
    require_admin(conn, actor_user_id)?;
    let existing = ai_agent_repo::get_skill(conn, id)?.ok_or_else(|| AppError::NotFound("Skill".into()))?;
    validate_skill_input(input)?;
    let updated = ai_agent_repo::update_skill(conn, id, input, actor_user_id)?;
    audit_repo::record(conn, &existing.workspace_id, actor_user_id, "update", Some("ai_skill"), Some(id), &format!("Updated AI skill '{}'", updated.name), None)?;
    Ok(updated)
}

pub fn list_skills(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<AiSkill>> {
    Ok(ai_agent_repo::list_skills(conn, workspace_id, active_only)?)
}

pub fn set_skill_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<AiSkill> {
    require_admin(conn, actor_user_id)?;
    ai_agent_repo::get_skill(conn, id)?.ok_or_else(|| AppError::NotFound("Skill".into()))?;
    Ok(ai_agent_repo::set_skill_active(conn, id, is_active, actor_user_id)?)
}
