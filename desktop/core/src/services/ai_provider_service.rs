//! AI & Agentic Layer, Phase 7a: admin-gated CRUD over `ai_providers` -
//! the named, multi-row sibling of `ai_service`'s single `ai_settings`
//! row (Admin -> LLM & MCP -> Gateway -> Providers). Secret handling
//! mirrors `ai_service::save_settings` exactly: a blank `api_key` on
//! update keeps whatever's already stored (rotated in place), a non-blank
//! one creates the secret on first save or rotates it on every later one.

use std::time::Instant;

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::ai::{AiProvider, AiProviderInput, AiTestResult, AI_PROVIDERS};
use crate::repositories::{ai_provider_repo, integration_secret_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate(input: &AiProviderInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("A provider connection needs a name".into()));
    }
    if !AI_PROVIDERS.contains(&input.provider.as_str()) {
        return Err(AppError::Validation(format!("Unknown AI provider '{}'", input.provider)));
    }
    if input.provider == "openai_compatible" && input.base_url.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::Validation("An OpenAI-compatible provider needs a base URL".into()));
    }
    Ok(())
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<AiProvider>> {
    Ok(ai_provider_repo::list(conn, workspace_id, active_only)?)
}

pub fn create(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], input: &AiProviderInput, actor_user_id: Option<&str>) -> AppResult<AiProvider> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    let secret_id = match input.api_key.as_deref().filter(|k| !k.is_empty()) {
        Some(key) => {
            let (ciphertext, nonce) = super::secret_service::encrypt(master_key, key)?;
            let id = new_uuid();
            integration_secret_repo::insert(conn, &id, workspace_id, &format!("AI provider key ({})", input.name), &ciphertext, &nonce, actor_user_id)?;
            Some(id)
        }
        None => None,
    };
    let id = new_uuid();
    Ok(ai_provider_repo::create(conn, &id, workspace_id, input, secret_id.as_deref(), actor_user_id)?)
}

pub fn update(conn: &Connection, id: &str, workspace_id: &str, master_key: &[u8; 32], input: &AiProviderInput, actor_user_id: Option<&str>) -> AppResult<AiProvider> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    let existing_secret_id = ai_provider_repo::get_secret_id(conn, id)?;
    let secret_id = match input.api_key.as_deref().filter(|k| !k.is_empty()) {
        Some(key) => {
            let (ciphertext, nonce) = super::secret_service::encrypt(master_key, key)?;
            match &existing_secret_id {
                Some(sid) => {
                    integration_secret_repo::rotate(conn, sid, &ciphertext, &nonce)?;
                    Some(sid.clone())
                }
                None => {
                    let sid = new_uuid();
                    integration_secret_repo::insert(conn, &sid, workspace_id, &format!("AI provider key ({})", input.name), &ciphertext, &nonce, actor_user_id)?;
                    Some(sid)
                }
            }
        }
        None => existing_secret_id,
    };
    Ok(ai_provider_repo::update(conn, id, input, secret_id.as_deref(), actor_user_id)?)
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<AiProvider> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_provider_repo::set_active(conn, id, is_active, actor_user_id)?)
}

/// The one genuinely-async operation on this model - a real outbound
/// connectivity probe, same reason `ai_service::test_key` is its own
/// admin-only async route rather than a plain dispatch/Tauri command -
/// see `server::admin_actions`'s own doc comment on why.
pub async fn test_key(conn: &Connection, id: &str, master_key: &[u8; 32], actor_user_id: Option<&str>) -> AppResult<AiTestResult> {
    require_admin(conn, actor_user_id)?;
    let provider = ai_provider_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("AI provider".into()))?;
    let secret_id = ai_provider_repo::get_secret_id(conn, id)?.ok_or_else(|| AppError::Validation("Configure an API key before testing it".into()))?;
    let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
    let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;

    let started = Instant::now();
    let result = super::ai_service::test_provider(&provider.provider, provider.base_url.as_deref(), &provider.model, &api_key).await;
    let latency_ms = started.elapsed().as_millis() as u64;
    Ok(match result {
        Ok(message) => AiTestResult { ok: true, latency_ms, message },
        Err(e) => AiTestResult { ok: false, latency_ms, message: e.to_string() },
    })
}
