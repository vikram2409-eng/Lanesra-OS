//! AI & Agentic Layer, Phase 1 (see `services::ai_service`'s own doc
//! comment for the full architecture): the bring-your-own-LLM-key
//! settings a workspace configures once, before any later agent feature
//! (MCP server, Activity Timeline summarization, meeting-prep/follow-up/
//! hygiene/reporting agents) can do anything. One row per workspace,
//! matching `models::integration::IntegrationSettings`'s own shape.

use serde::{Deserialize, Serialize};

/// AI & Agentic Layer, Phase 7a adds `"google_gemini"` - Gemini's REST
/// shape genuinely differs from OpenAI's, unlike Groq/Mistral/Ollama/
/// vLLM/llama.cpp, which all already work today through
/// `"openai_compatible"` (they each expose an OpenAI-compatible endpoint)
/// and are documented as such in the admin UI rather than given
/// near-duplicate adapters of their own.
pub const AI_PROVIDERS: &[&str] = &["anthropic", "openai_compatible", "google_gemini"];

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
    /// Phase 7a: the "System" tier of the Gateway's System -> Agent ->
    /// User token-budget hierarchy - `None` means unlimited, matching
    /// every other optional-cap field in this codebase (e.g. custom
    /// field `max_length`). Checked by `ai_gateway_service` before every
    /// agent dispatch, against the real per-day totals in
    /// `ai_token_usage`.
    pub daily_token_budget: Option<i64>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct AiDailyTokenBudgetInput {
    pub daily_token_budget: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiTestResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub message: String,
}

// --- Phase 7a: the Unified AI Gateway --------------------------------------

/// A named provider connection - the same "several named entries, not one
/// workspace-wide row" shape Integration Hub's own Connections already
/// use, so a workspace can configure e.g. a cloud Anthropic key, a cloud
/// Gemini key, and a self-hosted Ollama endpoint simultaneously, and an
/// agent's `AiAgentModelRouting` picks which of these to use per tier.
#[derive(Debug, Clone, Serialize)]
pub struct AiProvider {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model: String,
    pub has_key: bool,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiProviderInput {
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model: String,
    /// Same "leave blank to keep" convention `AiSettingsInput::api_key`
    /// already uses.
    pub api_key: Option<String>,
}

/// An agent's optional routing policy - `None` on `AiAgentDefinition`
/// means "no policy configured, use the workspace's `ai_settings`
/// default exactly like every agent did before this phase." Each tier
/// names an `AiProvider` id (or `None`, also meaning the workspace
/// default) - see `services::ai_gateway_service`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiAgentModelRouting {
    pub primary_provider_id: Option<String>,
    pub fallback_provider_id: Option<String>,
    pub local_fallback_provider_id: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i64>,
    /// The "Agent" tier of the System -> Agent -> User budget hierarchy.
    pub daily_token_budget: Option<i64>,
    /// Sensitive-entity classes (e.g. `"ssn"`, `"banking_credentials"`) -
    /// see `dlp_service::CLASSES` for the full recognized vocabulary -
    /// that force this agent's dispatch straight to
    /// `local_fallback_provider_id`, bypassing the normal primary/
    /// fallback order entirely, whenever a request payload matches one.
    pub force_air_gapped_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiGatewayFailoverEvent {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub served_by: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AiTokenUsageSummary {
    pub today_input_tokens: i64,
    pub today_output_tokens: i64,
    pub daily_token_budget: Option<i64>,
}
