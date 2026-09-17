//! Voice-First Mode, PR 1. Thin CRUD/read repo over the 9 `voice_*` tables
//! from `0057_voice_mode_v1.sql` - see `models::voice` for the shapes and
//! `services::voice_session_service`/`voice_planner_service`/
//! `voice_execution_service` for the logic that calls these.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::voice::*;

fn map_settings_record(row: &rusqlite::Row) -> rusqlite::Result<VoiceUserSettingsRecord> {
    Ok(VoiceUserSettingsRecord {
        user_id: row.get("user_id")?,
        enabled: row.get("enabled")?,
        pin_hash: row.get("pin_hash")?,
        pin_set_at: row.get("pin_set_at")?,
        failed_attempts: row.get("failed_attempts")?,
        locked_until: row.get("locked_until")?,
        response_channel: row.get("response_channel")?,
        spoken_detail: row.get("spoken_detail")?,
        auto_speak_confirmations: row.get("auto_speak_confirmations")?,
        quiet_mode: row.get("quiet_mode")?,
        unlock_duration_minutes: row.get("unlock_duration_minutes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn to_safe_settings(r: VoiceUserSettingsRecord) -> VoiceUserSettings {
    VoiceUserSettings {
        user_id: r.user_id,
        enabled: r.enabled,
        pin_set: r.pin_hash.is_some(),
        pin_set_at: r.pin_set_at,
        failed_attempts: r.failed_attempts,
        locked_until: r.locked_until,
        response_channel: r.response_channel,
        spoken_detail: r.spoken_detail,
        auto_speak_confirmations: r.auto_speak_confirmations,
        quiet_mode: r.quiet_mode,
        unlock_duration_minutes: r.unlock_duration_minutes,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

/// Every user starts with an implicit, all-defaults settings row (no PIN
/// set, Voice disabled) rather than requiring one be seeded per-user at
/// account creation - this inserts it lazily on first touch.
pub fn ensure_settings_row(conn: &Connection, user_id: &str) -> rusqlite::Result<VoiceUserSettingsRecord> {
    if let Some(existing) = get_settings_record(conn, user_id)? {
        return Ok(existing);
    }
    let now = now_iso();
    conn.execute(
        "INSERT INTO voice_user_settings (user_id, enabled, created_at, updated_at) VALUES (?1, 0, ?2, ?2)
         ON CONFLICT(user_id) DO NOTHING",
        (user_id, &now),
    )?;
    Ok(get_settings_record(conn, user_id)?.expect("just inserted"))
}

pub fn get_settings_record(conn: &Connection, user_id: &str) -> rusqlite::Result<Option<VoiceUserSettingsRecord>> {
    conn.query_row("SELECT * FROM voice_user_settings WHERE user_id = ?1", [user_id], map_settings_record).optional()
}

pub fn set_pin_hash(conn: &Connection, user_id: &str, pin_hash: &str) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE voice_user_settings SET enabled = 1, pin_hash = ?1, pin_set_at = ?2, failed_attempts = 0, locked_until = NULL, updated_at = ?2 WHERE user_id = ?3",
        (pin_hash, &now, user_id),
    )?;
    Ok(())
}

pub fn record_failed_pin_attempt(conn: &Connection, user_id: &str, attempts: i64, locked_until: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE voice_user_settings SET failed_attempts = ?1, locked_until = ?2, updated_at = ?3 WHERE user_id = ?4",
        (attempts, locked_until, now_iso(), user_id),
    )?;
    Ok(())
}

pub fn reset_pin_attempts(conn: &Connection, user_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE voice_user_settings SET failed_attempts = 0, locked_until = NULL, updated_at = ?1 WHERE user_id = ?2",
        (now_iso(), user_id),
    )?;
    Ok(())
}

pub fn update_preferences(conn: &Connection, user_id: &str, input: &VoicePreferencesInput) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE voice_user_settings SET response_channel = ?1, spoken_detail = ?2, auto_speak_confirmations = ?3, quiet_mode = ?4, unlock_duration_minutes = ?5, updated_at = ?6 WHERE user_id = ?7",
        rusqlite::params![input.response_channel, input.spoken_detail, input.auto_speak_confirmations, input.quiet_mode, input.unlock_duration_minutes, now_iso(), user_id],
    )?;
    Ok(())
}

// ---- voice_policy_bindings --------------------------------------------

