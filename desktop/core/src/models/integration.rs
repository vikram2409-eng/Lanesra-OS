//! Integration Hub (Lanesra_OS_Integration_Hub_Admin_Design_Development_Spec_v1.0)
//! - every model this feature's services/repositories operate on, grouped
//! by resource with a section comment each rather than split into many
//! small files (matches `models::industry_package`'s own precedent for a
//! comparably multi-shaped subsystem). See migration 0032's own comment
//! for the full table list and `services::connection_service` and
//! friends for the behavior built on top of these.

use serde::{Deserialize, Serialize};

// --- Connections (spec §4) --------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub connection_type: String,
    pub base_url: Option<String>,
    pub auth_mode: String,
    /// Never populated with the real secret value - only whether one
    /// exists, so the admin UI can show "configured" without ever
    /// receiving the plaintext back (spec §4.3: "never returned in
    /// plaintext after creation").
    pub has_secret: bool,
    pub config_json: String,
    pub owner_user_id: Option<String>,
    pub status: String,
    pub last_test_at: Option<String>,
    pub last_test_status: Option<String>,
    pub last_test_message: Option<String>,
    pub last_failure_at: Option<String>,
    pub credential_expires_at: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionInput {
    pub name: String,
    pub connection_type: String,
    pub base_url: Option<String>,
    pub auth_mode: String,
    /// The real secret value (API key/bearer token/basic-auth password/
    /// SFTP or Postgres password/SMTP password), plaintext over the wire
    /// exactly once - encrypted at rest immediately, never stored or
    /// echoed back as-is. `None`/absent when `auth_mode == "none"`.
    pub secret_value: Option<String>,
    pub config_json: String,
    pub owner_user_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionUpdate {
    pub name: String,
    pub base_url: Option<String>,
    pub auth_mode: String,
    /// `Some("")` or omitted leaves the existing secret untouched;
    /// `Some(value)` rotates it.
    pub secret_value: Option<String>,
    pub config_json: String,
    pub owner_user_id: Option<String>,
    pub status: String,
}

/// What `connection_service::test_connection` reports - spec §4.4: "Test
/// result displays latency, HTTP/error status and remediation message
/// without exposing secrets."
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionTestResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub status_code: Option<u16>,
    pub message: String,
}

// --- Connection References (spec §5) ----------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionRef {
    pub id: String,
    pub workspace_id: String,
    pub reference_name: String,
    pub reference_key: String,
    pub expected_connection_type: String,
    pub connection_id: Option<String>,
    pub connection_name: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionRefInput {
    pub reference_name: String,
    pub reference_key: String,
    pub expected_connection_type: String,
    pub connection_id: Option<String>,
}

// --- API Access (spec §8) ----------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ApiClient {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub client_id: String,
    pub status: String,
    pub scopes: Vec<String>,
    pub allowed_cidr: Option<String>,
    pub owner_user_id: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiClientInput {
    pub name: String,
    pub scopes: Vec<String>,
    pub allowed_cidr: Option<String>,
    pub owner_user_id: Option<String>,
}

/// Returned exactly once, at creation (and at rotation) - spec §8.1:
/// "Secrets shown only once at creation."
#[derive(Debug, Clone, Serialize)]
pub struct IssuedApiClient {
    pub client: ApiClient,
    /// `"{client_id}.{secret}"` - the full bearer token to hand to the
    /// integration's owner. Never recoverable again after this response.
    pub api_key: String,
}

// --- Webhooks & Events (spec §10) -------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Webhook {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub connection_id: String,
    pub endpoint_url: Option<String>,
    pub event_types: Vec<String>,
    pub object_scope: Option<String>,
    pub filter_json: Option<String>,
    pub payload_version: String,
    pub has_secret: bool,
    pub retry_policy_json: String,
    pub status: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookInput {
    pub name: String,
    pub connection_id: String,
    pub event_types: Vec<String>,
    pub object_scope: Option<String>,
    pub filter_json: Option<String>,
    pub payload_version: Option<String>,
    pub retry_policy_json: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event_id: String,
    pub event_type: String,
    pub attempt_number: i64,
    pub status: String,
    pub http_status: Option<i64>,
    pub duration_ms: Option<i64>,
    pub response_snippet: Option<String>,
    pub created_at: String,
}

