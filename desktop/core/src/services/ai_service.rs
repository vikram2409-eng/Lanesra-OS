//! AI & Agentic Layer, Phase 1: the bring-your-own-LLM-key foundation.
//!
//! Lanesra has no SaaS billing surface to meter inference through the way
//! a hosted "agentic CRM" competitor can - there's no subscription, no
//! seats, no usage plan. The answer here is the same shape every other
//! Lanesra feature already takes: a workspace supplies its own provider
//! key from a new Admin -> AI screen, Lanesra itself never resells,
//! proxies or bills for inference, and the key is stored exactly like any
//! other secret in this codebase - `integration_secrets` +
//! `secret_service::encrypt`/`decrypt` (reused directly, not a second
//! secret store), never returned by any read command (`AiSettings` only
//! ever exposes `has_key`, the same convention
//! `models::integration::Connection::has_secret` already established).
//!
//! Two provider modes for v1, matching what's realistically testable
//! without a live third-party account this environment doesn't have:
//! - `"anthropic"` - a fixed default base URL (overridable, e.g. to point
//!   at a self-hosted proxy in front of the real API - also what makes
//!   this testable against a local stub rather than a live endpoint), a
//!   minimal `max_tokens: 1` `messages.create` call as the connectivity
//!   test. There's no free "just validate this key" endpoint on
//!   Anthropic's API, so this is the standard way to prove a key works -
//!   stated plainly rather than pretending it's free.
//! - `"openai_compatible"` - an admin-supplied base URL (covers OpenAI
//!   itself, and any locally-hosted OpenAI-compatible server - Ollama's
//!   compat layer, vLLM, LM Studio - for a fully offline setup), a
//!   `GET {base_url}/models` call as the test, the standard
//!   OpenAI-compatible convention every one of those servers implements
//!   for free (no generation cost).
//!
//! Deliberately **not** built in Phase 1: any actual chat-completion
//! wrapper - that phase only proved a key works and stored it. Phase 4
//! (Agent Actions) added the generic `complete` primitive below once
//! natural-language reporting needed one; see `complete`'s own doc
//! comment.

use std::time::{Duration, Instant};

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::ai::{AiDailyTokenBudgetInput, AiEmbeddingSettingsInput, AiObservabilitySettingsInput, AiSettings, AiSettingsInput, AiTestResult, AiTokenUsageSummary, AI_PROVIDERS};
use crate::models::chat::ChatMessage;
use crate::repositories::{ai_settings_repo, ai_token_usage_repo, integration_secret_repo};

const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-haiku-4-5-20251001";
const ANTHROPIC_VERSION: &str = "2023-06-01";
// AI & Agentic Layer, Phase 7a: Gemini's own REST API - genuinely
// different from OpenAI's shape (key-in-query-string auth, its own
// `contents`/`functionCall`/`functionResponse` message format), unlike
// Groq/Mistral/Ollama/vLLM/llama.cpp, which all already work today
// through `openai_compatible`.
const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com";
const DEFAULT_GEMINI_MODEL: &str = "gemini-1.5-pro";

/// Real per-call token counts, taken straight from whichever provider's
/// own response reported them - never estimated. `ai_gateway_service`
/// records these into `ai_token_usage` for its System/Agent budget
/// checks; every provider adapter that makes a real completion call
/// returns one alongside its `CompletionOutcome`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokenUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}

/// Read-only - lazily creates the default (`unconfigured`) row on first
/// call, same as `integration_log_service::get_settings` does for
/// `integration_settings`. Never admin-gated on read: any signed-in user
/// can see whether AI is configured (not the key itself), matching how
/// every other Integration Hub settings read works.
pub fn get_settings(conn: &Connection, workspace_id: &str) -> AppResult<AiSettings> {
    Ok(ai_settings_repo::ensure_default(conn, workspace_id)?)
}

