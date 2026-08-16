use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::relationship::{RelationshipDefinition, RelationshipDefinitionInput, RelationshipDefinitionUpdate, RelationshipInstance};

fn map_def(row: &rusqlite::Row) -> rusqlite::Result<RelationshipDefinition> {
    Ok(RelationshipDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        key: row.get("key")?,
        source_entity_type: row.get("source_entity_type")?,
        target_entity_type: row.get("target_entity_type")?,
        relationship_type: row.get("relationship_type")?,
        forward_label: row.get("forward_label")?,
        reverse_label: row.get("reverse_label")?,
        is_required: row.get("is_required")?,
        show_related_list: row.get("show_related_list")?,
        delete_behavior: row.get("delete_behavior")?,
        is_protected: row.get("is_protected")?,
        is_active: row.get("is_active")?,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

fn map_instance(row: &rusqlite::Row) -> rusqlite::Result<RelationshipInstance> {
    Ok(RelationshipInstance {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        relationship_definition_id: row.get("relationship_definition_id")?,
        source_entity_type: row.get("source_entity_type")?,
        source_id: row.get("source_id")?,
        target_entity_type: row.get("target_entity_type")?,
        target_id: row.get("target_id")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_definition(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    key: &str,
    input: &RelationshipDefinitionInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<RelationshipDefinition> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO relationship_definitions
            (id, workspace_id, key, source_entity_type, target_entity_type, relationship_type,
             forward_label, reverse_label, is_required, show_related_list, delete_behavior,
             is_protected, is_active, sort_order, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, 1, ?12, ?13, ?14, ?13, ?14)",
        rusqlite::params![
            id, workspace_id, key, input.source_entity_type, input.target_entity_type, input.relationship_type,
            input.forward_label, input.reverse_label, input.is_required, input.show_related_list,
            input.delete_behavior, input.sort_order, now, actor_user_id,
        ],
    )?;
    get_definition(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get_definition(conn: &Connection, id: &str) -> rusqlite::Result<Option<RelationshipDefinition>> {
    conn.query_row("SELECT * FROM relationship_definitions WHERE id = ?1", [id], map_def)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_definitions(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<RelationshipDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM relationship_definitions WHERE workspace_id = ?1 ORDER BY sort_order, forward_label",
    )?;
    let rows = stmt.query_map([workspace_id], map_def)?.collect();
    rows
}

/// Every active definition where `entity_type` participates on either side
/// - used to work out which related lists a given record's detail page
/// should render.
pub fn list_definitions_for_entity(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<RelationshipDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM relationship_definitions
         WHERE workspace_id = ?1 AND is_active = 1 AND (source_entity_type = ?2 OR target_entity_type = ?2)
         ORDER BY sort_order, forward_label",
    )?;
    let rows = stmt.query_map((workspace_id, entity_type), map_def)?.collect();
    rows
}

pub fn update_definition(conn: &Connection, id: &str, input: &RelationshipDefinitionUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<RelationshipDefinition> {
    conn.execute(
        "UPDATE relationship_definitions
         SET forward_label = ?1, reverse_label = ?2, is_required = ?3, show_related_list = ?4,
             delete_behavior = ?5, sort_order = ?6, is_active = ?7, updated_at = ?8, updated_by = ?9
         WHERE id = ?10",
        rusqlite::params![
            input.forward_label, input.reverse_label, input.is_required, input.show_related_list,
            input.delete_behavior, input.sort_order, input.is_active, now_iso(), actor_user_id, id,
        ],
    )?;
    get_definition(conn, id).map(|d| d.expect("just updated"))
}

pub fn delete_definition(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM relationship_definitions WHERE id = ?1", [id])?;
    Ok(())
}

pub fn count_instances_for_definition(conn: &Connection, definition_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM relationship_instances WHERE relationship_definition_id = ?1",
        [definition_id],
        |r| r.get(0),
    )
}

pub fn count_by_source(conn: &Connection, definition_id: &str, source_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM relationship_instances WHERE relationship_definition_id = ?1 AND source_id = ?2",
        (definition_id, source_id),
        |r| r.get(0),
    )
}

pub fn count_by_target(conn: &Connection, definition_id: &str, target_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM relationship_instances WHERE relationship_definition_id = ?1 AND target_id = ?2",
        (definition_id, target_id),
        |r| r.get(0),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn create_instance(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    relationship_definition_id: &str,
    source_entity_type: &str,
    source_id: &str,
    target_entity_type: &str,
    target_id: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<RelationshipInstance> {
    conn.execute(
        "INSERT INTO relationship_instances
            (id, workspace_id, relationship_definition_id, source_entity_type, source_id, target_entity_type, target_id, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, workspace_id, relationship_definition_id, source_entity_type, source_id, target_entity_type, target_id, now_iso(), actor_user_id],
    )?;
    get_instance(conn, id).map(|i| i.expect("just inserted"))
}

pub fn get_instance(conn: &Connection, id: &str) -> rusqlite::Result<Option<RelationshipInstance>> {
    conn.query_row("SELECT * FROM relationship_instances WHERE id = ?1", [id], map_instance)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn delete_instance(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM relationship_instances WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_instances_for_record(conn: &Connection, entity_type: &str, entity_id: &str) -> rusqlite::Result<usize> {
    let a = conn.execute(
        "DELETE FROM relationship_instances WHERE source_entity_type = ?1 AND source_id = ?2",
        (entity_type, entity_id),
    )?;
    let b = conn.execute(
        "DELETE FROM relationship_instances WHERE target_entity_type = ?1 AND target_id = ?2",
        (entity_type, entity_id),
    )?;
    Ok(a + b)
}

pub fn list_instances_where_source(conn: &Connection, definition_id: &str, source_id: &str) -> rusqlite::Result<Vec<RelationshipInstance>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM relationship_instances WHERE relationship_definition_id = ?1 AND source_id = ?2 ORDER BY created_at",
    )?;
    let rows = stmt.query_map((definition_id, source_id), map_instance)?.collect();
    rows
}

pub fn list_instances_where_target(conn: &Connection, definition_id: &str, target_id: &str) -> rusqlite::Result<Vec<RelationshipInstance>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM relationship_instances WHERE relationship_definition_id = ?1 AND target_id = ?2 ORDER BY created_at",
    )?;
    let rows = stmt.query_map((definition_id, target_id), map_instance)?.collect();
    rows
}

/// Any instance - as source or target - referencing this exact record,
/// across every relationship definition. Used both to render "all related
/// records" on a detail page and to check whether deleting the record is
/// blocked by a `restrict` relationship.
pub fn list_instances_for_record(conn: &Connection, entity_type: &str, entity_id: &str) -> rusqlite::Result<Vec<RelationshipInstance>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM relationship_instances
         WHERE (source_entity_type = ?1 AND source_id = ?2) OR (target_entity_type = ?1 AND target_id = ?2)
         ORDER BY created_at",
    )?;
    let rows = stmt.query_map((entity_type, entity_id), map_instance)?.collect();
    rows
}
