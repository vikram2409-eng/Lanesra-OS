//! AI & Agentic Layer, phase 2: `lanesra-cli` pairs with the MCP server
//! (`server/src/mcp.rs`) as the backlog's other named half - a thin,
//! scriptable consumer of the same `/api/v1` REST API for humans and
//! shell scripts, not an MCP client itself. All the actual behavior
//! (permission checks, validation, business rules) lives server-side in
//! `api_object_service`; this crate is just argument parsing
//! (`cli.rs`) plus an HTTP call (`client.rs`) and JSON formatting.

pub mod cli;
pub mod client;

use serde_json::Value;

use cli::{Cli, Command, ObjectsCommand, RecordsCommand, WebhooksCommand};
use client::{ApiClient, ApiResponse};

/// Runs one CLI invocation, printing its result and returning the
/// process exit code (0 success, 1 an API/request error, 2 a bad
/// `--data` argument caught before any request was even sent).
pub fn run(cli: Cli) -> i32 {
    let client = ApiClient::new(cli.base_url, cli.api_key);

    let result = match cli.command {
        Command::Limits => client.get("/api/v1/limits"),
        Command::Objects { command } => match command {
            ObjectsCommand::List => client.get("/api/v1/objects"),
            ObjectsCommand::Metadata { object_key } => {
                client.get(&format!("/api/v1/objects/{object_key}/metadata"))
            }
        },
        Command::Records { command } => match command {
            RecordsCommand::List {
                object_key,
                filter,
                sort,
                page,
                page_size,
            } => {
                let mut query: Vec<(&str, String)> = Vec::new();
                if let Some(f) = filter {
                    query.push(("filter", f));
                }
                if let Some(s) = sort {
                    query.push(("sort", s));
                }
                if let Some(p) = page {
                    query.push(("page", p.to_string()));
                }
                if let Some(ps) = page_size {
                    query.push(("page_size", ps.to_string()));
                }
                client.get_with_query(&format!("/api/v1/objects/{object_key}/records"), &query)
            }
            RecordsCommand::Get { object_key, id } => {
                client.get(&format!("/api/v1/objects/{object_key}/records/{id}"))
            }
            RecordsCommand::Create { object_key, data } => match parse_data(&data) {
                Ok(body) => client.post(&format!("/api/v1/objects/{object_key}/records"), &body),
                Err(code) => return code,
            },
            RecordsCommand::Update {
                object_key,
                id,
                data,
            } => match parse_data(&data) {
                Ok(body) => {
                    client.patch(&format!("/api/v1/objects/{object_key}/records/{id}"), &body)
                }
                Err(code) => return code,
            },
            RecordsCommand::Archive { object_key, id } => {
                client.delete(&format!("/api/v1/objects/{object_key}/records/{id}"))
            }
            RecordsCommand::Export { object_key, format } => {
                return print_text(client.get_text(
                    &format!("/api/v1/objects/{object_key}/export"),
                    &[("format", format)],
                ));
            }
            RecordsCommand::Bulk { object_key, data } => match bulk_body(&data) {
                Ok(body) => client.post(&format!("/api/v1/objects/{object_key}/bulk"), &body),
                Err(code) => return code,
            },
        },
        Command::Webhooks { command } => match webhook_request(command) {
            Ok(WebhookRequest::Get(path, query)) => client.get_with_query(&path, &query),
            Ok(WebhookRequest::Post(path, body)) => client.post(&path, &body),
            Ok(WebhookRequest::Patch(path, body)) => client.patch(&path, &body),
            Ok(WebhookRequest::Delete(path)) => client.delete(&path),
            Err(code) => return code,
        },
        Command::Search {
            text,
            object,
            page,
            page_size,
        } => client.get_with_query(
            "/api/v1/search",
            &search_query(text, object, page, page_size),
        ),
    };

    print_result(result)
}

