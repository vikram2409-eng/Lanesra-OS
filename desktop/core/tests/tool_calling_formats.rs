//! WP-0122: native function calling and the strict JSON tool format.
//!
//! AC-1: a tool call round-trips through each provider adapter. A local
//! server speaks each provider's wire format; the test checks the call the
//! adapter reads, then that the tool's result goes back in that provider's
//! format on the next request.
//! AC-2: malformed arguments are answered with an error to the model and the
//! tool never runs; repeated malformed calls stop the run.
//!
//! Covers: REQ-LOOP-004 (WP-0122)
//! Threats: T-03 (prompt injection via tool args)

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::domain::AppError;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::chat::ChatMessage;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{
    self, CallOptions, CompletionOutcome, RequestedToolCall, ToolCallFormat, ToolSpec,
};
use lanesra_core::services::{chat_service, company_service, workspace_service};
use serde_json::{json, Value};

/// A local server that answers with `bodies` in order (repeating the last)
/// and records every request body it receives.
fn spawn_recording_stub(bodies: Vec<String>) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_clone = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            let _ = reader.read_line(&mut line);
            let mut length = 0usize;
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                    break;
                }
                let lower = header.to_ascii_lowercase();
                if let Some(v) = lower.strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0u8; length];
            let _ = reader.read_exact(&mut body);
            seen_clone
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&body).into_owned());
            let reply = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 {
                    q.pop_front().unwrap()
                } else {
                    q.front().cloned().unwrap_or_default()
                }
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply.len(),
                reply
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, seen)
}

fn get_record_tool() -> Vec<ToolSpec> {
    vec![ToolSpec {
        name: "get_record".into(),
        description: "Fetch one record by id".into(),
        input_schema: json!({
            "type": "object",
            "properties": {"id": {"type": "string"}, "limit": {"type": "integer"}},
            "required": ["id"]
        }),
    }]
}

fn message(
    role: &str,
    content: Option<&str>,
    tool_calls: Option<Value>,
    call_id: Option<&str>,
) -> ChatMessage {
    ChatMessage {
        id: String::new(),
        conversation_id: String::new(),
        role: role.into(),
        content: content.map(str::to_string),
        tool_calls,
        tool_call_id: call_id.map(str::to_string),
        created_at: String::new(),
    }
}

/// Sends one tool call through `provider`'s adapter and the tool result back,
/// returning the call the adapter read and the second request's body.
async fn round_trip(
    provider: &str,
    call_reply: Value,
    text_reply: Value,
) -> (RequestedToolCall, String) {
    let (port, seen) = spawn_recording_stub(vec![call_reply.to_string(), text_reply.to_string()]);
    let base = format!("http://127.0.0.1:{port}");
    let tools = get_record_tool();
    let mut history = vec![message("user", Some("Look up record 123"), None, None)];
    let (outcome, _) = ai_service::dispatch_with_tools_test(
        provider,
        Some(&base),
        "test-model",
        "test-key",
        "sys",
        &tools,
        &history,
    )
    .await
    .unwrap();
    let CompletionOutcome::ToolCalls {
        raw_assistant,
        calls,
    } = outcome
    else {
        panic!("{provider}: expected a tool call");
    };
    assert_eq!(calls.len(), 1, "{provider}");
    let call = calls[0].clone();
    history.push(message("assistant", None, Some(raw_assistant), None));
    history.push(message(
        "tool",
        Some(r#"{"name":"Acme Corp"}"#),
        None,
        Some(&call.id),
    ));
    let (outcome, _) = ai_service::dispatch_with_tools_test(
        provider,
        Some(&base),
        "test-model",
        "test-key",
        "sys",
        &tools,
        &history,
    )
    .await
    .unwrap();
    assert!(
        matches!(outcome, CompletionOutcome::Text(ref t) if t == "It is Acme Corp."),
        "{provider}"
    );
    let second = seen.lock().unwrap()[1].clone();
    (call, second)
}

#[tokio::test]
async fn ac1_anthropic_tool_calls_round_trip() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-1)
    let (call, second) = round_trip(
        "anthropic",
        json!({"content": [
            {"type": "text", "text": "Looking it up."},
            {"type": "tool_use", "id": "tu_1", "name": "get_record", "input": {"id": "123"}}
        ], "usage": {"input_tokens": 1, "output_tokens": 1}}),
        json!({"content": [{"type": "text", "text": "It is Acme Corp."}]}),
    )
    .await;
    assert_eq!(
        (call.id.as_str(), call.name.as_str()),
        ("tu_1", "get_record")
    );
    assert_eq!(call.arguments, json!({"id": "123"}));
    assert!(call.malformed.is_none());
    let body: Value = serde_json::from_str(&second).unwrap();
    let result = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["content"].as_array().cloned().unwrap_or_default())
        .find(|b| b["type"] == "tool_result")
        .expect("a tool_result block goes back");
    assert_eq!(result["tool_use_id"], "tu_1");
    assert!(result.to_string().contains("Acme Corp"));
}

