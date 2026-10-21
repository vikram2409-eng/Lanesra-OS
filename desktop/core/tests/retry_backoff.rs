//! WP-0124: retry with backoff and the ADR-0009 error classes.
//!
//! AC-1: 429 then success → the step succeeds on attempt 2 (a real run
//! against a provider that answers 429 once).
//! AC-2: policy denial → no retry (nor validation, configuration, internal).
//! AC-3: backoff timing verified with a controllable clock.
//!
//! Covers: REQ-ORCH-033 (WP-0124 AC-1, AC-2, AC-3). Threats: T-06.

use std::future::Future;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use lanesra_core::domain::{AppError, AppResult, ErrorClass};
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::principals::EffectiveIdentity;
use lanesra_core::services::retry::{self, RetryClock, RetryPolicy};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_service, workspace_service,
};

/// Records every sleep instead of sleeping, and fixes the jitter.
struct TestClock {
    jitter: f64,
    slept: Mutex<Vec<u64>>,
}

impl TestClock {
    fn new(jitter: f64) -> Self {
        Self {
            jitter,
            slept: Mutex::new(Vec::new()),
        }
    }
    fn slept(&self) -> Vec<u64> {
        self.slept.lock().unwrap().clone()
    }
}

impl RetryClock for TestClock {
    fn sleep(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.slept.lock().unwrap().push(ms);
        Box::pin(async {})
    }
    fn jitter_unit(&self) -> f64 {
        self.jitter
    }
}

fn err(class: ErrorClass, code: &'static str) -> AppError {
    AppError::classified(class, code, code)
}

/// Runs `retry` with an operation that always fails with `error`.
async fn always_failing(policy: &RetryPolicy, clock: &TestClock, error: fn() -> AppError) -> usize {
    let calls = AtomicUsize::new(0);
    let result: AppResult<()> = retry::retry(policy, clock, || {
        calls.fetch_add(1, Ordering::SeqCst);
        async move { Err(error()) }
    })
    .await;
    assert!(result.is_err());
    calls.load(Ordering::SeqCst)
}

#[tokio::test]
async fn ac2_policy_denial_is_never_retried() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-2)
    let clock = TestClock::new(1.0);
    let calls = always_failing(&RetryPolicy::default(), &clock, || {
        err(ErrorClass::Policy, "policy.denied")
    })
    .await;
    assert_eq!(calls, 1);
    assert!(clock.slept().is_empty());
}

#[tokio::test]
async fn ac2_validation_configuration_and_internal_errors_are_never_retried() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-2)
    let errors: [fn() -> AppError; 4] = [
        || err(ErrorClass::Validation, "validation.arguments"),
        || err(ErrorClass::Configuration, "config.missing_key"),
        || err(ErrorClass::Internal, "internal"),
        // An unclassified validation error is the validation class too.
        || AppError::Validation("bad request".into()),
    ];
    for error in errors {
        let clock = TestClock::new(1.0);
        assert_eq!(
            always_failing(&RetryPolicy::default(), &clock, error).await,
            1
        );
        assert!(clock.slept().is_empty());
    }
}

#[tokio::test]
async fn ac3_backoff_doubles_from_500ms_with_full_jitter() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-3)
    let policy = RetryPolicy {
        max_attempts: 4,
        ..RetryPolicy::default()
    };
    let transient = || err(ErrorClass::Transient, "provider.unavailable");

    // Jitter at its maximum shows the window itself.
    let clock = TestClock::new(1.0);
    assert_eq!(always_failing(&policy, &clock, transient).await, 4);
    assert_eq!(clock.slept(), [500, 1000, 2000]);

    // Full jitter: the delay is a fraction of the window.
    let clock = TestClock::new(0.5);
    always_failing(&policy, &clock, transient).await;
    assert_eq!(clock.slept(), [250, 500, 1000]);
}