/// The query string `lanesra search` sends (WP-0186).
pub fn search_query(
    text: String,
    object: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Vec<(&'static str, String)> {
    let mut query = vec![("q", text)];
    if let Some(o) = object {
        query.push(("object", o));
    }
    if let Some(p) = page {
        query.push(("page", p.to_string()));
    }
    if let Some(ps) = page_size {
        query.push(("page_size", ps.to_string()));
    }
    query
}

/// The request body for `lanesra records bulk`: `--data` is a JSON array of
/// operations, or `@path` to a file holding one (WP-0187).
pub fn bulk_body(data: &str) -> Result<Value, i32> {
    let text = match data.strip_prefix('@') {
        Some(path) => std::fs::read_to_string(path).map_err(|e| {
            eprintln!("Could not read {path}: {e}");
            2
        })?,
        None => data.to_string(),
    };
    let operations = parse_data(&text)?;
    if !operations.is_array() {
        eprintln!("--data must be a JSON array of operations");
        return Err(2);
    }
    Ok(serde_json::json!({ "operations": operations }))
}

fn print_text(result: Result<client::TextResponse, String>) -> i32 {
    match result {
        Ok(response) if response.status.is_success() => {
            print!("{}", response.body);
            0
        }
        Ok(response) => {
            eprintln!("{}", response.body);
            1
        }
        Err(message) => {
            eprintln!("Request failed: {message}");
            1
        }
    }
}

/// The HTTP call one `lanesra webhooks` command makes (WP-0188).
#[derive(Debug, PartialEq)]
pub enum WebhookRequest {
    Get(String, Vec<(&'static str, String)>),
    Post(String, Value),
    Patch(String, Value),
    Delete(String),
}

pub fn webhook_request(command: WebhooksCommand) -> Result<WebhookRequest, i32> {
    let one = |id: &str| format!("/api/v1/webhooks/{id}");
    let action = |id: &str, verb: &str| format!("/api/v1/webhooks/{id}/{verb}");
    Ok(match command {
        WebhooksCommand::List => WebhookRequest::Get("/api/v1/webhooks".into(), vec![]),
        WebhooksCommand::Get { id } => WebhookRequest::Get(one(&id), vec![]),
        WebhooksCommand::Create { data } => {
            WebhookRequest::Post("/api/v1/webhooks".into(), parse_data(&data)?)
        }
        WebhooksCommand::Update { id, data } => WebhookRequest::Patch(one(&id), parse_data(&data)?),
        WebhooksCommand::Delete { id } => WebhookRequest::Delete(one(&id)),
        WebhooksCommand::Pause { id } => WebhookRequest::Post(action(&id, "pause"), Value::Null),
        WebhooksCommand::Reactivate { id } => {
            WebhookRequest::Post(action(&id, "reactivate"), Value::Null)
        }
        WebhooksCommand::RotateSecret { id } => {
            WebhookRequest::Post(action(&id, "rotate-secret"), Value::Null)
        }
        WebhooksCommand::Test { id } => WebhookRequest::Post(action(&id, "test"), Value::Null),
        WebhooksCommand::Deliveries {
            id,
            page,
            page_size,
        } => {
            let mut query = Vec::new();
            if let Some(p) = page {
                query.push(("page", p.to_string()));
            }
            if let Some(ps) = page_size {
                query.push(("page_size", ps.to_string()));
            }
            WebhookRequest::Get(action(&id, "deliveries"), query)
        }
    })
}

fn parse_data(data: &str) -> Result<Value, i32> {
    serde_json::from_str(data).map_err(|e| {
        eprintln!("Invalid JSON for --data: {e}");
        2
    })
}

fn print_result(result: Result<ApiResponse, String>) -> i32 {
    match result {
        Ok(response) => {
            let pretty = serde_json::to_string_pretty(&response.body)
                .unwrap_or_else(|_| response.body.to_string());
            if response.status.is_success() {
                println!("{pretty}");
                0
            } else {
                eprintln!("{pretty}");
                1
            }
        }
        Err(message) => {
            eprintln!("Request failed: {message}");
            1
        }
    }
}
