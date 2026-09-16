use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::work_team::TeamMembership;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<TeamMembership> {
    Ok(TeamMembership {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        team_id: row.get("team_id")?,
        user_id: row.get("user_id")?,
        role_in_team: row.get("role_in_team")?,
        effective_from: row.get("effective_from")?,
        effective_to: row.get("effective_to")?,
        created_at: row.get("created_at")?,
    })
}

pub fn add(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    team_id: &str,
    user_id: &str,
    role_in_team: Option<&str>,
    effective_from: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<TeamMembership> {
    conn.execute(
        "INSERT INTO team_memberships (id, workspace_id, team_id, user_id, role_in_team, effective_from, effective_to, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8)",
        (id, workspace_id, team_id, user_id, role_in_team, effective_from, now_iso(), actor_user_id),
    )?;
    get(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<TeamMembership>> {
    conn.query_row("SELECT * FROM team_memberships WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Active members only (no effective_to, or one still in the future) -
/// matches the same "effective dating" convention the rest of this
/// codebase already uses (e.g. Organization Units).
pub fn list_active_members(conn: &Connection, team_id: &str) -> rusqlite::Result<Vec<TeamMembership>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM team_memberships WHERE team_id = ?1 AND (effective_to IS NULL OR effective_to > ?2) ORDER BY effective_from",
    )?;
    let rows = stmt.query_map((team_id, now_iso()), map_row)?.collect();
    rows
}

pub fn list_for_user(conn: &Connection, user_id: &str) -> rusqlite::Result<Vec<TeamMembership>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM team_memberships WHERE user_id = ?1 AND (effective_to IS NULL OR effective_to > ?2) ORDER BY effective_from",
    )?;
    let rows = stmt.query_map((user_id, now_iso()), map_row)?.collect();
    rows
}

/// Ends a membership by stamping effective_to - never deletes the row
/// (history stays queryable) and never touches any record's ownership;
/// see 0052_work_teams.sql's own doc comment on why the two are kept
/// deliberately independent.
pub fn end_membership(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE team_memberships SET effective_to = ?1 WHERE id = ?2", (now_iso(), id))?;
    Ok(())
}
