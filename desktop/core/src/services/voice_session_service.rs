//! Voice-First Mode, PR 1: Voice PIN setup and the Voice Session state
//! machine (spec §3/§4). A Voice Session is a secondary, short-lived unlock
//! layered on top of an already-authenticated Lanesra session - it must
//! never be treated as a replacement for primary authentication (spec's own
//! "Security boundary" callout), and it automatically has no meaning once
//! the primary session ends (nothing here extends its own life past the
//! primary session's, since every Tauri command already requires a live
//! `actor_user_id` to call any of this in the first place).

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::voice::{SetVoicePinInput, VoicePreferencesInput, VoiceSession, VoiceUserSettings};
use crate::repositories::{audit_repo, user_repo, voice_repo};
use crate::services::{auth_service, voice_policy_service};

/// After this many wrong PINs in a row, the PIN locks out for
/// `PIN_LOCKOUT_MINUTES` and the user must fall back to full Lanesra
/// reauthentication to keep trying sooner (spec §4 / VOICE-AC-02) - there is
/// no "unlock the lock early" path here on purpose.
const MAX_PIN_ATTEMPTS: i64 = 5;
const PIN_LOCKOUT_MINUTES: i64 = 15;

fn minutes_from_now(minutes: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::minutes(minutes)).to_rfc3339()
}

pub fn get_settings(conn: &Connection, user_id: &str) -> AppResult<VoiceUserSettings> {
    Ok(voice_repo::to_safe_settings(voice_repo::ensure_settings_row(conn, user_id)?))
}

/// Sets (or resets) the user's Voice PIN - always requires the user has
/// already passed primary authentication to even call this (spec: "User can
/// reset the Voice PIN only after normal authentication"), which is already
/// true of every Tauri command's `actor_user_id`. Clears any prior lockout,
/// matching how changing a password also resets its own failure state.
pub fn set_pin(conn: &Connection, user_id: &str, input: &SetVoicePinInput) -> AppResult<VoiceUserSettings> {
    if input.pin.len() != 4 || !input.pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation("Voice PIN must be exactly 4 digits".into()));
    }
    voice_repo::ensure_settings_row(conn, user_id)?;
    let hash = auth_service::hash_password(&input.pin)?;
    voice_repo::set_pin_hash(conn, user_id, &hash)?;

    let user = user_repo::find_by_id(conn, user_id)?.ok_or_else(|| AppError::NotFound("User".into()))?;
    audit_repo::record(conn, &user.workspace_id, Some(user_id), "voice_pin_set", Some("user"), Some(user_id), "Voice PIN set", None)?;
    get_settings(conn, user_id)
}

pub fn update_preferences(conn: &Connection, user_id: &str, input: &VoicePreferencesInput) -> AppResult<VoiceUserSettings> {
    voice_repo::ensure_settings_row(conn, user_id)?;
    voice_repo::update_preferences(conn, user_id, input)?;
    get_settings(conn, user_id)
}

/// Unlocks Voice Mode for one bounded session (spec §4/VOICE-AC-01) -
/// verifies the PIN against the exact same Argon2 hash/verify
/// `auth_service` uses for the primary password (never a second scheme),
/// rate-limits/locks out after `MAX_PIN_ATTEMPTS` failures (writing a
/// `failed_login`-style audit event each time, mirroring
/// `auth_service::login`'s own), and only then creates a fresh
/// `voice_sessions` row with a TTL capped by both the user's own preference
/// and the workspace/role policy's ceiling (spec: "Administrator configures
/// allowed Voice unlock duration").
pub fn unlock(conn: &Connection, user_id: &str, pin: &str) -> AppResult<VoiceSession> {
    let settings = voice_repo::ensure_settings_row(conn, user_id)?;
    let user = user_repo::find_by_id(conn, user_id)?.ok_or_else(|| AppError::NotFound("User".into()))?;

    if voice_repo::is_pin_locked(conn, user_id)? {
        return Err(AppError::Validation(
            "Voice PIN is locked after too many failed attempts - reauthenticate to Lanesra normally, then try again".into(),
        ));
    }
    let Some(hash) = &settings.pin_hash else {
        return Err(AppError::Validation("No Voice PIN set yet - set one from Personal Settings first".into()));
    };
    if !auth_service::verify_password(pin, hash) {
        let attempts = settings.failed_attempts + 1;
        let locked_until = if attempts >= MAX_PIN_ATTEMPTS { Some(minutes_from_now(PIN_LOCKOUT_MINUTES)) } else { None };
        voice_repo::record_failed_pin_attempt(conn, user_id, attempts, locked_until.as_deref())?;
        audit_repo::record(conn, &user.workspace_id, Some(user_id), "voice_pin_failed", Some("user"), Some(user_id), "Failed Voice PIN attempt", None)?;
        return Err(AppError::Validation("Incorrect Voice PIN".into()));
    }
    voice_repo::reset_pin_attempts(conn, user_id)?;

    let policy = voice_policy_service::effective_policy(conn, user_id)?;
    if !policy.can_use_voice {
        return Err(AppError::Validation("Voice Mode is not enabled for your Access Role - ask an Administrator to grant it under Voice Governance".into()));
    }
    let minutes = settings.unlock_duration_minutes.min(policy.max_unlock_minutes).max(1);
    let expires_at = minutes_from_now(minutes);

    let session = voice_repo::create_session(conn, &new_uuid(), &user.workspace_id, user_id, &expires_at)?;
    audit_repo::record(conn, &user.workspace_id, Some(user_id), "voice_session_unlocked", Some("voice_session"), Some(&session.id), "Voice Mode unlocked", None)?;
    Ok(session)
}

