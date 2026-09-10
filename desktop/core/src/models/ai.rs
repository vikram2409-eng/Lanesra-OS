//! AI & Agentic Layer, Phase 1 (see `services::ai_service`'s own doc
//! comment for the full architecture): the bring-your-own-LLM-key
//! settings a workspace configures once, before any later agent feature
//! (MCP server, Activity Timeline summarization, meeting-prep/follow-up/
//! hygiene/reporting agents) can do anything. One row per workspace,
//! matching `models::integration::IntegrationSettings`'s own shape.

use serde::{Deserialize, Serialize};

pub const AI_PROVIDERS: &[&str] = &["anthropic", "openai_compatible"];

#[derive(Debug, Clone, Serialize)]
pub struct AiSettings {
    pub workspace_id: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model: String,
    /// Never populated with the real key - only whether one exists, the
    /// same convention `models::integration::Connection::has_secret`
    /// already uses for exactly this reason.
    pub has_key: bool,
    pub status: String,
    pub last_test_message: Option<String>,
    pub last_tested_at: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiSettingsInput {
    pub provider: String,
    pub base_url: Option<String>,
    pub model: String,
    /// `None` or empty keeps whatever key is already stored (rotates it in
    /// place via `secret_service`/`integration_secret_repo::rotate` rather
    /// than orphaning the old secret row) - the same "leave blank to keep"
    /// convention the Connections admin screen already uses. Provide a
    /// value to set a key for the first time or replace it.
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiTestResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub message: String,
}