/// Admin-only. Setting a new `api_key` rotates the existing stored secret
/// in place (via `integration_secret_repo::rotate`) rather than orphaning
/// it, or creates one for the first time - see `AiSettingsInput::api_key`'s
/// own doc comment for the "leave blank to keep" convention this follows.
/// Any settings change resets `status` back to `unconfigured`: a test
/// result against the old configuration isn't meaningful evidence about
/// the new one.
pub fn save_settings(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], input: &AiSettingsInput, actor_user_id: Option<&str>) -> AppResult<AiSettings> {
    require_admin(conn, actor_user_id)?;
    if !AI_PROVIDERS.contains(&input.provider.as_str()) {
        return Err(AppError::Validation(format!("Unknown AI provider '{}'", input.provider)));
    }
    if input.provider == "openai_compatible" && input.base_url.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::Validation("An OpenAI-compatible provider needs a base URL".into()));
    }
    ai_settings_repo::ensure_default(conn, workspace_id)?;
    let existing_secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?;
    let secret_id = match input.api_key.as_deref().filter(|k| !k.is_empty()) {
        Some(key) => {
            let (ciphertext, nonce) = super::secret_service::encrypt(master_key, key)?;
            match &existing_secret_id {
                Some(id) => {
                    integration_secret_repo::rotate(conn, id, &ciphertext, &nonce)?;
                    Some(id.clone())
                }
                None => {
                    let id = new_uuid();
                    integration_secret_repo::insert(conn, &id, workspace_id, "AI provider API key", &ciphertext, &nonce, actor_user_id)?;
                    Some(id)
                }
            }
        }
        None => existing_secret_id,
    };
    Ok(ai_settings_repo::update(
        conn,
        workspace_id,
        &input.provider,
        input.base_url.as_deref().filter(|u| !u.is_empty()),
        &input.model,
        secret_id.as_deref(),
        actor_user_id,
    )?)
}

/// Admin-only. Makes a real outbound call proving the currently-stored
/// key actually works, and persists the result. Errors (no key configured
/// at all) are distinct from a "failed" test result (a key that's
/// configured but rejected, or a provider that's unreachable) - the
/// former means there's nothing to test yet, the latter is a real,
/// recorded outcome, matching `connection_service::test_connection`'s own
/// `Err` vs. `ok: false` distinction.
pub async fn test_key(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor_user_id: Option<&str>) -> AppResult<AiTestResult> {
    require_admin(conn, actor_user_id)?;
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    let secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?.ok_or_else(|| AppError::Validation("Configure an API key before testing it".into()))?;
    let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
    let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;

    let started = Instant::now();
    let result = match settings.provider.as_str() {
        "anthropic" => test_anthropic(settings.base_url.as_deref(), &settings.model, &api_key).await,
        "openai_compatible" => test_openai_compatible(settings.base_url.as_deref(), &api_key).await,
        "google_gemini" => test_gemini(settings.base_url.as_deref(), &api_key).await,
        other => Err(AppError::Validation(format!("No test implemented for provider '{other}'"))),
    };
    let latency_ms = started.elapsed().as_millis() as u64;

    let test_result = match result {
        Ok(message) => AiTestResult { ok: true, latency_ms, message },
        Err(e) => AiTestResult { ok: false, latency_ms, message: e.to_string() },
    };
    ai_settings_repo::set_test_result(conn, workspace_id, if test_result.ok { "connected" } else { "failed" }, &test_result.message)?;
    Ok(test_result)
}

/// Phase 7a: the "System" tier of the Gateway's budget hierarchy - its
/// own admin action, not folded into `save_settings`, the same "own
/// action, not part of the main form payload" shape `ai_agent_service::
/// set_model_routing` uses for the Agent tier - this is a Gateway health-
/// view dial, not an LLM connection-config field.
pub fn set_daily_token_budget(conn: &Connection, workspace_id: &str, input: &AiDailyTokenBudgetInput, actor_user_id: Option<&str>) -> AppResult<AiSettings> {
    require_admin(conn, actor_user_id)?;
    ai_settings_repo::ensure_default(conn, workspace_id)?;
    ai_settings_repo::set_daily_token_budget(conn, workspace_id, input.daily_token_budget)?;
    Ok(ai_settings_repo::ensure_default(conn, workspace_id)?)
}

/// Phase 7f: where `ai_orchestration_service::push_run_trace_to_otlp`
/// reads the endpoint it POSTs a run's trace to - same "own dial" shape
/// `set_daily_token_budget` above already uses. A blank/whitespace-only
/// value clears it, matching this codebase's "blank means unset"
/// convention rather than storing an empty string as if it were a real
/// endpoint.
pub fn set_otlp_endpoint(conn: &Connection, workspace_id: &str, input: &AiObservabilitySettingsInput, actor_user_id: Option<&str>) -> AppResult<AiSettings> {
    require_admin(conn, actor_user_id)?;
    ai_settings_repo::ensure_default(conn, workspace_id)?;
    let endpoint = input.otlp_endpoint.as_deref().map(str::trim).filter(|s| !s.is_empty());
    ai_settings_repo::set_otlp_endpoint(conn, workspace_id, endpoint)?;
    Ok(ai_settings_repo::ensure_default(conn, workspace_id)?)
}

