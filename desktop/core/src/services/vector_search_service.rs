//! AI & Agentic Layer, Phase 7g (part 2): semantic search over Custom
//! Object records - real embeddings from the workspace's already-
//! configured provider (`ai_service::embed_texts`), stored as plain BLOBs
//! (`record_embedding_repo`), compared by brute-force cosine similarity
//! here. This reverses this project's own long-stated position (see
//! migration 0042_context_layer.sql's doc comment: "no vector database,
//! bm25 is enough at this scale") - a deliberate choice for this pass,
//! made explicit rather than silently walked back. `search_service::
//! search_custom_records` (FTS5 lexical/bm25 ranking) is unchanged and
//! stays the default - `semantic_search_records` below is a second,
//! genuinely-semantic ranking next to it, not a replacement; an agent (or
//! a future search UI) picks whichever fits the question, the same way
//! `get_platform_overview`/`get_object_metadata` are opt-in tools rather
//! than something forced into every answer.
//!
//! No vector database, no `sqlite-vec` or other native extension - at
//! this project's per-workspace-SQLite-file scale, a workspace's Custom
//! Object records number in the thousands, not millions, so loading every
//! embedding for one workspace (or one object) and ranking in Rust is a
//! few milliseconds of real work, not a bottleneck worth a new dependency
//! for. If that stops being true at some future scale, the fix is an
//! index, not a rewrite - `record_embedding_repo`'s own storage shape
//! doesn't change either way.
//!
//! Embeddings are computed asynchronously: a synchronous SQL trigger
//! can't make an HTTP call, so migration 0048's triggers on
//! `custom_records`/`custom_field_values` only ever enqueue a
//! `pending_record_embeddings` row (see that migration's own doc
//! comment). The real provider call happens only when
//! `drain_pending_embeddings` runs - desktop's `job_scheduler` tick,
//! alongside `ai_orchestration_service::drain_pending_runs`, or the
//! admin's own "Reindex now" action (`reindex_workspace`) - the same
//! "enqueue now, drain later" shape that pending-run queue already
//! established.

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::AppResult;
use crate::repositories::record_embedding_repo;

/// One ranked hit from `semantic_search_records` - same shape as
/// `search_service::RankedSearchHit`, `similarity` in place of `score`
/// (cosine similarity, -1..1, **higher** is a better match - the inverse
/// direction of bm25's "lower is better", called out here so a caller
/// never mixes the two up when sorting).
#[derive(Debug, Clone, Serialize)]
pub struct SemanticSearchHit {
    pub object_key: String,
    pub record_id: String,
    pub title: String,
    pub similarity: f64,
}

const MAX_SEMANTIC_RESULTS: i64 = 25;
/// Batches of this size per `embed_texts` call while draining - matches
/// `ai_orchestration_service::drain_pending_runs`'s own `limit` parameter
/// shape (the caller decides how much of the queue to work through per
/// tick), keeping one HTTP request's payload bounded rather than
/// submitting an unbounded batch to whatever provider is configured.
const REINDEX_BATCH_SIZE: i64 = 25;

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

/// Embeds `query`, then ranks every already-embedded record in the
/// workspace (optionally scoped to one `object_key`) by cosine similarity
/// against it - a brute-force scan, not an index lookup; see this
/// module's own doc comment on why that's the honest tradeoff here. A
/// record with no embedding yet (still `pending_record_embeddings`, or
/// never written because no provider key is configured) simply can't be
/// found this way yet - `search_custom_records`'s lexical FTS5 pass is
/// unaffected and keeps working regardless.
pub async fn semantic_search_records(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], query: &str, object_key: Option<&str>, limit: i64) -> AppResult<Vec<SemanticSearchHit>> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let limit = limit.clamp(1, MAX_SEMANTIC_RESULTS) as usize;

    let query_embedding = super::ai_service::embed_texts(conn, workspace_id, master_key, std::slice::from_ref(&query.to_string())).await?;
    let query_vec = query_embedding.into_iter().next().unwrap_or_default();

    let candidates = record_embedding_repo::list_for_workspace(conn, workspace_id, object_key)?;
    let mut scored: Vec<(f64, record_embedding_repo::StoredEmbedding)> = candidates.into_iter().map(|c| (cosine_similarity(&query_vec, &c.embedding), c)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut hits = Vec::with_capacity(limit.min(scored.len()));
    for (similarity, candidate) in scored.into_iter().take(limit) {
        // The title isn't stored on the embedding row itself (it's
        // derived, not part of what's embedded) - a fresh, tiny lookup
        // per hit, cheap at `limit`'s scale (<=25) and always current,
        // rather than a title snapshot that could go stale the moment a
        // record is renamed without a field value actually changing.
        if let Some((_, _, title, _)) = record_embedding_repo::searchable_text(conn, &candidate.record_id)? {
            hits.push(SemanticSearchHit { object_key: candidate.object_key, record_id: candidate.record_id, title, similarity });
        }
    }
    Ok(hits)
}

