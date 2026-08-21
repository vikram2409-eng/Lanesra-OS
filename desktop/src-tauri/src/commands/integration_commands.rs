//! Integration Hub: Tauri commands for every admin-side operation built
//! in `lanesra-core`'s `integration_*`/connection/connector/api_client/
//! webhook/mapping/external_object/data_exchange services. Plain sync
//! commands mirror every other resource in this codebase 1:1 with their
//! `server::dispatch` counterpart. The handful of genuinely-async
//! operations (`test_connection`, `test_connector_action`,
//! `test_webhook_delivery`, `run_integration_job_now`,
//! `preview_external_object_records`) each run via
//! `run_with_own_connection` - see that helper's own doc comment for why
//! a plain `.await` against `state.conn` doesn't compile here.

use std::future::Future;
use std::path::PathBuf;

use tauri::State;

use crate::commands::{current_actor, require_workspace_id, resolve_master_key};
use crate::state::AppState;
use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::integration::{
    ApiClientInput, ApiListQuery, ConnectionInput, ConnectionRefInput, ConnectionTestResult, ConnectionUpdate, Connector,
    ConnectorImportInput, CsvImportInput, ExternalObject, ExternalObjectInput, IntegrationJob, IntegrationJobInput, IntegrationJobRun,
    IntegrationSettingsUpdate, IssuedApiClient, MappingInput, OpenApiImportPreview, WebhookInput,
};
use lanesra_core::services::{
    api_client_service, api_object_service, connection_ref_service, connection_service, connector_execution_service, connector_service,
    data_exchange_service, external_object_service, integration_job_service, integration_log_service, mapping_service, webhook_service,
};

/// Runs `f` - an async closure handed a fresh, exclusively-owned
/// `rusqlite::Connection` opened from `db_path` - to completion on a
/// blocking-pool thread with its own single-threaded Tokio runtime, and
/// returns the result. Every genuinely-async command below needs this
/// rather than a plain `.await` against `state.conn`: Tauri (like axum)
/// runs an async command's future through its own runtime, which
/// requires that future to be `Send`; `rusqlite::Connection` isn't
/// `Sync`, so a `&Connection` held across an inner `.await` (which every
/// one of these core functions does, since they interleave sync DB work
/// with a real outbound network call) can never be `Send`. Building a
/// throwaway `current_thread` runtime and driving it with `block_on` -
/// which has no such requirement - sidesteps the problem entirely,
/// exactly like `lanesra-server`'s `job_scheduler` does for the same
/// reason.
async fn run_with_own_connection<T, F, Fut>(db_path: PathBuf, f: F) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce(rusqlite::Connection) -> Fut + Send + 'static,
    Fut: Future<Output = AppResult<T>>,
{
    tauri::async_runtime::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AppError::Validation(format!("could not start a background runtime: {e}")))?;
        let conn = lanesra_core::db::open_workspace_db(&db_path)?;
        rt.block_on(f(conn))
    })
    .await
    .map_err(|e| AppError::Validation(format!("background task panicked: {e}")))?
}

// --- Connections (spec §4) ---------------------------------------------------