/// Phase 7g: same shape as `set_otlp_endpoint` above.
pub fn set_embedding_model(conn: &Connection, workspace_id: &str, input: &AiEmbeddingSettingsInput, actor_user_id: Option<&str>) -> AppResult<AiSettings> {
    require_admin(conn, actor_user_id)?;
    ai_settings_repo::ensure_default(conn, workspace_id)?;
    let model = input.embedding_model.as_deref().map(str::trim).filter(|s| !s.is_empty());
    ai_settings_repo::set_embedding_model(conn, workspace_id, model)?;
    Ok(ai_settings_repo::ensure_default(conn, workspace_id)?)
}

/// The System tier's real usage-vs-budget snapshot for the Gateway health
/// view - any authenticated user can see it, same "read needs no more
/// privilege than the numbers behind it" reasoning `get_settings` above
/// already documents.
pub fn token_usage_today(conn: &Connection, workspace_id: &str) -> AppResult<AiTokenUsageSummary> {
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    let (input_tokens, output_tokens) = ai_token_usage_repo::today_totals_for_workspace(conn, workspace_id)?;
    Ok(AiTokenUsageSummary { today_input_tokens: input_tokens, today_output_tokens: output_tokens, daily_token_budget: settings.daily_token_budget })
}

/// AI & Agentic Layer, Phase 4 (Agent Actions): the generic completion
/// primitive this module's own doc comment above said Phase 1 wouldn't
/// build - now something needs it (`agent_service::ask_report`, and
/// later meeting-prep/commitment-capture/hygiene actions once those are
/// scoped). Resolves the stored settings/secret exactly like `test_key`
/// does, then makes a real completion call - not the 1-token
/// connectivity probe `test_anthropic`/`test_openai_compatible` make.
///
/// No admin gate here: *configuring* the key is admin-only
/// (`save_settings`), but *using* an already-configured key to answer a
/// question needs no more privilege than viewing an existing report's
/// numbers already does (`custom_report_service::run`) - the caller
/// decides its own access model, same as every other read-mostly
/// service function in this codebase.
pub async fn complete(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], system_prompt: &str, user_message: &str) -> AppResult<String> {
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    let secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?
        .ok_or_else(|| AppError::Validation("Configure an AI provider key first (Admin -> LLM & MCP -> LLM)".into()))?;
    let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
    let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;

    match settings.provider.as_str() {
        "anthropic" => complete_anthropic(settings.base_url.as_deref(), &settings.model, &api_key, system_prompt, user_message).await,
        "openai_compatible" => complete_openai_compatible(settings.base_url.as_deref(), &settings.model, &api_key, system_prompt, user_message).await,
        "google_gemini" => complete_gemini(settings.base_url.as_deref(), &settings.model, &api_key, system_prompt, user_message).await,
        other => Err(AppError::Validation(format!("No completion implemented for provider '{other}'"))),
    }
}

async fn complete_anthropic(base_url: Option<&str>, model: &str, api_key: &str, system_prompt: &str, user_message: &str) -> AppResult<String> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_ANTHROPIC_BASE_URL);
    let model = if model.trim().is_empty() { DEFAULT_ANTHROPIC_MODEL } else { model };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({
        "model": model, "max_tokens": 1024, "system": system_prompt,
        "messages": [{"role": "user", "content": user_message}],
    });
    let response = client
        .post(format!("{}/v1/messages", base.trim_end_matches('/')))
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    value
        .get("content")
        .and_then(|c| c.get(0))
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Validation(format!("Unexpected response shape from {base}: {}", truncate(&text, 200))))
}

async fn complete_openai_compatible(base_url: Option<&str>, model: &str, api_key: &str, system_prompt: &str, user_message: &str) -> AppResult<String> {
    let base = base_url
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| AppError::Validation("An OpenAI-compatible provider needs a base URL".into()))?;
    if model.trim().is_empty() {
        return Err(AppError::Validation("An OpenAI-compatible provider needs a model name configured".into()));
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "system", "content": system_prompt}, {"role": "user", "content": user_message}],
        "max_tokens": 1024,
    });
    let response = client
        .post(format!("{}/chat/completions", base.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Validation(format!("Unexpected response shape from {base}: {}", truncate(&text, 200))))
}

