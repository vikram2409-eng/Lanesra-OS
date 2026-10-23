//! Scripted model provider for deterministic offline tests (WP-0005, REQ-PRE-003).
//!
//! Compiled only for tests (`cfg(test)` or the `test-support` feature,
//! which CI enables for the test run and no release build enables), so it
//! is never selectable in the product UI or settings API.
//!
//! Tests script a sequence of responses; each `dispatch` call pops the next
//! one. Requests are captured for assertion. Failures (429, timeout, 5xx,
//! malformed JSON) can be injected to test retry logic.
//!
//! Covers: REQ-PRE-003 (WP-0005)

use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

use super::ai_service::{CallOptions, CompletionOutcome, RequestedToolCall, TokenUsage, ToolSpec};
use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::chat::ChatMessage;

/// One scripted response in the queue.
#[derive(Debug, Clone)]
pub enum ScriptedResponse {
    Text {
        text: String,
        usage: TokenUsage,
    },
    ToolCalls {
        calls: Vec<RequestedToolCall>,
        usage: TokenUsage,
    },
    /// Inject a failure. The string names the failure kind:
    /// "rate_limit" (429), "timeout", "server_error" (5xx), "malformed_json".
    Failure {
        kind: String,
    },
    /// Never answer, so a caller's time limit is what ends the call
    /// (WP-0121). Pair with tokio's paused clock to keep the test instant.
    Stall,
}

/// A captured request for test assertions.
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub system_prompt: String,
    pub tools: Vec<ToolSpec>,
    pub history: Vec<ChatMessage>,
    pub model: String,
    /// Temperature, output limit and whether tools were switched off.
    pub options: CallOptions,
}

thread_local! {
    static SCRIPT: RefCell<VecDeque<ScriptedResponse>> = const { RefCell::new(VecDeque::new()) };
    static CAPTURED: RefCell<Vec<CapturedRequest>> = const { RefCell::new(Vec::new()) };
    static LATENCY: RefCell<Duration> = const { RefCell::new(Duration::ZERO) };
}

/// Queue scripted responses. Each `dispatch` pops the next one.
pub fn script_responses(responses: Vec<ScriptedResponse>) {
    SCRIPT.with(|s| {
        let mut q = s.borrow_mut();
        q.clear();
        q.extend(responses);
    });
}

/// Set simulated latency applied to each dispatch.
pub fn set_latency(d: Duration) {
    LATENCY.with(|l| *l.borrow_mut() = d);
}

/// Take the captured requests, clearing the buffer.
pub fn take_captured() -> Vec<CapturedRequest> {
    CAPTURED.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Clear script and captures. Call in test setup.
pub fn reset() {
    SCRIPT.with(|s| s.borrow_mut().clear());
    CAPTURED.with(|c| c.borrow_mut().clear());
    LATENCY.with(|l| *l.borrow_mut() = Duration::ZERO);
}

fn next_response() -> AppResult<ScriptedResponse> {
    SCRIPT.with(|s| {
        s.borrow_mut().pop_front().ok_or_else(|| {
            AppError::classified(
                ErrorClass::Internal,
                "internal",
                "scripted provider: no more scripted responses",
            )
        })
    })
}

/// The dispatch entry point, called from `ai_service::dispatch_with_tools`
/// in test builds when `provider == "scripted"`.
pub async fn dispatch_scripted(
    model: &str,
    system_prompt: &str,
    tools: &[ToolSpec],
    history: &[ChatMessage],
    options: CallOptions,
) -> AppResult<(CompletionOutcome, TokenUsage)> {
    // Capture the request first.
    CAPTURED.with(|c| {
        c.borrow_mut().push(CapturedRequest {
            system_prompt: system_prompt.to_string(),
            tools: tools.to_vec(),
            history: history.to_vec(),
            model: model.to_string(),
            options,
        })
    });

    // Simulated latency.
    let latency = LATENCY.with(|l| *l.borrow());
    if !latency.is_zero() {
        tokio::time::sleep(latency).await;
    }

    match next_response()? {
        ScriptedResponse::Stall => std::future::pending().await,
        ScriptedResponse::Text { text, usage } => Ok((CompletionOutcome::Text(text), usage)),
        ScriptedResponse::ToolCalls { calls, usage } => {
            let raw = serde_json::json!({
                "scripted": true,
                "calls": calls.iter().map(|c| &c.name).collect::<Vec<_>>(),
            });
            Ok((
                CompletionOutcome::ToolCalls {
                    raw_assistant: raw,
                    calls,
                },
                usage,
            ))
        }
        ScriptedResponse::Failure { kind } => {
            // Surface as the ADR-0009 classes and codes a real provider
            // adapter returns, so retry and failover can be tested.
            let provider = |class, code, status, message: &str| AppError::Classified {
                class,
                code,
                message: message.to_string(),
                retry_after_ms: None,
                http_status: status,
            };
            Err(match kind.as_str() {
                "rate_limit" => provider(
                    ErrorClass::Transient,
                    "provider.rate_limited",
                    Some(429),
                    "scripted 429",
                ),
                "timeout" => provider(
                    ErrorClass::Transient,
                    "provider.timeout",
                    None,
                    "scripted timeout",
                ),
                "server_error" => provider(
                    ErrorClass::Transient,
                    "provider.unavailable",
                    Some(503),
                    "scripted 5xx",
                ),
                "malformed_json" => provider(
                    ErrorClass::Validation,
                    "validation.arguments",
                    None,
                    "scripted malformed tool JSON",
                ),
                other => AppError::classified(
                    ErrorClass::Internal,
                    "internal",
                    format!("unknown scripted failure '{other}'"),
                ),
            })
        }
    }
}
