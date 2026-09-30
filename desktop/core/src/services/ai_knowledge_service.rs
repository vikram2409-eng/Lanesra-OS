//! AI Agent Platform v2, Phase 4 (GitHub issue #169): Document RAG -
//! ingestion -> chunking -> embeddings -> retrieval, built on the exact
//! provider call `vector_search_service` already established
//! (`ai_service::embed_texts`), not a second embeddings pipeline.
//!
//! **Honest scope line**: "ingestion" in this pass means an admin (or a
//! future programmatic caller) supplies the source's full text directly -
//! there is no file upload / binary parsing (PDF, DOCX, ...) in this
//! codebase yet, so that's not silently half-built here either. A
//! `KnowledgeSource.content` update re-chunks and re-embeds in place
//! (`update_source`), which is what "re-index on version change" means
//! for a text source with no separate file version to track.
//!
//! Unlike `vector_search_service`'s enqueue-now/drain-later shape (a SQL
//! trigger can't make an HTTP call), creating or updating a Knowledge
//! Source is already an explicit, already-async admin action - so chunking
//! and embedding happen synchronously, as one atomic step: either the
//! whole source is created/updated with real embeddings, or nothing is
//! persisted and the admin sees the real error and can retry. No pending
//! queue table exists for this table for exactly that reason.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::ai_knowledge::{KnowledgeCollection, KnowledgeCollectionInput, KnowledgeSearchHit, KnowledgeSource, KnowledgeSourceInput};
use crate::repositories::ai_knowledge_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// A chunk's target size in characters, with overlap so a fact spanning a
/// chunk boundary is still fully present in at least one chunk. Splits on
/// whitespace so a chunk never cuts a word in half.
const CHUNK_SIZE_CHARS: usize = 800;
const CHUNK_OVERLAP_CHARS: usize = 100;
const MAX_SEARCH_RESULTS: i64 = 25;

/// Splits `text` into overlapping, word-boundary-respecting windows - no
/// NLP/sentence-boundary library, a deliberately simple approach that's
/// good enough for retrieval at this project's scale (see this module's
/// own top doc comment on the equally deliberate "no vector database"
/// choice `vector_search_service` already made for the same reason).
fn chunk_text(text: &str) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < words.len() {
        let mut end = start;
        let mut len = 0usize;
        while end < words.len() && (len == 0 || len + 1 + words[end].len() <= CHUNK_SIZE_CHARS) {
            len += if len == 0 { words[end].len() } else { 1 + words[end].len() };
            end += 1;
        }
        chunks.push(words[start..end].join(" "));
        if end >= words.len() {
            break;
        }
        // Step back by roughly CHUNK_OVERLAP_CHARS worth of words so the
        // next chunk overlaps this one, then always advance at least one
        // word to guarantee forward progress.
        let mut back = 0usize;
        let mut back_len = 0usize;
        while back < end - start && back_len < CHUNK_OVERLAP_CHARS {
            back += 1;
            back_len += 1 + words[end - back].len();
        }
        start = (end - back).max(start + 1);
    }
    chunks
}

async fn chunk_and_embed(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], content: &str) -> AppResult<Vec<(String, Vec<f32>, String)>> {
    let chunks = chunk_text(content);
    if chunks.is_empty() {
        return Ok(Vec::new());
    }
    let embeddings = super::ai_service::embed_texts(conn, workspace_id, master_key, &chunks).await?;
    let model = super::ai_service::get_settings(conn, workspace_id)?.embedding_model.unwrap_or_default();
    Ok(chunks.into_iter().zip(embeddings).map(|(content, embedding)| (content, embedding, model.clone())).collect())
}

// --- Collections -----------------------------------------------------------

pub fn create_collection(conn: &Connection, workspace_id: &str, input: &KnowledgeCollectionInput, actor_user_id: Option<&str>) -> AppResult<KnowledgeCollection> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_knowledge_repo::create_collection(conn, workspace_id, input, actor_user_id)?)
}

pub fn update_collection(conn: &Connection, workspace_id: &str, id: &str, input: &KnowledgeCollectionInput, actor_user_id: Option<&str>) -> AppResult<KnowledgeCollection> {
    require_admin(conn, actor_user_id)?;
    get_owned_collection(conn, workspace_id, id)?;
    ai_knowledge_repo::update_collection(conn, id, input, actor_user_id)?;
    get_owned_collection(conn, workspace_id, id)
}

