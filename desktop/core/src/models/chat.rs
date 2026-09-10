//! AI & Agentic Layer, Phase 5: the LLM Chat Assistant. Two modes -
//! `"records"` (any authenticated user; the same 7 tools MCP already
//! exposes over `api_object_service`) and `"admin"` (Administrator
//! only; create/list tools over most of the admin configuration
//! surface). See `services::chat_service`'s own doc comment for the
//! tool-calling loop and the full tool catalog.

use serde::{Deserialize, Serialize};

pub const CHAT_MODES: &[&str] = &["records", "admin"];

#[derive(Debug, Clone, Serialize)]
pub struct ChatConversation {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub mode: String,
    pub created_at: String,
    pub updated_at: String,
}

/// One turn. `tool_calls` is only meaningful for `role == "assistant"`
/// (the raw tool-call requests that turn made, if any); `tool_call_id`
/// is only meaningful for `role == "tool"` (which call this result
/// answers). Both are opaque JSON as far as this struct is concerned -
/// `ai_service::complete_with_tools` is the only place that shape is
/// interpreted, since it's provider-defined, not something this model
/// invents.
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<serde_json::Value>,
    pub tool_call_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageInput {
    pub mode: String,
    pub text: String,
}
