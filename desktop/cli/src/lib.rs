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

use cli::{Cli, Command, ObjectsCommand, RecordsCommand};
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
            ObjectsCommand::Metadata { object_key } => client.get(&format!("/api/v1/objects/{object_key}/metadata")),
        },
        Command::Records { command } => match command {
            RecordsCommand::List { object_key, filter, sort, page, page_size } => {
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
            RecordsCommand::Get { object_key, id } => client.get(&format!("/api/v1/objects/{object_key}/records/{id}")),
            RecordsCommand::Create { object_key, data } => match parse_data(&data) {
                Ok(body) => client.post(&format!("/api/v1/objects/{object_key}/records"), &body),
                Err(code) => return code,
            },
            RecordsCommand::Update { object_key, id, data } => match parse_data(&data) {
                Ok(body) => client.patch(&format!("/api/v1/objects/{object_key}/records/{id}"), &body),
                Err(code) => return code,
            },
            RecordsCommand::Archive { object_key, id } => client.delete(&format!("/api/v1/objects/{object_key}/records/{id}")),
        },
    };

    print_result(result)
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
            let pretty = serde_json::to_string_pretty(&response.body).unwrap_or_else(|_| response.body.to_string());
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