/// Embeds and stores exactly one record's current searchable text - the
/// unit both `drain_pending_embeddings` and `reindex_workspace` below
/// call per record. `Ok(false)` (not an error) when the record has
/// nothing to embed (deleted, or archived since being enqueued) - the
/// caller still clears its own pending-queue row either way.
async fn reindex_one(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], record_id: &str) -> AppResult<bool> {
    let Some((record_workspace_id, object_key, _title, text)) = record_embedding_repo::searchable_text(conn, record_id)? else {
        record_embedding_repo::delete(conn, record_id)?;
        return Ok(false);
    };
    let embeddings = super::ai_service::embed_texts(conn, workspace_id, master_key, std::slice::from_ref(&text)).await?;
    let Some(embedding) = embeddings.into_iter().next() else {
        return Ok(false);
    };
    let model = super::ai_service::get_settings(conn, workspace_id)?.embedding_model.unwrap_or_default();
    record_embedding_repo::upsert(conn, &record_workspace_id, &object_key, record_id, &embedding, &model)?;
    Ok(true)
}

/// The async drain for what migration 0048's triggers queued - called
/// from `job_scheduler.rs`'s tick (server) and a matching desktop poll,
/// right alongside `ai_orchestration_service::drain_pending_runs`. A
/// single record's failure (no provider key configured, a provider
/// error) stops the batch rather than silently skipping it and clearing
/// its queue row - an admin should see the real error (surfaced by
/// whichever caller checks this function's `Err`), not have records
/// quietly fall out of semantic search with no explanation.
pub async fn drain_pending_embeddings(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], limit: i64) -> AppResult<usize> {
    let batch = record_embedding_repo::list_pending_batch(conn, workspace_id, limit)?;
    let mut drained = 0;
    for item in &batch {
        reindex_one(conn, workspace_id, master_key, &item.record_id).await?;
        record_embedding_repo::delete_pending(conn, &item.record_id)?;
        drained += 1;
    }
    Ok(drained)
}

/// Admin-triggered "Reindex now": drains the full pending queue right
/// away instead of waiting for the next scheduler tick, in
/// `REINDEX_BATCH_SIZE` batches so one call never holds the connection
/// through an unbounded number of provider requests. Same admin gate
/// `ai_orchestration_service::run_manual` uses - this makes real outbound
/// calls to an admin-configured provider, same reasoning.
pub async fn reindex_workspace(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor_user_id: Option<&str>) -> AppResult<usize> {
    super::user_service::require_admin(conn, actor_user_id)?;
    let mut total = 0;
    loop {
        let drained = drain_pending_embeddings(conn, workspace_id, master_key, REINDEX_BATCH_SIZE).await?;
        total += drained;
        if drained < REINDEX_BATCH_SIZE as usize {
            break;
        }
    }
    Ok(total)
}

/// The admin Vector Search panel's own progress readout - proves
/// reindexing actually happened (or that it hasn't, and why: a nonzero
/// `pending_count` with a stuck `embedded_count` usually means no
/// provider key is configured yet) rather than a silent black box.
pub fn status(conn: &Connection, workspace_id: &str) -> AppResult<crate::models::ai::VectorSearchStatus> {
    let (embedded_count, pending_count) = record_embedding_repo::counts(conn, workspace_id)?;
    Ok(crate::models::ai::VectorSearchStatus { embedded_count, pending_count })
}
