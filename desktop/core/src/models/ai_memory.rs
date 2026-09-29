//! AI Agent Platform v2, Phase 4 (migration `0063_memory_architecture.sql`):
//! the three new, itemized memory types - Session, Working and Entity.
//! Agent Memory (today's `memory_md`/`AiAgentMemorySnapshot`, `models::
//! ai_agent`) is unchanged and lives entirely outside this table - see the
//! migration's own doc comment for why. See `services::ai_memory_service`
//! for exactly what each type is scoped by, how long it lives, and how a
//! write is gated by Phase 2's Policy Engine.

pub const MEMORY_TYPES: &[&str] = &["session", "working", "entity"];

pub const CLASSIFICATIONS: &[&str] = &["standard", "sensitive", "restricted"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub workspace_id: String,
    pub memory_type: String,
    pub agent_id: Option<String>,
    pub session_key: Option<String>,
    pub run_id: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub content: String,
    pub source: String,
    pub confidence: Option<f64>,
    pub classification: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MemoryItemInput {
    pub memory_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub content: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default = "default_classification")]
    pub classification: String,
    /// Seconds from now this item should expire, `None` = never (Entity
    /// Memory's normal case). Session/Working writes that omit this get a
    /// service-level default TTL - see `ai_memory_service::remember`.
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
}

fn default_source() -> String {
    "agent_inference".to_string()
}

fn default_classification() -> String {
    "standard".to_string()
}

/// Which run/session a `remember`/`get_memory` tool call is scoped to -
/// threaded through `chat_service::run_agent_once`/`run_agent_once_with_text`/
/// `execute_agent_tool` from whichever caller actually has one. Both `None`
/// for a caller with no real session or run to scope memory to (e.g. an
/// Orchestration Pipeline step doesn't have a session, only a run) - a
/// `remember` call for a type this context can't scope simply errors with a
/// clear message rather than silently no-oping or guessing a key.
#[derive(Debug, Clone, Default)]
pub struct AgentMemoryContext {
    pub session_key: Option<String>,
    pub run_id: Option<String>,
}
