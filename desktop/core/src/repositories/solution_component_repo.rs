//! Raw CRUD for `solution_components` (migration 0030) - see
//! `services::solution_component_service` for the tag/retag logic this
//! backs.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::solution_component::SolutionComponent;

fn map_component(row: &rusqlite::Row) -> rusqlite::Result<SolutionComponent> {
    Ok(SolutionComponent {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        artifact_type: row.get("artifact_type")?,
        metadata_id: row.get("metadata_id")?,
        publisher_id: row.get("publisher_id")?,
        installed_app_id: row.get("installed_app_id")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

/// Tags `(artifact_type, metadata_id)` with `publisher_id`, creating the
/// row the first time or overwriting an existing tag - the single
/// operation both `solution_component_service::tag_local` (unconditional,
/// called from every component-creating service function) and
/// `::retag` (called by `run_install` right after, to correct the tag to
/// the installing package's real publisher) reduce to.
#[allow(clippy::too_many_arguments)]
pub fn upsert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    artifact_type: &str,
    metadata_id: &str,
    publisher_id: &str,
    installed_app_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<SolutionComponent> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO solution_components (id, workspace_id, artifact_type, metadata_id, publisher_id, installed_app_id, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(workspace_id, artifact_type, metadata_id)
         DO UPDATE SET publisher_id = excluded.publisher_id, installed_app_id = excluded.installed_app_id",
        rusqlite::params![id, workspace_id, artifact_type, metadata_id, publisher_id, installed_app_id, now, actor_user_id],
    )?;
    get(conn, workspace_id, artifact_type, metadata_id).map(|c| c.expect("just upserted"))
}

pub fn get(conn: &Connection, workspace_id: &str, artifact_type: &str, metadata_id: &str) -> rusqlite::Result<Option<SolutionComponent>> {
    conn.query_row(
        "SELECT * FROM solution_components WHERE workspace_id = ?1 AND artifact_type = ?2 AND metadata_id = ?3",
        (workspace_id, artifact_type, metadata_id),
        map_component,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Every component tagged to `publisher_id` in this workspace - what the
/// "Local Workspace" grouping filters by (`is_local` publisher's id) and
/// what a package export reads back from.
pub fn list_for_publisher(conn: &Connection, workspace_id: &str, publisher_id: &str) -> rusqlite::Result<Vec<SolutionComponent>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM solution_components WHERE workspace_id = ?1 AND publisher_id = ?2 ORDER BY artifact_type, created_at",
    )?;
    let rows = stmt.query_map((workspace_id, publisher_id), map_component)?;
    rows.collect()
}

/// Every component in the workspace, joined with its owning publisher's
/// key/name/is_local and (when installed) the installed app's name - the
/// single query the "Components" tab needs, across both hand-built and
/// package-installed components.
pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<(SolutionComponent, String, String, bool, Option<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT c.*, p.key AS publisher_key, p.name AS publisher_name, p.is_local AS publisher_is_local, i.name AS installed_app_name
         FROM solution_components c
         JOIN publishers p ON p.id = c.publisher_id
         LEFT JOIN installed_apps i ON i.id = c.installed_app_id
         WHERE c.workspace_id = ?1
         ORDER BY c.artifact_type, c.created_at",
    )?;
    let rows = stmt.query_map([workspace_id], |row| {
        Ok((
            map_component(row)?,
            row.get::<_, String>("publisher_key")?,
            row.get::<_, String>("publisher_name")?,
            row.get::<_, bool>("publisher_is_local")?,
            row.get::<_, Option<String>>("installed_app_name")?,
        ))
    })?;
    rows.collect()
}
