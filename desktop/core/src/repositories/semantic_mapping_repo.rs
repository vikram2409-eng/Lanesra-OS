//! Raw CRUD for `semantic_mappings` (migration 0074). See
//! `services::semantic_mapping_service` for validation/System-Graph sync.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::semantic::{SemanticMapping, SemanticMappingInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<SemanticMapping> {
    Ok(SemanticMapping {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        entity_type: row.get("entity_type")?,
        field_key: row.get("field_key")?,
        glossary_term_id: row.get("glossary_term_id")?,
        semantic_role: row.get("semantic_role")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &SemanticMappingInput, actor_user_id: Option<&str>) -> rusqlite::Result<SemanticMapping> {
    conn.execute(
        "INSERT INTO semantic_mappings (id, workspace_id, entity_type, field_key, glossary_term_id, semantic_role, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, workspace_id, input.entity_type, input.field_key, input.glossary_term_id, input.semantic_role, now_iso(), actor_user_id],
    )?;
    get(conn, id).map(|m| m.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<SemanticMapping>> {
    conn.query_row("SELECT * FROM semantic_mappings WHERE id = ?1", [id], map_row).optional()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM semantic_mappings WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list_for_entity(conn: &Connection, workspace_id: &str, entity_type: &str) -> rusqlite::Result<Vec<SemanticMapping>> {
    let mut stmt = conn.prepare("SELECT * FROM semantic_mappings WHERE workspace_id = ?1 AND entity_type = ?2 ORDER BY created_at")?;
    let rows: rusqlite::Result<Vec<SemanticMapping>> = stmt.query_map(params![workspace_id, entity_type], map_row)?.collect();
    rows
}

/// Every mapping pointing at a given glossary term - what
/// `glossary_service::sync_graph_edges` reads to rebuild that term's
/// outgoing `derives_from` edges whenever a mapping is added/removed.
pub fn list_for_term(conn: &Connection, term_id: &str) -> rusqlite::Result<Vec<SemanticMapping>> {
    let mut stmt = conn.prepare("SELECT * FROM semantic_mappings WHERE glossary_term_id = ?1 ORDER BY created_at")?;
    let rows: rusqlite::Result<Vec<SemanticMapping>> = stmt.query_map([term_id], map_row)?.collect();
    rows
}

pub fn exact_duplicate_exists(conn: &Connection, workspace_id: &str, input: &SemanticMappingInput) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM semantic_mappings WHERE workspace_id = ?1 AND entity_type = ?2
         AND field_key IS ?3 AND glossary_term_id IS ?4 AND semantic_role IS ?5",
        params![workspace_id, input.entity_type, input.field_key, input.glossary_term_id, input.semantic_role],
        |r| r.get(0),
    )?;
    Ok(count > 0)
}