// --- AI & Agentic Layer, Phase 5 (LLM Chat Assistant): tool-calling ---
//
// `complete` above is one request, one response - enough for Phase 4's
// single JSON directive. A real chat agent needs a genuine back-and-
// forth: the model asks to call one or more tools, the caller executes
// them and feeds the results back, and this repeats until the model
// answers in plain text instead. `complete_with_tools` is one round of
// that exchange; `services::chat_service::send_message` owns the loop
// itself (execute the requested tools, persist the results, call this
// again) and the tool tables both chat modes use.
//
// `history` is the persisted `ChatMessage` row list for a conversation,
// straight from `chat_repo::list_messages` - the provider-specific
// request shape is rebuilt from those rows fresh on every call, rather
// than this module owning any conversation state of its own. A `role:
// "assistant"` row's `tool_calls` field, when present, holds that
// provider's own raw response content verbatim (Anthropic's full
// `content` block array, or an OpenAI-compatible `tool_calls` array) -
// captured once when the response first arrived and echoed straight
// back unchanged, since each provider expects its own prior turns
// exactly as it produced them, not reconstructed from scratch.

/// One tool the model may call. `input_schema` follows the same loose,
/// prose-friendly convention `server/src/mcp.rs`'s own tool definitions
/// already use - a `{"type": "object"}` schema whose `description`
/// tells the model the target Rust struct's real shape, not a
/// hand-authored per-field schema for every admin input type.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RequestedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum CompletionOutcome {
    Text(String),
    ToolCalls { raw_assistant: serde_json::Value, calls: Vec<RequestedToolCall> },
}

/// Same settings/secret resolution as `complete`, then one tool-calling
/// completion call per provider via `dispatch_with_tools` below, using
/// the workspace's plain default settings - the path every call site
/// used before Phase 7a, and still exactly what `chat_service::
/// send_message`'s fixed records/admin chat modes use (no agent, so no
/// routing policy to resolve). No admin gate here either - see
/// `complete`'s own doc comment; `chat_service::send_message` decides
/// its own access model per mode (any authenticated user for records,
/// Administrator for admin).
pub async fn complete_with_tools(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
) -> AppResult<(CompletionOutcome, TokenUsage)> {
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    let secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?
        .ok_or_else(|| AppError::Validation("Configure an AI provider key first (Admin -> LLM & MCP -> LLM)".into()))?;
    let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
    let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;

    dispatch_with_tools(&settings.provider, settings.base_url.as_deref(), &settings.model, &api_key, system_prompt, tools, history).await
}

/// Phase 7a: the provider-matching logic `complete_with_tools` used to
/// inline directly, now its own shared entry point so
/// `ai_gateway_service::dispatch` (the routing/failover/budget layer
/// wrapping this for agent runs) can call it directly with an
/// explicitly-resolved provider/base_url/model/api_key - from a specific
/// `ai_providers` tier row, or the plain workspace `ai_settings` when an
/// agent has no routing policy configured - without duplicating this
/// match arm by arm.
pub(crate) async fn dispatch_with_tools(
    provider: &str,
    base_url: Option<&str>,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
) -> AppResult<(CompletionOutcome, TokenUsage)> {
    match provider {
        "anthropic" => complete_anthropic_with_tools(base_url, model, api_key, system_prompt, tools, history).await,
        "openai_compatible" => complete_openai_with_tools(base_url, model, api_key, system_prompt, tools, history).await,
        "google_gemini" => complete_gemini_with_tools(base_url, model, api_key, system_prompt, tools, history).await,
        other => Err(AppError::Validation(format!("No tool-calling completion implemented for provider '{other}'"))),
    }
}

fn anthropic_tools_json(tools: &[ToolSpec]) -> Vec<serde_json::Value> {
    tools.iter().map(|t| serde_json::json!({"name": t.name, "description": t.description, "input_schema": t.input_schema})).collect()
}

