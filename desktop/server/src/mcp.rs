//! AI & Agentic Layer, phase 2: an MCP (Model Context Protocol) server
//! exposing the exact same `api_object_service` dispatcher the inbound
//! `/api/v1` REST API (`api_v1.rs`) already wraps - so an MCP-capable
//! agent (Claude, or any other) reads and writes Lanesra records under
//! the identical permission/validation/business-rule checks a human's UI
//! action already goes through, not a separately-checked integration
//! surface. See this crate's own doc comment on `api_v1` for why this
//! only ever runs in Team Workspace mode: a pure desktop install has no
//! listening socket for an external agent to reach either.
//!
//! Transport is a single stateless endpoint, `POST /mcp`: one JSON-RPC
//! 2.0 request in, one JSON-RPC response out. No SSE/server-initiated
//! notifications - every tool call here is a synchronous read or write
//! against SQLite, nothing long-running to stream, and the MCP spec
//! explicitly allows a server to respond with a single JSON body instead
//! of an event stream when it has nothing to push asynchronously.
//!
//! Auth reuses `api_v1::authorize` exactly - the same `Authorization:
//! Bearer {client_id}.{secret}` API-client credential, the same
//! `RateLimiter`, the same scope vocabulary (`metadata.read` /
//! `objects.read` / `objects.write`). There is no separate MCP
//! credential type: an admin issues (or reuses) an API client from the
//! existing Integration Hub -> API Access screen and pastes it into an
//! MCP client's config. Every call - success or failure - is logged
//! through `api_v1::logged`, the same executions log a REST call or
//! webhook delivery already lands in.
//!
//! JSON-RPC error taxonomy used below: a malformed request, an unknown
//! method/tool, a missing required argument, or a missing scope is a
//! **protocol-level** JSON-RPC error object (the call never reached
//! `api_object_service`). An `AppError` raised by actually invoking a
//! valid tool with a bad id or invalid field value is a **tool-level**
//! error instead - returned as a normal JSON-RPC *result* with
//! `isError: true` and the message in `content`, per the MCP convention
//! that lets the calling agent see the failure and adapt, rather than
//! treating it as a broken connection.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};

use lanesra_core::models::integration::{ApiClient, ApiListQuery};
use lanesra_core::services::{api_client_service, api_object_service};

use crate::api_v1::{authorize, err_json, logged};
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new().route("/mcp", post(handle))
}

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "lanesra-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[serde(default)]
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

/// One JSON-RPC-protocol-level error: `(code, message)`. Distinct from a
/// tool-level `AppError`, which becomes an `isError: true` *result*
/// instead - see this module's own doc comment.
type RpcError = (i64, String);

async fn handle(State(state): State<SharedState>, headers: HeaderMap, Json(body): Json<Value>) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Every method needs at least a valid, active API client - `tools/call`
    // additionally checks the specific tool's own scope below, the same
    // two-tier check `api_v1.rs`'s routes already do per-route.
    let (client, workspace_id) = authorize(&state, &headers, "metadata.read")?;

    let req: RpcRequest = serde_json::from_value(body).map_err(|e| err_json(StatusCode::BAD_REQUEST, &format!("Malformed JSON-RPC request: {e}")))?;

    if req.method == "notifications/initialized" {
        // A notification - no id, no response body expected.
        return Ok((StatusCode::ACCEPTED, Json(json!({}))));
    }

    let outcome: Result<Value, RpcError> = match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tool_definitions()})),
        "tools/call" => handle_tools_call(&state, &client, &workspace_id, &req.params),
        other => Err((-32601, format!("Method not found: {other}"))),
    };

    let body = match outcome {
        Ok(result) => json!({"jsonrpc": "2.0", "id": req.id, "result": result}),
        Err((code, message)) => json!({"jsonrpc": "2.0", "id": req.id, "error": {"code": code, "message": message}}),
    };
    Ok((StatusCode::OK, Json(body)))
}

