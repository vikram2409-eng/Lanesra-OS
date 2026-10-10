//! Raw CRUD for `business_glossary_terms` (migration 0074). See
//! `services::glossary_service` for validation/System-Graph sync.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::semantic::{BusinessGlossaryTerm, BusinessGlossaryTermInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<BusinessGlossaryTerm> {
    let synonyms_json: String = row.get("synonyms_json")?;
    Ok(BusinessGlossaryTerm {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        definition: row.get("definition")?,
        owner_user_id: row.get("owner_user_id")?,
        synonyms: serde_json::from_str(&synonyms_json).unwrap_or_default(),
        data_classification: row.get("data_classification")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &BusinessGlossaryTermInput, actor_user_id: Option<&str>) -> rusqlite::Result<BusinessGlossaryTerm> {
    let now = now_iso();
    let synonyms_json = serde_json::to_string(&input.synonyms).expect("Vec<String> always serializes");
    conn.execute(
        "INSERT INTO business_glossary_terms (id, workspace_id, name, definition, owner_user_id, synonyms_json, data_classification, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?8, ?9)",
        params![id, workspace_id, input.name, input.definition, input.owner_user_id, synonyms_json, input.data_classification, now, actor_user_id],
    )?;
    get(conn, id).map(|t| t.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &BusinessGlossaryTermInput, actor_user_id: Option<&str>) -> rusqlite::Result<BusinessGlossaryTerm> {
    let now = now_iso();
    let synonyms_json = serde_json::to_string(&input.synonyms).expect("Vec<String> always serializes");
    conn.execute(
        "UPDATE business_glossary_terms SET name = ?1, definition = ?2, owner_user_id = ?3, synonyms_json = ?4, data_classification = ?5, updated_at = ?6, updated_by = ?7 WHERE id = ?8",
        params![input.name, input.definition, input.owner_user_id, synonyms_json, input.data_classification, now, actor_user_id, id],
    )?;
    get(conn, id).map(|t| t.expect("just updated"))
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE business_glossary_terms SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        params![is_active, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<BusinessGlossaryTerm>> {
    conn.query_row("SELECT * FROM business_glossary_terms WHERE id = ?1", [id], map_row).optional()
}

pub fn get_by_name(conn: &Connection, workspace_id: &str, name: &str) -> rusqlite::Result<Option<BusinessGlossaryTerm>> {
    conn.query_row("SELECT * FROM business_glossary_terms WHERE workspace_id = ?1 AND name = ?2", params![workspace_id, name], map_row).optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<BusinessGlossaryTerm>> {
    let mut stmt = conn.prepare("SELECT * FROM business_glossary_terms WHERE workspace_id = ?1 ORDER BY name")?;
    let rows: rusqlite::Result<Vec<BusinessGlossaryTerm>> = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}