/// Every consecutive `role: "tool"` row answering one assistant turn's
/// requested calls must land in a single Anthropic user message (one
/// `tool_result` block per call) - not one message per result, which
/// the API rejects.
fn anthropic_messages_json(history: &[ChatMessage]) -> Vec<serde_json::Value> {
    let mut messages = Vec::new();
    let mut i = 0;
    while i < history.len() {
        let m = &history[i];
        match m.role.as_str() {
            "assistant" => {
                let content = m.tool_calls.clone().unwrap_or_else(|| serde_json::json!([{"type": "text", "text": m.content.clone().unwrap_or_default()}]));
                messages.push(serde_json::json!({"role": "assistant", "content": content}));
                i += 1;
            }
            "tool" => {
                let mut blocks = Vec::new();
                while i < history.len() && history[i].role == "tool" {
                    let t = &history[i];
                    blocks.push(serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": t.tool_call_id.clone().unwrap_or_default(),
                        "content": t.content.clone().unwrap_or_default(),
                    }));
                    i += 1;
                }
                messages.push(serde_json::json!({"role": "user", "content": blocks}));
            }
            _ => {
                messages.push(serde_json::json!({"role": "user", "content": m.content.clone().unwrap_or_default()}));
                i += 1;
            }
        }
    }
    messages
}

async fn complete_anthropic_with_tools(
    base_url: Option<&str>,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
) -> AppResult<(CompletionOutcome, TokenUsage)> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_ANTHROPIC_BASE_URL);
    let model = if model.trim().is_empty() { DEFAULT_ANTHROPIC_MODEL } else { model };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({
        "model": model, "max_tokens": 2048, "system": system_prompt,
        "tools": anthropic_tools_json(tools),
        "messages": anthropic_messages_json(history),
    });
    let response = client
        .post(format!("{}/v1/messages", base.trim_end_matches('/')))
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    let usage = TokenUsage {
        input_tokens: value.get("usage").and_then(|u| u.get("input_tokens")).and_then(|v| v.as_i64()).unwrap_or(0),
        output_tokens: value.get("usage").and_then(|u| u.get("output_tokens")).and_then(|v| v.as_i64()).unwrap_or(0),
    };
    let content = value.get("content").cloned().unwrap_or(serde_json::Value::Null);
    let blocks = content.as_array().cloned().unwrap_or_default();
    let calls: Vec<RequestedToolCall> = blocks
        .iter()
        .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
        .filter_map(|b| {
            Some(RequestedToolCall {
                id: b.get("id")?.as_str()?.to_string(),
                name: b.get("name")?.as_str()?.to_string(),
                arguments: b.get("input").cloned().unwrap_or(serde_json::json!({})),
            })
        })
        .collect();
    if !calls.is_empty() {
        return Ok((CompletionOutcome::ToolCalls { raw_assistant: content, calls }, usage));
    }
    let text_reply = blocks.iter().filter_map(|b| b.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("\n");
    Ok((CompletionOutcome::Text(text_reply), usage))
}

fn openai_tools_json(tools: &[ToolSpec]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|t| serde_json::json!({"type": "function", "function": {"name": t.name, "description": t.description, "parameters": t.input_schema}}))
        .collect()
}

fn openai_messages_json(system_prompt: &str, history: &[ChatMessage]) -> Vec<serde_json::Value> {
    let mut messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];
    for m in history {
        match m.role.as_str() {
            "assistant" => {
                let mut msg = serde_json::json!({"role": "assistant", "content": m.content});
                if let Some(tool_calls) = &m.tool_calls {
                    msg["tool_calls"] = tool_calls.clone();
                }
                messages.push(msg);
            }
            "tool" => messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": m.tool_call_id.clone().unwrap_or_default(),
                "content": m.content.clone().unwrap_or_default(),
            })),
            _ => messages.push(serde_json::json!({"role": "user", "content": m.content.clone().unwrap_or_default()})),
        }
    }
    messages
}

