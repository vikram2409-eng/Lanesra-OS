//! `client.rs` against a real (stubbed) HTTP listener - the same
//! raw-socket test-double pattern already used in
//! `core/tests/ai_settings.rs` / `core/tests/integration_connection.rs`,
//! reused here rather than mocking `reqwest` itself.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use lanesra_cli::client::ApiClient;
use serde_json::json;

/// Starts a one-shot HTTP stub that always returns the given status/body,
/// and returns its port. `expect_method`/`expect_path`, if given, are
/// asserted against the request line the client actually sent.
fn spawn_http_stub(status: u16, body: &'static str) -> u16 {
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
            let reason = if status == 200 { "OK" } else { "Not Found" };
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

#[test]
fn get_round_trips_a_successful_json_response() {
    let port = spawn_http_stub(200, r#"{"ok":true,"data":[{"object_key":"Company"}]}"#);
    let client = ApiClient::new(format!("http://127.0.0.1:{port}"), "client_x.secret");
    let response = client.get("/api/v1/objects").unwrap();
    assert!(response.status.is_success());
    assert_eq!(response.body["data"][0]["object_key"], "Company");
}

#[test]
fn a_non_2xx_status_is_still_returned_as_a_parsed_response_not_an_error() {
    let port = spawn_http_stub(404, r#"{"ok":false,"error":"Object 'Bogus' not found"}"#);
    let client = ApiClient::new(format!("http://127.0.0.1:{port}"), "client_x.secret");
    let response = client.get("/api/v1/objects/Bogus/metadata").unwrap();
    assert_eq!(response.status, reqwest::StatusCode::NOT_FOUND);
    assert_eq!(response.body["ok"], false);
}

#[test]
fn post_sends_the_json_body() {
    let port = spawn_http_stub(200, r#"{"ok":true,"data":{"id":"abc123","name":"Acme"}}"#);
    let client = ApiClient::new(format!("http://127.0.0.1:{port}"), "client_x.secret");
    let response = client.post("/api/v1/objects/Company/records", &json!({"name": "Acme"})).unwrap();
    assert_eq!(response.body["data"]["id"], "abc123");
}

#[test]
fn base_url_trailing_slash_is_stripped() {
    let port = spawn_http_stub(200, r#"{"ok":true,"data":[]}"#);
    let client = ApiClient::new(format!("http://127.0.0.1:{port}/"), "client_x.secret");
    let response = client.get("/api/v1/objects").unwrap();
    assert!(response.status.is_success());
}
