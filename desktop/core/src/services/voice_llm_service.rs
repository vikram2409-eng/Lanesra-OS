//! Voice-First Mode: admin CRUD for the optional, workspace-wide LLM-backed
//! conversational fallback's settings (`voice_llm_settings`) - see
//! `voice_llm_planner_service`'s own doc comment for what actually uses
//! this once it's turned on. Administrator-only to view or change, same
//! "governance settings, not a per-user preference" gate
//! `voice_policy_service`'s own Voice Governance editor already uses.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::voice::{VoiceLlmSettings, VoiceLlmSettingsInput};
use crate::repositories::{ai_provider_repo, voice_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = crate::repositories::user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage Voice's conversational AI settings".into()));
    }
    Ok(())
}

/// A workspace that has never touched this setting reads back as
/// disabled/no-provider - the exact same behavior Voice Mode has always
/// had, not a row this function fabricates as "saved."
fn defaults(workspace_id: &str) -> VoiceLlmSettings {
    VoiceLlmSettings { workspace_id: workspace_id.into(), enabled: false, provider_id: None, updated_at: String::new(), updated_by: None }
}

pub fn get_settings(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<VoiceLlmSettings> {
    require_admin(conn, actor_user_id)?;
    Ok(voice_repo::get_llm_settings(conn, workspace_id)?.unwrap_or_else(|| defaults(workspace_id)))
}

pub fn upsert_settings(conn: &Connection, workspace_id: &str, input: &VoiceLlmSettingsInput, actor_user_id: Option<&str>) -> AppResult<VoiceLlmSettings> {
    require_admin(conn, actor_user_id)?;
    if let Some(provider_id) = &input.provider_id {
        let provider = ai_provider_repo::get(conn, provider_id)?.ok_or_else(|| AppError::Validation("Selected AI provider does not exist".into()))?;
        if provider.workspace_id != workspace_id {
            return Err(AppError::Validation("Selected AI provider does not exist".into()));
        }
        if !provider.is_active {
            return Err(AppError::Validation("Selected AI provider is deactivated".into()));
        }
    }
    Ok(voice_repo::upsert_llm_settings(conn, workspace_id, input, actor_user_id)?)
}