async fn complete_openai_with_tools(
    base_url: Option<&str>,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
) -> AppResult<(CompletionOutcome, TokenUsage)> {
    let base = base_url
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| AppError::Validation("An OpenAI-compatible provider needs a base URL".into()))?;
    if model.trim().is_empty() {
        return Err(AppError::Validation("An OpenAI-compatible provider needs a model name configured".into()));
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({
        "model": model,
        "messages": openai_messages_json(system_prompt, history),
        "tools": openai_tools_json(tools),
        "max_tokens": 2048,
    });
    let response = client
        .post(format!("{}/chat/completions", base.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    let usage = TokenUsage {
        input_tokens: value.get("usage").and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0),
        output_tokens: value.get("usage").and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0),
    };
    let message = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .cloned()
        .ok_or_else(|| AppError::Validation(format!("Unexpected response shape from {base}: {}", truncate(&text, 200))))?;
    if let Some(tool_calls_value) = message.get("tool_calls").filter(|v| v.is_array()) {
        let calls: Vec<RequestedToolCall> = tool_calls_value
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| {
                let function = c.get("function")?;
                let arguments_str = function.get("arguments")?.as_str()?;
                Some(RequestedToolCall {
                    id: c.get("id")?.as_str()?.to_string(),
                    name: function.get("name")?.as_str()?.to_string(),
                    arguments: serde_json::from_str(arguments_str).unwrap_or(serde_json::json!({})),
                })
            })
            .collect();
        if !calls.is_empty() {
            return Ok((CompletionOutcome::ToolCalls { raw_assistant: tool_calls_value.clone(), calls }, usage));
        }
    }
    let text_reply = message.get("content").and_then(|c| c.as_str()).unwrap_or_default().to_string();
    Ok((CompletionOutcome::Text(text_reply), usage))
}

fn gemini_tools_json(tools: &[ToolSpec]) -> serde_json::Value {
    serde_json::json!([{
        "functionDeclarations": tools.iter().map(|t| serde_json::json!({
            "name": t.name, "description": t.description, "parameters": t.input_schema,
        })).collect::<Vec<_>>(),
    }])
}

/// Gemini's `contents` array uses `role: "model"` for assistant turns and,
/// like Anthropic's `tool_use`/`tool_result` pairing, batches every
/// consecutive `role: "tool"` row answering one model turn's requested
/// calls into a single `role: "user"` turn (one `functionResponse` part
/// per call) rather than one turn per result, which Gemini's API rejects
/// the same way Anthropic's does.
///
/// Gemini's function-calling protocol is name-based, not call-id based -
/// there's no `tool_use_id`/`tool_call_id` concept in its wire format at
/// all. Rather than inventing a parallel id scheme (and a protocol-
/// specific branch in `chat_service.rs`'s otherwise-generic tool-
/// execution loop), the function *name* itself is stored as the
/// synthetic `RequestedToolCall.id`/`ChatMessage.tool_call_id` for
/// Gemini-routed runs (see `complete_gemini_with_tools` below) - reused
/// verbatim here as `functionResponse.name`, which is exactly what
/// Gemini's format actually keys responses by anyway.
fn gemini_contents_json(history: &[ChatMessage]) -> Vec<serde_json::Value> {
    let mut contents = Vec::new();
    let mut i = 0;
    while i < history.len() {
        let m = &history[i];
        match m.role.as_str() {
            "assistant" => {
                let parts = m.tool_calls.clone().unwrap_or_else(|| serde_json::json!([{"text": m.content.clone().unwrap_or_default()}]));
                contents.push(serde_json::json!({"role": "model", "parts": parts}));
                i += 1;
            }
            "tool" => {
                let mut parts = Vec::new();
                while i < history.len() && history[i].role == "tool" {
                    let t = &history[i];
                    let name = t.tool_call_id.clone().unwrap_or_default();
                    let response_text = t.content.clone().unwrap_or_default();
                    parts.push(serde_json::json!({
                        "functionResponse": { "name": name, "response": {"result": response_text} },
                    }));
                    i += 1;
                }
                contents.push(serde_json::json!({"role": "user", "parts": parts}));
            }
            _ => {
                contents.push(serde_json::json!({"role": "user", "parts": [{"text": m.content.clone().unwrap_or_default()}]}));
                i += 1;
            }
        }
    }
    contents
}

