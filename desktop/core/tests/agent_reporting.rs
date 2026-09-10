//! AI & Agentic Layer, Phase 4: proves `agent_service::ask_report` end
//! to end against a real local HTTP listener standing in for the LLM
//! provider (same raw-socket test-double pattern `ai_settings.rs`
//! already uses, not a live account) - a well-formed directive runs a
//! real report against seeded data, the model's own `{"error": ...}`
//! reply surfaces as a validation error, a malformed or markdown-fenced
//! reply is handled, and an out-of-scope field name is caught by the
//! exact same `validate_shape` a human's manual report creation goes
//! through (`custom_report_service::preview`).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_core::models::agent::NlReportQuery;
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{agent_service, ai_service, company_service, custom_field_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Agent Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [7u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

/// A minimal raw-socket HTTP server always returning the given
/// Anthropic-shaped `messages` response body containing `text` as the
/// assistant's reply - mirrors `ai_settings.rs`'s own `spawn_http_stub`.
fn spawn_anthropic_stub(reply_text: &str) -> u16 {
    let body = serde_json::json!({"content": [{"type": "text", "text": reply_text}]}).to_string();
    spawn_http_stub(200, body)
}

fn spawn_http_stub(status: u16, body: String) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            let _ = reader.read_line(&mut line);
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) | Ok(_) if l == "\r\n" || l.is_empty() => break,
                    _ => continue,
                }
            }
            let reason = if status == 200 { "OK" } else { "Bad Request" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn make_company(conn: &rusqlite::Connection, ws: &str, admin: &str, status: &str) -> String {
    company_service::create(conn, ws, &CompanyInput { name: format!("Co {status}"), status: status.into(), owner_user_id: None, ..Default::default() }, Some(admin)).unwrap().id
}

#[tokio::test]
async fn a_count_directive_runs_a_real_report_against_seeded_data() {
    let (conn, ws, admin) = setup_workspace();
    make_company(&conn, &ws, &admin, "Prospect");
    make_company(&conn, &ws, &admin, "Prospect");
    make_company(&conn, &ws, &admin, "Active Customer");

    let port = spawn_anthropic_stub(r#"{"entity_type":"Company","group_by_source":"builtin","group_by_field":"status","aggregate":"count","sum_field_key":null}"#);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "how many companies by status?".into() }).await.unwrap();
    assert_eq!(result.report.entity_type, "Company");
    assert_eq!(result.report.aggregate, "count");
    let by_group: HashMap<String, f64> = result.rows.into_iter().map(|r| (r.group, r.value)).collect();
    assert_eq!(by_group.get("Prospect"), Some(&2.0));
    assert_eq!(by_group.get("Active Customer"), Some(&1.0));
}

#[tokio::test]
async fn a_sum_directive_over_a_custom_field_runs_correctly() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Deal Size".into(), field_type: "number".into(), options: vec![],
            required: false, show_in_list: false, sort_order: 0, min_value: None, max_value: None, max_length: None,
            regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: true, default_value: None,
            is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
        },
        Some(&admin),
    )
    .unwrap();
    let c1 = make_company(&conn, &ws, &admin, "Prospect");
    let c2 = make_company(&conn, &ws, &admin, "Prospect");
    custom_field_service::set_entity_values(&conn, "Company", &c1, &HashMap::from([("deal_size".to_string(), "100".to_string())]), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &c2, &HashMap::from([("deal_size".to_string(), "250".to_string())]), Some(&admin)).unwrap();

    let port = spawn_anthropic_stub(r#"{"entity_type":"Company","group_by_source":"builtin","group_by_field":"status","aggregate":"sum","sum_field_key":"deal_size"}"#);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "total deal size by status".into() }).await.unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].group, "Prospect");
    assert_eq!(result.rows[0].value, 350.0);
}

#[tokio::test]
async fn the_models_own_error_reply_surfaces_as_a_validation_error() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_anthropic_stub(r#"{"error":"that needs a date range, which this engine doesn't support"}"#);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "revenue last quarter vs this quarter".into() }).await;
    let err = result.unwrap_err().to_string();
    assert!(err.contains("date range"), "{err}");
}

#[tokio::test]
async fn a_non_json_reply_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_anthropic_stub("Sure, I can help you with that!");
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "how many companies?".into() }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn a_markdown_fenced_reply_is_still_parsed() {
    let (conn, ws, admin) = setup_workspace();
    make_company(&conn, &ws, &admin, "Prospect");
    let port = spawn_anthropic_stub("```json\n{\"entity_type\":\"Company\",\"group_by_source\":\"builtin\",\"group_by_field\":\"status\",\"aggregate\":\"count\",\"sum_field_key\":null}\n```");
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "how many companies?".into() }).await.unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[tokio::test]
async fn an_invalid_field_name_is_rejected_by_the_reused_validation() {
    let (conn, ws, admin) = setup_workspace();
    make_company(&conn, &ws, &admin, "Prospect");
    // group_by_source "custom" naming a field that was never defined -
    // validate_shape (via custom_report_service::preview) must reject
    // this exactly as it would a human's own manual mistake.
    let port = spawn_anthropic_stub(r#"{"entity_type":"Company","group_by_source":"custom","group_by_field":"totally_made_up","aggregate":"count","sum_field_key":null}"#);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "group by made-up field".into() }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn an_empty_question_is_rejected_before_any_network_call() {
    let (conn, ws, _admin) = setup_workspace();
    // No key configured at all - if this reached ai_service::complete it
    // would fail for that reason instead, so succeeding here proves the
    // empty-question check runs first.
    let result = agent_service::ask_report(&conn, &ws, &master_key(), &NlReportQuery { question: "   ".into() }).await;
    let err = result.unwrap_err().to_string();
    assert!(err.contains("question"), "{err}");
}