#[tokio::test]
async fn ac1_openai_compatible_tool_calls_round_trip() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-1)
    let (call, second) = round_trip(
        "openai_compatible",
        json!({"choices": [{"message": {"role": "assistant", "content": null, "tool_calls": [
            {"id": "call_9", "type": "function", "function": {"name": "get_record", "arguments": "{\"id\": \"123\"}"}}
        ]}}]}),
        json!({"choices": [{"message": {"role": "assistant", "content": "It is Acme Corp."}}]}),
    )
    .await;
    assert_eq!(
        (call.id.as_str(), call.name.as_str()),
        ("call_9", "get_record")
    );
    assert_eq!(call.arguments, json!({"id": "123"}));
    assert!(call.malformed.is_none());
    let body: Value = serde_json::from_str(&second).unwrap();
    let result = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "tool")
        .expect("a tool message goes back");
    assert_eq!(result["tool_call_id"], "call_9");
    assert!(result["content"].as_str().unwrap().contains("Acme Corp"));
}

#[tokio::test]
async fn ac1_gemini_tool_calls_round_trip() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-1)
    let (call, second) = round_trip(
        "google_gemini",
        json!({"candidates": [{"content": {"role": "model", "parts": [
            {"functionCall": {"name": "get_record", "args": {"id": "123"}}}
        ]}}]}),
        json!({"candidates": [{"content": {"role": "model", "parts": [{"text": "It is Acme Corp."}]}}]}),
    )
    .await;
    assert_eq!(call.name, "get_record");
    assert_eq!(call.arguments, json!({"id": "123"}));
    assert!(call.malformed.is_none());
    let body: Value = serde_json::from_str(&second).unwrap();
    let response = body["contents"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["parts"].as_array().cloned().unwrap_or_default())
        .find(|p| p.get("functionResponse").is_some())
        .expect("a functionResponse goes back");
    assert_eq!(response["functionResponse"]["name"], "get_record");
    assert!(response.to_string().contains("Acme Corp"));
}

#[test]
fn every_adapter_uses_native_tool_calling_and_unknown_providers_get_strict_json() {
    // Covers: REQ-LOOP-004 (WP-0122): the capability matrix.
    for provider in ["anthropic", "openai_compatible", "google_gemini"] {
        assert_eq!(
            ai_service::tool_call_format(provider),
            ToolCallFormat::Native,
            "{provider}"
        );
    }
    assert_eq!(
        ai_service::tool_call_format("some_new_provider"),
        ToolCallFormat::StrictJson
    );
}