fn tool_definitions() -> Vec<Value> {
    let object_key_prop = json!({"type": "string", "description": "An object key from list_objects, e.g. 'Company' or a custom object's key"});
    let id_prop = json!({"type": "string", "description": "The record's id"});
    vec![
        json!({
            "name": "list_objects",
            "description": "List every built-in and custom object this workspace exposes, with its label and whether it's custom.",
            "inputSchema": {"type": "object", "properties": {}},
        }),
        json!({
            "name": "get_object_metadata",
            "description": "Get an object's label and its custom field definitions (key, label, type, required).",
            "inputSchema": {"type": "object", "properties": {"object_key": object_key_prop}, "required": ["object_key"]},
        }),
        json!({
            "name": "list_records",
            "description": "List records for an object, paginated. Optional 'filter' (an object of exact-match field/value pairs), 'sort' (array of field names, prefix with '-' for descending), 'page' (1-based), 'page_size' (max 500).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "object_key": object_key_prop,
                    "filter": {"type": "object", "description": "Exact-match field/value pairs"},
                    "sort": {"type": "array", "items": {"type": "string"}},
                    "page": {"type": "integer"},
                    "page_size": {"type": "integer"},
                },
                "required": ["object_key"],
            },
        }),
        json!({
            "name": "get_record",
            "description": "Get a single record by id.",
            "inputSchema": {"type": "object", "properties": {"object_key": object_key_prop, "id": id_prop}, "required": ["object_key", "id"]},
        }),
        json!({
            "name": "create_record",
            "description": "Create a record. 'data' is the record's fields as an object - the same shape POST /api/v1/objects/{key}/records accepts. Company, Contact, Product, Task and active Custom Objects only; Opportunity/Quote/Order/Invoice/Contract are read-only here.",
            "inputSchema": {"type": "object", "properties": {"object_key": object_key_prop, "data": {"type": "object"}}, "required": ["object_key", "data"]},
        }),
        json!({
            "name": "update_record",
            "description": "Update a record's fields by id. 'data' holds only the fields being changed.",
            "inputSchema": {"type": "object", "properties": {"object_key": object_key_prop, "id": id_prop, "data": {"type": "object"}}, "required": ["object_key", "id", "data"]},
        }),
        json!({
            "name": "archive_record",
            "description": "Archive (soft-delete) a record by id.",
            "inputSchema": {"type": "object", "properties": {"object_key": object_key_prop, "id": id_prop}, "required": ["object_key", "id"]},
        }),
    ]
}

fn tool_ok(value: &Value) -> Value {
    json!({"content": [{"type": "text", "text": value.to_string()}], "isError": false})
}

fn tool_err(message: &str) -> Value {
    json!({"content": [{"type": "text", "text": message}], "isError": true})
}

fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, RpcError> {
    args.get(key).and_then(Value::as_str).ok_or_else(|| (-32602, format!("Missing required argument '{key}'")))
}

fn required_object<'a>(args: &'a Value, key: &str) -> Result<&'a Value, RpcError> {
    let v = args.get(key).ok_or_else(|| (-32602, format!("Missing required argument '{key}'")))?;
    if v.is_object() { Ok(v) } else { Err((-32602, format!("'{key}' must be an object"))) }
}

fn handle_tools_call(state: &SharedState, client: &ApiClient, workspace_id: &str, params: &Value) -> Result<Value, RpcError> {
    let name = required_str(params, "name")?;
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

    let needed_scope = match name {
        "list_objects" | "get_object_metadata" => "metadata.read",
        "list_records" | "get_record" => "objects.read",
        "create_record" | "update_record" | "archive_record" => "objects.write",
        other => return Err((-32602, format!("Unknown tool '{other}'"))),
    };
    if !api_client_service::has_scope(client, needed_scope) {
        return Err((-32000, format!("This API client lacks the '{needed_scope}' scope required by tool '{name}'")));
    }

    let conn = state.conn.lock().unwrap();
    Ok(match name {
        "list_objects" => call(&conn, workspace_id, api_object_service::list_object_keys(&conn, workspace_id).map(|v| serde_json::to_value(v).unwrap_or(Value::Null))),
        "get_object_metadata" => {
            let key = required_str(&arguments, "object_key")?;
            call(&conn, workspace_id, api_object_service::get_metadata(&conn, workspace_id, key).map(|v| serde_json::to_value(v).unwrap_or(Value::Null)))
        }
        "list_records" => {
            let key = required_str(&arguments, "object_key")?;
            let query: ApiListQuery = serde_json::from_value(arguments.clone()).unwrap_or_default();
            call(&conn, workspace_id, api_object_service::list_records(&conn, workspace_id, key, &query).map(|v| serde_json::to_value(v).unwrap_or(Value::Null)))
        }
        "get_record" => {
            let key = required_str(&arguments, "object_key")?;
            let id = required_str(&arguments, "id")?;
            call(&conn, workspace_id, api_object_service::get_record(&conn, workspace_id, key, id))
        }
        "create_record" => {
            let key = required_str(&arguments, "object_key")?;
            let data = required_object(&arguments, "data")?;
            call(&conn, workspace_id, api_object_service::create_record(&conn, workspace_id, key, data, None))
        }
        "update_record" => {
            let key = required_str(&arguments, "object_key")?;
            let id = required_str(&arguments, "id")?;
            let data = required_object(&arguments, "data")?;
            call(&conn, workspace_id, api_object_service::update_record(&conn, workspace_id, key, id, data, None))
        }
        "archive_record" => {
            let key = required_str(&arguments, "object_key")?;
            let id = required_str(&arguments, "id")?;
            call(&conn, workspace_id, api_object_service::archive_record(&conn, workspace_id, key, id, None).map(|_| json!({"archived": true})))
        }
        _ => unreachable!("scope match above already rejects any other name"),
    })
}

/// Runs one dispatcher call through the same execution log every REST
/// route already writes to (`api_v1::logged`), then maps the outcome
/// into the tool-level `content`/`isError` shape - never a JSON-RPC
/// protocol error, per this module's own doc comment.
fn call(conn: &Connection, workspace_id: &str, result: lanesra_core::domain::AppResult<Value>) -> Value {
    match logged(conn, workspace_id, result) {
        Ok(value) => tool_ok(&value),
        Err(e) => tool_err(&e.to_string()),
    }
}