fn get_owned_collection(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<KnowledgeCollection> {
    let c = ai_knowledge_repo::get_collection(conn, id)?.ok_or_else(|| crate::domain::AppError::NotFound("Knowledge collection".into()))?;
    if c.workspace_id != workspace_id {
        return Err(crate::domain::AppError::NotFound("Knowledge collection".into()));
    }
    Ok(c)
}

pub fn list_collections(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<KnowledgeCollection>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_knowledge_repo::list_collections(conn, workspace_id)?)
}

pub fn delete_collection(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned_collection(conn, workspace_id, id)?;
    ai_knowledge_repo::delete_collection(conn, id)?;
    Ok(())
}

// --- Sources -----------------------------------------------------------

pub async fn create_source(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], input: &KnowledgeSourceInput, actor_user_id: Option<&str>) -> AppResult<KnowledgeSource> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(crate::domain::AppError::Validation("name is required".into()));
    }
    if input.content.trim().is_empty() {
        return Err(crate::domain::AppError::Validation("content is required".into()));
    }
    let chunks = chunk_and_embed(conn, workspace_id, master_key, &input.content).await?;
    let source = ai_knowledge_repo::create_source(conn, workspace_id, input, chunks.len() as i64, actor_user_id)?;
    ai_knowledge_repo::replace_chunks(conn, &source.id, workspace_id, &chunks)?;
    Ok(source)
}

/// Re-chunks and re-embeds the whole source in place - what "re-index on
/// version change" means for this pass's text-only ingestion (see this
/// module's own top doc comment).
pub async fn update_source(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], id: &str, input: &KnowledgeSourceInput, actor_user_id: Option<&str>) -> AppResult<KnowledgeSource> {
    require_admin(conn, actor_user_id)?;
    get_owned_source(conn, workspace_id, id)?;
    if input.content.trim().is_empty() {
        return Err(crate::domain::AppError::Validation("content is required".into()));
    }
    let chunks = chunk_and_embed(conn, workspace_id, master_key, &input.content).await?;
    ai_knowledge_repo::update_source(conn, id, input, chunks.len() as i64, actor_user_id)?;
    ai_knowledge_repo::replace_chunks(conn, id, workspace_id, &chunks)?;
    get_owned_source(conn, workspace_id, id)
}

fn get_owned_source(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<KnowledgeSource> {
    let s = ai_knowledge_repo::get_source(conn, id)?.ok_or_else(|| crate::domain::AppError::NotFound("Knowledge source".into()))?;
    if s.workspace_id != workspace_id {
        return Err(crate::domain::AppError::NotFound("Knowledge source".into()));
    }
    Ok(s)
}

pub fn get_source(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<KnowledgeSource> {
    require_admin(conn, actor_user_id)?;
    get_owned_source(conn, workspace_id, id)
}

pub fn list_sources(conn: &Connection, workspace_id: &str, collection_id: Option<&str>, actor_user_id: Option<&str>) -> AppResult<Vec<KnowledgeSource>> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_knowledge_repo::list_sources(conn, workspace_id, collection_id)?)
}

pub fn delete_source(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned_source(conn, workspace_id, id)?;
    ai_knowledge_repo::delete_source(conn, id)?;
    Ok(())
}

/// Embeds `query`, then ranks every already-embedded chunk in the
/// workspace (optionally scoped to one collection) by cosine similarity -
/// the same brute-force-in-Rust tradeoff `vector_search_service::
/// semantic_search_records` already makes at this project's scale. Every
/// hit carries `source_id`/`source_name`/`chunk_index` - the citation this
/// issue's own scope text requires ("never an unattributed answer").
pub async fn search_knowledge(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], query: &str, collection_id: Option<&str>, limit: i64) -> AppResult<Vec<KnowledgeSearchHit>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let limit = limit.clamp(1, MAX_SEARCH_RESULTS) as usize;

    let query_embedding = super::ai_service::embed_texts(conn, workspace_id, master_key, std::slice::from_ref(&query.to_string())).await?;
    let query_vec = query_embedding.into_iter().next().unwrap_or_default();

    let candidates = ai_knowledge_repo::list_chunks_for_workspace(conn, workspace_id, collection_id)?;
    let mut scored: Vec<(f64, ai_knowledge_repo::StoredChunk)> = candidates.into_iter().map(|c| (cosine_similarity(&query_vec, &c.embedding), c)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut hits = Vec::with_capacity(limit.min(scored.len()));
    for (similarity, chunk) in scored.into_iter().take(limit) {
        if let Some(source) = ai_knowledge_repo::get_source(conn, &chunk.source_id)? {
            hits.push(KnowledgeSearchHit { source_id: chunk.source_id, source_name: source.name, chunk_index: chunk.chunk_index, content: chunk.content, similarity });
        }
    }
    Ok(hits)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)) as f64
}
