#![cfg(feature = "test-support")]
//! Scripted model provider tests (WP-0005, REQ-PRE-003).
//!
//! Covers: REQ-PRE-003 (WP-0005 AC-1, AC-2). AC-3 is in scripted_provider_gate.rs.

use lanesra_core::domain::{AppError, ErrorClass};
use lanesra_core::services::ai_service::CompletionOutcome;
use lanesra_core::services::ai_service::RequestedToolCall;
use lanesra_core::services::ai_service::TokenUsage;
use lanesra_core::services::ai_service::ToolSpec;
use lanesra_core::services::scripted_provider::{self, CapturedRequest, ScriptedResponse};

fn tool(name: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        description: format!("test tool {name}"),
        input_schema: serde_json::json!({"type": "object"}),
    }
}

fn usage() -> TokenUsage {
    TokenUsage {
        input_tokens: 10,
        output_tokens: 20,
    }
}

fn call(id: &str, name: &str) -> RequestedToolCall {
    RequestedToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments: serde_json::json!({"q": "test"}),
    }
}

async fn dispatch_scripted(
    script: Vec<ScriptedResponse>,
) -> lanesra_core::domain::AppResult<(CompletionOutcome, TokenUsage)> {
    scripted_provider::reset();
    scripted_provider::script_responses(script);
    lanesra_core::services::ai_service::dispatch_with_tools_test(
        "scripted",
        None,
        "scripted-test-model",
        "not-a-real-key",
        "you are a test assistant",
        &[tool("search"), tool("calculator")],
        &[],
    )
    .await
}

/// AC-1: A test can script a 3-round tool-calling exchange and assert the
/// exact tool calls executed.
/// Covers: REQ-PRE-003 (WP-0005 AC-1)
#[tokio::test]
async fn three_round_tool_calling_exchange() {
    let script = vec![
        ScriptedResponse::ToolCalls {
            calls: vec![call("c1", "search")],
            usage: usage(),
        },
        ScriptedResponse::ToolCalls {
            calls: vec![call("c2", "calculator"), call("c3", "search")],
            usage: usage(),
        },
        ScriptedResponse::Text {
            text: "done".to_string(),
            usage: usage(),
        },
    ];
    scripted_provider::reset();
    scripted_provider::script_responses(script);

    // Round 1
    let (o1, _) = lanesra_core::services::ai_service::dispatch_with_tools_test(
        "scripted",
        None,
        "m",
        "k",
        "sys",
        &[tool("search")],
        &[],
    )
    .await
    .expect("round 1");
    match o1 {
        CompletionOutcome::ToolCalls { calls, .. } => {
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].name, "search");
            assert_eq!(calls[0].id, "c1");
        }
        other => panic!("expected tool calls, got {other:?}"),
    }

    // Round 2
    let (o2, _) = lanesra_core::services::ai_service::dispatch_with_tools_test(
        "scripted",
        None,
        "m",
        "k",
        "sys",
        &[tool("search"), tool("calculator")],
        &[],
    )
    .await
    .expect("round 2");
    match o2 {
        CompletionOutcome::ToolCalls { calls, .. } => {
            assert_eq!(calls.len(), 2);
            assert_eq!(calls[0].name, "calculator");
            assert_eq!(calls[1].name, "search");
        }
        other => panic!("expected tool calls, got {other:?}"),
    }

    // Round 3: final text
    let (o3, u3) = lanesra_core::services::ai_service::dispatch_with_tools_test(
        "scripted",
        None,
        "m",
        "k",
        "sys",
        &[],
        &[],
    )
    .await
    .expect("round 3");
    match o3 {
        CompletionOutcome::Text(t) => assert_eq!(t, "done"),
        other => panic!("expected text, got {other:?}"),
    }
    assert_eq!(u3.input_tokens + u3.output_tokens, 30);

    // Request capture: all three requests recorded with tools and prompt.
    let captured: Vec<CapturedRequest> = scripted_provider::take_captured();
    assert_eq!(captured.len(), 3);
    assert_eq!(captured[0].system_prompt, "sys");
    assert_eq!(captured[0].tools.len(), 1);
}

/// AC-2: Injected 429, timeout and 5xx errors surface as the ADR-0009
/// classes and codes a real provider adapter returns, so retry and
/// failover can be tested against them.
/// Covers: REQ-PRE-003 (WP-0005 AC-2)
#[tokio::test]
async fn injected_failures_surface_as_gateway_errors() {
    for (kind, class, code, status) in [
        (
            "rate_limit",
            ErrorClass::Transient,
            "provider.rate_limited",
            Some(429),
        ),
        ("timeout", ErrorClass::Transient, "provider.timeout", None),
        (
            "server_error",
            ErrorClass::Transient,
            "provider.unavailable",
            Some(503),
        ),
        (
            "malformed_json",
            ErrorClass::Validation,
            "validation.arguments",
            None,
        ),
    ] {
        let err = dispatch_scripted(vec![ScriptedResponse::Failure {
            kind: kind.to_string(),
        }])
        .await
        .expect_err("expected an injected failure");
        assert_eq!(err.class(), class, "{kind}: {err:?}");
        assert_eq!(err.code(), Some(code), "{kind}: {err:?}");
        match err {
            AppError::Classified { http_status, .. } => assert_eq!(http_status, status, "{kind}"),
            other => panic!("{kind}: expected Classified, got {other:?}"),
        }
    }
}
