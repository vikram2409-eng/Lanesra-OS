//! Raw CRUD for `ai_knowledge_collections`/`ai_knowledge_sources`/
//! `ai_knowledge_chunks` (migration `0063_memory_architecture.sql`). Same
//! `f32`-vector BLOB codec `record_embedding_repo` already established for
//! `record_embeddings` - see this module's own `encode`/`decode`, kept as
//! an exact copy rather than a shared helper so either table's storage
//! shape can change independently later.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_knowledge::{KnowledgeCollection, KnowledgeCollectionInput, KnowledgeSource, KnowledgeSourceInput};

fn encode(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn decode(bytes: &[u8]) -> Vec<f32> {
    bytes.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

// --- Collections -----------------------------------------------------------

fn map_collection_row(row: &rusqlite::Row) -> rusqlite::Result<KnowledgeCollection> {
    Ok(KnowledgeCollection {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn create_collection(conn: &Connection, workspace_id: &str, input: &KnowledgeCollectionInput, actor_user_id: Option<&str>) -> rusqlite::Result<KnowledgeCollection> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_knowledge_collections (id, workspace_id, name, description, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, ?6)",
        rusqlite::params![id, workspace_id, input.name, input.description, now, actor_user_id],
    )?;
    get_collection(conn, &id).map(|r| r.expect("just inserted"))
}

pub fn update_collection(conn: &Connection, id: &str, input: &KnowledgeCollectionInput, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_knowledge_collections SET name = ?1, description = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?5",
        rusqlite::params![input.name, input.description, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn get_collection(conn: &Connection, id: &str) -> rusqlite::Result<Option<KnowledgeCollection>> {
    conn.query_row("SELECT * FROM ai_knowledge_collections WHERE id = ?1", [id], map_collection_row).optional()
}

pub fn list_collections(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<KnowledgeCollection>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_knowledge_collections WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_collection_row)?.collect();
    rows
}

pub fn delete_collection(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_knowledge_collections WHERE id = ?1", [id])?;
    Ok(())
}

// --- Sources -----------------------------------------------------------

fn map_source_row(row: &rusqlite::Row) -> rusqlite::Result<KnowledgeSource> {
    Ok(KnowledgeSource {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        collection_id: row.get("collection_id")?,
        name: row.get("name")?,
        content: row.get("content")?,
        status: row.get("status")?,
        chunk_count: row.get("chunk_count")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn create_source(conn: &Connection, workspace_id: &str, input: &KnowledgeSourceInput, chunk_count: i64, actor_user_id: Option<&str>) -> rusqlite::Result<KnowledgeSource> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_knowledge_sources (id, workspace_id, collection_id, name, content, status, chunk_count, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, 'indexed', ?6, ?7, ?8, ?7, ?8)",
        rusqlite::params![id, workspace_id, input.collection_id, input.name, input.content, chunk_count, now, actor_user_id],
    )?;
    get_source(conn, &id).map(|r| r.expect("just inserted"))
}

pub fn update_source(conn: &Connection, id: &str, input: &KnowledgeSourceInput, chunk_count: i64, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_knowledge_sources SET collection_id = ?1, name = ?2, content = ?3, status = 'indexed', chunk_count = ?4, updated_at = ?5, updated_by = ?6 WHERE id = ?7",
        rusqlite::params![input.collection_id, input.name, input.content, chunk_count, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn get_source(conn: &Connection, id: &str) -> rusqlite::Result<Option<KnowledgeSource>> {
    conn.query_row("SELECT * FROM ai_knowledge_sources WHERE id = ?1", [id], map_source_row).optional()
}

pub fn list_sources(conn: &Connection, workspace_id: &str, collection_id: Option<&str>) -> rusqlite::Result<Vec<KnowledgeSource>> {
    match collection_id {
        Some(cid) => {
            let mut stmt = conn.prepare("SELECT * FROM ai_knowledge_sources WHERE workspace_id = ?1 AND collection_id = ?2 ORDER BY name")?;
            let rows = stmt.query_map(rusqlite::params![workspace_id, cid], map_source_row)?.collect();
            rows
        }
        None => {
            let mut stmt = conn.prepare("SELECT * FROM ai_knowledge_sources WHERE workspace_id = ?1 ORDER BY name")?;
            let rows = stmt.query_map([workspace_id], map_source_row)?.collect();
            rows
        }
    }
}

pub fn delete_source(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_knowledge_sources WHERE id = ?1", [id])?;
    Ok(())
}

// --- Chunks -----------------------------------------------------------

/// Deletes whatever chunks a source already had and inserts the freshly
/// chunked+embedded replacement set - a source is always re-indexed
/// wholesale on update, the same "replace-all" shape `execution_graph_repo
/// ::replace_nodes_and_edges` already established for a draft graph's
/// nodes/edges.
pub fn replace_chunks(conn: &Connection, source_id: &str, workspace_id: &str, chunks: &[(String, Vec<f32>, String)]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_knowledge_chunks WHERE source_id = ?1", [source_id])?;
    let now = now_iso();
    for (i, (content, embedding, model)) in chunks.iter().enumerate() {
        let bytes = encode(embedding);
        conn.execute(
            "INSERT INTO ai_knowledge_chunks (id, source_id, workspace_id, chunk_index, content, embedding, dim, model, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![new_uuid(), source_id, workspace_id, i as i64, content, bytes, embedding.len() as i64, model, now],
        )?;
    }
    Ok(())
}

pub struct StoredChunk {
    pub source_id: String,
    pub chunk_index: i64,
    pub content: String,
    pub embedding: Vec<f32>,
}

/// Every embedded chunk in a workspace, optionally scoped to sources in
/// one collection - `ai_knowledge_service::search_knowledge` loads this
/// wholesale and ranks in Rust, the same brute-force-cosine tradeoff
/// `vector_search_service::semantic_search_records` already makes at this
/// project's scale.
pub fn list_chunks_for_workspace(conn: &Connection, workspace_id: &str, collection_id: Option<&str>) -> rusqlite::Result<Vec<StoredChunk>> {
    let rows: Vec<(String, i64, String, Vec<u8>)> = match collection_id {
        Some(cid) => {
            let mut stmt = conn.prepare(
                "SELECT c.source_id, c.chunk_index, c.content, c.embedding FROM ai_knowledge_chunks c
                 JOIN ai_knowledge_sources s ON s.id = c.source_id
                 WHERE c.workspace_id = ?1 AND s.collection_id = ?2 AND c.embedding IS NOT NULL",
            )?;
            let mapped = stmt.query_map(rusqlite::params![workspace_id, cid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?.collect::<rusqlite::Result<_>>()?;
            mapped
        }
        None => {
            let mut stmt = conn.prepare("SELECT source_id, chunk_index, content, embedding FROM ai_knowledge_chunks WHERE workspace_id = ?1 AND embedding IS NOT NULL")?;
            let mapped = stmt.query_map([workspace_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?.collect::<rusqlite::Result<_>>()?;
            mapped
        }
    };
    Ok(rows.into_iter().map(|(source_id, chunk_index, content, bytes)| StoredChunk { source_id, chunk_index, content, embedding: decode(&bytes) }).collect())
}