async fn complete_gemini_with_tools(
    base_url: Option<&str>,
    model: &str,
    api_key: &str,
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
) -> AppResult<(CompletionOutcome, TokenUsage)> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_GEMINI_BASE_URL);
    let model = if model.trim().is_empty() { DEFAULT_GEMINI_MODEL } else { model };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({
        "contents": gemini_contents_json(history),
        "systemInstruction": {"parts": [{"text": system_prompt}]},
        "tools": gemini_tools_json(tools),
    });
    let response = client
        .post(format!("{}/v1beta/models/{}:generateContent?key={}", base.trim_end_matches('/'), model, api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    let usage = TokenUsage {
        input_tokens: value.get("usageMetadata").and_then(|u| u.get("promptTokenCount")).and_then(|v| v.as_i64()).unwrap_or(0),
        output_tokens: value.get("usageMetadata").and_then(|u| u.get("candidatesTokenCount")).and_then(|v| v.as_i64()).unwrap_or(0),
    };
    let content = value.get("candidates").and_then(|c| c.get(0)).and_then(|c| c.get("content")).cloned().unwrap_or(serde_json::Value::Null);
    let parts = content.get("parts").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    let calls: Vec<RequestedToolCall> = parts
        .iter()
        .filter_map(|p| p.get("functionCall"))
        .filter_map(|fc| {
            let name = fc.get("name")?.as_str()?.to_string();
            Some(RequestedToolCall { id: name.clone(), name, arguments: fc.get("args").cloned().unwrap_or(serde_json::json!({})) })
        })
        .collect();
    if !calls.is_empty() {
        return Ok((CompletionOutcome::ToolCalls { raw_assistant: serde_json::Value::Array(parts), calls }, usage));
    }
    let text_reply = parts.iter().filter_map(|p| p.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("\n");
    Ok((CompletionOutcome::Text(text_reply), usage))
}

async fn test_anthropic(base_url: Option<&str>, model: &str, api_key: &str) -> AppResult<String> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_ANTHROPIC_BASE_URL);
    let model = if model.trim().is_empty() { DEFAULT_ANTHROPIC_MODEL } else { model };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({ "model": model, "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}] });
    let response = client
        .post(format!("{}/v1/messages", base.trim_end_matches('/')))
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Key valid - reached {base} (HTTP {status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))))
    }
}

async fn test_openai_compatible(base_url: Option<&str>, api_key: &str) -> AppResult<String> {
    let base = base_url
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| AppError::Validation("An OpenAI-compatible provider needs a base URL".into()))?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let response = client
        .get(format!("{}/models", base.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Key valid - reached {base} (HTTP {status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))))
    }
}

/// Gemini's key is a query parameter, not a header, so the free
/// connectivity probe is a `GET .../v1beta/models?key=...` list call -
/// the same "free, no generation cost" shape `test_openai_compatible`'s
/// own `GET /models` already uses, not a 1-token generation charge like
/// `test_anthropic` has to fall back to.
async fn test_gemini(base_url: Option<&str>, api_key: &str) -> AppResult<String> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_GEMINI_BASE_URL);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let response = client
        .get(format!("{}/v1beta/models?key={}", base.trim_end_matches('/'), api_key))
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Key valid - reached {base} (HTTP {status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))))
    }
}

async fn complete_gemini(base_url: Option<&str>, model: &str, api_key: &str, system_prompt: &str, user_message: &str) -> AppResult<String> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_GEMINI_BASE_URL);
    let model = if model.trim().is_empty() { DEFAULT_GEMINI_MODEL } else { model };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({
        "contents": [{"role": "user", "parts": [{"text": user_message}]}],
        "systemInstruction": {"parts": [{"text": system_prompt}]},
    });
    let response = client
        .post(format!("{}/v1beta/models/{}:generateContent?key={}", base.trim_end_matches('/'), model, api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    value
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Validation(format!("Unexpected response shape from {base}: {}", truncate(&text, 200))))
}

/// Phase 7a: the same provider-matching shape `dispatch_with_tools`
/// gives the tool-calling path, for the plain connectivity test -
/// `ai_provider_service::test_key` calls this directly for a named
/// `ai_providers` row, exactly like `test_key` above does for the plain
/// workspace `ai_settings` row.
pub(crate) async fn test_provider(provider: &str, base_url: Option<&str>, model: &str, api_key: &str) -> AppResult<String> {
    match provider {
        "anthropic" => test_anthropic(base_url, model, api_key).await,
        "openai_compatible" => test_openai_compatible(base_url, api_key).await,
        "google_gemini" => test_gemini(base_url, api_key).await,
        other => Err(AppError::Validation(format!("No test implemented for provider '{other}'"))),
    }
}

// --- AI & Agentic Layer, Phase 7g: embeddings for vector search ---------
//
// Only "openai_compatible"/"google_gemini" expose an embeddings endpoint
// - Anthropic's API has none at all, so a workspace configured for it
// gets one clear, named error the moment reindexing/searching is
// attempted, not a confusing HTTP failure partway through. `embed_texts`
// batches everything it can in one request (openai_compatible's own
// array `input`; Gemini's `batchEmbedContents`) so reindexing many
// records doesn't mean one round trip per record.

