//! AI Agent Platform v2, Phase 1: raw CRUD for `ai_approvals` (migration
//! `0059_ai_agent_versioning.sql`) - a durable, generalized pending-action
//! row. See `services::approval_service` for validation (only a pending
//! approval can be resolved, `approve`/`reject` are the only outcomes).

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_approval::{AiApproval, AiApprovalInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AiApproval> {
    let proposal_json: String = row.get("proposal_json")?;
    Ok(AiApproval {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        subject_type: row.get("subject_type")?,
        subject_id: row.get("subject_id")?,
        proposal: serde_json::from_str(&proposal_json).unwrap_or(serde_json::Value::Null),
        status: row.get("status")?,
        requested_by: row.get("requested_by")?,
        resolved_by: row.get("resolved_by")?,
        resolution_notes: row.get("resolution_notes")?,
        created_at: row.get("created_at")?,
        resolved_at: row.get("resolved_at")?,
    })
}

pub fn create(conn: &Connection, workspace_id: &str, input: &AiApprovalInput, requested_by: Option<&str>) -> rusqlite::Result<AiApproval> {
    let id = new_uuid();
    let now = now_iso();
    let proposal_json = input.proposal.to_string();
    conn.execute(
        "INSERT INTO ai_approvals (id, workspace_id, subject_type, subject_id, proposal_json, status, requested_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)",
        rusqlite::params![id, workspace_id, input.subject_type, input.subject_id, proposal_json, requested_by, now],
    )?;
    Ok(get(conn, &id)?.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiApproval>> {
    conn.query_row("SELECT * FROM ai_approvals WHERE id = ?1", [id], map_row).optional()
}

/// Most recent first. `status` narrows to one of `pending`/`approved`/
/// `rejected` when given - the admin approvals inbox always wants
/// `Some("pending")`, a full audit view wants `None`.
pub fn list(conn: &Connection, workspace_id: &str, status: Option<&str>) -> rusqlite::Result<Vec<AiApproval>> {
    let mut stmt = if status.is_some() {
        conn.prepare("SELECT * FROM ai_approvals WHERE workspace_id = ?1 AND status = ?2 ORDER BY created_at DESC")?
    } else {
        conn.prepare("SELECT * FROM ai_approvals WHERE workspace_id = ?1 ORDER BY created_at DESC")?
    };
    let rows = if let Some(status) = status {
        stmt.query_map(rusqlite::params![workspace_id, status], map_row)?.collect()
    } else {
        stmt.query_map(rusqlite::params![workspace_id], map_row)?.collect()
    };
    rows
}

pub fn resolve(conn: &Connection, id: &str, status: &str, resolved_by: Option<&str>, resolution_notes: Option<&str>) -> rusqlite::Result<AiApproval> {
    let now = now_iso();
    conn.execute(
        "UPDATE ai_approvals SET status = ?1, resolved_by = ?2, resolution_notes = ?3, resolved_at = ?4 WHERE id = ?5",
        rusqlite::params![status, resolved_by, resolution_notes, now, id],
    )?;
    Ok(get(conn, id)?.expect("just updated"))
}