fn map_policy(row: &rusqlite::Row) -> rusqlite::Result<VoicePolicyBinding> {
    let level: String = row.get("max_action_level")?;
    Ok(VoicePolicyBinding {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        access_role_id: row.get("access_role_id")?,
        can_use_voice: row.get("can_use_voice")?,
        can_search: row.get("can_search")?,
        can_create: row.get("can_create")?,
        can_update: row.get("can_update")?,
        can_act: row.get("can_act")?,
        can_bulk_act: row.get("can_bulk_act")?,
        can_external_act: row.get("can_external_act")?,
        can_use_agents: row.get("can_use_agents")?,
        max_action_level: MaxActionLevel::from_str(&level).unwrap_or(MaxActionLevel::AskOnly),
        processing_boundary: row.get("processing_boundary")?,
        confidence_thresholds_json: row.get("confidence_thresholds_json")?,
        max_unlock_minutes: row.get("max_unlock_minutes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_policy_bindings(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<VoicePolicyBinding>> {
    let mut stmt = conn.prepare("SELECT * FROM voice_policy_bindings WHERE workspace_id = ?1 ORDER BY (access_role_id IS NULL) DESC")?;
    let rows = stmt.query_map([workspace_id], map_policy)?.collect();
    rows
}

/// The workspace-default binding (`access_role_id IS NULL`) - present for
/// any workspace that existed when migration 0057 ran (its own bootstrap
/// insert), and for every workspace created afterwards via
/// `ensure_default_policy_binding` below, called from
/// `workspace_service::first_run_setup`.
pub fn get_default_policy_binding(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<VoicePolicyBinding>> {
    conn.query_row(
        "SELECT * FROM voice_policy_bindings WHERE workspace_id = ?1 AND access_role_id IS NULL",
        [workspace_id],
        map_policy,
    )
    .optional()
}

/// Idempotently creates the workspace-default policy binding for a
/// workspace that didn't exist yet when migration 0057's own bootstrap
/// `INSERT ... SELECT FROM workspaces` ran - true of every workspace
/// created after that migration first shipped, which is the normal case
/// (migrations apply once to the schema; `first_run_setup` runs afterwards).
/// Same values as the migration's own default row: Voice off by default,
/// an Administrator must opt each Access Role in.
pub fn ensure_default_policy_binding(conn: &Connection, workspace_id: &str) -> rusqlite::Result<()> {
    if get_default_policy_binding(conn, workspace_id)?.is_some() {
        return Ok(());
    }
    let now = now_iso();
    conn.execute(
        "INSERT INTO voice_policy_bindings (id, workspace_id, access_role_id, can_use_voice, can_search, can_create, can_update, can_act, can_bulk_act, can_external_act, can_use_agents, max_action_level, processing_boundary, confidence_thresholds_json, max_unlock_minutes, created_at, updated_at)
         VALUES (?1, ?2, NULL, 0, 1, 0, 0, 0, 0, 0, 0, 'ask_only', 'cloud', '{}', 15, ?3, ?3)",
        (crate::domain::ids::new_uuid(), workspace_id, &now),
    )?;
    Ok(())
}

pub fn get_policy_binding_for_role(conn: &Connection, workspace_id: &str, access_role_id: &str) -> rusqlite::Result<Option<VoicePolicyBinding>> {
    conn.query_row(
        "SELECT * FROM voice_policy_bindings WHERE workspace_id = ?1 AND access_role_id = ?2",
        (workspace_id, access_role_id),
        map_policy,
    )
    .optional()
}

#[allow(clippy::too_many_arguments)]
pub fn upsert_policy_binding(conn: &Connection, id: &str, workspace_id: &str, input: &VoicePolicyBindingInput) -> rusqlite::Result<VoicePolicyBinding> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO voice_policy_bindings (id, workspace_id, access_role_id, can_use_voice, can_search, can_create, can_update, can_act, can_bulk_act, can_external_act, can_use_agents, max_action_level, processing_boundary, confidence_thresholds_json, max_unlock_minutes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, '{}', ?14, ?15, ?15)
         ON CONFLICT(workspace_id, access_role_id) DO UPDATE SET
            can_use_voice = excluded.can_use_voice, can_search = excluded.can_search, can_create = excluded.can_create,
            can_update = excluded.can_update, can_act = excluded.can_act, can_bulk_act = excluded.can_bulk_act,
            can_external_act = excluded.can_external_act, can_use_agents = excluded.can_use_agents,
            max_action_level = excluded.max_action_level, processing_boundary = excluded.processing_boundary,
            max_unlock_minutes = excluded.max_unlock_minutes, updated_at = excluded.updated_at",
        rusqlite::params![
            id, workspace_id, input.access_role_id, input.can_use_voice, input.can_search, input.can_create,
            input.can_update, input.can_act, input.can_bulk_act, input.can_external_act, input.can_use_agents,
            input.max_action_level, input.processing_boundary, input.max_unlock_minutes, now,
        ],
    )?;
    let found = match &input.access_role_id {
        Some(role_id) => get_policy_binding_for_role(conn, workspace_id, role_id)?,
        None => get_default_policy_binding(conn, workspace_id)?,
    };
    Ok(found.expect("just upserted"))
}

// ---- voice_sessions -----------------------------------------------------

fn map_session(row: &rusqlite::Row) -> rusqlite::Result<VoiceSession> {
    Ok(VoiceSession {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        user_id: row.get("user_id")?,
        state: row.get("state")?,
        unlocked_at: row.get("unlocked_at")?,
        expires_at: row.get("expires_at")?,
        context_object_key: row.get("context_object_key")?,
        context_record_id: row.get("context_record_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create_session(conn: &Connection, id: &str, workspace_id: &str, user_id: &str, expires_at: &str) -> rusqlite::Result<VoiceSession> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO voice_sessions (id, workspace_id, user_id, state, unlocked_at, expires_at, conversation_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'idle', ?4, ?5, '[]', ?4, ?4)",
        (id, workspace_id, user_id, &now, expires_at),
    )?;
    Ok(get_session(conn, id)?.expect("just inserted"))
}

pub fn get_session(conn: &Connection, id: &str) -> rusqlite::Result<Option<VoiceSession>> {
    conn.query_row("SELECT * FROM voice_sessions WHERE id = ?1", [id], map_session).optional()
}

/// The user's most recently created session, regardless of whether it has
/// since expired - callers check `expires_at`/`state` themselves
/// (`voice_session_service::current_session` is the one place that does).
pub fn get_latest_session_for_user(conn: &Connection, user_id: &str) -> rusqlite::Result<Option<VoiceSession>> {
    conn.query_row(
        "SELECT * FROM voice_sessions WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 1",
        [user_id],
        map_session,
    )
    .optional()
}

pub fn set_session_state(conn: &Connection, id: &str, state: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_sessions SET state = ?1, updated_at = ?2 WHERE id = ?3", (state, now_iso(), id))?;
    Ok(())
}

pub fn set_session_context(conn: &Connection, id: &str, object_key: Option<&str>, record_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE voice_sessions SET context_object_key = ?1, context_record_id = ?2, updated_at = ?3 WHERE id = ?4",
        (object_key, record_id, now_iso(), id),
    )?;
    Ok(())
}

/// Flips a session to `expired` if its TTL has actually passed - the
/// comparison happens in SQL (against the same `now_iso()` text format
/// every `expires_at` was written in), matching `session_repo`'s own
/// `expires_at > ?`-in-SQL convention for `web_sessions` rather than
/// comparing timestamp strings in Rust.
pub fn expire_stale_session(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE voice_sessions SET state = 'expired', updated_at = ?1 WHERE id = ?2 AND expires_at <= ?1 AND state != 'expired'",
        (now_iso(), id),
    )?;
    Ok(())
}

pub fn session_is_active(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM voice_sessions WHERE id = ?1 AND state != 'expired' AND expires_at > ?2)",
        (id, now_iso()),
        |r| r.get(0),
    )
}