const DEFAULT_OPENAI_EMBEDDING_MODEL: &str = "text-embedding-3-small";
const DEFAULT_GEMINI_EMBEDDING_MODEL: &str = "text-embedding-004";

/// The effective embedding model name - `settings.embedding_model` when
/// an admin has set one (`set_embedding_model`), else this per-provider
/// default. Deliberately never `settings.model` - that's the *chat*
/// model, not necessarily an embeddings-capable one.
fn default_embedding_model(provider: &str) -> &'static str {
    match provider {
        "google_gemini" => DEFAULT_GEMINI_EMBEDDING_MODEL,
        _ => DEFAULT_OPENAI_EMBEDDING_MODEL,
    }
}

/// Real embedding vectors for `texts`, in the same order - the one place
/// both `vector_search_service::drain_pending_embeddings` (reindexing)
/// and `semantic_search_records` (embedding the query itself) funnel
/// through, the same "one place every call funnels through" shape
/// `ai_orchestration_service::run_step` already established for agent
/// calls. Resolves settings/secret exactly like `complete` above.
pub async fn embed_texts(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    let secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?
        .ok_or_else(|| AppError::Validation("Configure an AI provider key first (Admin -> LLM & MCP -> LLM)".into()))?;
    let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
    let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;
    let model = settings.embedding_model.as_deref().filter(|m| !m.trim().is_empty()).unwrap_or_else(|| default_embedding_model(&settings.provider)).to_string();

    match settings.provider.as_str() {
        "openai_compatible" => embed_openai_compatible(settings.base_url.as_deref(), &model, &api_key, texts).await,
        "google_gemini" => embed_gemini(settings.base_url.as_deref(), &model, &api_key, texts).await,
        "anthropic" => Err(AppError::Validation(
            "Anthropic has no embeddings API - configure an OpenAI-compatible or Google Gemini provider (Admin -> LLM & MCP -> LLM) to use vector search".into(),
        )),
        other => Err(AppError::Validation(format!("No embeddings implemented for provider '{other}'"))),
    }
}

async fn embed_openai_compatible(base_url: Option<&str>, model: &str, api_key: &str, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
    let base = base_url.filter(|u| !u.trim().is_empty()).ok_or_else(|| AppError::Validation("An OpenAI-compatible provider needs a base URL".into()))?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let body = serde_json::json!({"model": model, "input": texts});
    let response = client
        .post(format!("{}/embeddings", base.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    let data = value.get("data").and_then(|d| d.as_array()).ok_or_else(|| AppError::Validation(format!("Unexpected response shape from {base}: {}", truncate(&text, 200))))?;
    data.iter()
        .map(|d| {
            d.get("embedding")
                .and_then(|e| e.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_f64()).map(|v| v as f32).collect::<Vec<f32>>())
                .ok_or_else(|| AppError::Validation(format!("Unexpected embedding shape from {base}: {}", truncate(&text, 200))))
        })
        .collect()
}

async fn embed_gemini(base_url: Option<&str>, model: &str, api_key: &str, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
    let base = base_url.filter(|u| !u.is_empty()).unwrap_or(DEFAULT_GEMINI_BASE_URL);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let requests: Vec<serde_json::Value> = texts.iter().map(|t| serde_json::json!({"model": format!("models/{model}"), "content": {"parts": [{"text": t}]}})).collect();
    let body = serde_json::json!({"requests": requests});
    let response = client
        .post(format!("{}/v1beta/models/{}:batchEmbedContents?key={}", base.trim_end_matches('/'), model, api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {base}: {e}")))?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Validation(format!("{base} responded with HTTP {status}: {}", truncate(&text, 200))));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| AppError::Validation(format!("Could not parse {base}'s response: {e}")))?;
    let embeddings = value.get("embeddings").and_then(|e| e.as_array()).ok_or_else(|| AppError::Validation(format!("Unexpected response shape from {base}: {}", truncate(&text, 200))))?;
    embeddings
        .iter()
        .map(|e| {
            e.get("values")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_f64()).map(|v| v as f32).collect::<Vec<f32>>())
                .ok_or_else(|| AppError::Validation(format!("Unexpected embedding shape from {base}: {}", truncate(&text, 200))))
        })
        .collect()
}
