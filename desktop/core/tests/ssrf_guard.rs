//! WP-0143: the outbound network guard on real connections (threat T-13,
//! RAID R-27). This binary declares Team Workspace with one allow-listed
//! address, 127.0.0.3, as the server would at startup. Every refusal is
//! checked to have happened before a byte reached the target.
//!
//! Covers: REQ-TOOL-007 (WP-0143)

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::feature_flag::Edition;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ssrf_guard::{self, AllowEntry, NetworkPolicy};
use lanesra_core::services::{ai_service, workspace_service};

const ALLOWED: &str = "127.0.0.3";

fn declare_team_workspace() {
    ssrf_guard::install(NetworkPolicy::workspace(vec![
        AllowEntry::parse(ALLOWED).unwrap()
    ]))
    .unwrap();
}

/// A server on `ip` that counts connections and answers every request
/// with `response`.
fn spawn_server(ip: &str, response: String) -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind((ip, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            counter.fetch_add(1, Ordering::SeqCst);
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
                    break;
                }
            }
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, hits)
}

fn ok_response() -> String {
    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string()
}

fn assert_blocked(err: &lanesra_core::domain::AppError) {
    assert_eq!(err.code(), Some("policy.network_blocked"), "{err:?}");
}

async fn get(url: &str) -> Result<reqwest::Response, lanesra_core::domain::AppError> {
    ssrf_guard::check_url(url)?;
    let client = ssrf_guard::client_builder().build().unwrap();
    client.get(url).send().await.map_err(|e| {
        ssrf_guard::blocked_in(&e).unwrap_or_else(|| panic!("not a guard refusal: {e}"))
    })
}