/// The user's current session, if one exists - `expired` in place (not
/// deleted) once its TTL has passed, so "Voice session expired, unlock
/// again" is a real, visible state rather than silently reverting to
/// looking un-unlocked with no explanation (spec §3's `Expired` state).
pub fn current_session(conn: &Connection, user_id: &str) -> AppResult<Option<VoiceSession>> {
    let Some(session) = voice_repo::get_latest_session_for_user(conn, user_id)? else { return Ok(None) };
    voice_repo::expire_stale_session(conn, &session.id)?;
    Ok(voice_repo::get_session(conn, &session.id)?)
}

fn require_owned_session(conn: &Connection, session_id: &str, user_id: &str) -> AppResult<VoiceSession> {
    let session = voice_repo::get_session(conn, session_id)?.ok_or_else(|| AppError::NotFound("Voice session".into()))?;
    if session.user_id != user_id {
        return Err(AppError::Validation("Not your Voice session".into()));
    }
    Ok(session)
}

/// The one gate every other Voice service call goes through before doing
/// anything - `voice_planner_service`/`voice_execution_service` all take a
/// session_id and call this first, so an expired session can never plan or
/// execute a command no matter how it's invoked.
pub fn require_active_session(conn: &Connection, session_id: &str, user_id: &str) -> AppResult<VoiceSession> {
    let session = require_owned_session(conn, session_id, user_id)?;
    voice_repo::expire_stale_session(conn, &session.id)?;
    if !voice_repo::session_is_active(conn, &session.id)? {
        return Err(AppError::Validation("Voice session expired - unlock again".into()));
    }
    Ok(session)
}

pub fn extend(conn: &Connection, session_id: &str, user_id: &str) -> AppResult<VoiceSession> {
    let session = require_active_session(conn, session_id, user_id)?;
    let settings = voice_repo::ensure_settings_row(conn, user_id)?;
    let policy = voice_policy_service::effective_policy(conn, user_id)?;
    let minutes = settings.unlock_duration_minutes.min(policy.max_unlock_minutes).max(1);
    voice_repo::extend_session(conn, &session.id, &minutes_from_now(minutes))?;
    Ok(voice_repo::get_session(conn, &session.id)?.expect("just extended"))
}

pub fn expire(conn: &Connection, session_id: &str, user_id: &str) -> AppResult<()> {
    let session = require_owned_session(conn, session_id, user_id)?;
    voice_repo::set_session_state(conn, &session.id, "expired").map_err(AppError::from)
}

pub fn set_context(conn: &Connection, session_id: &str, user_id: &str, object_key: Option<&str>, record_id: Option<&str>) -> AppResult<VoiceSession> {
    let session = require_active_session(conn, session_id, user_id)?;
    voice_repo::set_session_context(conn, &session.id, object_key, record_id)?;
    Ok(voice_repo::get_session(conn, &session.id)?.expect("just updated"))
}

/// Clears conversational references ("this", "the CRM one", ...) without
/// logging the user out or re-locking Voice Mode (spec §14's "Reset voice
/// context" action) - a fresh empty history, same session.
pub fn reset_conversation(conn: &Connection, session_id: &str, user_id: &str) -> AppResult<VoiceSession> {
    let session = require_owned_session(conn, session_id, user_id)?;
    voice_repo::reset_session_conversation(conn, &session.id)?;
    Ok(voice_repo::get_session(conn, &session.id)?.expect("just reset"))
}

pub fn set_state(conn: &Connection, session_id: &str, user_id: &str, state: &str) -> AppResult<VoiceSession> {
    let session = require_owned_session(conn, session_id, user_id)?;
    voice_repo::set_session_state(conn, &session.id, state)?;
    Ok(voice_repo::get_session(conn, &session.id)?.expect("just updated"))
}
