//! AI Agent Platform v2, Phase 4 (migration `0063_memory_architecture.sql`):
//! Document RAG - a Knowledge Collection groups Knowledge Sources, each
//! chunked and embedded via the workspace's already-configured provider.
//! See `services::ai_knowledge_service`'s own doc comment for the honest
//! scope line on what "ingestion" means in this pass.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeCollection {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KnowledgeCollectionInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub workspace_id: String,
    pub collection_id: Option<String>,
    pub name: String,
    pub content: String,
    pub status: String,
    pub chunk_count: i64,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KnowledgeSourceInput {
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub collection_id: Option<String>,
}

/// One ranked chunk from `ai_knowledge_service::search_knowledge` - the
/// citation shape this issue's own scope text requires ("retrieved chunks
/// carry source document/record IDs and displayable citations - never an
/// unattributed answer"). `similarity` mirrors `vector_search_service::
/// SemanticSearchHit`'s own convention: cosine similarity, -1..1, higher is
/// a better match.
#[derive(Debug, Clone, serde::Serialize)]
pub struct KnowledgeSearchHit {
    pub source_id: String,
    pub source_name: String,
    pub chunk_index: i64,
    pub content: String,
    pub similarity: f64,
}
