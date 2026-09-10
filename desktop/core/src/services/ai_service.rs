//! AI & Agentic Layer, Phase 1: the bring-your-own-LLM-key foundation.
//!
//! Lanesra has no SaaS billing surface to meter inference through the way
//! a hosted "agentic CRM" competitor can - there's no subscription, no
//! seats, no usage plan. The answer here is the same shape every other
//! Lanesra feature already takes: a workspace supplies its own provider
//! key from a new Admin -> AI screen, Lanesra itself never resells,
//! proxies or bills for inference, and the key is stored exactly like any
//! other secret in this codebase - `integration_secrets` +
//! `secret_service::encrypt`/`decrypt` (reused directly, not a second
//! secret store), never returned by any read command (`AiSettings` only
//! ever exposes `has_key`, the same convention
//! `models::integration::Connection::has_secret` already established).
//!
//! Two provider modes for v1, matching what's realistically testable
//! without a live third-party account this environment doesn't have:
//! - `"anthropic"` - a fixed default base URL (overridable, e.g. to point
//!   at a self-hosted proxy in front of the real API - also what makes
//!   this testable against a local stub rather than a live endpoint), a
//!   minimal `max_tokens: 1` `messages.create` call as the connectivity
//!   test. There's no free "just validate this key" endpoint on
//!   Anthropic's API, so this is the standard way to prove a key works -
//!   stated plainly rather than pretending it's free.
//! - `"openai_compatible"` - an admin-supplied base URL (covers OpenAI
//!   itself, and any locally-hosted OpenAI-compatible server - Ollama's
//!   compat layer, vLLM, LM Studio - for a fully offline setup), a
//!   `GET {base_url}/models` call as the test, the standard
//!   OpenAI-compatible convention every one of those servers implements
//!   for free (no generation cost).
//!
//! Deliberately **not** built here: any actual chat-completion wrapper.
//! This phase only proves a key works and stores it - the later
//! Agent Actions item (meeting-prep briefings, follow-up capture, record
//! hygiene, natural-language reporting) is what makes real calls against
//! it, once the Activity Timeline item also exists for it to draw on.

use std::time::{Duration, Instant};

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::ai::{AiSettings, AiSettingsInput, AiTestResult, AI_PROVIDERS};
use crate::repositories::{ai_settings_repo, integration_secret_repo};

const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-haiku-4-5-20251001";
const ANTHROPIC_VERSION: &str = "2023-06-01";

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}

/// Read-only - lazily creates the default (`unconfigured`) row on first
/// call, same as `integration_log_service::get_settings` does for
/// `integration_settings`. Never admin-gated on read: any signed-in user
/// can see whether AI is configured (not the key itself), matching how
/// every other Integration Hub settings read works.
pub fn get_settings(conn: &Connection, workspace_id: &str) -> AppResult<AiSettings> {
    Ok(ai_settings_repo::ensure_default(conn, workspace_id)?)
}

/// Admin-only. Setting a new `api_key` rotates the existing stored secret
/// in place (via `integration_secret_repo::rotate`) rather than orphaning
/// it, or creates one for the first time - see `AiSettingsInput::api_key`'s
/// own doc comment for the "leave blank to keep" convention this follows.
/// Any settings change resets `status` back to `unconfigured`: a test
/// result against the old configuration isn't meaningful evidence about
/// the new one.
pub fn save_settings(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], input: &AiSettingsInput, actor_user_id: Option<&str>) -> AppResult<AiSettings> {
    require_admin(conn, actor_user_id)?;
    if !AI_PROVIDERS.contains(&input.provider.as_str()) {
        return Err(AppError::Validation(format!("Unknown AI provider '{}'", input.provider)));
    }
    if input.provider == "openai_compatible" && input.base_url.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::Validation("An OpenAI-compatible provider needs a base URL".into()));
    }
    ai_settings_repo::ensure_default(conn, workspace_id)?;
    let existing_secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?;
    let secret_id = match input.api_key.as_deref().filter(|k| !k.is_empty()) {
        Some(key) => {
            let (ciphertext, nonce) = super::secret_service::encrypt(master_key, key)?;
            match &existing_secret_id {
                Some(id) => {
                    integration_secret_repo::rotate(conn, id, &ciphertext, &nonce)?;
                    Some(id.clone())
                }
                None => {
                    let id = new_uuid();
                    integration_secret_repo::insert(conn, &id, workspace_id, "AI provider API key", &ciphertext, &nonce, actor_user_id)?;
                    Some(id)
                }
            }
        }
        None => existing_secret_id,
    };
    Ok(ai_settings_repo::update(
        conn,
        workspace_id,
        &input.provider,
        input.base_url.as_deref().filter(|u| !u.is_empty()),
        &input.model,
        secret_id.as_deref(),
        actor_user_id,
    )?)
}

/// Admin-only. Makes a real outbound call proving the currently-stored
/// key actually works, and persists the result. Errors (no key configured
/// at all) are distinct from a "failed" test result (a key that's
/// configured but rejected, or a provider that's unreachable) - the
/// former means there's nothing to test yet, the latter is a real,
/// recorded outcome, matching `connection_service::test_connection`'s own
/// `Err` vs. `ok: false` distinction.
pub async fn test_key(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor_user_id: Option<&str>) -> AppResult<AiTestResult> {
    require_admin(conn, actor_user_id)?;
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    let secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?.ok_or_else(|| AppError::Validation("Configure an API key before testing it".into()))?;
    let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
    let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;

    let started = Instant::now();
    let result = match settings.provider.as_str() {
        "anthropic" => test_anthropic(settings.base_url.as_deref(), &settings.model, &api_key).await,
        "openai_compatible" => test_openai_compatible(settings.base_url.as_deref(), &api_key).await,
        other => Err(AppError::Validation(format!("No test implemented for provider '{other}'"))),
    };
    let latency_ms = started.elapsed().as_millis() as u64;

    let test_result = match result {
        Ok(message) => AiTestResult { ok: true, latency_ms, message },
        Err(e) => AiTestResult { ok: false, latency_ms, message: e.to_string() },
    };
    ai_settings_repo::set_test_result(conn, workspace_id, if test_result.ok { "connected" } else { "failed" }, &test_result.message)?;
    Ok(test_result)
}

async fn test_anthropic(base_url: Option<&str>, model: &str, api_key: &str) -> AppResult<String> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_ANTHROPIC_BASE_URL);
    let model = if model.trim().is_empty() { DEFAULT_ANTHROPIC_MODEL } else { model };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({ "model": model, "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}] });
    let response = client
        .post(format!("{}/v1/messages", base.trim_end_matches('/')))
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Key valid - reached {base} (HTTP {status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))))
    }
}

async fn test_openai_compatible(base_url: Option<&str>, api_key: &str) -> AppResult<String> {
    let base = base_url
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| AppError::Validation("An OpenAI-compatible provider needs a base URL".into()))?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let response = client
        .get(format!("{}/models", base.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Key valid - reached {base} (HTTP {status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))))
    }
}