#[tokio::test]
async fn a_literal_internal_address_is_refused_before_connecting() {
    // Covers: REQ-TOOL-007 (WP-0143)
    declare_team_workspace();
    let (port, hits) = spawn_server("127.0.0.1", ok_response());
    assert_blocked(&get(&format!("http://127.0.0.1:{port}/")).await.unwrap_err());
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_name_is_checked_when_it_is_resolved() {
    // Covers: REQ-TOOL-007 (WP-0143). `localhost` only becomes an address at
    // connect time, which is where a rebinding name is caught too.
    declare_team_workspace();
    let (port, hits) = spawn_server("127.0.0.1", ok_response());
    assert_blocked(&get(&format!("http://localhost:{port}/")).await.unwrap_err());
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn an_allow_listed_address_is_reachable() {
    // Covers: REQ-TOOL-007 (WP-0143)
    declare_team_workspace();
    let (port, hits) = spawn_server(ALLOWED, ok_response());
    let response = get(&format!("http://{ALLOWED}:{port}/")).await.unwrap();
    assert_eq!(response.text().await.unwrap(), "ok");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_redirect_to_a_refused_address_is_not_followed() {
    // Covers: REQ-TOOL-007 (WP-0143)
    declare_team_workspace();
    let (target_port, target_hits) = spawn_server("127.0.0.1", ok_response());
    for location in [
        format!("http://127.0.0.1:{target_port}/"),
        "http://169.254.169.254/latest/meta-data/".to_string(),
    ] {
        let redirect = format!(
            "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let (port, hits) = spawn_server(ALLOWED, redirect);
        assert_blocked(&get(&format!("http://{ALLOWED}:{port}/")).await.unwrap_err());
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "the allowed first hop is reached"
        );
    }
    assert_eq!(target_hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn metadata_is_refused_even_when_the_allow_list_covers_it() {
    // Covers: REQ-TOOL-007 (WP-0143)
    let open = NetworkPolicy::workspace(vec![AllowEntry::parse("0.0.0.0/0").unwrap()]);
    for url in [
        "http://169.254.169.254/latest/meta-data/",
        "http://[::ffff:169.254.169.254]/",
        "http://2852039166/",
        "http://[fd00:ec2::254]/",
    ] {
        assert!(ssrf_guard::check_url_with(&open, url).is_err(), "{url}");
    }
    assert!(ssrf_guard::check_url_with(&open, "file:///etc/passwd").is_err());
}

#[tokio::test]
async fn desktop_reaches_local_services_and_names_resolve_by_policy() {
    // Covers: REQ-TOOL-007 (WP-0143)
    let desktop = NetworkPolicy::desktop();
    assert_eq!(desktop.edition, Edition::Desktop);
    assert!(ssrf_guard::check_url_with(&desktop, "http://127.0.0.1:11434/").is_ok());
    assert!(ssrf_guard::check_url_with(&desktop, "http://192.168.1.20/").is_ok());
    assert!(ssrf_guard::check_url_with(&desktop, "http://169.254.169.254/").is_err());
    assert!(ssrf_guard::resolve_allowed(&desktop, "localhost")
        .await
        .is_ok());

    let workspace = NetworkPolicy::workspace(Vec::new());
    assert!(ssrf_guard::resolve_allowed(&workspace, "localhost")
        .await
        .is_err());
}

#[tokio::test]
async fn a_model_provider_on_an_internal_address_is_refused() {
    // Covers: REQ-TOOL-007 (WP-0143): an admin-configured OpenAI-compatible
    // base URL cannot point the server at its own network.
    declare_team_workspace();
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Guard Co".into(),
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
    let key = [71u8; 32];
    let (port, hits) = spawn_server("127.0.0.1", ok_response());
    let internal = AiSettingsInput {
        provider: "openai_compatible".into(),
        base_url: Some(format!("http://127.0.0.1:{port}/v1")),
        model: "local-model".into(),
        api_key: Some("sk-test".into()),
    };
    // WP-0191: saving it is refused now, naming the setting to change.
    let refused = ai_service::save_settings(&conn, &workspace.id, &key, &internal, Some(&admin.id))
        .unwrap_err();
    assert_blocked(&refused);
    assert!(
        refused.to_string().contains(ssrf_guard::ALLOW_LIST_ENV),
        "{refused}"
    );
    // A URL saved before that check existed is still refused when used.
    ai_service::save_settings(
        &conn,
        &workspace.id,
        &key,
        &AiSettingsInput {
            base_url: Some(format!("http://{ALLOWED}:{port}/v1")),
            ..internal.clone()
        },
        Some(&admin.id),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_settings SET base_url = ?1 WHERE workspace_id = ?2",
        (internal.base_url.as_deref(), &workspace.id),
    )
    .unwrap();
    // "Test key" reports the refusal to the admin instead of probing the network.
    let tested = ai_service::test_key(&conn, &workspace.id, &key, Some(&admin.id))
        .await
        .unwrap();
    assert!(!tested.ok, "{tested:?}");
    assert!(
        tested.message.contains(ssrf_guard::ALLOW_LIST_ENV),
        "{tested:?}"
    );
    let err = ai_service::complete_with_tools(&conn, &workspace.id, &key, "sys", &[], &[])
        .await
        .unwrap_err();
    assert_blocked(&err);
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[test]
fn every_outbound_http_client_is_built_by_the_guard() {
    // Covers: REQ-TOOL-007 (WP-0143): a new client that skips the guard fails
    // this test.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path.ends_with("ssrf_guard.rs")
            {
                let text = std::fs::read_to_string(&path).unwrap();
                if text.contains("reqwest::Client::builder()")
                    || text.contains("reqwest::Client::new()")
                {
                    offenders.push(path.display().to_string());
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "build HTTP clients with ssrf_guard::client_builder(): {offenders:?}"
    );
}

#[tokio::test]
async fn sftp_and_smtp_hosts_resolve_only_to_allowed_addresses() {
    // Covers: REQ-TOOL-007 (WP-0143): SFTP and SMTP connect to these
    // addresses, not to the configured name.
    declare_team_workspace();
    assert_blocked(
        &ssrf_guard::allowed_addrs("127.0.0.1", 22)
            .await
            .unwrap_err(),
    );
    assert_blocked(
        &ssrf_guard::allowed_addrs("localhost", 25)
            .await
            .unwrap_err(),
    );
    let addrs = ssrf_guard::allowed_addrs(ALLOWED, 2222).await.unwrap();
    assert_eq!(addrs, ["127.0.0.3:2222".parse().unwrap()]);
}

#[tokio::test]
async fn admins_see_a_refused_address_when_they_save_it() {
    // Covers: REQ-PRE-019 (WP-0191 AC-1)
    use lanesra_core::models::ai::{AiObservabilitySettingsInput, AiProviderInput};
    use lanesra_core::models::ai_mcp::McpServerInput;
    use lanesra_core::models::integration::{ConnectionInput, ConnectionUpdate};
    use lanesra_core::services::{ai_provider_service, connection_service, mcp_client_service};

    declare_team_workspace();
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Save Co".into(),
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
    let (ws, actor, key) = (workspace.id.as_str(), Some(admin.id.as_str()), [72u8; 32]);
    let refused_with = |err: lanesra_core::domain::AppError, address: &str| {
        assert_blocked(&err);
        let message = err.to_string();
        assert!(message.contains(address), "{message}");
        assert!(message.contains(ssrf_guard::ALLOW_LIST_ENV), "{message}");
    };
    let connection = |url: &str| ConnectionInput {
        name: format!("API at {url}"),
        connection_type: "rest".into(),
        base_url: Some(url.into()),
        auth_mode: "none".into(),
        secret_value: None,
        config_json: "{}".into(),
        owner_user_id: None,
    };

    // A literal internal address, and a name that resolves only to one.
    refused_with(
        connection_service::create(&conn, ws, &key, &connection("http://10.1.2.3/api"), actor)
            .unwrap_err(),
        "10.1.2.3",
    );
    refused_with(
        connection_service::create(
            &conn,
            ws,
            &key,
            &connection("http://localhost:9/api"),
            actor,
        )
        .unwrap_err(),
        "localhost",
    );
    // An allow-listed address saves, and cannot then be moved off the list.
    let saved = connection_service::create(
        &conn,
        ws,
        &key,
        &connection(&format!("http://{ALLOWED}:9/api")),
        actor,
    )
    .unwrap();
    let moved = connection_service::update(
        &conn,
        ws,
        &key,
        &saved.id,
        &ConnectionUpdate {
            name: saved.name.clone(),
            base_url: Some("http://192.168.7.7/api".into()),
            auth_mode: "none".into(),
            secret_value: None,
            config_json: "{}".into(),
            owner_user_id: None,
            status: saved.status.clone(),
        },
        actor,
    )
    .unwrap_err();
    refused_with(moved, "192.168.7.7");
    // A name that cannot be resolved now is accepted and checked on use.
    connection_service::create(
        &conn,
        ws,
        &key,
        &connection("http://lanesra-save-check.invalid/api"),
        actor,
    )
    .unwrap();

    // MCP servers, model providers and the trace exporter.
    refused_with(
        mcp_client_service::create_server(
            &conn,
            ws,
            &key,
            &McpServerInput {
                name: "Local MCP".into(),
                base_url: "http://127.0.0.1:7000/mcp".into(),
                auth_mode: "none".into(),
                secret_value: None,
            },
            actor,
        )
        .unwrap_err(),
        "127.0.0.1",
    );
    refused_with(
        ai_provider_service::create(
            &conn,
            ws,
            &key,
            &AiProviderInput {
                name: "Ollama".into(),
                provider: "openai_compatible".into(),
                base_url: Some("http://127.0.0.1:11434/v1".into()),
                model: "llama3".into(),
                api_key: None,
            },
            actor,
        )
        .unwrap_err(),
        "127.0.0.1",
    );
    refused_with(
        ai_service::set_otlp_endpoint(
            &conn,
            ws,
            &AiObservabilitySettingsInput {
                otlp_endpoint: Some("http://172.16.0.9:4318/v1/traces".into()),
            },
            actor,
        )
        .unwrap_err(),
        "172.16.0.9",
    );
}
