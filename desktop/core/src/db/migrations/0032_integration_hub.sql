-- Integration Hub (Lanesra_OS_Integration_Hub_Admin_Design_Development_Spec_v1.0)
-- - APIs, Connections, Connectors, Webhooks & Data Exchange, built to the
-- spec's own §34/table-28 phasing rather than as one undifferentiated
-- blob: this migration lays down every table the v1 through v2 slice this
-- build actually implements needs. See each service module's own doc
-- comment for what's genuinely proven (tested against a real local
-- listener/database/SSH server spun up inside the test itself) versus
-- best-effort (the connection/auth *mechanics* are real and tested, but a
-- live third-party provider - Google, Microsoft, a customer's own SFTP/SQL
-- server - can't be verified from this environment). Not built at all:
-- mTLS (the spec itself marks this "Future", not even P2) and vendor-
-- specific Google/Microsoft prebuilt connector action packs (contacts/
-- calendar/email) - the generic OAuth2 connection + generic REST/OpenAPI
-- connector covers the same ground without fabricating untested actions.
--
-- integration_secrets is the one shared secret store every other table
-- with sensitive material points into by id, never inline - see
-- services/secret_service.rs for the AES-256-GCM encryption and
-- master-key resolution (env var, else a key file kept separate from the
-- SQLite database itself, per the spec's own §20 "practical fallback"
-- language). integration_api_credentials is the one exception: an issued
-- API client secret is *hashed* (one-way, like a password), never
-- encrypted, since Lanesra only ever needs to verify a presented key, not
-- re-send it anywhere - see api_client_service's own doc comment.
CREATE TABLE integration_secrets (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- Admin-facing label only ("Accounting API bearer token") - never the
    -- secret value itself, which is encrypted below.
    label TEXT NOT NULL,
    ciphertext TEXT NOT NULL,
    nonce TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    rotated_at TEXT
);

-- A workspace-specific authenticated endpoint instance (spec §4).
-- connection_type is one of 'rest' | 'webhook' | 'sftp' | 'postgres' |
-- 'odata' | 'smtp' - see connection_service::ConnectionType. auth_mode is
-- one of 'none' | 'api_key' | 'basic' | 'bearer' | 'custom_header' |
-- 'oauth2_client_credentials' | 'oauth2_authorization_code'. config_json
-- holds the connection_type-specific extra properties (timeout_ms, retry
-- policy, tls_verify, default headers, SFTP/Postgres host/port/database,
-- OAuth2 authorize/token URLs and scopes, ...) as one flexible JSON blob -
-- the same "don't add a sparse column per connection type" convention
-- business_rule_conditions/workflow actions already use for their own
-- type-varying payloads.
CREATE TABLE integration_connections (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    connection_type TEXT NOT NULL,
    base_url TEXT,
    auth_mode TEXT NOT NULL DEFAULT 'none',
    -- Points into integration_secrets for whatever this auth_mode needs
    -- (an API key, a bearer token, a basic-auth password, an OAuth2
    -- client secret + refresh token bundle, an SMTP password, ...). NULL
    -- for auth_mode = 'none'.
    secret_id TEXT REFERENCES integration_secrets(id) ON DELETE SET NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    owner_user_id TEXT REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'disabled',
    last_test_at TEXT,
    last_test_status TEXT,
    last_test_message TEXT,
    last_failure_at TEXT,
    credential_expires_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_integration_connections_workspace ON integration_connections(workspace_id);

-- Portable logical reference (spec §5, table 6/7) - what a Workflow
-- "Call Connector Action" step or a Webhook subscription actually points
-- at, so packaging it into a Solution never has to package a physical
-- connection or its secrets. reference_key follows the same dotted-
-- namespace convention publisher keys already use (e.g.
-- "local.accounting_api"). connection_id is nullable - unresolved until an
-- admin binds it, exactly like the spec's own "Import prompts
-- administrator to bind unresolved references before activating dependent
-- automation" (table 7).
CREATE TABLE integration_connection_refs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    reference_name TEXT NOT NULL,
    reference_key TEXT NOT NULL,
    expected_connection_type TEXT NOT NULL,
    connection_id TEXT REFERENCES integration_connections(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE UNIQUE INDEX idx_integration_connection_refs_key ON integration_connection_refs(workspace_id, reference_key);

-- An inbound API client / service account (spec §8). client_id is the
-- public half; integration_api_credentials (below) holds the hashed
-- secret half issued alongside it. scopes_json is an array of strings
-- from the spec §8.2 vocabulary (objects.read, objects.write,
-- metadata.read, search.read, bulk.read, bulk.write, webhooks.manage,
-- admin.integration.read, admin.integration.manage).
CREATE TABLE integration_api_clients (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    client_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    scopes_json TEXT NOT NULL DEFAULT '[]',
    allowed_cidr TEXT,
    owner_user_id TEXT REFERENCES users(id),
    last_used_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE UNIQUE INDEX idx_integration_api_clients_client_id ON integration_api_clients(client_id);

-- The hashed (SHA-256, one-way) secret half of an API client's issued
-- key, shown to the admin exactly once at creation - see
-- api_client_service's own doc comment for the `{client_id}.{secret}`
-- shape and why this is hashed rather than encrypted.
CREATE TABLE integration_api_credentials (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    api_client_id TEXT NOT NULL REFERENCES integration_api_clients(id) ON DELETE CASCADE,
    secret_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    rotated_at TEXT
);
CREATE INDEX idx_integration_api_credentials_client ON integration_api_credentials(api_client_id);

-- An outbound event subscription (spec §10). event_types_json is an array
-- drawn from record.created/updated/archived, field.changed,
-- workflow.completed, workflow.failed. secret_id is the HMAC signing
-- secret (never the connection's own auth secret, even though both live
-- in integration_secrets) - see webhook_service's own doc comment.
CREATE TABLE integration_webhooks (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    connection_id TEXT NOT NULL REFERENCES integration_connections(id) ON DELETE CASCADE,
    event_types_json TEXT NOT NULL DEFAULT '[]',
    object_scope TEXT,
    filter_json TEXT,
    payload_version TEXT NOT NULL DEFAULT '1',
    secret_id TEXT REFERENCES integration_secrets(id) ON DELETE SET NULL,
    retry_policy_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_integration_webhooks_workspace ON integration_webhooks(workspace_id);

-- Per-event delivery attempt history (spec §10.3).
CREATE TABLE integration_webhook_deliveries (
    id TEXT PRIMARY KEY,
    webhook_id TEXT NOT NULL REFERENCES integration_webhooks(id) ON DELETE CASCADE,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL,
    http_status INTEGER,
    duration_ms INTEGER,
    response_snippet TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_integration_webhook_deliveries_webhook ON integration_webhook_deliveries(webhook_id, created_at);

-- A reusable, saved source->target field mapping (spec §14) - powers both
-- the Data Exchange CSV wizard and (later, if a job uses it) Integration
-- Jobs. field_map_json is an array of
-- {source_column, target_field, transform, default_value, constant}.
-- needs_review is flipped by mapping_service when target metadata changes
-- (a field renamed/deactivated) invalidate a saved mapping - spec §14's
-- own "metadata changes that invalidate mappings mark them Needs Review".
CREATE TABLE integration_mappings (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    target_object_key TEXT NOT NULL,
    operation TEXT NOT NULL DEFAULT 'upsert',
    match_key TEXT,
    field_map_json TEXT NOT NULL DEFAULT '[]',
    duplicate_policy TEXT NOT NULL DEFAULT 'update_matched',
    needs_review INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_integration_mappings_workspace ON integration_mappings(workspace_id);

-- The unified cross-integration observability index (spec §23.1) -
-- every API call, connector action, webhook delivery, import/export run
-- and job run writes one row here, regardless of which subsystem it came
-- from, so Logs & Monitoring and the Overview KPIs are real aggregates
-- over real data rather than a per-subsystem view stitched together in
-- the UI.
CREATE TABLE integration_executions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    execution_type TEXT NOT NULL,
    correlation_id TEXT,
    ref_id TEXT,
    direction TEXT NOT NULL DEFAULT 'outbound',
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_ms INTEGER,
    status TEXT NOT NULL,
    http_status INTEGER,
    records_read INTEGER NOT NULL DEFAULT 0,
    records_written INTEGER NOT NULL DEFAULT 0,
    records_skipped INTEGER NOT NULL DEFAULT 0,
    records_failed INTEGER NOT NULL DEFAULT 0,
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_category TEXT,
    error_message TEXT,
    actor_user_id TEXT REFERENCES users(id)
);
CREATE INDEX idx_integration_executions_workspace ON integration_executions(workspace_id, started_at);
CREATE INDEX idx_integration_executions_correlation ON integration_executions(correlation_id);

-- One row per workspace - rate limits, retention and network policy
-- (spec §21/§22), a real dedicated table rather than resurrecting the
-- orphaned generic key-value `settings` table from migration 0001 (which
-- has zero references anywhere in this codebase) or bolting yet more
-- columns onto `workspaces`.
CREATE TABLE integration_settings (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    api_rate_limit_per_minute INTEGER NOT NULL DEFAULT 300,
    global_rate_limit_per_minute INTEGER NOT NULL DEFAULT 3000,
    log_retention_days INTEGER NOT NULL DEFAULT 90,
    file_retention_days INTEGER NOT NULL DEFAULT 7,
    allow_insecure_connections INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);

-- A reusable Connector definition (spec §6) - either hand-imported from an
-- OpenAPI 3.x document (spec_source = 'openapi') or a small built-in
-- shape. raw_spec keeps the original document text so it can be
-- re-parsed/diffed later; publisher_id lets a Connector be a genuine
-- Solution Component (spec §6.1's "may be a Solution Component").
CREATE TABLE integration_connectors (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    connection_type TEXT NOT NULL DEFAULT 'rest',
    spec_source TEXT NOT NULL DEFAULT 'openapi',
    raw_spec TEXT,
    publisher_id TEXT REFERENCES publishers(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_integration_connectors_workspace ON integration_connectors(workspace_id);

-- One derived Action per exposed OpenAPI operation (spec §6.2/§6.3).
-- action_key defaults to the operation's own operationId (spec: "support
-- operationId as technical action key"); display_name is the
-- business-friendly override.
CREATE TABLE integration_connector_actions (
    id TEXT PRIMARY KEY,
    connector_id TEXT NOT NULL REFERENCES integration_connectors(id) ON DELETE CASCADE,
    action_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    http_method TEXT NOT NULL,
    path_template TEXT NOT NULL,
    params_json TEXT NOT NULL DEFAULT '[]',
    request_schema_json TEXT,
    response_schema_json TEXT,
    created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_integration_connector_actions_key ON integration_connector_actions(connector_id, action_key);

-- A recurring or on-demand pull-sync job (spec §15): re-fetches an
-- External Object (below) on `interval_minutes` and upserts its records
-- into `target_object_key` via the same generic dispatcher the CSV
-- wizard/REST API use, checkpointing a "cursor" (the max value seen of
-- `cursor_field`) between runs, per INT-AC-10's "survives a host
-- restart" - it's a plain persisted column, not in-memory state. Push
-- direction (Lanesra -> external system) is a stated, deliberate gap in
-- this pass - see services::integration_job_service's own doc comment.
CREATE TABLE integration_jobs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    external_object_id TEXT NOT NULL REFERENCES integration_external_objects(id) ON DELETE CASCADE,
    target_object_key TEXT NOT NULL,
    match_key TEXT NOT NULL,
    cursor_field TEXT,
    cursor_value TEXT,
    interval_minutes INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    last_run_at TEXT,
    last_run_status TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_integration_jobs_workspace ON integration_jobs(workspace_id);

CREATE TABLE integration_job_runs (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES integration_jobs(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL,
    records_processed INTEGER NOT NULL DEFAULT 0,
    records_failed INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    cursor_before TEXT,
    cursor_after TEXT
);
CREATE INDEX idx_integration_job_runs_job ON integration_job_runs(job_id, started_at);

-- External/Virtual Object metadata (spec §16) - read-only records shown
-- through the existing generic list view without copying the source data
-- into Lanesra. Explicitly not included in backup (spec: "External data
-- itself is not included in Lanesra backup; only metadata/configuration
-- is backed up") - only this metadata row is ever backed up, never the
-- records it resolves at read time.
CREATE TABLE integration_external_objects (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    object_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    connection_id TEXT NOT NULL REFERENCES integration_connections(id) ON DELETE CASCADE,
    resource_path TEXT NOT NULL,
    field_map_json TEXT NOT NULL DEFAULT '[]',
    cache_ttl_seconds INTEGER,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE UNIQUE INDEX idx_integration_external_objects_key ON integration_external_objects(workspace_id, object_key);

-- The bridge between every entity's plain *synchronous* create/update/
-- archive path (`services::event_hooks::emit`, called from
-- company_service/contact_service/etc.) and webhook delivery, which
-- genuinely needs to be async (a real outbound HTTP call). Rather than
-- making every entity service function async - a sweeping, risky change
-- touching every Tauri command and test in this crate - `emit` only ever
-- does a cheap synchronous insert here; a separate, genuinely-async drain
-- (`webhook_service::drain_pending_events`) delivers them, called
-- periodically the same way `run_scheduled_workflows` already is (a
-- client-side poll on desktop, a real background tokio interval on the
-- Team Workspace server - see `integration_job_service`'s own comment for
-- that scheduler). A queued row with zero matching webhooks is simply
-- never written in the first place (`emit` checks first), so the common
-- case - no webhooks subscribed at all - costs nothing beyond that check.
CREATE TABLE integration_pending_events (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    object_key TEXT NOT NULL,
    record_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    correlation_id TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_integration_pending_events_workspace ON integration_pending_events(workspace_id, created_at);
