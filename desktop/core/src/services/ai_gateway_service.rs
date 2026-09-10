//! AI & Agentic Layer, Phase 7a: the Unified AI Gateway - the dispatch
//! entry point every agent run (`chat_service::run_agent_once`) now calls
//! instead of going straight to `ai_service::complete_with_tools`.
//!
//! An agent with no `model_routing` policy configured (`agent.model_routing
//! == None`) keeps behaving exactly as it did before this phase - the
//! plain workspace `ai_settings` default, no failover, no per-agent
//! budget - strict backward compatibility with everything Phase 6a/6b
//! shipped. Configuring a routing policy adds, in order:
//!
//! 1. **System-tier budget** (`ai_settings.daily_token_budget`) - checked
//!    for every agent run, routed or not, since it's a workspace-wide
//!    ceiling independent of any one agent's policy.
//! 2. **Agent-tier budget** (`AiAgentModelRouting::daily_token_budget`) -
//!    only when a routing policy exists.
//! 3. **Forced air-gapping** - if the outbound payload matches one of the
//!    agent's `force_air_gapped_for` sensitive-entity classes
//!    (`dlp_service::scan`), dispatch goes straight to `local_fallback`,
//!    skipping `primary`/`fallback` entirely, regardless of whether they
//!    would have succeeded.
//! 4. **Ordered failover** - otherwise, `primary` is tried first, then
//!    `fallback`, then `local_fallback`, in order. Any failure - a
//!    network error, a non-2xx response, a deactivated or deleted
//!    provider, a missing key - advances to the next tier; this is a
//!    deliberate simplification over parsing specific HTTP status codes
//!    into "retryable" vs. not, stated plainly rather than presented as
//!    fine-grained circuit-breaking.
//!
//! Every tier field on `AiAgentModelRouting` is independently optional:
//! `None` for a given tier means "use the workspace `ai_settings` default
//! for this attempt," not "skip this tier" - see the model's own doc
//! comment. Every real dispatch's actual token usage (never estimated) is
//! recorded into `ai_token_usage`; every dispatch that didn't end up
//! served by `primary` - a real failover, or a forced air-gap - is logged
//! to `ai_gateway_failover_events` for the Gateway health view.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ai::AiGatewayFailoverEvent;
use crate::models::ai_agent::AiAgentDefinition;
use crate::models::chat::ChatMessage;
use crate::repositories::{ai_gateway_failover_repo, ai_provider_repo, ai_settings_repo, ai_token_usage_repo, integration_secret_repo};

use super::ai_service::{self, CompletionOutcome, ToolSpec, TokenUsage};
use super::dlp_service;

/// What `dispatch` actually did, beyond the raw model outcome - the
/// caller (`chat_service::run_agent_once`) doesn't need `served_by` today,
/// but Phase 7e's run-trace spans will.
pub struct GatewayOutcome {
    pub outcome: CompletionOutcome,
    pub usage: TokenUsage,
    pub served_by: &'static str,
}

struct ResolvedTier {
    provider: String,
    base_url: Option<String>,
    model: String,
    api_key: String,
}

/// `None` resolves to the workspace's plain `ai_settings` row - see this
/// module's own doc comment on why that's "the default," not "skip."
fn resolve_tier(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], provider_id: Option<&str>) -> AppResult<ResolvedTier> {
    match provider_id {
        Some(pid) => {
            let provider = ai_provider_repo::get(conn, pid)?.ok_or_else(|| AppError::Validation(format!("Configured AI provider '{pid}' no longer exists")))?;
            if !provider.is_active {
                return Err(AppError::Validation(format!("AI provider '{}' is deactivated", provider.name)));
            }
            let secret_id = ai_provider_repo::get_secret_id(conn, pid)?.ok_or_else(|| AppError::Validation(format!("AI provider '{}' has no key configured", provider.name)))?;
            let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
            let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;
            Ok(ResolvedTier { provider: provider.provider, base_url: provider.base_url, model: provider.model, api_key })
        }
        None => {
            let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
            let secret_id = ai_settings_repo::get_secret_id(conn, workspace_id)?.ok_or_else(|| AppError::Validation("Configure an AI provider key first (Admin -> LLM & MCP -> LLM)".into()))?;
            let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::Validation("Stored key not found - reconfigure it".into()))?;
            let api_key = super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?;
            Ok(ResolvedTier { provider: settings.provider, base_url: settings.base_url, model: settings.model, api_key })
        }
    }
}

