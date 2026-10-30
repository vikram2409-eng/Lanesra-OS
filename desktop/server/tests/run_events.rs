//! WP-0112: run events over SSE on a real spawned server (ADR-0008 §3, §5).
//!
//! AC-2: only a signed-in Administrator, or an API client with the
//! `events.read` scope, receives a run's events; a member, an anonymous
//! caller and a key without the scope receive none.
//! AC-1 and AC-3 over the wire: every streamed event matches its schema,
//! arrives in `seq` order, replay after `Last-Event-ID` resumes exactly,
//! and a paused run's stream carries on live when the run resumes.
//!
//! The run executes on its own connection to the server's database file,
//! as a background run does; its events reach the server through the
//! in-process bus.
//!
//! Covers: REQ-LOOP-005, REQ-OBS-003, REQ-API-005 (WP-0112).

#[path = "../../core/tests/support/event_schema.rs"]
mod event_schema;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use lanesra_core::db::open_workspace_db;
use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::integration::ApiClientInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::TokenUsage;
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_orchestration_service, ai_provider_service, api_client_service,
    user_service, workspace_service,
};
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

const KEY: [u8; 32] = [113u8; 32];

struct Fixture {
    addr: SocketAddr,
    db_path: PathBuf,
    _dir: tempfile::TempDir,
    workspace_id: String,
    admin_id: String,
    pipeline_id: String,
    key_with_scope: String,
    key_without_scope: String,
}

async fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let conn = open_workspace_db(&db_path).unwrap();
    let state: SharedState = ServerState::new(conn, db_path.clone(), SecurityConfig::default());
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let conn = state.conn.lock().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Stream Co".into(),
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
    user_service::create(
        &conn,
        &workspace.id,
        &NewUser {
            username: "member".into(),
            display_name: "Member".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".to_string()],
        },
        Some(&admin.id),
    )
    .unwrap();
    let key = |scopes: &[&str], name: &str| {
        api_client_service::create(
            &conn,
            &workspace.id,
            &ApiClientInput {
                name: name.into(),
                scopes: scopes.iter().map(|s| s.to_string()).collect(),
                allowed_cidr: None,
                owner_user_id: None,
            },
            Some(&admin.id),
        )
        .unwrap()
        .api_key
    };
    let key_with_scope = key(&["events.read"], "Events reader");
    let key_without_scope = key(&["objects.read"], "Objects reader");

    let provider = ai_provider_service::create(
        &conn,
        &workspace.id,
        &KEY,
        &AiProviderInput {
            name: "scripted".into(),
            provider: "anthropic".into(),
            base_url: None,
            model: "scripted-model".into(),
            api_key: Some("k".into()),
        },
        Some(&admin.id),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_providers SET provider = 'scripted' WHERE id = ?1",
        [&provider.id],
    )
    .unwrap();
    let agent = ai_agent_service::create(
        &conn,
        &workspace.id,
        &AiAgentInput {
            name: "Writer".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "You write.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(&admin.id),
    )
    .unwrap();
    ai_agent_service::set_model_routing(
        &conn,
        &agent.id,
        &workspace.id,
        Some(AiAgentModelRouting {
            primary_provider_id: Some(provider.id),
            ..Default::default()
        }),
        Some(&admin.id),
    )
    .unwrap();
    // Two steps; the first needs approval, so a run pauses between them.
    let pipeline = ai_orchestration_service::create_pipeline(
        &conn,
        &workspace.id,
        &AiAgentPipelineInput {
            name: "Brief".into(),
            description: None,
            topology: "sequential".into(),
            steps: vec![
                PipelineStepInput {
                    agent_id: agent.id.clone(),
                    input_template: "{{trigger_input}}".into(),
                    requires_approval: true,
                },
                PipelineStepInput {
                    agent_id: agent.id.clone(),
                    input_template: "{{previous_output}}".into(),
                    requires_approval: false,
                },
            ],
        },
        Some(&admin.id),
    )
    .unwrap();
    drop(conn);
    Fixture {
        addr,
        db_path,
        _dir: dir,
        workspace_id: workspace.id,
        admin_id: admin.id,
        pipeline_id: pipeline.id,
        key_with_scope,
        key_without_scope,
    }
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: TokenUsage {
            input_tokens: 50,
            output_tokens: 10,
        },
    }
}

