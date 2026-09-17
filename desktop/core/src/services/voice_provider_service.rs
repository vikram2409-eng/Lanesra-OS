//! Voice-First Mode, PR 1: read-side access to `voice_provider_profiles`
//! (spec §18's Speech Provider Abstraction), same shape as
//! `ai_provider_service.rs`'s own config-table pattern. PR 1 ships exactly
//! one real adapter per workspace (`web_speech`, seeded by the 0057
//! migration) - its "health" is whatever the browser itself reports
//! (`SpeechRecognition` support), which is inherently client-side and
//! per-viewer, not something this server can meaningfully probe; `notes`
//! on `health_check` says so rather than faking a check that doesn't exist.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::voice::VoiceProviderProfile;
use crate::repositories::voice_repo;

pub fn list_providers(conn: &Connection, workspace_id: &str) -> AppResult<Vec<VoiceProviderProfile>> {
    Ok(voice_repo::list_provider_profiles(conn, workspace_id)?)
}

/// Called once from `workspace_service::first_run_setup` for every new
/// workspace - see `voice_policy_service::ensure_workspace_default`'s doc
/// comment for why this is needed at all (migration 0057's own bootstrap
/// insert only reaches workspaces that already existed when it ran).
pub fn ensure_defaults(conn: &Connection, workspace_id: &str) -> AppResult<()> {
    Ok(voice_repo::ensure_default_provider_profile(conn, workspace_id)?)
}

/// Records that a provider's status was checked - for `web_speech` this is
/// always "browser-dependent" rather than a real network probe, since
/// speech recognition support lives entirely in the viewer's own browser,
/// not on this server.
pub fn health_check(conn: &Connection, workspace_id: &str, provider_id: &str) -> AppResult<VoiceProviderProfile> {
    let providers = voice_repo::list_provider_profiles(conn, workspace_id)?;
    let provider = providers.into_iter().find(|p| p.id == provider_id).ok_or_else(|| crate::domain::AppError::NotFound("Voice provider".into()))?;
    let status = match provider.kind.as_str() {
        "web_speech" => "browser-dependent (checked client-side, not by this server)",
        _ => "not configured",
    };
    voice_repo::set_provider_health(conn, provider_id, status)?;
    let updated = voice_repo::list_provider_profiles(conn, workspace_id)?.into_iter().find(|p| p.id == provider_id).expect("just updated");
    Ok(updated)
}