pub fn is_pin_locked(conn: &Connection, user_id: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM voice_user_settings WHERE user_id = ?1 AND locked_until IS NOT NULL AND locked_until > ?2)",
        (user_id, now_iso()),
        |r| r.get(0),
    )
}

pub fn extend_session(conn: &Connection, id: &str, expires_at: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_sessions SET expires_at = ?1, state = 'idle', updated_at = ?2 WHERE id = ?3", (expires_at, now_iso(), id))?;
    Ok(())
}

pub fn reset_session_conversation(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_sessions SET conversation_json = '[]', updated_at = ?1 WHERE id = ?2", (now_iso(), id))?;
    Ok(())
}

pub fn get_session_conversation(conn: &Connection, id: &str) -> rusqlite::Result<String> {
    conn.query_row("SELECT conversation_json FROM voice_sessions WHERE id = ?1", [id], |r| r.get(0))
}

pub fn set_session_conversation(conn: &Connection, id: &str, conversation_json: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_sessions SET conversation_json = ?1, updated_at = ?2 WHERE id = ?3", (conversation_json, now_iso(), id))?;
    Ok(())
}

// ---- voice_commands -------------------------------------------------------

fn map_command(row: &rusqlite::Row) -> rusqlite::Result<VoiceCommand> {
    Ok(VoiceCommand {
        id: row.get("id")?,
        session_id: row.get("session_id")?,
        workspace_id: row.get("workspace_id")?,
        user_id: row.get("user_id")?,
        transcript: row.get("transcript")?,
        language: row.get("language")?,
        speech_confidence: row.get("speech_confidence")?,
        status: row.get("status")?,
        correlation_id: row.get("correlation_id")?,
        created_at: row.get("created_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_command(
    conn: &Connection,
    id: &str,
    session_id: &str,
    workspace_id: &str,
    user_id: &str,
    transcript: &str,
    language: &str,
    speech_confidence: Option<f64>,
    correlation_id: &str,
) -> rusqlite::Result<VoiceCommand> {
    conn.execute(
        "INSERT INTO voice_commands (id, session_id, workspace_id, user_id, transcript, language, speech_confidence, status, correlation_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'received', ?8, ?9)",
        rusqlite::params![id, session_id, workspace_id, user_id, transcript, language, speech_confidence, correlation_id, now_iso()],
    )?;
    Ok(get_command(conn, id)?.expect("just inserted"))
}

pub fn get_command(conn: &Connection, id: &str) -> rusqlite::Result<Option<VoiceCommand>> {
    conn.query_row("SELECT * FROM voice_commands WHERE id = ?1", [id], map_command).optional()
}

pub fn set_command_status(conn: &Connection, id: &str, status: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_commands SET status = ?1 WHERE id = ?2", (status, id))?;
    Ok(())
}

pub fn list_commands_for_user(conn: &Connection, user_id: &str, limit: i64) -> rusqlite::Result<Vec<VoiceCommand>> {
    let mut stmt = conn.prepare("SELECT * FROM voice_commands WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2")?;
    let rows = stmt.query_map((user_id, limit), map_command)?.collect();
    rows
}

pub fn list_commands_for_workspace(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<VoiceCommand>> {
    let mut stmt = conn.prepare("SELECT * FROM voice_commands WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT ?2")?;
    let rows = stmt.query_map((workspace_id, limit), map_command)?.collect();
    rows
}

// ---- voice_resolutions -----------------------------------------------------

fn map_resolution(row: &rusqlite::Row) -> rusqlite::Result<VoiceResolution> {
    let candidates_json: String = row.get("candidates_json")?;
    Ok(VoiceResolution {
        id: row.get("id")?,
        command_id: row.get("command_id")?,
        intent: row.get("intent")?,
        object_key: row.get("object_key")?,
        record_reference_text: row.get("record_reference_text")?,
        resolved_record_id: row.get("resolved_record_id")?,
        intent_confidence: row.get("intent_confidence")?,
        entity_confidence: row.get("entity_confidence")?,
        candidates: serde_json::from_str(&candidates_json).unwrap_or_default(),
        created_at: row.get("created_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_resolution(
    conn: &Connection,
    id: &str,
    command_id: &str,
    intent: &str,
    object_key: Option<&str>,
    record_reference_text: Option<&str>,
    resolved_record_id: Option<&str>,
    intent_confidence: f64,
    entity_confidence: Option<f64>,
    candidates: &[ResolutionCandidate],
) -> rusqlite::Result<VoiceResolution> {
    let candidates_json = serde_json::to_string(candidates).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO voice_resolutions (id, command_id, intent, object_key, record_reference_text, resolved_record_id, intent_confidence, entity_confidence, candidates_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, command_id, intent, object_key, record_reference_text, resolved_record_id, intent_confidence, entity_confidence, candidates_json, now_iso()],
    )?;
    Ok(get_resolution_for_command(conn, command_id)?.expect("just inserted"))
}

pub fn get_resolution_for_command(conn: &Connection, command_id: &str) -> rusqlite::Result<Option<VoiceResolution>> {
    conn.query_row(
        "SELECT * FROM voice_resolutions WHERE command_id = ?1 ORDER BY created_at DESC LIMIT 1",
        [command_id],
        map_resolution,
    )
    .optional()
}

// ---- voice_action_plans -----------------------------------------------------

fn map_plan(row: &rusqlite::Row) -> rusqlite::Result<VoiceActionPlan> {
    let plan_json: String = row.get("plan_json")?;
    let risk_str: String = row.get("risk")?;
    Ok(VoiceActionPlan {
        id: row.get("id")?,
        command_id: row.get("command_id")?,
        plan: serde_json::from_str(&plan_json).unwrap_or(VoiceActionPlanBody { steps: vec![] }),
        risk: VoiceRisk::from_str(&risk_str).unwrap_or(VoiceRisk::None),
        confirmation_required: row.get("confirmation_required")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create_plan(conn: &Connection, id: &str, command_id: &str, plan: &VoiceActionPlanBody, risk: VoiceRisk, confirmation_required: bool, status: &str) -> rusqlite::Result<VoiceActionPlan> {
    let plan_json = serde_json::to_string(plan).unwrap_or_else(|_| "{\"steps\":[]}".to_string());
    let now = now_iso();
    conn.execute(
        "INSERT INTO voice_action_plans (id, command_id, plan_json, risk, confirmation_required, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![id, command_id, plan_json, risk.as_str(), confirmation_required, status, now],
    )?;
    Ok(get_plan(conn, id)?.expect("just inserted"))
}

pub fn get_plan(conn: &Connection, id: &str) -> rusqlite::Result<Option<VoiceActionPlan>> {
    conn.query_row("SELECT * FROM voice_action_plans WHERE id = ?1", [id], map_plan).optional()
}

pub fn set_plan_status(conn: &Connection, id: &str, status: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_action_plans SET status = ?1, updated_at = ?2 WHERE id = ?3", (status, now_iso(), id))?;
    Ok(())
}

// ---- voice_confirmations -----------------------------------------------------

pub fn create_confirmation(conn: &Connection, id: &str, plan_id: &str, method: &str, outcome: &str, user_id: &str) -> rusqlite::Result<VoiceConfirmation> {
    conn.execute(
        "INSERT INTO voice_confirmations (id, plan_id, method, outcome, user_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (id, plan_id, method, outcome, user_id, now_iso()),
    )?;
    Ok(VoiceConfirmation { id: id.to_string(), plan_id: plan_id.to_string(), method: method.to_string(), outcome: outcome.to_string(), user_id: user_id.to_string(), created_at: now_iso() })
}

// ---- voice_executions -----------------------------------------------------

fn map_execution(row: &rusqlite::Row) -> rusqlite::Result<VoiceExecution> {
    Ok(VoiceExecution {
        id: row.get("id")?,
        plan_id: row.get("plan_id")?,
        step_index: row.get("step_index")?,
        entity_type: row.get("entity_type")?,
        entity_id: row.get("entity_id")?,
        action: row.get("action")?,
        result: row.get("result")?,
        error_message: row.get("error_message")?,
        undo_token: row.get("undo_token")?,
        undone_at: row.get("undone_at")?,
        correlation_id: row.get("correlation_id")?,
        executed_at: row.get("executed_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_execution(
    conn: &Connection,
    id: &str,
    plan_id: &str,
    step_index: i64,
    entity_type: &str,
    entity_id: Option<&str>,
    action: &str,
    result: &str,
    error_message: Option<&str>,
    undo_token: Option<&str>,
    correlation_id: &str,
) -> rusqlite::Result<VoiceExecution> {
    conn.execute(
        "INSERT INTO voice_executions (id, plan_id, step_index, entity_type, entity_id, action, result, error_message, undo_token, correlation_id, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![id, plan_id, step_index, entity_type, entity_id, action, result, error_message, undo_token, correlation_id, now_iso()],
    )?;
    Ok(get_execution(conn, id)?.expect("just inserted"))
}

pub fn get_execution(conn: &Connection, id: &str) -> rusqlite::Result<Option<VoiceExecution>> {
    conn.query_row("SELECT * FROM voice_executions WHERE id = ?1", [id], map_execution).optional()
}

pub fn list_executions_for_plan(conn: &Connection, plan_id: &str) -> rusqlite::Result<Vec<VoiceExecution>> {
    let mut stmt = conn.prepare("SELECT * FROM voice_executions WHERE plan_id = ?1 ORDER BY step_index")?;
    let rows = stmt.query_map([plan_id], map_execution)?.collect();
    rows
}

pub fn get_execution_by_undo_token(conn: &Connection, undo_token: &str) -> rusqlite::Result<Option<VoiceExecution>> {
    conn.query_row("SELECT * FROM voice_executions WHERE undo_token = ?1", [undo_token], map_execution).optional()
}

pub fn mark_execution_undone(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE voice_executions SET undone_at = ?1 WHERE id = ?2", (now_iso(), id))?;
    Ok(())
}

// ---- voice_provider_profiles -----------------------------------------------------

fn map_provider(row: &rusqlite::Row) -> rusqlite::Result<VoiceProviderProfile> {
    Ok(VoiceProviderProfile {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        kind: row.get("kind")?,
        privacy_class: row.get("privacy_class")?,
        is_default: row.get("is_default")?,
        last_health_check_at: row.get("last_health_check_at")?,
        last_health_status: row.get("last_health_status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_provider_profiles(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<VoiceProviderProfile>> {
    let mut stmt = conn.prepare("SELECT * FROM voice_provider_profiles WHERE workspace_id = ?1 ORDER BY is_default DESC, name")?;
    let rows = stmt.query_map([workspace_id], map_provider)?.collect();
    rows
}

pub fn set_provider_health(conn: &Connection, id: &str, status: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE voice_provider_profiles SET last_health_status = ?1, last_health_check_at = ?2, updated_at = ?2 WHERE id = ?3",
        (status, now_iso(), id),
    )?;
    Ok(())
}

/// Idempotently seeds the one real PR-1 adapter (browser Web Speech API)
/// for a workspace created after migration 0057's own bootstrap insert
/// already ran - same reasoning as `ensure_default_policy_binding` above.
pub fn ensure_default_provider_profile(conn: &Connection, workspace_id: &str) -> rusqlite::Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM voice_provider_profiles WHERE workspace_id = ?1)",
        [workspace_id],
        |r| r.get(0),
    )?;
    if exists {
        return Ok(());
    }
    let now = now_iso();
    conn.execute(
        "INSERT INTO voice_provider_profiles (id, workspace_id, name, kind, privacy_class, is_default, created_at, updated_at)
         VALUES (?1, ?2, 'Browser (Web Speech API)', 'web_speech', 'external', 1, ?3, ?3)",
        (crate::domain::ids::new_uuid(), workspace_id, &now),
    )?;
    Ok(())
}

// ---- Voice Activity (My Voice Activity + admin search) --------------------

fn map_activity(row: &rusqlite::Row) -> rusqlite::Result<VoiceActivityEntry> {
    Ok(VoiceActivityEntry {
        command_id: row.get("command_id")?,
        transcript: row.get("transcript")?,
        intent: row.get("intent")?,
        object_key: row.get("object_key")?,
        resolved_record_id: row.get("resolved_record_id")?,
        plan_status: row.get("plan_status")?,
        risk: row.get("risk")?,
        user_id: row.get("user_id")?,
        created_at: row.get("created_at")?,
    })
}

const ACTIVITY_SELECT: &str = "SELECT c.id AS command_id, c.transcript, r.intent, r.object_key, r.resolved_record_id, p.status AS plan_status, p.risk, c.user_id, c.created_at
     FROM voice_commands c
     LEFT JOIN voice_resolutions r ON r.command_id = c.id
     LEFT JOIN voice_action_plans p ON p.command_id = c.id";

pub fn list_activity_for_user(conn: &Connection, user_id: &str, limit: i64) -> rusqlite::Result<Vec<VoiceActivityEntry>> {
    let sql = format!("{ACTIVITY_SELECT} WHERE c.user_id = ?1 ORDER BY c.created_at DESC LIMIT ?2");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map((user_id, limit), map_activity)?.collect();
    rows
}

pub fn search_activity_for_workspace(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<VoiceActivityEntry>> {
    let sql = format!("{ACTIVITY_SELECT} WHERE c.workspace_id = ?1 ORDER BY c.created_at DESC LIMIT ?2");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map((workspace_id, limit), map_activity)?.collect();
    rows
}
