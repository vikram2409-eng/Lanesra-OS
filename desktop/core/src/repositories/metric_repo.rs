//! Raw CRUD + version history for `metric_definitions`/`metric_versions`
//! (migration 0074). See `services::metric_service` for validation,
//! versioning orchestration and System-Graph sync.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::semantic::{MetricDefinition, MetricDefinitionInput};

/// Same cap `business_rule_repo`/`workflow_repo` already use for their
/// own version history.
const VERSION_HISTORY_LIMIT: i64 = 20;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<MetricDefinition> {
    Ok(MetricDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        key: row.get("key")?,
        name: row.get("name")?,
        description: row.get("description")?,
        source_entity_type: row.get("source_entity_type")?,
        source_field_key: row.get("source_field_key")?,
        aggregation: row.get("aggregation")?,
        grain: row.get("grain")?,
        filters_json: row.get("filters_json")?,
        time_logic: row.get("time_logic")?,
        owner_user_id: row.get("owner_user_id")?,
        glossary_term_id: row.get("glossary_term_id")?,
        version: row.get("version")?,
        effective_start_date: row.get("effective_start_date")?,
        effective_end_date: row.get("effective_end_date")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create(conn: &Connection, id: &str, workspace_id: &str, key: &str, input: &MetricDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<MetricDefinition> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO metric_definitions (id, workspace_id, key, name, description, source_entity_type, source_field_key, aggregation, grain, filters_json, time_logic, owner_user_id, glossary_term_id, version, effective_start_date, effective_end_date, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1, ?14, ?15, 1, ?16, ?17, ?16, ?17)",
        params![
            id, workspace_id, key, input.name, input.description, input.source_entity_type, input.source_field_key, input.aggregation,
            input.grain, input.filters_json, input.time_logic, input.owner_user_id, input.glossary_term_id,
            input.effective_start_date, input.effective_end_date, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|m| m.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &MetricDefinitionInput, actor_user_id: Option<&str>) -> rusqlite::Result<MetricDefinition> {
    let now = now_iso();
    conn.execute(
        "UPDATE metric_definitions SET name = ?1, description = ?2, source_entity_type = ?3, source_field_key = ?4, aggregation = ?5,
            grain = ?6, filters_json = ?7, time_logic = ?8, owner_user_id = ?9, glossary_term_id = ?10,
            effective_start_date = ?11, effective_end_date = ?12, version = version + 1, updated_at = ?13, updated_by = ?14
         WHERE id = ?15",
        params![
            input.name, input.description, input.source_entity_type, input.source_field_key, input.aggregation, input.grain,
            input.filters_json, input.time_logic, input.owner_user_id, input.glossary_term_id, input.effective_start_date,
            input.effective_end_date, now, actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|m| m.expect("just updated"))
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute("UPDATE metric_definitions SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", params![is_active, now_iso(), actor_user_id, id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM metric_definitions WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<MetricDefinition>> {
    conn.query_row("SELECT * FROM metric_definitions WHERE id = ?1", [id], map_row).optional()
}

pub fn get_by_key(conn: &Connection, workspace_id: &str, key: &str) -> rusqlite::Result<Option<MetricDefinition>> {
    conn.query_row("SELECT * FROM metric_definitions WHERE workspace_id = ?1 AND key = ?2", params![workspace_id, key], map_row).optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<MetricDefinition>> {
    let mut stmt = conn.prepare("SELECT * FROM metric_definitions WHERE workspace_id = ?1 ORDER BY name")?;
    let rows: rusqlite::Result<Vec<MetricDefinition>> = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

/// Appends a snapshot and prunes to `VERSION_HISTORY_LIMIT` - the exact
/// shape `business_rule_repo::insert_version` already established.
pub fn insert_version(conn: &Connection, metric_id: &str, snapshot_json: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO metric_versions (id, metric_definition_id, snapshot_json, saved_at) VALUES (?1, ?2, ?3, ?4)",
        params![crate::domain::ids::new_uuid(), metric_id, snapshot_json, now_iso()],
    )?;
    conn.execute(
        "DELETE FROM metric_versions WHERE metric_definition_id = ?1 AND id NOT IN (
            SELECT id FROM metric_versions WHERE metric_definition_id = ?1 ORDER BY saved_at DESC LIMIT ?2
        )",
        params![metric_id, VERSION_HISTORY_LIMIT],
    )?;
    Ok(())
}

pub fn list_version_rows(conn: &Connection, metric_id: &str) -> rusqlite::Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, snapshot_json, saved_at FROM metric_versions WHERE metric_definition_id = ?1 ORDER BY saved_at DESC")?;
    let rows = stmt.query_map([metric_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))?.collect();
    rows
}