#[tokio::test]
async fn ac3_default_policy_is_three_attempts_capped_at_20s() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-3)
    let clock = TestClock::new(1.0);
    let calls = always_failing(&RetryPolicy::default(), &clock, || {
        err(ErrorClass::Transient, "provider.timeout")
    })
    .await;
    assert_eq!(calls, 3);
    assert_eq!(clock.slept(), [500, 1000]);

    let long = RetryPolicy {
        max_attempts: 9,
        ..RetryPolicy::default()
    };
    let clock = TestClock::new(1.0);
    always_failing(&long, &clock, || {
        err(ErrorClass::Transient, "provider.timeout")
    })
    .await;
    assert_eq!(clock.slept().last(), Some(&20_000));
}

#[tokio::test]
async fn ac3_retry_after_replaces_the_backoff() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-3)
    let clock = TestClock::new(1.0);
    let calls = AtomicUsize::new(0);
    let result = retry::retry(&RetryPolicy::default(), &clock, || {
        let n = calls.fetch_add(1, Ordering::SeqCst);
        async move {
            if n == 0 {
                Err(AppError::Classified {
                    class: ErrorClass::Transient,
                    code: "provider.rate_limited",
                    message: "slow down".into(),
                    retry_after_ms: Some(7000),
                    http_status: Some(429),
                })
            } else {
                Ok("ok")
            }
        }
    })
    .await;
    assert_eq!(result.unwrap(), "ok");
    assert_eq!(clock.slept(), [7000]);
}

// --- AC-1 on a real run ------------------------------------------------

/// A provider that answers each request with the next canned response,
/// repeating the last one. Returns its port and a request counter.
fn spawn_provider(responses: Vec<String>) -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = v.trim().parse().unwrap_or(0);
                }
            }
            let mut buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut buf);
            let n = seen.fetch_add(1, Ordering::SeqCst);
            let response = responses[n.min(responses.len() - 1)].clone();
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, count)
}

fn http(status: &str, extra_headers: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{extra_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn ok_text(text: &str) -> String {
    http(
        "200 OK",
        "",
        &serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string(),
    )
}

async fn run_against(
    responses: Vec<String>,
) -> (lanesra_core::models::ai_agent_pipeline::AiAgentRun, usize) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Retry Co".into(),
            legal_name: None,
            currency_code: "USD".into(),
            locale: "en-US".into(),
            timezone: "UTC".into(),
            default_tax_rate_bp: 0,
            admin_username: "admin".into(),
            admin_display_name: "Admin".into(),
            admin_password: "supersecretpassword".into(),
            load_sample_data: false,
        },
    )
    .unwrap();
    let key = [61u8; 32];
    let (port, requests) = spawn_provider(responses);
    ai_service::save_settings(
        &conn,
        &workspace.id,
        &key,
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            model: "claude-haiku-4-5-20251001".into(),
            api_key: Some("sk-ant-test".into()),
        },
        Some(&admin.id),
    )
    .unwrap();
    let agent = ai_agent_service::create(
        &conn,
        &workspace.id,
        &AiAgentInput {
            name: "Helper".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "You help.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin.id),
    )
    .unwrap();
    let run = ai_orchestration_service::run_triggered(
        &conn,
        &workspace.id,
        &key,
        "agent",
        &agent.id,
        &EffectiveIdentity::interactive(&admin.id),
        "hello",
        Some("manual"),
        None,
        None,
    )
    .await
    .unwrap();
    (run, requests.load(Ordering::SeqCst))
}

#[tokio::test]
async fn ac1_a_429_then_success_succeeds_on_attempt_2() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-1)
    let (run, requests) = run_against(vec![
        http(
            "429 Too Many Requests",
            "Retry-After: 0\r\n",
            r#"{"error":"rate limited"}"#,
        ),
        ok_text("second time lucky"),
    ])
    .await;
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(requests, 2, "the step must succeed on its second attempt");
}

#[tokio::test]
async fn ac2_a_rejected_request_is_not_retried() {
    // Covers: REQ-ORCH-033 (WP-0124 AC-2)
    let (run, requests) = run_against(vec![http(
        "400 Bad Request",
        "",
        r#"{"error":"bad request"}"#,
    )])
    .await;
    assert_eq!(run.status, "failed", "{run:?}");
    assert_eq!(
        requests, 1,
        "a validation error must not be retried or failed over"
    );
}
