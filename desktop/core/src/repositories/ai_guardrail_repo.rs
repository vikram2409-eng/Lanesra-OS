//! WP-0133: raw CRUD for the guardrail tables (migration 0090). A guardrail
//! carries its current version's config (joined), versions are append-only,
//! and evaluations tie a run to the version it checked.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_guardrail::{Guardrail, GuardrailEvaluation};

const SELECT: &str = "SELECT g.id, g.workspace_id, g.builtin_key, g.name, g.description,
        g.checker_type, g.is_builtin, g.current_version, g.enabled, g.hook_point,
        v.config_json, g.updated_at, g.updated_by
    FROM ai_guardrails g
    JOIN ai_guardrail_versions v
      ON v.guardrail_id = g.id AND v.version = g.current_version";

fn map(row: &rusqlite::Row) -> rusqlite::Result<Guardrail> {
    Ok(Guardrail {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        builtin_key: row.get("builtin_key")?,
        name: row.get("name")?,
        description: row.get("description")?,
        checker_type: row.get("checker_type")?,
        is_builtin: row.get::<_, i64>("is_builtin")? != 0,
        current_version: row.get("current_version")?,
        enabled: row.get::<_, i64>("enabled")? != 0,
        hook_point: row.get("hook_point")?,
        config_json: row.get("config_json")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Guardrail>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE g.workspace_id = ?1 ORDER BY g.is_builtin DESC, g.name"
    ))?;
    let rows = stmt
        .query_map([workspace_id], map)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// WP-0128: the enabled guardrails wired at one hook point, in a stable
/// order (built-ins first, then by name) so the loop evaluates them the
/// same way every run.
pub fn list_for_hook(
    conn: &Connection,
    workspace_id: &str,
    hook_point: &str,
) -> rusqlite::Result<Vec<Guardrail>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE g.workspace_id = ?1 AND g.hook_point = ?2 AND g.enabled = 1
         ORDER BY g.is_builtin DESC, g.name, g.id"
    ))?;
    let rows = stmt
        .query_map([workspace_id, hook_point], map)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> rusqlite::Result<Option<Guardrail>> {
    conn.query_row(
        &format!("{SELECT} WHERE g.workspace_id = ?1 AND g.id = ?2"),
        [workspace_id, id],
        map,
    )
    .optional()
}

pub fn get_by_builtin_key(
    conn: &Connection,
    workspace_id: &str,
    key: &str,
) -> rusqlite::Result<Option<Guardrail>> {
    conn.query_row(
        &format!("{SELECT} WHERE g.workspace_id = ?1 AND g.builtin_key = ?2"),
        [workspace_id, key],
        map,
    )
    .optional()
}

/// Inserts a guardrail at version 1 with its first config, returning its id.
#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    builtin_key: Option<&str>,
    name: &str,
    description: Option<&str>,
    checker_type: &str,
    is_builtin: bool,
    config_json: &str,
    hook_point: &str,
    actor: Option<&str>,
) -> rusqlite::Result<String> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_guardrails
            (id, workspace_id, builtin_key, name, description, checker_type, is_builtin,
             current_version, enabled, hook_point, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 1, ?10, ?8, ?9, ?8, ?9)",
        rusqlite::params![
            id,
            workspace_id,
            builtin_key,
            name,
            description,
            checker_type,
            is_builtin as i64,
            now,
            actor,
            hook_point,
        ],
    )?;
    conn.execute(
        "INSERT INTO ai_guardrail_versions (id, guardrail_id, version, config_json, created_at, created_by)
         VALUES (?1, ?2, 1, ?3, ?4, ?5)",
        rusqlite::params![new_uuid(), id, config_json, now, actor],
    )?;
    Ok(id)
}

/// Appends a new version with `config_json` and points current_version at it.
pub fn add_version(
    conn: &Connection,
    guardrail_id: &str,
    config_json: &str,
    actor: Option<&str>,
) -> rusqlite::Result<i64> {
    let next: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM ai_guardrail_versions WHERE guardrail_id = ?1",
        [guardrail_id],
        |r| r.get(0),
    )?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_guardrail_versions (id, guardrail_id, version, config_json, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![new_uuid(), guardrail_id, next, config_json, now, actor],
    )?;
    conn.execute(
        "UPDATE ai_guardrails SET current_version = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        rusqlite::params![next, now, actor, guardrail_id],
    )?;
    Ok(next)
}

#[allow(clippy::too_many_arguments)]
pub fn update_meta(
    conn: &Connection,
    guardrail_id: &str,
    name: &str,
    description: Option<&str>,
    enabled: bool,
    hook_point: &str,
    actor: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_guardrails SET name = ?1, description = ?2, enabled = ?3, hook_point = ?4, updated_at = ?5, updated_by = ?6
         WHERE id = ?7",
        rusqlite::params![name, description, enabled as i64, hook_point, now_iso(), actor, guardrail_id],
    )?;
    Ok(())
}

pub fn record_evaluation(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
    guardrail_id: &str,
    version: i64,
    passed: bool,
    detail: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_guardrail_evaluations
            (id, workspace_id, run_kind, run_id, guardrail_id, guardrail_version, passed, detail, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            new_uuid(), workspace_id, run_kind, run_id, guardrail_id, version,
            passed as i64, detail, now_iso(),
        ],
    )?;
    Ok(())
}

pub fn list_evaluations(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
) -> rusqlite::Result<Vec<GuardrailEvaluation>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, run_kind, run_id, guardrail_id, guardrail_version, passed, detail, created_at
           FROM ai_guardrail_evaluations
          WHERE workspace_id = ?1 AND run_kind = ?2 AND run_id = ?3 ORDER BY created_at",
    )?;
    let rows = stmt
        .query_map([workspace_id, run_kind, run_id], |r| {
            Ok(GuardrailEvaluation {
                id: r.get(0)?,
                workspace_id: r.get(1)?,
                run_kind: r.get(2)?,
                run_id: r.get(3)?,
                guardrail_id: r.get(4)?,
                guardrail_version: r.get(5)?,
                passed: r.get::<_, i64>(6)? != 0,
                detail: r.get(7)?,
                created_at: r.get(8)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
