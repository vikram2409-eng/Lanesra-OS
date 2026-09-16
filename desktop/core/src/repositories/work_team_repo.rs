use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::work_team::{WorkTeam, WorkTeamInput, WorkTeamUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<WorkTeam> {
    Ok(WorkTeam {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        code: row.get("code")?,
        team_type: row.get("team_type")?,
        primary_org_unit_id: row.get("primary_org_unit_id")?,
        owner_user_id: row.get("owner_user_id")?,
        can_own_records: row.get("can_own_records")?,
        status: row.get("status")?,
        effective_from: row.get("effective_from")?,
        effective_to: row.get("effective_to")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &WorkTeamInput, actor_user_id: Option<&str>) -> rusqlite::Result<WorkTeam> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO work_teams
            (id, workspace_id, name, code, team_type, primary_org_unit_id, owner_user_id, can_own_records, status, effective_from, effective_to, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'Active', ?9, ?10, ?11, ?12, ?11, ?12)",
        rusqlite::params![
            id, workspace_id, input.name, input.code, input.team_type, input.primary_org_unit_id, input.owner_user_id,
            input.can_own_records, input.effective_from, input.effective_to, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<WorkTeam>> {
    conn.query_row("SELECT * FROM work_teams WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_code(conn: &Connection, workspace_id: &str, code: &str) -> rusqlite::Result<Option<WorkTeam>> {
    conn.query_row("SELECT * FROM work_teams WHERE workspace_id = ?1 AND code = ?2", (workspace_id, code), map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<WorkTeam>> {
    let mut stmt = conn.prepare("SELECT * FROM work_teams WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn update(conn: &Connection, id: &str, input: &WorkTeamUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<WorkTeam> {
    conn.execute(
        "UPDATE work_teams SET name = ?1, team_type = ?2, primary_org_unit_id = ?3, owner_user_id = ?4, can_own_records = ?5, status = ?6, effective_from = ?7, effective_to = ?8, updated_at = ?9, updated_by = ?10
         WHERE id = ?11",
        rusqlite::params![
            input.name, input.team_type, input.primary_org_unit_id, input.owner_user_id, input.can_own_records,
            input.status, input.effective_from, input.effective_to, now_iso(), actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just updated"))
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM work_teams WHERE id = ?1", [id])?;
    Ok(())
}