#[tokio::test]
async fn ac2_malformed_arguments_are_read_as_malformed_by_every_adapter() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-2)
    let cases = [
        (
            "openai_compatible",
            json!({"choices": [{"message": {"role": "assistant", "content": null, "tool_calls": [
                {"id": "c1", "type": "function", "function": {"name": "get_record", "arguments": "{\"id\": "}}
            ]}}]}),
        ),
        (
            "anthropic",
            json!({"content": [{"type": "tool_use", "id": "t1", "name": "get_record", "input": "id=123"}]}),
        ),
        (
            "google_gemini",
            json!({"candidates": [{"content": {"parts": [{"functionCall": {"name": "get_record", "args": [1, 2]}}]}}]}),
        ),
    ];
    for (provider, reply) in cases {
        let (port, _) = spawn_recording_stub(vec![reply.to_string()]);
        let (outcome, _) = ai_service::dispatch_with_tools_test(
            provider,
            Some(&format!("http://127.0.0.1:{port}")),
            "m",
            "k",
            "sys",
            &get_record_tool(),
            &[message("user", Some("hi"), None, None)],
        )
        .await
        .unwrap_or_else(|e| panic!("{provider}: a malformed call must not fail the response: {e}"));
        let CompletionOutcome::ToolCalls { calls, .. } = outcome else {
            panic!("{provider}: expected a tool call");
        };
        assert!(calls[0].malformed.is_some(), "{provider}");
        assert!(
            ai_service::check_tool_call(&calls[0], &get_record_tool()).is_err(),
            "{provider}"
        );
    }
}

#[test]
fn ac2_arguments_are_checked_against_the_tool_parameters() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-2)
    let tools = get_record_tool();
    let call = |args: Value| RequestedToolCall {
        id: "c".into(),
        name: "get_record".into(),
        arguments: args,
        malformed: None,
    };
    assert!(ai_service::check_tool_call(&call(json!({"id": "1"})), &tools).is_ok());
    assert!(ai_service::check_tool_call(&call(json!({"id": "1", "limit": 5})), &tools).is_ok());
    let missing = ai_service::check_tool_call(&call(json!({})), &tools).unwrap_err();
    assert!(missing.contains("'id'"), "{missing}");
    let wrong_type = ai_service::check_tool_call(&call(json!({"id": 7})), &tools).unwrap_err();
    assert!(wrong_type.contains("must be string"), "{wrong_type}");
    assert!(
        ai_service::check_tool_call(&call(json!({"id": "1", "limit": "five"})), &tools).is_err()
    );
    assert!(ai_service::check_tool_call(&call(json!("id=1")), &tools).is_err());
}

#[test]
fn strict_json_replies_are_read_without_repair() {
    // Covers: REQ-LOOP-004 (WP-0122): the strict JSON fallback format.
    match ai_service::parse_strict_json_reply(r#"{"answer": "It is Acme Corp."}"#).unwrap() {
        CompletionOutcome::Text(t) => assert_eq!(t, "It is Acme Corp."),
        other => panic!("{other:?}"),
    }
    match ai_service::parse_strict_json_reply(
        r#"{"tool_calls": [{"name": "get_record", "arguments": {"id": "123"}}, {"name": "get_record", "arguments": "id=4"}]}"#,
    )
    .unwrap()
    {
        CompletionOutcome::ToolCalls { calls, .. } => {
            assert_eq!(calls[0].id, "call_1");
            assert_eq!(calls[0].arguments, json!({"id": "123"}));
            assert!(calls[0].malformed.is_none());
            assert!(calls[1].malformed.is_some());
        }
        other => panic!("{other:?}"),
    }
    for bad in [
        "```json\n{\"answer\": \"x\"}\n```",
        "Sure! {\"answer\": \"x\"}",
        "{\"answer\": \"x\"} and more",
        "{\"tool_calls\": []}",
        "{\"tool_calls\": [{\"arguments\": {}}]}",
        "[1, 2]",
    ] {
        assert!(ai_service::parse_strict_json_reply(bad).is_err(), "{bad}");
    }
    let section = ai_service::json_tool_format_section(&get_record_tool()).unwrap();
    assert!(section.starts_with("Answer with one JSON object and nothing else"));
    assert!(section.contains("\"get_record\"") && section.contains("\"required\""));
}

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Tool Format Co".into(),
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
    (conn, workspace.id, admin.id)
}

