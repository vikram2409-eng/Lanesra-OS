//! Raw CRUD for `record_embeddings`/`pending_record_embeddings` (migration
//! 0048) - one embedding row per Custom Object record, keyed by
//! `record_id` (a record has at most one embedding, always its latest,
//! so there's no surrogate id to key by), plus the "enqueue now, drain
//! later" reindex queue `services::vector_search_service` drains.
//! Deliberately no vector index/extension - `vector_search_service` loads
//! every row for a workspace (optionally scoped to one `object_key`) and
//! computes cosine similarity in Rust; see that module's own doc comment
//! on why, at this project's scale, that's the honest tradeoff rather
//! than a new native dependency.

use rusqlite::{params, Connection};

use crate::domain::ids::now_iso;

/// Packs an `f32` vector into little-endian bytes for the `BLOB` column -
/// `decode` below is its exact inverse. Not a general-purpose codec, just
/// enough to round-trip what `ai_service::embed_texts` returns.
fn encode(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn decode(bytes: &[u8]) -> Vec<f32> {
    bytes.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

/// Replaces whatever embedding this record already had (if any) - a
/// record is always reindexed wholesale, never partially updated.
pub fn upsert(conn: &Connection, workspace_id: &str, object_key: &str, record_id: &str, embedding: &[f32], model: &str) -> rusqlite::Result<()> {
    let bytes = encode(embedding);
    conn.execute(
        "INSERT INTO record_embeddings (workspace_id, object_key, record_id, embedding, dim, model, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (record_id) DO UPDATE SET
            workspace_id = excluded.workspace_id, object_key = excluded.object_key,
            embedding = excluded.embedding, dim = excluded.dim, model = excluded.model, updated_at = excluded.updated_at",
        params![workspace_id, object_key, record_id, bytes, embedding.len() as i64, model, now_iso()],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, record_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM record_embeddings WHERE record_id = ?1", params![record_id])?;
    Ok(())
}

pub struct StoredEmbedding {
    pub object_key: String,
    pub record_id: String,
    pub embedding: Vec<f32>,
}

/// Every embedded record in a workspace, optionally scoped to one
/// `object_key` - `vector_search_service::semantic_search_records` loads
/// this wholesale and ranks in Rust; see that module's own doc comment on
/// why a brute-force scan is the honest tradeoff here.
pub fn list_for_workspace(conn: &Connection, workspace_id: &str, object_key: Option<&str>) -> rusqlite::Result<Vec<StoredEmbedding>> {
    let rows: Vec<(String, String, Vec<u8>)> = if let Some(object_key) = object_key {
        let mut stmt = conn.prepare("SELECT object_key, record_id, embedding FROM record_embeddings WHERE workspace_id = ?1 AND object_key = ?2")?;
        let mapped = stmt.query_map(params![workspace_id, object_key], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<rusqlite::Result<_>>()?;
        mapped
    } else {
        let mut stmt = conn.prepare("SELECT object_key, record_id, embedding FROM record_embeddings WHERE workspace_id = ?1")?;
        let mapped = stmt.query_map(params![workspace_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<rusqlite::Result<_>>()?;
        mapped
    };
    Ok(rows.into_iter().map(|(object_key, record_id, bytes)| StoredEmbedding { object_key, record_id, embedding: decode(&bytes) }).collect())
}

pub struct PendingEmbedding {
    pub object_key: String,
    pub record_id: String,
}

/// One batch of `pending_record_embeddings`, oldest first -
/// `vector_search_service::drain_pending_embeddings` processes and clears
/// these, the same "enqueue now, drain later" shape
/// `ai_agent_pending_run_repo::list_batch` already established for agent
/// runs.
pub fn list_pending_batch(conn: &Connection, workspace_id: &str, limit: i64) -> rusqlite::Result<Vec<PendingEmbedding>> {
    let mut stmt = conn.prepare("SELECT object_key, record_id FROM pending_record_embeddings WHERE workspace_id = ?1 ORDER BY created_at LIMIT ?2")?;
    let rows = stmt.query_map(params![workspace_id, limit], |r| Ok(PendingEmbedding { object_key: r.get(0)?, record_id: r.get(1)? }))?;
    rows.collect()
}

pub fn delete_pending(conn: &Connection, record_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM pending_record_embeddings WHERE record_id = ?1", params![record_id])?;
    Ok(())
}

/// Total embedded records + total still pending, for the admin Vector
/// Search panel - proves reindexing actually happened rather than
/// silently doing nothing when no provider key is configured.
pub fn counts(conn: &Connection, workspace_id: &str) -> rusqlite::Result<(i64, i64)> {
    let embedded: i64 = conn.query_row("SELECT COUNT(*) FROM record_embeddings WHERE workspace_id = ?1", params![workspace_id], |r| r.get(0))?;
    let pending: i64 = conn.query_row("SELECT COUNT(*) FROM pending_record_embeddings WHERE workspace_id = ?1", params![workspace_id], |r| r.get(0))?;
    Ok((embedded, pending))
}

/// The exact same combined searchable text `record_search_fts`'s own
/// triggers compute (migration 0042) - `primary_name`, `display_number`,
/// then every `is_searchable` custom field's value - for one record, so
/// what gets embedded is what a lexical search over the same record
/// would already match against. `None` when the record doesn't exist (or
/// is archived) - `vector_search_service::drain_pending_embeddings`
/// treats that as "nothing to do", not an error, since a record can be
/// archived or deleted between being enqueued and drained.
pub fn searchable_text(conn: &Connection, record_id: &str) -> rusqlite::Result<Option<(String, String, String, String)>> {
    conn.query_row(
        "SELECT r.workspace_id, r.object_key, r.primary_name,
                r.primary_name || ' ' || r.display_number || ' ' || COALESCE((
                    SELECT GROUP_CONCAT(v.value_text, ' ')
                    FROM custom_field_values v
                    JOIN custom_field_definitions d ON d.id = v.definition_id
                    WHERE v.entity_id = r.id AND d.is_searchable = 1
                ), '')
         FROM custom_records r
         WHERE r.id = ?1 AND r.archived_at IS NULL",
        params![record_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}
