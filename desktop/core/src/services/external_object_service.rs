//! Integration Hub (spec §16): External/Virtual Objects - read-only
//! records surfaced from an external system through an existing
//! Connection, displayed via the same generic object metadata/list shape
//! `api_object_service` already uses for built-in and Custom Object
//! records. Deliberately narrow scope, stated plainly: **read-only**
//! (list only, no get/create/update/archive - the spec's own "virtual"
//! framing is about visibility, not writing back to a system Lanesra
//! doesn't own), and the external response must be a JSON array or an
//! object with a top-level `data`/`items`/`results`/`value` array (the
//! shapes real REST/OData APIs commonly use) - anything else is a clear
//! validation error, not a silent empty list.

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{ExternalObject, ExternalObjectInput};
use crate::repositories::integration_external_object_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &ExternalObjectInput, actor_user_id: Option<&str>) -> AppResult<ExternalObject> {
    require_admin(conn, actor_user_id)?;
    if input.object_key.trim().is_empty() || input.display_name.trim().is_empty() {
        return Err(AppError::Validation("Object key and display name are required".into()));
    }
    if integration_external_object_repo::get_by_object_key(conn, workspace_id, &input.object_key)?.is_some() {
        return Err(AppError::Conflict(format!("An external object with key '{}' already exists in this workspace", input.object_key)));
    }
    let connection = super::connection_service::get(conn, workspace_id, &input.connection_id)?;
    if !matches!(connection.connection_type.as_str(), "rest" | "odata") {
        return Err(AppError::Validation("External Objects need a 'rest' or 'odata' connection".into()));
    }
    let field_map_json = serde_json::to_string(&input.field_map).unwrap_or_else(|_| "[]".to_string());
    Ok(integration_external_object_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        input.object_key.trim(),
        input.display_name.trim(),
        &input.connection_id,
        &input.resource_path,
        &field_map_json,
        input.cache_ttl_seconds,
        actor_user_id,
    )?)
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<ExternalObject> {
    let object = integration_external_object_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("External object".into()))?;
    if object.workspace_id != workspace_id {
        return Err(AppError::NotFound("External object".into()));
    }
    Ok(object)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ExternalObject>> {
    Ok(integration_external_object_repo::list_for_workspace(conn, workspace_id)?)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    if crate::repositories::integration_job_repo::count_by_external_object(conn, id)? > 0 {
        return Err(AppError::Validation("This External Object still has an Integration Job pointing at it - delete or repoint the Job first".into()));
    }
    Ok(integration_external_object_repo::delete(conn, id)?)
}

fn extract_array(body: &Value) -> AppResult<Vec<Value>> {
    match body {
        Value::Array(items) => Ok(items.clone()),
        Value::Object(obj) => {
            for key in ["data", "items", "results", "value"] {
                if let Some(Value::Array(items)) = obj.get(key) {
                    return Ok(items.clone());
                }
            }
            Err(AppError::Validation("Response was a JSON object with no recognized 'data'/'items'/'results'/'value' array - could not list records".into()))
        }
        _ => Err(AppError::Validation("Response was neither a JSON array nor an object wrapping one".into())),
    }
}

fn apply_field_map(record: &Value, def: &ExternalObject) -> Value {
    if def.field_map.is_empty() {
        return record.clone();
    }
    let mut mapped = serde_json::Map::new();
    for entry in &def.field_map {
        if let Some(value) = record.get(&entry.source_column) {
            mapped.insert(entry.target_field.clone(), value.clone());
        } else if let Some(default) = &entry.default_value {
            mapped.insert(entry.target_field.clone(), Value::String(default.clone()));
        }
    }
    Value::Object(mapped)
}

/// Shared by `list_records` and `list_records_by_id`/`integration_job_service`
/// - a real GET against `resource_path` on the bound Connection's base
/// URL, not a cached/simulated response (spec §16: "read-only, live or
/// cached per `cache_ttl_seconds`" - the cache layer itself is left to a
/// future pass; every call here is live). `since`, when given, is
/// appended as a plain `?since=<value>` (or `&since=...` if
/// `resource_path` already has a query string) query parameter - this is
/// the Integration Job checkpoint mechanism (spec §15); not URL-encoded,
/// a stated simplification fine for the token/timestamp-shaped cursor
/// values this build produces, not arbitrary user text.
async fn fetch_records(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], def: &ExternalObject, since: Option<&str>) -> AppResult<Vec<Value>> {
    let connection = super::connection_service::get(conn, workspace_id, &def.connection_id)?;
    let secret = super::connection_service::resolve_secret(conn, master_key, &def.connection_id)?;
    let base_url = connection.base_url.as_deref().ok_or_else(|| AppError::Validation("The bound connection has no base URL configured".into()))?;
    let mut url = format!("{}{}", base_url.trim_end_matches('/'), def.resource_path);
    if let Some(since) = since.filter(|s| !s.is_empty()) {
        let sep = if url.contains('?') { '&' } else { '?' };
        url = format!("{url}{sep}since={since}");
    }

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build().map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let builder = super::connection_service::apply_auth(client.get(&url), &connection.auth_mode, secret.as_deref());
    let response = builder.send().await.map_err(|e| AppError::Validation(format!("Could not reach {url}: {e}")))?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Validation(format!("External system responded with HTTP {status}")));
    }
    let body: Value = response.json().await.map_err(|e| AppError::Validation(format!("Response was not valid JSON: {e}")))?;
    let records = extract_array(&body)?;
    Ok(records.iter().map(|r| apply_field_map(r, def)).collect())
}

/// Fetches and lists this External Object's current records, looked up
/// by its `object_key` - the shape the future generic object browser/
/// REST surface will call.
pub async fn list_records(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], object_key: &str) -> AppResult<Vec<Value>> {
    let def = integration_external_object_repo::get_by_object_key(conn, workspace_id, object_key)?.ok_or_else(|| AppError::NotFound(format!("External object '{object_key}'")))?;
    fetch_records(conn, workspace_id, master_key, &def, None).await
}

/// Same fetch, looked up by id and with an optional checkpoint value -
/// what `integration_job_service::run_now` calls, since a Job stores the
/// External Object's id, not its (mutable) `object_key`.
pub async fn list_records_by_id(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], external_object_id: &str, since: Option<&str>) -> AppResult<Vec<Value>> {
    let def = integration_external_object_repo::get(conn, external_object_id)?.ok_or_else(|| AppError::NotFound("External object".into()))?;
    if def.workspace_id != workspace_id {
        return Err(AppError::NotFound("External object".into()));
    }
    fetch_records(conn, workspace_id, master_key, &def, since).await
}