fn use_anthropic(conn: &rusqlite::Connection, ws: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn,
        ws,
        &[91u8; 32],
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            model: "claude-test".into(),
            api_key: Some("sk-ant-test".into()),
        },
        Some(admin),
    )
    .unwrap();
}

fn malformed_create() -> String {
    json!({"content": [{"type": "tool_use", "id": "t1", "name": "create_record", "input": "name=Acme"}]})
        .to_string()
}

#[tokio::test]
async fn ac2_a_malformed_call_is_answered_with_an_error_and_never_runs() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-2)
    let (conn, ws, admin) = setup_workspace();
    let (port, seen) = spawn_recording_stub(vec![
        malformed_create(),
        json!({"content": [{"type": "text", "text": "Sorry, I could not create it."}]}).to_string(),
    ]);
    use_anthropic(&conn, &ws, &admin, port);
    let appended =
        chat_service::send_message(&conn, &ws, &[91u8; 32], &admin, "records", "create Acme")
            .await
            .unwrap();
    let tool_reply = appended
        .iter()
        .find(|m| m.role == "tool")
        .expect("the model gets an answer");
    let text = tool_reply.content.as_deref().unwrap();
    assert!(
        text.contains("validation.arguments") && text.contains("was not run"),
        "{text}"
    );
    assert!(
        company_service::list(&conn, &ws).unwrap().is_empty(),
        "the tool must not run"
    );
    // The error went back to the model on the next request.
    assert!(seen.lock().unwrap()[1].contains("was not run"));
}

#[tokio::test]
async fn ac2_repeated_malformed_calls_stop_the_run() {
    // Covers: REQ-LOOP-004 (WP-0122 AC-2): malformed calls count toward a cap.
    let (conn, ws, admin) = setup_workspace();
    let (port, seen) = spawn_recording_stub(vec![malformed_create()]);
    use_anthropic(&conn, &ws, &admin, port);
    let err = chat_service::send_message(&conn, &ws, &[91u8; 32], &admin, "records", "create Acme")
        .await
        .unwrap_err();
    assert!(
        matches!(
            &err,
            AppError::Classified {
                code: "validation.arguments",
                ..
            }
        ),
        "{err:?}"
    );
    assert_eq!(
        seen.lock().unwrap().len(),
        chat_service::MAX_MALFORMED_TOOL_CALLS as usize
    );
    assert!(company_service::list(&conn, &ws).unwrap().is_empty());
}