#[tauri::command]
pub fn list_connections(state: State<AppState>) -> AppResult<Vec<lanesra_core::models::integration::Connection>> {
    let conn = state.conn.lock().unwrap();
    connection_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_connection(state: State<AppState>, input: ConnectionInput) -> AppResult<lanesra_core::models::integration::Connection> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connection_service::create(&conn, &workspace_id, &master_key, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_connection(state: State<AppState>, id: String, input: ConnectionUpdate) -> AppResult<lanesra_core::models::integration::Connection> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connection_service::update(&conn, &workspace_id, &master_key, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_connection(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connection_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

/// Opens its own connection - see `run_with_own_connection`'s doc comment.
#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>, id: String) -> AppResult<ConnectionTestResult> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = { let conn = state.conn.lock().unwrap(); require_workspace_id(&conn)? };
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move {
        connection_service::test_connection(&conn, &workspace_id, &master_key, &id, actor.as_deref()).await
    })
    .await
}

// --- Connection References (spec §5) ----------------------------------------

#[tauri::command]
pub fn list_connection_refs(state: State<AppState>) -> AppResult<Vec<lanesra_core::models::integration::ConnectionRef>> {
    let conn = state.conn.lock().unwrap();
    connection_ref_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_connection_ref(state: State<AppState>, input: ConnectionRefInput) -> AppResult<lanesra_core::models::integration::ConnectionRef> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connection_ref_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bind_connection_ref(state: State<AppState>, id: String, connection_id: Option<String>) -> AppResult<lanesra_core::models::integration::ConnectionRef> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connection_ref_service::bind(&conn, &workspace_id, &id, connection_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_connection_ref(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connection_ref_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

// --- Connectors (spec §6) ----------------------------------------------------

#[tauri::command]
pub fn preview_connector_import(spec_text: String, spec_format: String) -> AppResult<OpenApiImportPreview> {
    connector_service::preview_import(&spec_text, &spec_format)
}

#[tauri::command]
pub fn import_connector(state: State<AppState>, input: ConnectorImportInput) -> AppResult<Connector> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connector_service::import(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_connectors(state: State<AppState>) -> AppResult<Vec<Connector>> {
    let conn = state.conn.lock().unwrap();
    connector_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn get_connector(state: State<AppState>, id: String) -> AppResult<Connector> {
    let conn = state.conn.lock().unwrap();
    connector_service::get(&conn, &require_workspace_id(&conn)?, &id)
}

#[tauri::command]
pub fn delete_connector(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    connector_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

/// A manual "Test Action" run - opens its own connection, same reason as
/// `test_connection` above.
#[tauri::command]
pub async fn test_connector_action(state: State<'_, AppState>, connector_id: String, action_key: String, reference_key: String, params: serde_json::Value) -> AppResult<lanesra_core::models::integration::ConnectorExecutionResult> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = { let conn = state.conn.lock().unwrap(); require_workspace_id(&conn)? };
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move {
        connector_execution_service::execute(&conn, &workspace_id, &master_key, &connector_id, &action_key, &reference_key, &params, actor.as_deref()).await
    })
    .await
}

// --- API Access (spec §8) ----------------------------------------------------

#[tauri::command]
pub fn list_api_clients(state: State<AppState>) -> AppResult<Vec<lanesra_core::models::integration::ApiClient>> {
    let conn = state.conn.lock().unwrap();
    api_client_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_api_client(state: State<AppState>, input: ApiClientInput) -> AppResult<IssuedApiClient> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    api_client_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn rotate_api_client_secret(state: State<AppState>, id: String) -> AppResult<IssuedApiClient> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    api_client_service::rotate_secret(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn revoke_api_client(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    api_client_service::revoke(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn reactivate_api_client(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    api_client_service::reactivate(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_api_client(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    api_client_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

// --- Webhooks & Events (spec §10) --------------------------------------------

#[tauri::command]
pub fn list_webhooks(state: State<AppState>) -> AppResult<Vec<lanesra_core::models::integration::Webhook>> {
    let conn = state.conn.lock().unwrap();
    webhook_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_webhook(state: State<AppState>, input: WebhookInput) -> AppResult<lanesra_core::models::integration::Webhook> {
    let master_key = resolve_master_key(&state)?;
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    webhook_service::create(&conn, &workspace_id, &master_key, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_webhook_deliveries(state: State<AppState>, webhook_id: String) -> AppResult<Vec<lanesra_core::models::integration::WebhookDelivery>> {
    let conn = state.conn.lock().unwrap();
    webhook_service::list_deliveries(&conn, &require_workspace_id(&conn)?, &webhook_id)
}

#[tauri::command]
pub fn pause_webhook(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    webhook_service::pause(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn reactivate_webhook(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    webhook_service::reactivate(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_webhook(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    webhook_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

/// Opens its own connection - see `run_with_own_connection`'s doc comment.
#[tauri::command]
pub async fn test_webhook_delivery(state: State<'_, AppState>, webhook_id: String) -> AppResult<()> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = { let conn = state.conn.lock().unwrap(); require_workspace_id(&conn)? };
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move {
        webhook_service::test_delivery(&conn, &workspace_id, &master_key, &webhook_id, actor.as_deref()).await
    })
    .await
}

// --- Mappings (spec §14) ------------------------------------------------------

#[tauri::command]
pub fn list_mappings(state: State<AppState>) -> AppResult<Vec<lanesra_core::models::integration::Mapping>> {
    let conn = state.conn.lock().unwrap();
    mapping_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_mapping(state: State<AppState>, input: MappingInput) -> AppResult<lanesra_core::models::integration::Mapping> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mapping_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_mapping(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    mapping_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

// --- Data Exchange (spec §12/§13) --------------------------------------------

#[tauri::command]
pub fn import_csv(state: State<AppState>, input: CsvImportInput) -> AppResult<lanesra_core::models::integration::CsvImportResult> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    data_exchange_service::import_csv(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn export_csv(state: State<AppState>, object_key: String, query: ApiListQuery) -> AppResult<String> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    data_exchange_service::export_csv(&conn, &workspace_id, &object_key, &query)
}

#[tauri::command]
pub fn list_integration_object_keys(state: State<AppState>) -> AppResult<Vec<lanesra_core::models::integration::ApiObjectMetadata>> {
    let conn = state.conn.lock().unwrap();
    api_object_service::list_object_keys(&conn, &require_workspace_id(&conn)?)
}

// --- External / Virtual Objects (spec §16) -----------------------------------

#[tauri::command]
pub fn list_external_objects(state: State<AppState>) -> AppResult<Vec<ExternalObject>> {
    let conn = state.conn.lock().unwrap();
    external_object_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_external_object(state: State<AppState>, input: ExternalObjectInput) -> AppResult<ExternalObject> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    external_object_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_external_object(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    external_object_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

/// A live preview of this External Object's current records - opens its
/// own connection, see `run_with_own_connection`'s doc comment.
#[tauri::command]
pub async fn preview_external_object_records(state: State<'_, AppState>, object_key: String) -> AppResult<Vec<serde_json::Value>> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = { let conn = state.conn.lock().unwrap(); require_workspace_id(&conn)? };
    run_with_own_connection(db_path, move |conn| async move {
        external_object_service::list_records(&conn, &workspace_id, &master_key, &object_key).await
    })
    .await
}

// --- Integration Jobs (spec §15) ---------------------------------------------

#[tauri::command]
pub fn list_integration_jobs(state: State<AppState>) -> AppResult<Vec<IntegrationJob>> {
    let conn = state.conn.lock().unwrap();
    integration_job_service::list_for_workspace(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_integration_job(state: State<AppState>, input: IntegrationJobInput) -> AppResult<IntegrationJob> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    integration_job_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_integration_job(state: State<AppState>, id: String, input: IntegrationJobInput, status: String) -> AppResult<IntegrationJob> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    integration_job_service::update(&conn, &workspace_id, &id, &input, &status, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_integration_job(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    integration_job_service::delete(&conn, &workspace_id, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_integration_job_runs(state: State<AppState>, job_id: String, limit: i64) -> AppResult<Vec<IntegrationJobRun>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    integration_job_service::list_runs(&conn, &workspace_id, &job_id, limit)
}

/// Manual "Run Now" - opens its own connection, see this module's own
/// doc comment. Desktop has no OS-level scheduler for these (see
/// `integration_job_service`'s own doc comment on why), so this is also
/// the *only* way a desktop-hosted Job ever runs - a real future gap
/// worth a client-poll equivalent, not attempted this pass.
#[tauri::command]
pub async fn run_integration_job_now(state: State<'_, AppState>, id: String) -> AppResult<IntegrationJobRun> {
    let master_key = resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let workspace_id = { let conn = state.conn.lock().unwrap(); require_workspace_id(&conn)? };
    let actor = current_actor(&state);
    run_with_own_connection(db_path, move |conn| async move {
        integration_job_service::run_now(&conn, &workspace_id, &master_key, &id, actor.as_deref()).await
    })
    .await
}

// --- Logs & Monitoring / Settings (spec §21/§22/§23) -------------------------

#[tauri::command]
pub fn get_integration_overview(state: State<AppState>) -> AppResult<lanesra_core::models::integration::IntegrationOverview> {
    let conn = state.conn.lock().unwrap();
    integration_log_service::overview(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn list_integration_executions(state: State<AppState>, query: integration_log_service::ExecutionQuery) -> AppResult<Vec<lanesra_core::models::integration::IntegrationExecution>> {
    let conn = state.conn.lock().unwrap();
    integration_log_service::list_executions(&conn, &require_workspace_id(&conn)?, &query)
}

#[tauri::command]
pub fn get_integration_settings(state: State<AppState>) -> AppResult<lanesra_core::models::integration::IntegrationSettings> {
    let conn = state.conn.lock().unwrap();
    integration_log_service::get_settings(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn update_integration_settings(state: State<AppState>, input: IntegrationSettingsUpdate) -> AppResult<lanesra_core::models::integration::IntegrationSettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    integration_log_service::update_settings(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn purge_expired_integration_logs(state: State<AppState>) -> AppResult<usize> {
    let conn = state.conn.lock().unwrap();
    integration_log_service::purge_expired(&conn, &require_workspace_id(&conn)?)
}