/// Starts a run on its own connection; it pauses after the first step.
async fn start_paused_run(f: &Fixture) -> String {
    let conn = open_workspace_db(&f.db_path).unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text("notes")]);
    let run = ai_orchestration_service::run_manual(
        &conn,
        &f.workspace_id,
        &KEY,
        "pipeline",
        &f.pipeline_id,
        "Acme",
        Some(&f.admin_id),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "awaiting_approval");
    run.id
}

async fn approve(f: &Fixture, run_id: &str) {
    let conn = open_workspace_db(&f.db_path).unwrap();
    scripted_provider::script_responses(vec![text("the brief")]);
    let run = ai_orchestration_service::approve_pending_step(
        &conn,
        &f.workspace_id,
        &KEY,
        run_id,
        None,
        Some(&f.admin_id),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "succeeded");
}

async fn complete_run(f: &Fixture) -> String {
    let run_id = start_paused_run(f).await;
    approve(f, &run_id).await;
    run_id
}

async fn signed_in(f: &Fixture, username: &str, password: &str) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let login: Value = client
        .post(format!("http://{}/api/invoke/login", f.addr))
        .json(&json!({"credentials": {"username": username, "password": password}}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(login["ok"], true, "{login}");
    client
}

/// One SSE message: its id and parsed data.
#[derive(Debug)]
struct Message {
    id: String,
    data: Value,
}

/// Reads SSE messages until `done` holds or the stream ends; fails after
/// five seconds.
async fn read_messages(
    response: reqwest::Response,
    done: impl Fn(&[Message]) -> bool,
) -> Vec<Message> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut messages: Vec<Message> = Vec::new();
    let reading = async {
        while let Some(chunk) = stream.next().await {
            buffer.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
            while let Some(end) = buffer.find("\n\n") {
                let block: String = buffer.drain(..end + 2).collect();
                let mut id = String::new();
                let mut data = String::new();
                for line in block.lines() {
                    if let Some(v) = line.strip_prefix("id:") {
                        id = v.trim().to_string();
                    } else if let Some(v) = line.strip_prefix("data:") {
                        data.push_str(v.trim_start());
                    }
                }
                if !data.is_empty() {
                    messages.push(Message {
                        id,
                        data: serde_json::from_str(&data).unwrap(),
                    });
                }
            }
            if done(&messages) {
                break;
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), reading)
        .await
        .unwrap_or_else(|_| panic!("timed out after {messages:?}"));
    messages
}

fn types(messages: &[Message]) -> Vec<String> {
    messages
        .iter()
        .map(|m| m.data["type"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn an_administrator_follows_a_paused_run_live_until_it_completes() {
    let f = fixture().await;
    let run_id = start_paused_run(&f).await;
    let admin = signed_in(&f, "admin", "supersecretpassword").await;

    let response = admin
        .get(format!("http://{}/api/v1/runs/{run_id}/events", f.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(response.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/event-stream"));

    // The stream replays what the paused run has done, then waits; the
    // resumed run's events arrive live and the stream ends after the last.
    let reader = tokio::spawn(read_messages(response, |_| false));
    tokio::time::sleep(Duration::from_millis(300)).await;
    approve(&f, &run_id).await;
    let messages = reader.await.unwrap();

    assert_eq!(
        types(&messages),
        [
            "run.started",
            "step.started",
            "round.completed",
            "step.completed",
            "approval.requested",
            "step.started",
            "round.completed",
            "step.completed",
            "run.completed",
        ]
    );
    for (i, m) in messages.iter().enumerate() {
        event_schema::assert_valid(&m.data);
        assert_eq!(m.data["seq"], (i + 1) as i64);
        assert_eq!(m.id, (i + 1).to_string());
        assert_eq!(m.data["run_id"], run_id.as_str());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn only_administrators_and_clients_with_events_read_receive_events() {
    let f = fixture().await;
    let run_id = complete_run(&f).await;
    let url = format!("http://{}/api/v1/runs/{run_id}/events", f.addr);

    let anonymous = reqwest::Client::new().get(&url).send().await.unwrap();
    assert_eq!(anonymous.status(), 401);
    assert!(anonymous
        .text()
        .await
        .unwrap()
        .find("run.started")
        .is_none());

    let member = signed_in(&f, "member", "anothersecretpw").await;
    let refused = member.get(&url).send().await.unwrap();
    assert_eq!(refused.status(), 403);
    assert!(refused.text().await.unwrap().find("run.started").is_none());

    let without_scope = reqwest::Client::new()
        .get(&url)
        .bearer_auth(&f.key_without_scope)
        .send()
        .await
        .unwrap();
    assert_eq!(without_scope.status(), 403);

    let with_scope = reqwest::Client::new()
        .get(&url)
        .bearer_auth(&f.key_with_scope)
        .send()
        .await
        .unwrap();
    assert_eq!(with_scope.status(), 200);
    let messages = read_messages(with_scope, |_| false).await;
    assert_eq!(messages.len(), 9);
    assert_eq!(messages.last().unwrap().data["type"], "run.completed");

    // The member's own command read is refused too.
    let listed: Value = member
        .post(format!("http://{}/api/invoke/list_run_events", f.addr))
        .json(&json!({"runId": run_id, "afterSeq": 0}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(listed["ok"], false, "{listed}");

    let unknown = reqwest::Client::new()
        .get(format!("http://{}/api/v1/runs/no-such-run/events", f.addr))
        .bearer_auth(&f.key_with_scope)
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn replay_resumes_after_last_event_id_and_summary_keeps_run_and_step_events() {
    let f = fixture().await;
    let run_id = complete_run(&f).await;
    let url = format!("http://{}/api/v1/runs/{run_id}/events", f.addr);
    let client = reqwest::Client::new();

    let resumed = client
        .get(&url)
        .bearer_auth(&f.key_with_scope)
        .header("Last-Event-ID", "4")
        .send()
        .await
        .unwrap();
    let messages = read_messages(resumed, |_| false).await;
    let seqs: Vec<i64> = messages
        .iter()
        .map(|m| m.data["seq"].as_i64().unwrap())
        .collect();
    assert_eq!(seqs, (5..=9).collect::<Vec<_>>());

    let summary = client
        .get(format!("{url}?level=summary"))
        .bearer_auth(&f.key_with_scope)
        .send()
        .await
        .unwrap();
    let messages = read_messages(summary, |_| false).await;
    assert!(types(&messages)
        .iter()
        .all(|t| t.starts_with("run.") || t.starts_with("step.")));
    assert_eq!(messages.len(), 6);

    let refused = client
        .get(format!("{url}?level=tokens"))
        .bearer_auth(&f.key_with_scope)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_shared_stream_follows_only_the_runs_asked_for() {
    let f = fixture().await;
    let first = complete_run(&f).await;
    let second = complete_run(&f).await;
    let other = complete_run(&f).await;

    let response = reqwest::Client::new()
        .get(format!(
            "http://{}/api/v1/events/stream?run_id={first}&run_id={second}",
            f.addr
        ))
        .bearer_auth(&f.key_with_scope)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let messages = read_messages(response, |m| m.len() >= 18).await;
    assert_eq!(messages.len(), 18);
    assert!(messages
        .iter()
        .all(|m| m.data["run_id"] == first.as_str() || m.data["run_id"] == second.as_str()));
    assert!(!messages.iter().any(|m| m.data["run_id"] == other.as_str()));
    // Ids are a cursor across runs: increasing.
    let ids: Vec<i64> = messages.iter().map(|m| m.id.parse().unwrap()).collect();
    assert!(ids.windows(2).all(|w| w[0] < w[1]));

    // Resuming after the ninth message sends only what followed it.
    let resumed = reqwest::Client::new()
        .get(format!(
            "http://{}/api/v1/events/stream?run_id={first}&run_id={second}",
            f.addr
        ))
        .bearer_auth(&f.key_with_scope)
        .header("Last-Event-ID", messages[8].id.clone())
        .send()
        .await
        .unwrap();
    let rest = read_messages(resumed, |m| m.len() >= 9).await;
    assert_eq!(
        rest.iter().map(|m| m.id.clone()).collect::<Vec<_>>(),
        messages[9..]
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    );

    // Without run_id the route still streams integration events only, for
    // API keys only.
    let session = signed_in(&f, "admin", "supersecretpassword").await;
    let integration = session
        .get(format!("http://{}/api/v1/events/stream", f.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(integration.status(), 401);
}