/// The internal event this event-family fires for (spec table 12) - what
/// `record.created`/`updated`/`archived`/`field.changed` actually carry.
#[derive(Debug, Clone, Serialize)]
pub struct IntegrationEvent {
    pub event_id: String,
    pub event_type: String,
    pub workspace_id: String,
    pub object_key: String,
    pub record_id: String,
    pub occurred_at: String,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
}

// --- Mappings (spec §14) -----------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapEntry {
    pub source_column: String,
    pub target_field: String,
    /// One of "none" | "trim" | "uppercase" | "lowercase" | "concatenate"
    /// | "numeric" | "date". See mapping_service::apply_transform.
    pub transform: Option<String>,
    pub default_value: Option<String>,
    pub constant: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Mapping {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub target_object_key: String,
    pub operation: String,
    pub match_key: Option<String>,
    pub field_map: Vec<FieldMapEntry>,
    pub duplicate_policy: String,
    pub needs_review: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MappingInput {
    pub name: String,
    pub target_object_key: String,
    pub operation: String,
    pub match_key: Option<String>,
    pub field_map: Vec<FieldMapEntry>,
    pub duplicate_policy: String,
}

// --- Data Exchange (CSV import/export, spec §12/13) -------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CsvImportInput {
    pub target_object_key: String,
    pub csv_text: String,
    pub operation: String,
    pub match_key: Option<String>,
    pub field_map: Vec<FieldMapEntry>,
    pub duplicate_policy: String,
    /// When true, only validates and reports what *would* happen -
    /// nothing is written (spec §12.1 step 8 "Validate and preview").
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvRowResult {
    pub row_index: usize,
    pub status: String,
    pub record_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvImportResult {
    pub total_rows: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped_duplicates: usize,
    pub row_results: Vec<CsvRowResult>,
    pub duration_ms: u64,
}

// --- Unified logs (spec §23) -------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationExecution {
    pub id: String,
    pub workspace_id: String,
    pub execution_type: String,
    pub correlation_id: Option<String>,
    pub ref_id: Option<String>,
    pub direction: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub http_status: Option<i64>,
    pub records_read: i64,
    pub records_written: i64,
    pub records_skipped: i64,
    pub records_failed: i64,
    pub retry_count: i64,
    pub error_category: Option<String>,
    pub error_message: Option<String>,
    pub actor_user_id: Option<String>,
}

/// The Overview screen's KPI row - real aggregates over
/// `integration_executions`/`integration_connections`/
/// `integration_webhook_deliveries`, not a placeholder.
#[derive(Debug, Clone, Serialize)]
pub struct IntegrationOverview {
    pub active_connections: i64,
    pub failed_connections: i64,
    pub api_calls_today: i64,
    pub failed_webhooks_today: i64,
    pub jobs_running: i64,
    pub jobs_failed_today: i64,
}

// --- Settings (spec §21/22) --------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationSettings {
    pub workspace_id: String,
    pub api_rate_limit_per_minute: i64,
    pub global_rate_limit_per_minute: i64,
    pub log_retention_days: i64,
    pub file_retention_days: i64,
    pub allow_insecure_connections: bool,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationSettingsUpdate {
    pub api_rate_limit_per_minute: i64,
    pub global_rate_limit_per_minute: i64,
    pub log_retention_days: i64,
    pub file_retention_days: i64,
    pub allow_insecure_connections: bool,
}

// --- Connectors (spec §6) ----------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Connector {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub connection_type: String,
    pub spec_source: String,
    pub publisher_id: Option<String>,
    pub actions: Vec<ConnectorAction>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorActionParam {
    pub name: String,
    /// "path" | "query" | "header" | "body"
    pub location: String,
    pub required: bool,
    pub schema_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectorAction {
    pub id: String,
    pub connector_id: String,
    pub action_key: String,
    pub display_name: String,
    pub http_method: String,
    pub path_template: String,
    pub params: Vec<ConnectorActionParam>,
    pub request_schema_json: Option<String>,
    pub response_schema_json: Option<String>,
}

/// What OpenAPI import (spec §6.2) reports back before anything is saved -
/// "Admin chooses which operations to expose" (step 4).
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredOperation {
    pub operation_id: String,
    pub http_method: String,
    pub path_template: String,
    pub summary: Option<String>,
    pub params: Vec<ConnectorActionParam>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenApiImportPreview {
    pub title: String,
    pub version: String,
    pub operations: Vec<DiscoveredOperation>,
    /// Constructs this import encountered but could not represent - spec
    /// §6.2: "Reject unsupported OpenAPI constructs with actionable
    /// warnings", surfaced rather than silently dropped.
    pub warnings: Vec<String>,
}

/// What `connector_execution_service::execute` reports - the Workflow
/// "Call Connector Action" step (spec §17) surfaces this directly rather
/// than a bare HTTP response, so a workflow author can branch on `ok`
/// without parsing status codes themselves.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectorExecutionResult {
    pub ok: bool,
    pub status_code: Option<u16>,
    pub duration_ms: u64,
    pub response_body: serde_json::Value,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectorImportInput {
    pub name: String,
    pub description: Option<String>,
    pub spec_text: String,
    /// "json" | "yaml"
    pub spec_format: String,
    /// Which of the preview's discovered `operation_id`s to actually save
    /// as Actions - spec §6.2 step 4's "Admin chooses which operations to
    /// expose".
    pub selected_operation_ids: Vec<String>,
}

// Integration Jobs (spec §15) models live further down this file, next to
// External/Virtual Objects (§16) - a Job's pull source - rather than here;
// an earlier, never-wired-up sketch of this section (direction/connection_id/
// mapping_id/schedule/batch_size/etc., with no repo or service ever built
// against it) was removed rather than kept alongside the real one.

// --- External / Virtual Objects (spec §16) ----------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ExternalObject {
    pub id: String,
    pub workspace_id: String,
    pub object_key: String,
    pub display_name: String,
    pub connection_id: String,
    pub resource_path: String,
    pub field_map: Vec<FieldMapEntry>,
    pub cache_ttl_seconds: Option<i64>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalObjectInput {
    pub object_key: String,
    pub display_name: String,
    pub connection_id: String,
    pub resource_path: String,
    pub field_map: Vec<FieldMapEntry>,
    pub cache_ttl_seconds: Option<i64>,
}

// --- Generic object dispatcher (backs the REST API, Bulk API, CSV wizard,
// and External Objects) -------------------------------------------------

/// One column's worth of metadata for `GET /api/v1/objects/{key}/metadata`
/// - deliberately the same shape whether `object_key` resolves to a
/// built-in entity or a Custom Object, since both go through
/// `api_object_service`.
#[derive(Debug, Clone, Serialize)]
pub struct ApiFieldMetadata {
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiObjectMetadata {
    pub object_key: String,
    pub label: String,
    pub is_custom: bool,
    pub fields: Vec<ApiFieldMetadata>,
}

/// The generic paged list result `GET /api/v1/objects/{key}/records`
/// returns, and what a Bulk export/CSV export ultimately iterates over.
#[derive(Debug, Clone, Serialize)]
pub struct ApiRecordPage {
    pub records: Vec<serde_json::Value>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ApiListQuery {
    pub select: Option<Vec<String>>,
    pub filter: Option<serde_json::Value>,
    pub sort: Option<Vec<String>>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// --- Integration Jobs (spec §15) --------------------------------------------
//
// Recurring pull-sync from an External Object into a Lanesra object, on an
// interval, with a checkpoint ("cursor") - see `services::integration_job_service`
// for what's actually implemented (pull only; push is a stated gap).

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationJob {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub external_object_id: String,
    pub target_object_key: String,
    pub match_key: String,
    pub cursor_field: Option<String>,
    pub cursor_value: Option<String>,
    pub interval_minutes: i64,
    pub status: String,
    pub last_run_at: Option<String>,
    pub last_run_status: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationJobInput {
    pub name: String,
    pub external_object_id: String,
    pub target_object_key: String,
    pub match_key: String,
    pub cursor_field: Option<String>,
    pub interval_minutes: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrationJobRun {
    pub id: String,
    pub job_id: String,
    pub workspace_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub records_processed: i64,
    pub records_failed: i64,
    pub error_message: Option<String>,
    pub cursor_before: Option<String>,
    pub cursor_after: Option<String>,
}