#[tokio::test]
async fn an_answer_that_misses_the_output_schema_gets_the_catalog_repair_prompt() {
    // Covers: REQ-LOOP-016 (WP-0130): the structured-output repair message
    // comes from the prompt catalog (`loop/output-repair`).
    use lanesra_core::models::ai_agent::{AiAgentInput, AiAgentVersionInput};
    use lanesra_core::models::ai_memory::AgentMemoryContext;
    use lanesra_core::services::{agent_version_service, ai_agent_service};
    let (conn, ws, admin) = setup_workspace();
    let (port, seen) = spawn_recording_stub(vec![
        json!({"content": [{"type": "text", "text": "Here is the answer: ok"}]}).to_string(),
        json!({"content": [{"type": "text", "text": "{\"answer\": \"ok\"}"}]}).to_string(),
    ]);
    use_anthropic(&conn, &ws, &admin, port);
    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &AiAgentInput {
            name: "Summarizer".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Summarise the request.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();
    let draft = agent_version_service::create_draft(
        &conn,
        &agent.id,
        &ws,
        &AiAgentVersionInput {
            name: "Summarizer".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Summarise the request.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            model_routing: None,
            output_schema: Some(json!({"type": "object", "required": ["answer"], "properties": {"answer": {"type": "string"}}})),
        },
        Some(&admin),
    )
    .unwrap();
    for status in ["test", "published"] {
        agent_version_service::transition_status(
            &conn,
            &agent.id,
            &ws,
            &draft.id,
            status,
            Some(&admin),
        )
        .unwrap();
    }
    let agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();
    let outcome = chat_service::run_agent_once_with_text(
        &conn,
        &ws,
        &[91u8; 32],
        Some(&admin),
        &agent,
        "Summarise",
        &AgentMemoryContext {
            session_key: None,
            run_id: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(outcome.final_text, "{\"answer\": \"ok\"}");
    let second = seen.lock().unwrap()[1].clone();
    assert!(
        second.contains("Send your answer again as valid JSON"),
        "{second}"
    );
}

/// The request body `provider`'s adapter sends with `opts`.
async fn request_body(provider: &str, reply: Value, opts: CallOptions) -> Value {
    let (port, seen) = spawn_recording_stub(vec![reply.to_string()]);
    ai_service::dispatch_with_options_test(
        provider,
        Some(&format!("http://127.0.0.1:{port}")),
        "test-model",
        "test-key",
        "sys",
        &get_record_tool(),
        &[message("user", Some("Look up record 123"), None, None)],
        opts,
    )
    .await
    .unwrap();
    let body = seen.lock().unwrap()[0].clone();
    serde_json::from_str(&body).unwrap()
}

#[tokio::test]
async fn every_adapter_sends_the_agents_temperature_output_limit_and_tools_off() {
    // Covers: REQ-LOOP-001, REQ-LOOP-008 (WP-0121): routing temperature and
    // max tokens take effect, and a final-answer call tells each provider in
    // its own format not to call a tool.
    let opts = CallOptions {
        temperature: Some(0.4),
        max_tokens: Some(300),
        tools_off: true,
    };
    let anthropic = request_body(
        "anthropic",
        json!({"content": [{"type": "text", "text": "ok"}]}),
        opts,
    )
    .await;
    assert_eq!(anthropic["temperature"], json!(0.4));
    assert_eq!(anthropic["max_tokens"], json!(300));
    assert_eq!(anthropic["tool_choice"], json!({"type": "none"}));

    let openai = request_body(
        "openai_compatible",
        json!({"choices": [{"message": {"role": "assistant", "content": "ok"}}]}),
        opts,
    )
    .await;
    assert_eq!(openai["temperature"], json!(0.4));
    assert_eq!(openai["max_tokens"], json!(300));
    assert_eq!(openai["tool_choice"], json!("none"));

    let gemini = request_body(
        "google_gemini",
        json!({"candidates": [{"content": {"parts": [{"text": "ok"}]}}]}),
        opts,
    )
    .await;
    assert_eq!(gemini["generationConfig"]["temperature"], json!(0.4));
    assert_eq!(gemini["generationConfig"]["maxOutputTokens"], json!(300));
    assert_eq!(
        gemini["toolConfig"]["functionCallingConfig"]["mode"],
        json!("NONE")
    );

    // Without settings: no temperature, the default output limit, tools on.
    let plain = request_body(
        "anthropic",
        json!({"content": [{"type": "text", "text": "ok"}]}),
        CallOptions::default(),
    )
    .await;
    assert!(plain.get("temperature").is_none() && plain.get("tool_choice").is_none());
    assert_eq!(
        plain["max_tokens"],
        json!(ai_service::DEFAULT_MAX_OUTPUT_TOKENS)
    );
    // Anthropic takes 0 to 1, so a higher routing temperature is capped.
    let hot = request_body(
        "anthropic",
        json!({"content": [{"type": "text", "text": "ok"}]}),
        CallOptions {
            temperature: Some(1.6),
            ..CallOptions::default()
        },
    )
    .await;
    assert_eq!(hot["temperature"], json!(1.0));
}