/// The single dispatch entry point for every agent run - see this
/// module's own doc comment for the full budget/air-gap/failover
/// sequence. `actor` is the same optional acting-user id
/// `run_agent_once` already threads through everywhere else (recorded as
/// the User tier of the token-usage hierarchy - not yet enforced, only
/// visible in the Gateway health view, per `ai_token_usage_repo::
/// today_totals_for_user`'s own doc comment).
pub async fn dispatch(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    agent: &AiAgentDefinition,
    actor: Option<&str>,
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
) -> AppResult<GatewayOutcome> {
    let user_id = actor.unwrap_or("");

    // System tier: applies to every agent run, routed or not.
    let settings = ai_settings_repo::ensure_default(conn, workspace_id)?;
    if let Some(budget) = settings.daily_token_budget {
        let (input, output) = ai_token_usage_repo::today_totals_for_workspace(conn, workspace_id)?;
        if input + output >= budget {
            return Err(AppError::Validation(format!(
                "This workspace's daily AI token budget ({budget}) has been reached - try again tomorrow, or raise it in Admin -> LLM & MCP -> Gateway."
            )));
        }
    }

    let routing = match agent.model_routing.clone() {
        Some(r) => r,
        None => {
            // No routing policy configured - unchanged pre-7a behavior.
            let (outcome, usage) = ai_service::complete_with_tools(conn, workspace_id, master_key, system_prompt, tools, history).await?;
            ai_token_usage_repo::increment(conn, workspace_id, &agent.id, user_id, usage.input_tokens, usage.output_tokens)?;
            return Ok(GatewayOutcome { outcome, usage, served_by: "primary" });
        }
    };

    // Agent tier: only checked once a routing policy exists to check it against.
    if let Some(budget) = routing.daily_token_budget {
        let (input, output) = ai_token_usage_repo::today_totals_for_agent(conn, workspace_id, &agent.id)?;
        if input + output >= budget {
            return Err(AppError::Validation(format!(
                "'{}' has reached its daily AI token budget ({budget}) - try again tomorrow, or raise it in this agent's Model Routing settings.",
                agent.name
            )));
        }
    }

    // Forced air-gapping: scan the outbound payload - the system prompt
    // plus the latest turn's own text, not the whole history (which was
    // already dispatched and accepted on a prior round of this same
    // run) - against this agent's configured sensitive-entity classes.
    let outbound_text = {
        let mut parts = vec![system_prompt.to_string()];
        if let Some(last) = history.last() {
            if let Some(c) = &last.content {
                parts.push(c.clone());
            }
        }
        parts.join("\n")
    };
    let matched_classes = dlp_service::scan(&outbound_text);
    let forced_air_gap = matched_classes.iter().any(|m| routing.force_air_gapped_for.iter().any(|c| c == m));

    let tiers: Vec<(&'static str, Option<&str>)> = if forced_air_gap {
        vec![("local_fallback", routing.local_fallback_provider_id.as_deref())]
    } else {
        vec![
            ("primary", routing.primary_provider_id.as_deref()),
            ("fallback", routing.fallback_provider_id.as_deref()),
            ("local_fallback", routing.local_fallback_provider_id.as_deref()),
        ]
    };

    let mut last_err: Option<AppError> = None;
    for (idx, (tier_name, provider_id)) in tiers.iter().enumerate() {
        let resolved = match resolve_tier(conn, workspace_id, master_key, *provider_id) {
            Ok(r) => r,
            Err(e) => {
                last_err = Some(e);
                continue;
            }
        };
        match ai_service::dispatch_with_tools(&resolved.provider, resolved.base_url.as_deref(), &resolved.model, &resolved.api_key, system_prompt, tools, history).await {
            Ok((outcome, usage)) => {
                ai_token_usage_repo::increment(conn, workspace_id, &agent.id, user_id, usage.input_tokens, usage.output_tokens)?;
                if forced_air_gap {
                    let reason = format!("Forced air-gapped routing: outbound payload matched {}", matched_classes.join(", "));
                    ai_gateway_failover_repo::record(conn, workspace_id, &agent.id, tier_name, &reason)?;
                } else if idx > 0 {
                    let reason = last_err.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "the prior tier failed".into());
                    ai_gateway_failover_repo::record(conn, workspace_id, &agent.id, tier_name, &reason)?;
                }
                return Ok(GatewayOutcome { outcome, usage, served_by: tier_name });
            }
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or_else(|| AppError::Validation("No AI provider tier is configured for this agent".into())))
}

/// The Gateway health view's own data source (Admin -> LLM & MCP ->
/// Gateway) - any authenticated user can read it, same "the numbers
/// aren't the secret, the key is" reasoning `ai_service::get_settings`
/// already documents.
pub fn recent_failover_events(conn: &Connection, workspace_id: &str, limit: i64) -> AppResult<Vec<AiGatewayFailoverEvent>> {
    Ok(ai_gateway_failover_repo::list_recent(conn, workspace_id, limit)?)
}
