import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type {
  ApiClient,
  ApiClientInput,
  Connection,
  ConnectionInput,
  ConnectionRef,
  ConnectionRefInput,
  ConnectionUpdate,
  Connector,
  ConnectorImportInput,
  CsvImportInput,
  ExternalObject,
  ExternalObjectInput,
  FieldMapEntry,
  IntegrationExecutionQuery,
  IntegrationJob,
  IntegrationJobInput,
  IntegrationSettingsUpdate,
  IssuedApiClient,
  Mapping,
  MappingInput,
  OpenApiImportPreview,
  Webhook,
  WebhookInput,
} from "../../lib/types";

// Spec vocabularies enforced server-side (connection_service::CONNECTION_TYPES,
// AUTH_MODES; webhook_service::EVENT_TYPES; mapping_service::OPERATIONS,
// DUPLICATE_POLICIES; api_client_service::VALID_SCOPES) - mirrored here only
// to drive select options, not re-validated client-side; the service is
// always the source of truth and rejects anything else.
const CONNECTION_TYPES = ["rest", "webhook", "sftp", "postgres", "odata", "smtp"];
const AUTH_MODES = ["none", "api_key", "basic", "bearer", "custom_header", "oauth2_client_credentials", "oauth2_authorization_code"];
const EVENT_TYPES = ["record.created", "record.updated", "record.archived", "field.changed", "workflow.completed", "workflow.failed"];
const OPERATIONS = ["insert", "update", "upsert"];
const DUPLICATE_POLICIES = ["skip", "update_matched", "create_new"];
const TRANSFORMS = ["none", "trim", "uppercase", "lowercase", "concatenate", "numeric", "date"];
const API_SCOPES = [
  "objects.read", "objects.write", "metadata.read", "search.read", "bulk.read", "bulk.write",
  "webhooks.manage", "events.read", "admin.integration.read", "admin.integration.manage",
];

type IntegrationTab =
  | "overview" | "connections" | "connectionRefs" | "connectors" | "apiAccess"
  | "webhooks" | "dataExchange" | "externalObjects" | "jobs" | "logs" | "settings";

const INTEGRATION_TABS: { key: IntegrationTab; label: string }[] = [
  { key: "overview", label: "Overview" },
  { key: "connections", label: "Connections" },
  { key: "connectionRefs", label: "Connection References" },
  { key: "connectors", label: "Connectors" },
  { key: "apiAccess", label: "API Access" },
  { key: "webhooks", label: "Webhooks & Events" },
  { key: "dataExchange", label: "Data Exchange" },
  { key: "externalObjects", label: "External Objects" },
  { key: "jobs", label: "Integration Jobs" },
  { key: "logs", label: "Logs & Monitoring" },
  { key: "settings", label: "Settings" },
];

function apiErrorMessage(err: unknown, fallback: string): string {
  return err instanceof ApiError ? err.message : fallback;
}

function downloadText(filename: string, content: string, mime: string): void {
  const blob = new Blob([content], { type: `${mime};charset=utf-8;` });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Integration Hub (Lanesra_OS_Integration_Hub_Admin_Design_Development_Spec_v1.0).
 * Every screen here talks to a real, working backend - generic REST
 * Connections with a real Test Connection call, Connection References,
 * OpenAPI-imported Connectors with a real Test Action call, inbound API
 * Access (issued once, hashed at rest), HMAC-signed Webhooks with real
 * delivery history, reusable field Mappings backing a generalized CSV
 * import/export wizard, read-only External Objects backed by a real
 * Connection, recurring Integration Jobs (server-hosted only - see the
 * Jobs tab's own note), a unified execution log with real KPIs, and
 * workspace-level policy Settings. Nothing here is a UI-only simulation -
 * that's the online demo's Integrations tab, not this one.
 */
export function IntegrationHubAdmin() {
  const [tab, setTab] = useState<IntegrationTab>("overview");

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Connect Lanesra to the outside world: outbound Connections and Connectors, inbound API clients and
        webhooks, CSV data exchange, recurring sync Jobs, and one place to watch it all run.
      </p>

      <div className="tab-row">
        {INTEGRATION_TABS.map((t) => (
          <button key={t.key} className={`tab${tab === t.key ? " active" : ""}`} onClick={() => setTab(t.key)}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === "overview" && <OverviewTab />}
      {tab === "connections" && <ConnectionsTab />}
      {tab === "connectionRefs" && <ConnectionRefsTab />}
      {tab === "connectors" && <ConnectorsTab />}
      {tab === "apiAccess" && <ApiAccessTab />}
      {tab === "webhooks" && <WebhooksTab />}
      {tab === "dataExchange" && <DataExchangeTab />}
      {tab === "externalObjects" && <ExternalObjectsTab />}
      {tab === "jobs" && <JobsTab />}
      {tab === "logs" && <LogsTab />}
      {tab === "settings" && <SettingsTab />}
    </div>
  );
}

// --- Overview ----------------------------------------------------------------

function OverviewTab() {
  const overview = useQuery({ queryKey: ["integrationOverview"], queryFn: () => api.getIntegrationOverview() });
  const o = overview.data;
  const kpi = (label: string, value: number | undefined, danger?: boolean) => (
    <div className="card" style={{ textAlign: "center" }}>
      <div style={{ fontSize: 28, fontWeight: 600, color: danger && value ? "var(--danger, #c0392b)" : undefined }}>
        {value ?? "—"}
      </div>
      <div style={{ color: "var(--text-muted)", fontSize: 13 }}>{label}</div>
    </div>
  );
  return (
    <div>
      {overview.isLoading && <p>Loading...</p>}
      {o && (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))", gap: 12 }}>
          {kpi("Active connections", o.active_connections)}
          {kpi("Failed connections", o.failed_connections, true)}
          {kpi("API calls today", o.api_calls_today)}
          {kpi("Failed webhooks today", o.failed_webhooks_today, true)}
          {kpi("Jobs running", o.jobs_running)}
          {kpi("Jobs failed today", o.jobs_failed_today, true)}
        </div>
      )}
    </div>
  );
}

// --- Connections (spec §4) ----------------------------------------------------

function ConnectionsTab() {
  const queryClient = useQueryClient();
  const connections = useQuery({ queryKey: ["connections"], queryFn: () => api.listConnections() });
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, string>>({});

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["connections"] });

  const del = useMutation({
    mutationFn: (id: string) => api.deleteConnection(id),
    onSuccess: invalidate,
  });

  const test = useMutation({
    mutationFn: (id: string) => api.testConnection(id),
    onSuccess: (result, id) => {
      setTestResults((r) => ({ ...r, [id]: result.ok ? `OK (${result.latency_ms}ms)` : `Failed: ${result.message}` }));
      invalidate();
    },
    onError: (err, id) => setTestResults((r) => ({ ...r, [id]: apiErrorMessage(err, "Test failed") })),
  });

  const rows = connections.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Connections</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Generic REST/webhook/SFTP/Postgres/OData/SMTP connections. Secrets are encrypted at rest
            (AES-256-GCM) and never returned in plaintext once saved.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>
          {creating ? "Cancel" : "+ New connection"}
        </button>
      </div>

      {creating && <ConnectionForm onDone={() => { setCreating(false); invalidate(); }} onCancel={() => setCreating(false)} />}

      {connections.isLoading && <p>Loading...</p>}
      {!connections.isLoading && rows.length === 0 && !creating && <p className="empty-state">No connections yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Name</th><th>Type</th><th>Auth</th><th>Status</th><th>Last test</th><th></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((c) => (
              <>
                <tr key={c.id}>
                  <td>{c.name}</td>
                  <td>{c.connection_type}</td>
                  <td>{c.auth_mode}{c.has_secret ? "" : c.auth_mode !== "none" ? " (no secret set)" : ""}</td>
                  <td><span className={`badge${c.status === "active" ? " badge-success" : ""}`}>{c.status}</span></td>
                  <td style={{ fontSize: 12 }}>
                    {testResults[c.id] ?? (c.last_test_status ? `${c.last_test_status}${c.last_test_message ? `: ${c.last_test_message}` : ""}` : "Never tested")}
                  </td>
                  <td style={{ display: "flex", gap: 6, whiteSpace: "nowrap" }}>
                    <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => test.mutate(c.id)} disabled={test.isPending}>
                      {test.isPending ? "Testing..." : "Test"}
                    </button>
                    <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => setEditingId(editingId === c.id ? null : c.id)}>
                      {editingId === c.id ? "Close" : "Edit"}
                    </button>
                    <button
                      className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }}
                      onClick={() => { if (confirm(`Delete connection "${c.name}"?`)) del.mutate(c.id); }}
                    >
                      Delete
                    </button>
                  </td>
                </tr>
                {editingId === c.id && (
                  <tr key={`${c.id}-edit`}>
                    <td colSpan={6} style={{ background: "var(--bg-subtle, rgba(0,0,0,0.02))" }}>
                      <ConnectionEditForm connection={c} onDone={() => { setEditingId(null); invalidate(); }} onCancel={() => setEditingId(null)} />
                    </td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ConnectionForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<ConnectionInput>({
    name: "", connection_type: "rest", base_url: "", auth_mode: "none", secret_value: "", config_json: "{}", owner_user_id: null,
  });
  const [error, setError] = useState<string | null>(null);
  const create = useMutation({
    mutationFn: () => api.createConnection({ ...input, base_url: input.base_url || null, secret_value: input.secret_value || null }),
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not create that connection")),
  });
  return (
    <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      <div className="form-field"><label>Name</label><input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required /></div>
      <div className="form-field">
        <label>Type</label>
        <select value={input.connection_type} onChange={(e) => setInput({ ...input, connection_type: e.target.value })}>
          {CONNECTION_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
        </select>
      </div>
      <div className="form-field full"><label>Base URL / host</label><input value={input.base_url ?? ""} onChange={(e) => setInput({ ...input, base_url: e.target.value })} placeholder="https://api.example.com" /></div>
      <div className="form-field">
        <label>Auth mode</label>
        <select value={input.auth_mode} onChange={(e) => setInput({ ...input, auth_mode: e.target.value })}>
          {AUTH_MODES.map((m) => <option key={m} value={m}>{m}</option>)}
        </select>
      </div>
      {input.auth_mode !== "none" && (
        <div className="form-field"><label>Secret value</label><input type="password" value={input.secret_value ?? ""} onChange={(e) => setInput({ ...input, secret_value: e.target.value })} placeholder="API key / token / password" /></div>
      )}
      <div className="form-field full"><label>Config (JSON)</label><textarea value={input.config_json} onChange={(e) => setInput({ ...input, config_json: e.target.value })} rows={3} /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={create.isPending}>{create.isPending ? "Creating..." : "Create connection"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function ConnectionEditForm({ connection, onDone, onCancel }: { connection: Connection; onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<ConnectionUpdate>({
    name: connection.name, base_url: connection.base_url, auth_mode: connection.auth_mode,
    secret_value: "", config_json: connection.config_json, owner_user_id: connection.owner_user_id, status: connection.status,
  });
  const [error, setError] = useState<string | null>(null);
  const update = useMutation({
    mutationFn: () => api.updateConnection(connection.id, { ...input, secret_value: input.secret_value || null }),
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not save that connection")),
  });
  return (
    <form className="form-grid" style={{ padding: "12px 4px" }} onSubmit={(e) => { e.preventDefault(); update.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      <div className="form-field"><label>Name</label><input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required /></div>
      <div className="form-field full"><label>Base URL / host</label><input value={input.base_url ?? ""} onChange={(e) => setInput({ ...input, base_url: e.target.value })} /></div>
      <div className="form-field">
        <label>Auth mode</label>
        <select value={input.auth_mode} onChange={(e) => setInput({ ...input, auth_mode: e.target.value })}>
          {AUTH_MODES.map((m) => <option key={m} value={m}>{m}</option>)}
        </select>
      </div>
      <div className="form-field"><label>Secret (leave blank to keep)</label><input type="password" value={input.secret_value ?? ""} onChange={(e) => setInput({ ...input, secret_value: e.target.value })} /></div>
      <div className="form-field">
        <label>Status</label>
        <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
          <option value="active">active</option>
          <option value="disabled">disabled</option>
        </select>
      </div>
      <div className="form-field full"><label>Config (JSON)</label><textarea value={input.config_json} onChange={(e) => setInput({ ...input, config_json: e.target.value })} rows={3} /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={update.isPending}>{update.isPending ? "Saving..." : "Save"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

// --- Connection References (spec §5) -----------------------------------------

function ConnectionRefsTab() {
  const queryClient = useQueryClient();
  const refs = useQuery({ queryKey: ["connectionRefs"], queryFn: () => api.listConnectionRefs() });
  const connections = useQuery({ queryKey: ["connections"], queryFn: () => api.listConnections() });
  const [creating, setCreating] = useState(false);
  const [input, setInput] = useState<ConnectionRefInput>({ reference_name: "", reference_key: "", expected_connection_type: "rest", connection_id: null });
  const [error, setError] = useState<string | null>(null);

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["connectionRefs"] });
  const create = useMutation({
    mutationFn: () => api.createConnectionRef({ ...input, reference_key: input.reference_key.trim() }),
    onSuccess: () => { setCreating(false); setInput({ reference_name: "", reference_key: "", expected_connection_type: "rest", connection_id: null }); invalidate(); },
    onError: (err) => setError(apiErrorMessage(err, "Could not create that reference")),
  });
  const bind = useMutation({ mutationFn: ({ id, connectionId }: { id: string; connectionId: string | null }) => api.bindConnectionRef(id, connectionId), onSuccess: invalidate });
  const del = useMutation({ mutationFn: (id: string) => api.deleteConnectionRef(id), onSuccess: invalidate });

  const rows = refs.data ?? [];
  const matchingConnections = (type: string) => (connections.data ?? []).filter((c) => c.connection_type === type);

  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Connection References</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            A portable logical name a Solution or workflow binds to - point it at a real Connection per
            workspace, so an exported Solution never carries a physical connection or secret with it.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>{creating ? "Cancel" : "+ New reference"}</button>
      </div>

      {creating && (
        <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
          {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
          <div className="form-field"><label>Display name</label><input value={input.reference_name} onChange={(e) => setInput({ ...input, reference_name: e.target.value })} required /></div>
          <div className="form-field"><label>Key</label><input value={input.reference_key} onChange={(e) => setInput({ ...input, reference_key: e.target.value })} placeholder="crm_primary" required /></div>
          <div className="form-field">
            <label>Expected type</label>
            <select value={input.expected_connection_type} onChange={(e) => setInput({ ...input, expected_connection_type: e.target.value })}>
              {CONNECTION_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
            </select>
          </div>
          <div className="form-field full">
            <button className="btn btn-primary" type="submit" disabled={create.isPending}>{create.isPending ? "Creating..." : "Create reference"}</button>
          </div>
        </form>
      )}

      {refs.isLoading && <p>Loading...</p>}
      {!refs.isLoading && rows.length === 0 && !creating && <p className="empty-state">No references yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead><tr><th>Name</th><th>Key</th><th>Type</th><th>Bound connection</th><th></th></tr></thead>
          <tbody>
            {rows.map((r: ConnectionRef) => (
              <tr key={r.id}>
                <td>{r.reference_name}</td>
                <td><code>{r.reference_key}</code></td>
                <td>{r.expected_connection_type}</td>
                <td>
                  <select
                    value={r.connection_id ?? ""}
                    onChange={(e) => bind.mutate({ id: r.id, connectionId: e.target.value || null })}
                  >
                    <option value="">Unbound</option>
                    {matchingConnections(r.expected_connection_type).map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
                  </select>
                </td>
                <td>
                  <button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete reference "${r.reference_name}"?`)) del.mutate(r.id); }}>Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

// --- Connectors (spec §6) ------------------------------------------------------

function ConnectorsTab() {
  const queryClient = useQueryClient();
  const connectors = useQuery({ queryKey: ["connectors"], queryFn: () => api.listConnectors() });
  const [importing, setImporting] = useState(false);
  const [testingActionOf, setTestingActionOf] = useState<{ connector: Connector; actionKey: string } | null>(null);

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["connectors"] });
  const del = useMutation({ mutationFn: (id: string) => api.deleteConnector(id), onSuccess: invalidate });

  const rows = connectors.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Connectors</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Import an OpenAPI 3.x spec to derive reusable Actions - each becomes a step Workflow Automation
            can call ("Call Connector Action").
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setImporting((v) => !v)}>{importing ? "Cancel" : "+ Import connector"}</button>
      </div>

      {importing && <ConnectorImportWizard onDone={() => { setImporting(false); invalidate(); }} onCancel={() => setImporting(false)} />}

      {connectors.isLoading && <p>Loading...</p>}
      {!connectors.isLoading && rows.length === 0 && !importing && <p className="empty-state">No connectors imported yet.</p>}
      {rows.map((c) => (
        <div key={c.id} style={{ borderTop: "1px solid var(--border, #e5e5e5)", paddingTop: 12, marginTop: 12 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div>
              <b>{c.name}</b> <span style={{ color: "var(--text-muted)", fontSize: 12 }}>({c.connection_type}, {c.actions.length} action{c.actions.length === 1 ? "" : "s"})</span>
              {c.description && <p style={{ margin: "2px 0 0", fontSize: 13, color: "var(--text-muted)" }}>{c.description}</p>}
            </div>
            <button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete connector "${c.name}"?`)) del.mutate(c.id); }}>Delete</button>
          </div>
          {c.actions.length > 0 && (
            <table style={{ marginTop: 8, marginBottom: 0 }}>
              <thead><tr><th>Action</th><th>Method</th><th>Path</th><th></th></tr></thead>
              <tbody>
                {c.actions.map((a) => (
                  <tr key={a.id}>
                    <td>{a.display_name} <code style={{ fontSize: 11 }}>{a.action_key}</code></td>
                    <td>{a.http_method.toUpperCase()}</td>
                    <td><code style={{ fontSize: 12 }}>{a.path_template}</code></td>
                    <td><button className="btn btn-secondary" style={{ fontSize: 12, padding: "2px 8px" }} onClick={() => setTestingActionOf({ connector: c, actionKey: a.action_key })}>Test</button></td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      ))}

      {testingActionOf && <TestConnectorActionModal connector={testingActionOf.connector} actionKey={testingActionOf.actionKey} onClose={() => setTestingActionOf(null)} />}
    </div>
  );
}

function ConnectorImportWizard({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [specText, setSpecText] = useState("");
  const [specFormat, setSpecFormat] = useState<"json" | "yaml">("json");
  const [preview, setPreview] = useState<OpenApiImportPreview | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);

  const previewMutation = useMutation({
    mutationFn: () => api.previewConnectorImport(specText, specFormat),
    onSuccess: (p) => {
      setPreview(p);
      setName(p.title);
      setSelected(new Set(p.operations.map((o) => o.operation_id)));
      setError(null);
    },
    onError: (err) => setError(apiErrorMessage(err, "Could not parse that OpenAPI spec")),
  });

  const importMutation = useMutation({
    mutationFn: () => {
      const input: ConnectorImportInput = { name, description: description || null, spec_text: specText, spec_format: specFormat, selected_operation_ids: [...selected] };
      return api.importConnector(input);
    },
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not import that connector")),
  });

  return (
    <div className="card" style={{ background: "var(--bg-subtle, rgba(0,0,0,0.02))", marginBottom: 16 }}>
      {error && <div className="error-banner">{error}</div>}
      {!preview && (
        <div className="form-grid">
          <div className="form-field">
            <label>Format</label>
            <select value={specFormat} onChange={(e) => setSpecFormat(e.target.value as "json" | "yaml")}>
              <option value="json">JSON</option>
              <option value="yaml">YAML</option>
            </select>
          </div>
          <div className="form-field full">
            <label>OpenAPI 3.x spec</label>
            <textarea value={specText} onChange={(e) => setSpecText(e.target.value)} rows={8} placeholder="Paste the spec text here..." />
          </div>
          <div className="form-field full" style={{ display: "flex", gap: 8 }}>
            <button className="btn btn-primary" type="button" onClick={() => previewMutation.mutate()} disabled={previewMutation.isPending || !specText.trim()}>
              {previewMutation.isPending ? "Parsing..." : "Parse spec"}
            </button>
            <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
          </div>
        </div>
      )}
      {preview && (
        <div>
          <p style={{ fontSize: 13, color: "var(--text-muted)" }}>{preview.title} v{preview.version} - {preview.operations.length} operation{preview.operations.length === 1 ? "" : "s"} found</p>
          {preview.warnings.length > 0 && (
            <ul style={{ fontSize: 12, color: "var(--warn, #b58900)" }}>
              {preview.warnings.map((w, i) => <li key={i}>{w}</li>)}
            </ul>
          )}
          <div className="form-grid">
            <div className="form-field"><label>Connector name</label><input value={name} onChange={(e) => setName(e.target.value)} required /></div>
            <div className="form-field full"><label>Description (optional)</label><input value={description} onChange={(e) => setDescription(e.target.value)} /></div>
          </div>
          <table>
            <thead><tr><th></th><th>Operation</th><th>Method</th><th>Path</th></tr></thead>
            <tbody>
              {preview.operations.map((op) => (
                <tr key={op.operation_id}>
                  <td>
                    <input
                      type="checkbox"
                      checked={selected.has(op.operation_id)}
                      onChange={(e) => {
                        const next = new Set(selected);
                        if (e.target.checked) next.add(op.operation_id); else next.delete(op.operation_id);
                        setSelected(next);
                      }}
                    />
                  </td>
                  <td>{op.operation_id}{op.summary ? <span style={{ color: "var(--text-muted)", fontSize: 12 }}> - {op.summary}</span> : null}</td>
                  <td>{op.http_method.toUpperCase()}</td>
                  <td><code style={{ fontSize: 12 }}>{op.path_template}</code></td>
                </tr>
              ))}
            </tbody>
          </table>
          <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
            <button className="btn btn-primary" onClick={() => importMutation.mutate()} disabled={importMutation.isPending || !name.trim() || selected.size === 0}>
              {importMutation.isPending ? "Importing..." : `Import ${selected.size} action${selected.size === 1 ? "" : "s"}`}
            </button>
            <button className="btn btn-secondary" onClick={() => setPreview(null)}>Back</button>
            <button className="btn btn-secondary" onClick={onCancel}>Cancel</button>
          </div>
        </div>
      )}
    </div>
  );
}

function TestConnectorActionModal({ connector, actionKey, onClose }: { connector: Connector; actionKey: string; onClose: () => void }) {
  const [referenceKey, setReferenceKey] = useState("");
  const [paramsJson, setParamsJson] = useState("{}");
  const [error, setError] = useState<string | null>(null);
  const run = useMutation({
    mutationFn: () => {
      let params: unknown;
      try { params = JSON.parse(paramsJson); } catch { throw new Error("Params must be valid JSON"); }
      return api.testConnectorAction(connector.id, actionKey, referenceKey, params);
    },
    onError: (err) => setError(err instanceof Error ? err.message : apiErrorMessage(err, "Test failed")),
  });
  const action = connector.actions.find((a) => a.action_key === actionKey);
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" style={{ maxWidth: 520 }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ marginTop: 0 }}>Test action: {action?.display_name ?? actionKey}</h3>
        {error && <div className="error-banner">{error}</div>}
        <div className="form-grid">
          <div className="form-field full"><label>Connection Reference key</label><input value={referenceKey} onChange={(e) => setReferenceKey(e.target.value)} placeholder="crm_primary" /></div>
          <div className="form-field full"><label>Params (JSON)</label><textarea value={paramsJson} onChange={(e) => setParamsJson(e.target.value)} rows={4} /></div>
        </div>
        {run.data && (
          <div style={{ marginTop: 12 }}>
            <p><span className={`badge${run.data.ok ? " badge-success" : " badge-danger"}`}>{run.data.ok ? "OK" : "Failed"}</span> {run.data.status_code ?? ""} - {run.data.duration_ms}ms</p>
            <pre style={{ fontSize: 12, maxHeight: 200, overflow: "auto", background: "var(--bg-subtle, rgba(0,0,0,0.03))", padding: 8 }}>{JSON.stringify(run.data.response_body, null, 2)}</pre>
          </div>
        )}
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 16 }}>
          <button className="btn btn-secondary" onClick={onClose}>Close</button>
          <button className="btn btn-primary" onClick={() => run.mutate()} disabled={run.isPending || !referenceKey.trim()}>{run.isPending ? "Running..." : "Run test"}</button>
        </div>
      </div>
    </div>
  );
}

// --- API Access (spec §8) ------------------------------------------------------

function ApiAccessTab() {
  const queryClient = useQueryClient();
  const clients = useQuery({ queryKey: ["apiClients"], queryFn: () => api.listApiClients() });
  const [creating, setCreating] = useState(false);
  const [issued, setIssued] = useState<IssuedApiClient | null>(null);
  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["apiClients"] });

  const revoke = useMutation({ mutationFn: (id: string) => api.revokeApiClient(id), onSuccess: invalidate });
  const reactivate = useMutation({ mutationFn: (id: string) => api.reactivateApiClient(id), onSuccess: invalidate });
  const del = useMutation({ mutationFn: (id: string) => api.deleteApiClient(id), onSuccess: invalidate });
  const rotate = useMutation({ mutationFn: (id: string) => api.rotateApiClientSecret(id), onSuccess: (result) => { setIssued(result); invalidate(); } });

  const rows = clients.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>API Access</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Service-account style clients for the generic <code>/api/v1/objects/...</code> REST API - only
            available where a Team Workspace server is actually running (a pure desktop install has no
            listening socket to receive external calls on). Secrets are hashed, never stored plaintext, and
            shown only once.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>{creating ? "Cancel" : "+ New API client"}</button>
      </div>

      {creating && <ApiClientForm onDone={(result) => { setCreating(false); setIssued(result); invalidate(); }} onCancel={() => setCreating(false)} />}

      {clients.isLoading && <p>Loading...</p>}
      {!clients.isLoading && rows.length === 0 && !creating && <p className="empty-state">No API clients yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead><tr><th>Name</th><th>Client ID</th><th>Scopes</th><th>Status</th><th>Last used</th><th></th></tr></thead>
          <tbody>
            {rows.map((c: ApiClient) => (
              <tr key={c.id}>
                <td>{c.name}</td>
                <td><code style={{ fontSize: 12 }}>{c.client_id}</code></td>
                <td style={{ fontSize: 12 }}>{c.scopes.join(", ")}</td>
                <td><span className={`badge${c.status === "active" ? " badge-success" : ""}`}>{c.status}</span></td>
                <td style={{ fontSize: 12 }}>{c.last_used_at ? new Date(c.last_used_at).toLocaleString() : "Never"}</td>
                <td style={{ display: "flex", gap: 6, whiteSpace: "nowrap" }}>
                  <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => rotate.mutate(c.id)} disabled={rotate.isPending}>Rotate</button>
                  {c.status === "active" ? (
                    <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => revoke.mutate(c.id)}>Revoke</button>
                  ) : (
                    <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => reactivate.mutate(c.id)}>Reactivate</button>
                  )}
                  <button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete API client "${c.name}"?`)) del.mutate(c.id); }}>Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {issued && <IssuedSecretModal issued={issued} onClose={() => setIssued(null)} />}
    </div>
  );
}

function ApiClientForm({ onDone, onCancel }: { onDone: (result: IssuedApiClient) => void; onCancel: () => void }) {
  const [input, setInput] = useState<ApiClientInput>({ name: "", scopes: ["objects.read"], allowed_cidr: null, owner_user_id: null });
  const [error, setError] = useState<string | null>(null);
  const create = useMutation({
    mutationFn: () => api.createApiClient(input),
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not create that API client")),
  });
  const toggleScope = (scope: string) => {
    setInput((cur) => ({ ...cur, scopes: cur.scopes.includes(scope) ? cur.scopes.filter((s) => s !== scope) : [...cur.scopes, scope] }));
  };
  return (
    <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      <div className="form-field full"><label>Name</label><input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required /></div>
      <div className="form-field full">
        <label>Scopes</label>
        <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
          {API_SCOPES.map((s) => (
            <label key={s} style={{ fontSize: 13, display: "flex", alignItems: "center", gap: 4 }}>
              <input type="checkbox" checked={input.scopes.includes(s)} onChange={() => toggleScope(s)} /> {s}
            </label>
          ))}
        </div>
      </div>
      <div className="form-field full"><label>Allowed CIDR (optional)</label><input value={input.allowed_cidr ?? ""} onChange={(e) => setInput({ ...input, allowed_cidr: e.target.value || null })} placeholder="10.0.0.0/8" /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={create.isPending || input.scopes.length === 0}>{create.isPending ? "Creating..." : "Create client"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function IssuedSecretModal({ issued, onClose }: { issued: IssuedApiClient; onClose: () => void }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" style={{ maxWidth: 480 }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ marginTop: 0 }}>API key issued</h3>
        <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
          Copy this now - it is never shown again. Send as <code>Authorization: Bearer {"<key>"}</code>.
        </p>
        <pre style={{ fontSize: 13, wordBreak: "break-all", whiteSpace: "pre-wrap", background: "var(--bg-subtle, rgba(0,0,0,0.03))", padding: 10 }}>{issued.api_key}</pre>
        <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 12 }}>
          <button className="btn btn-primary" onClick={onClose}>Done</button>
        </div>
      </div>
    </div>
  );
}

// --- Webhooks & Events (spec §10) ----------------------------------------------

function WebhooksTab() {
  const queryClient = useQueryClient();
  const webhooks = useQuery({ queryKey: ["webhooks"], queryFn: () => api.listWebhooks() });
  const connections = useQuery({ queryKey: ["connections"], queryFn: () => api.listConnections() });
  const [creating, setCreating] = useState(false);
  const [deliveriesOpenFor, setDeliveriesOpenFor] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, string>>({});

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["webhooks"] });
  const pause = useMutation({ mutationFn: (id: string) => api.pauseWebhook(id), onSuccess: invalidate });
  const reactivate = useMutation({ mutationFn: (id: string) => api.reactivateWebhook(id), onSuccess: invalidate });
  const del = useMutation({ mutationFn: (id: string) => api.deleteWebhook(id), onSuccess: invalidate });
  const test = useMutation({
    mutationFn: (id: string) => api.testWebhookDelivery(id),
    onSuccess: (_v, id) => setTestResults((r) => ({ ...r, [id]: "Test delivery sent" })),
    onError: (err, id) => setTestResults((r) => ({ ...r, [id]: apiErrorMessage(err, "Test delivery failed") })),
  });

  const rows = webhooks.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Webhooks & Events</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Outbound subscriptions on record and workflow events - HMAC-SHA256 signed
            (<code>X-Lanesra-Signature</code>), retried with exponential backoff on failure.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>{creating ? "Cancel" : "+ New webhook"}</button>
      </div>

      {creating && (
        <WebhookForm
          connections={connections.data ?? []}
          onDone={() => { setCreating(false); invalidate(); }}
          onCancel={() => setCreating(false)}
        />
      )}

      {webhooks.isLoading && <p>Loading...</p>}
      {!webhooks.isLoading && rows.length === 0 && !creating && <p className="empty-state">No webhooks yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead><tr><th>Name</th><th>Events</th><th>Status</th><th></th></tr></thead>
          <tbody>
            {rows.map((w: Webhook) => {
              const open = deliveriesOpenFor === w.id;
              return (
                <>
                  <tr key={w.id}>
                    <td>{w.name}</td>
                    <td style={{ fontSize: 12 }}>{w.event_types.join(", ")}</td>
                    <td><span className={`badge${w.status === "active" ? " badge-success" : ""}`}>{w.status}</span></td>
                    <td style={{ display: "flex", gap: 6, whiteSpace: "nowrap", flexWrap: "wrap" }}>
                      <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => test.mutate(w.id)} disabled={test.isPending}>Test</button>
                      <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => setDeliveriesOpenFor(open ? null : w.id)}>{open ? "Hide deliveries" : "Deliveries"}</button>
                      {w.status === "active" ? (
                        <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => pause.mutate(w.id)}>Pause</button>
                      ) : (
                        <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => reactivate.mutate(w.id)}>Reactivate</button>
                      )}
                      <button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete webhook "${w.name}"?`)) del.mutate(w.id); }}>Delete</button>
                    </td>
                  </tr>
                  {testResults[w.id] && (
                    <tr key={`${w.id}-result`}><td colSpan={4} style={{ fontSize: 12, color: "var(--text-muted)" }}>{testResults[w.id]}</td></tr>
                  )}
                  {open && (
                    <tr key={`${w.id}-deliveries`}>
                      <td colSpan={4} style={{ background: "var(--bg-subtle, rgba(0,0,0,0.02))" }}>
                        <WebhookDeliveriesPanel webhookId={w.id} />
                      </td>
                    </tr>
                  )}
                </>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}

function WebhookForm({ connections, onDone, onCancel }: { connections: Connection[]; onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<WebhookInput>({
    name: "", connection_id: connections[0]?.id ?? "", event_types: [], object_scope: null, filter_json: null, payload_version: "v1", retry_policy_json: null,
  });
  const [error, setError] = useState<string | null>(null);
  const create = useMutation({
    mutationFn: () => api.createWebhook(input),
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not create that webhook")),
  });
  const toggleEvent = (evt: string) => {
    setInput((cur) => ({ ...cur, event_types: cur.event_types.includes(evt) ? cur.event_types.filter((e) => e !== evt) : [...cur.event_types, evt] }));
  };
  return (
    <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      {connections.length === 0 && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>Create a Connection first - webhooks deliver to a Connection's base URL.</div>}
      <div className="form-field"><label>Name</label><input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required /></div>
      <div className="form-field">
        <label>Deliver via connection</label>
        <select value={input.connection_id} onChange={(e) => setInput({ ...input, connection_id: e.target.value })}>
          {connections.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
        </select>
      </div>
      <div className="form-field full">
        <label>Event types</label>
        <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
          {EVENT_TYPES.map((e) => (
            <label key={e} style={{ fontSize: 13, display: "flex", alignItems: "center", gap: 4 }}>
              <input type="checkbox" checked={input.event_types.includes(e)} onChange={() => toggleEvent(e)} /> {e}
            </label>
          ))}
        </div>
      </div>
      <div className="form-field full"><label>Object scope (optional)</label><input value={input.object_scope ?? ""} onChange={(e) => setInput({ ...input, object_scope: e.target.value || null })} placeholder="companies" /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={create.isPending || !input.connection_id || input.event_types.length === 0}>{create.isPending ? "Creating..." : "Create webhook"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function WebhookDeliveriesPanel({ webhookId }: { webhookId: string }) {
  const deliveries = useQuery({ queryKey: ["webhookDeliveries", webhookId], queryFn: () => api.listWebhookDeliveries(webhookId) });
  const rows = deliveries.data ?? [];
  if (deliveries.isLoading) return <p style={{ margin: "8px 0" }}>Loading...</p>;
  if (rows.length === 0) return <p className="empty-state">No deliveries yet.</p>;
  return (
    <div style={{ padding: "10px 4px" }}>
      <table style={{ marginBottom: 0 }}>
        <thead><tr><th>Event</th><th>Attempt</th><th>Status</th><th>HTTP</th><th>Duration</th><th>When</th></tr></thead>
        <tbody>
          {rows.map((d) => (
            <tr key={d.id}>
              <td>{d.event_type}</td>
              <td>{d.attempt_number}</td>
              <td><span className={`badge${d.status === "success" ? " badge-success" : d.status === "failed" ? " badge-danger" : ""}`}>{d.status}</span></td>
              <td>{d.http_status ?? "—"}</td>
              <td>{d.duration_ms ? `${d.duration_ms}ms` : "—"}</td>
              <td style={{ fontSize: 12 }}>{new Date(d.created_at).toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// --- Data Exchange (spec §12/13/14) --------------------------------------------

function FieldMapEditor({ fieldMap, onChange, targetFieldOptions }: { fieldMap: FieldMapEntry[]; onChange: (m: FieldMapEntry[]) => void; targetFieldOptions: string[] }) {
  const update = (i: number, patch: Partial<FieldMapEntry>) => onChange(fieldMap.map((f, idx) => (idx === i ? { ...f, ...patch } : f)));
  const remove = (i: number) => onChange(fieldMap.filter((_, idx) => idx !== i));
  const add = () => onChange([...fieldMap, { source_column: "", target_field: "", transform: "none", default_value: null, constant: null }]);
  return (
    <div>
      <table style={{ marginBottom: 8 }}>
        <thead><tr><th>Source column</th><th>Target field</th><th>Transform</th><th>Default</th><th></th></tr></thead>
        <tbody>
          {fieldMap.map((f, i) => (
            <tr key={i}>
              <td><input value={f.source_column} onChange={(e) => update(i, { source_column: e.target.value })} style={{ width: 130 }} /></td>
              <td>
                {targetFieldOptions.length > 0 ? (
                  <select value={f.target_field} onChange={(e) => update(i, { target_field: e.target.value })}>
                    <option value="">Select...</option>
                    {targetFieldOptions.map((k) => <option key={k} value={k}>{k}</option>)}
                  </select>
                ) : (
                  <input value={f.target_field} onChange={(e) => update(i, { target_field: e.target.value })} style={{ width: 130 }} />
                )}
              </td>
              <td>
                <select value={f.transform ?? "none"} onChange={(e) => update(i, { transform: e.target.value })}>
                  {TRANSFORMS.map((t) => <option key={t} value={t}>{t}</option>)}
                </select>
              </td>
              <td><input value={f.default_value ?? ""} onChange={(e) => update(i, { default_value: e.target.value || null })} style={{ width: 90 }} /></td>
              <td><button className="btn btn-secondary" style={{ fontSize: 12, padding: "2px 6px" }} type="button" onClick={() => remove(i)}>×</button></td>
            </tr>
          ))}
        </tbody>
      </table>
      <button className="btn btn-secondary" type="button" style={{ fontSize: 12, padding: "4px 8px" }} onClick={add}>+ Add field mapping</button>
    </div>
  );
}

function DataExchangeTab() {
  const objectKeys = useQuery({ queryKey: ["integrationObjectKeys"], queryFn: () => api.listIntegrationObjectKeys() });
  return (
    <div style={{ display: "grid", gap: 16 }}>
      <CsvImportCard objectKeys={objectKeys.data ?? []} />
      <CsvExportCard objectKeys={objectKeys.data ?? []} />
      <MappingsCard objectKeys={objectKeys.data ?? []} />
    </div>
  );
}

function CsvImportCard({ objectKeys }: { objectKeys: { object_key: string; label: string; fields: { key: string }[] }[] }) {
  const [targetObjectKey, setTargetObjectKey] = useState("");
  const [csvText, setCsvText] = useState("");
  const [operation, setOperation] = useState("insert");
  const [matchKey, setMatchKey] = useState("");
  const [duplicatePolicy, setDuplicatePolicy] = useState("skip");
  const [fieldMap, setFieldMap] = useState<FieldMapEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const targetFields = objectKeys.find((o) => o.object_key === targetObjectKey)?.fields.map((f) => f.key) ?? [];

  const runImport = useMutation({
    mutationFn: (dryRun: boolean) => {
      const input: CsvImportInput = { target_object_key: targetObjectKey, csv_text: csvText, operation, match_key: matchKey || null, field_map: fieldMap, duplicate_policy: duplicatePolicy, dry_run: dryRun };
      return api.importCsv(input);
    },
    onError: (err) => setError(apiErrorMessage(err, "Import failed")),
  });

  const autoMapFromHeader = () => {
    const header = csvText.split("\n")[0]?.split(",").map((h) => h.trim()) ?? [];
    setFieldMap(header.filter(Boolean).map((h) => ({ source_column: h, target_field: targetFields.find((f) => f.toLowerCase() === h.toLowerCase()) ?? "", transform: "none", default_value: null, constant: null })));
  };

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>CSV Import</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Any built-in or active Custom Object, going through the same generic record-write path the REST API
        uses - the same validation, business rules and permissions apply.
      </p>
      {error && <div className="error-banner">{error}</div>}
      <div className="form-grid">
        <div className="form-field">
          <label>Target object</label>
          <select value={targetObjectKey} onChange={(e) => setTargetObjectKey(e.target.value)}>
            <option value="">Select...</option>
            {objectKeys.map((o) => <option key={o.object_key} value={o.object_key}>{o.label}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Operation</label>
          <select value={operation} onChange={(e) => setOperation(e.target.value)}>
            {OPERATIONS.map((o) => <option key={o} value={o}>{o}</option>)}
          </select>
        </div>
        {operation !== "insert" && (
          <div className="form-field"><label>Match key (target field)</label><input value={matchKey} onChange={(e) => setMatchKey(e.target.value)} placeholder="email" /></div>
        )}
        <div className="form-field">
          <label>Duplicate policy</label>
          <select value={duplicatePolicy} onChange={(e) => setDuplicatePolicy(e.target.value)}>
            {DUPLICATE_POLICIES.map((p) => <option key={p} value={p}>{p}</option>)}
          </select>
        </div>
        <div className="form-field full"><label>CSV text (first row = header)</label><textarea value={csvText} onChange={(e) => setCsvText(e.target.value)} rows={5} placeholder="name,email\nAcme Corp,info@acme.com" /></div>
        <div className="form-field full">
          <button className="btn btn-secondary" type="button" style={{ fontSize: 12, padding: "4px 8px" }} onClick={autoMapFromHeader} disabled={!csvText.trim()}>Auto-map from header row</button>
        </div>
        <div className="form-field full">
          <label>Field mapping</label>
          <FieldMapEditor fieldMap={fieldMap} onChange={setFieldMap} targetFieldOptions={targetFields} />
        </div>
        <div className="form-field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-secondary" type="button" onClick={() => runImport.mutate(true)} disabled={runImport.isPending || !targetObjectKey || !csvText.trim() || fieldMap.length === 0}>
            {runImport.isPending ? "Running..." : "Preview (dry run)"}
          </button>
          <button className="btn btn-primary" type="button" onClick={() => runImport.mutate(false)} disabled={runImport.isPending || !targetObjectKey || !csvText.trim() || fieldMap.length === 0}>
            {runImport.isPending ? "Importing..." : "Import"}
          </button>
        </div>
      </div>
      {runImport.data && (
        <div style={{ marginTop: 12 }}>
          <p style={{ fontSize: 13 }}>
            {runImport.data.total_rows} rows - <span style={{ color: "var(--success, #2e7d32)" }}>{runImport.data.successful} succeeded</span>,{" "}
            <span style={{ color: "var(--danger, #c0392b)" }}>{runImport.data.failed} failed</span>, {runImport.data.skipped_duplicates} skipped as duplicates ({runImport.data.duration_ms}ms)
          </p>
          {runImport.data.row_results.some((r) => r.status !== "success") && (
            <table style={{ marginTop: 8 }}>
              <thead><tr><th>Row</th><th>Status</th><th>Error</th></tr></thead>
              <tbody>
                {runImport.data.row_results.filter((r) => r.status !== "success").slice(0, 50).map((r) => (
                  <tr key={r.row_index}><td>{r.row_index + 1}</td><td>{r.status}</td><td style={{ fontSize: 12 }}>{r.error ?? "—"}</td></tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  );
}

function CsvExportCard({ objectKeys }: { objectKeys: { object_key: string; label: string }[] }) {
  const [objectKey, setObjectKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const runExport = useMutation({
    mutationFn: () => api.exportCsv(objectKey, {}),
    onSuccess: (csv) => downloadText(`${objectKey || "export"}.csv`, csv, "text/csv"),
    onError: (err) => setError(apiErrorMessage(err, "Export failed")),
  });
  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>CSV Export</h3>
      {error && <div className="error-banner">{error}</div>}
      <div style={{ display: "flex", gap: 8, alignItems: "flex-end" }}>
        <div className="form-field" style={{ margin: 0 }}>
          <label>Object</label>
          <select value={objectKey} onChange={(e) => setObjectKey(e.target.value)}>
            <option value="">Select...</option>
            {objectKeys.map((o) => <option key={o.object_key} value={o.object_key}>{o.label}</option>)}
          </select>
        </div>
        <button className="btn btn-primary" onClick={() => runExport.mutate()} disabled={runExport.isPending || !objectKey}>{runExport.isPending ? "Exporting..." : "Export CSV"}</button>
      </div>
    </div>
  );
}

function MappingsCard({ objectKeys }: { objectKeys: { object_key: string; label: string; fields: { key: string }[] }[] }) {
  const queryClient = useQueryClient();
  const mappings = useQuery({ queryKey: ["mappings"], queryFn: () => api.listMappings() });
  const [creating, setCreating] = useState(false);
  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["mappings"] });
  const del = useMutation({ mutationFn: (id: string) => api.deleteMapping(id), onSuccess: invalidate });
  const rows = mappings.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Mappings</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>Reusable field mappings - saved once, reused across CSV imports or an Integration Job.</p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>{creating ? "Cancel" : "+ New mapping"}</button>
      </div>
      {creating && <MappingForm objectKeys={objectKeys} onDone={() => { setCreating(false); invalidate(); }} onCancel={() => setCreating(false)} />}
      {mappings.isLoading && <p>Loading...</p>}
      {!mappings.isLoading && rows.length === 0 && !creating && <p className="empty-state">No saved mappings yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead><tr><th>Name</th><th>Target</th><th>Operation</th><th>Fields</th><th></th></tr></thead>
          <tbody>
            {rows.map((m: Mapping) => (
              <tr key={m.id}>
                <td>{m.name}{m.needs_review && <span className="badge" style={{ marginLeft: 6 }}>Needs review</span>}</td>
                <td>{m.target_object_key}</td>
                <td>{m.operation}</td>
                <td>{m.field_map.length}</td>
                <td><button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete mapping "${m.name}"?`)) del.mutate(m.id); }}>Delete</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function MappingForm({ objectKeys, onDone, onCancel }: { objectKeys: { object_key: string; label: string; fields: { key: string }[] }[]; onDone: () => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [targetObjectKey, setTargetObjectKey] = useState("");
  const [operation, setOperation] = useState("insert");
  const [matchKey, setMatchKey] = useState("");
  const [duplicatePolicy, setDuplicatePolicy] = useState("skip");
  const [fieldMap, setFieldMap] = useState<FieldMapEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const targetFields = objectKeys.find((o) => o.object_key === targetObjectKey)?.fields.map((f) => f.key) ?? [];
  const create = useMutation({
    mutationFn: () => {
      const input: MappingInput = { name, target_object_key: targetObjectKey, operation, match_key: matchKey || null, field_map: fieldMap, duplicate_policy: duplicatePolicy };
      return api.createMapping(input);
    },
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not create that mapping")),
  });
  return (
    <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      <div className="form-field"><label>Name</label><input value={name} onChange={(e) => setName(e.target.value)} required /></div>
      <div className="form-field">
        <label>Target object</label>
        <select value={targetObjectKey} onChange={(e) => setTargetObjectKey(e.target.value)}>
          <option value="">Select...</option>
          {objectKeys.map((o) => <option key={o.object_key} value={o.object_key}>{o.label}</option>)}
        </select>
      </div>
      <div className="form-field">
        <label>Operation</label>
        <select value={operation} onChange={(e) => setOperation(e.target.value)}>
          {OPERATIONS.map((o) => <option key={o} value={o}>{o}</option>)}
        </select>
      </div>
      {operation !== "insert" && <div className="form-field"><label>Match key</label><input value={matchKey} onChange={(e) => setMatchKey(e.target.value)} /></div>}
      <div className="form-field">
        <label>Duplicate policy</label>
        <select value={duplicatePolicy} onChange={(e) => setDuplicatePolicy(e.target.value)}>
          {DUPLICATE_POLICIES.map((p) => <option key={p} value={p}>{p}</option>)}
        </select>
      </div>
      <div className="form-field full"><FieldMapEditor fieldMap={fieldMap} onChange={setFieldMap} targetFieldOptions={targetFields} /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={create.isPending || !name.trim() || !targetObjectKey}>{create.isPending ? "Creating..." : "Create mapping"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

// --- External / Virtual Objects (spec §16) -------------------------------------

function ExternalObjectsTab() {
  const queryClient = useQueryClient();
  const objects = useQuery({ queryKey: ["externalObjects"], queryFn: () => api.listExternalObjects() });
  const connections = useQuery({ queryKey: ["connections"], queryFn: () => api.listConnections() });
  const [creating, setCreating] = useState(false);
  const [previewOf, setPreviewOf] = useState<string | null>(null);
  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["externalObjects"] });
  const del = useMutation({ mutationFn: (id: string) => api.deleteExternalObject(id), onSuccess: invalidate });
  const rows = objects.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>External Objects</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Read-only records surfaced live from an external system through a Connection - a Job's pull
            source, or just a live preview on its own.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>{creating ? "Cancel" : "+ New external object"}</button>
      </div>
      {creating && <ExternalObjectForm connections={connections.data ?? []} onDone={() => { setCreating(false); invalidate(); }} onCancel={() => setCreating(false)} />}
      {objects.isLoading && <p>Loading...</p>}
      {!objects.isLoading && rows.length === 0 && !creating && <p className="empty-state">No external objects yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead><tr><th>Key</th><th>Display name</th><th>Resource path</th><th></th></tr></thead>
          <tbody>
            {rows.map((o: ExternalObject) => (
              <>
                <tr key={o.id}>
                  <td><code>{o.object_key}</code></td>
                  <td>{o.display_name}</td>
                  <td><code style={{ fontSize: 12 }}>{o.resource_path}</code></td>
                  <td style={{ display: "flex", gap: 6, whiteSpace: "nowrap" }}>
                    <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => setPreviewOf(previewOf === o.object_key ? null : o.object_key)}>
                      {previewOf === o.object_key ? "Hide preview" : "Preview"}
                    </button>
                    <button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete external object "${o.display_name}"?`)) del.mutate(o.id); }}>Delete</button>
                  </td>
                </tr>
                {previewOf === o.object_key && (
                  <tr key={`${o.id}-preview`}>
                    <td colSpan={4} style={{ background: "var(--bg-subtle, rgba(0,0,0,0.02))" }}>
                      <ExternalObjectPreviewPanel objectKey={o.object_key} />
                    </td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ExternalObjectForm({ connections, onDone, onCancel }: { connections: Connection[]; onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<ExternalObjectInput>({ object_key: "", display_name: "", connection_id: connections[0]?.id ?? "", resource_path: "", field_map: [], cache_ttl_seconds: null });
  const [error, setError] = useState<string | null>(null);
  const create = useMutation({
    mutationFn: () => api.createExternalObject(input),
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not create that external object")),
  });
  return (
    <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      {connections.length === 0 && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>Create a Connection first.</div>}
      <div className="form-field"><label>Object key</label><input value={input.object_key} onChange={(e) => setInput({ ...input, object_key: e.target.value })} placeholder="ext_widgets" required /></div>
      <div className="form-field"><label>Display name</label><input value={input.display_name} onChange={(e) => setInput({ ...input, display_name: e.target.value })} required /></div>
      <div className="form-field">
        <label>Connection</label>
        <select value={input.connection_id} onChange={(e) => setInput({ ...input, connection_id: e.target.value })}>
          {connections.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
        </select>
      </div>
      <div className="form-field"><label>Resource path</label><input value={input.resource_path} onChange={(e) => setInput({ ...input, resource_path: e.target.value })} placeholder="/widgets" /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={create.isPending || !input.connection_id}>{create.isPending ? "Creating..." : "Create"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function ExternalObjectPreviewPanel({ objectKey }: { objectKey: string }) {
  const preview = useQuery({ queryKey: ["externalObjectPreview", objectKey], queryFn: () => api.previewExternalObjectRecords(objectKey) });
  if (preview.isLoading) return <p style={{ margin: "8px 0" }}>Fetching live records...</p>;
  if (preview.isError) return <p className="error-banner">{apiErrorMessage(preview.error, "Could not fetch records")}</p>;
  const records = preview.data ?? [];
  if (records.length === 0) return <p className="empty-state">No records returned.</p>;
  return <pre style={{ fontSize: 12, maxHeight: 260, overflow: "auto", margin: "10px 4px" }}>{JSON.stringify(records.slice(0, 20), null, 2)}</pre>;
}

// --- Integration Jobs (spec §15) ------------------------------------------------

function JobsTab() {
  const queryClient = useQueryClient();
  const jobs = useQuery({ queryKey: ["integrationJobs"], queryFn: () => api.listIntegrationJobs() });
  const externalObjects = useQuery({ queryKey: ["externalObjects"], queryFn: () => api.listExternalObjects() });
  const objectKeys = useQuery({ queryKey: ["integrationObjectKeys"], queryFn: () => api.listIntegrationObjectKeys() });
  const [creating, setCreating] = useState(false);
  const [runsOpenFor, setRunsOpenFor] = useState<string | null>(null);
  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["integrationJobs"] });
  const del = useMutation({ mutationFn: (id: string) => api.deleteIntegrationJob(id), onSuccess: invalidate });
  const runNow = useMutation({ mutationFn: (id: string) => api.runIntegrationJobNow(id), onSuccess: invalidate });
  const rows = jobs.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Integration Jobs</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Recurring pull-sync from an External Object into a Lanesra object, on an interval, with a
            checkpoint cursor. The background scheduler only runs where a Team Workspace server is hosting
            this workspace - on a pure desktop install there's no long-running process to host it, so "Run
            Now" is the only way a desktop-hosted Job ever runs (see the Rust core's own note on this gap).
            Push-direction sync (Lanesra → external system) isn't built - only pull.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setCreating((v) => !v)}>{creating ? "Cancel" : "+ New job"}</button>
      </div>
      {creating && (
        <JobForm
          externalObjects={externalObjects.data ?? []}
          objectKeys={objectKeys.data ?? []}
          onDone={() => { setCreating(false); invalidate(); }}
          onCancel={() => setCreating(false)}
        />
      )}
      {jobs.isLoading && <p>Loading...</p>}
      {!jobs.isLoading && rows.length === 0 && !creating && <p className="empty-state">No integration jobs yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead><tr><th>Name</th><th>Target</th><th>Interval</th><th>Status</th><th>Last run</th><th></th></tr></thead>
          <tbody>
            {rows.map((j: IntegrationJob) => {
              const open = runsOpenFor === j.id;
              return (
                <>
                  <tr key={j.id}>
                    <td>{j.name}</td>
                    <td>{j.target_object_key}</td>
                    <td>{j.interval_minutes}m</td>
                    <td><span className={`badge${j.status === "active" ? " badge-success" : ""}`}>{j.status}</span></td>
                    <td style={{ fontSize: 12 }}>{j.last_run_at ? `${j.last_run_status} @ ${new Date(j.last_run_at).toLocaleString()}` : "Never run"}</td>
                    <td style={{ display: "flex", gap: 6, whiteSpace: "nowrap" }}>
                      <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => runNow.mutate(j.id)} disabled={runNow.isPending}>Run Now</button>
                      <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => setRunsOpenFor(open ? null : j.id)}>{open ? "Hide runs" : "Runs"}</button>
                      <button className="btn btn-danger" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => { if (confirm(`Delete job "${j.name}"?`)) del.mutate(j.id); }}>Delete</button>
                    </td>
                  </tr>
                  {open && (
                    <tr key={`${j.id}-runs`}>
                      <td colSpan={6} style={{ background: "var(--bg-subtle, rgba(0,0,0,0.02))" }}>
                        <JobRunsPanel jobId={j.id} />
                      </td>
                    </tr>
                  )}
                </>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}

function JobForm({
  externalObjects, objectKeys, onDone, onCancel,
}: {
  externalObjects: ExternalObject[]; objectKeys: { object_key: string; label: string }[]; onDone: () => void; onCancel: () => void;
}) {
  const [input, setInput] = useState<IntegrationJobInput>({
    name: "", external_object_id: externalObjects[0]?.id ?? "", target_object_key: "", match_key: "", cursor_field: null, interval_minutes: 60,
  });
  const [error, setError] = useState<string | null>(null);
  const create = useMutation({
    mutationFn: () => api.createIntegrationJob(input),
    onSuccess: onDone,
    onError: (err) => setError(apiErrorMessage(err, "Could not create that job")),
  });
  return (
    <form className="form-grid" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); create.mutate(); }}>
      {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
      {externalObjects.length === 0 && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>Create an External Object first - a Job pulls from one.</div>}
      <div className="form-field"><label>Name</label><input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required /></div>
      <div className="form-field">
        <label>Source (External Object)</label>
        <select value={input.external_object_id} onChange={(e) => setInput({ ...input, external_object_id: e.target.value })}>
          {externalObjects.map((o) => <option key={o.id} value={o.id}>{o.display_name}</option>)}
        </select>
      </div>
      <div className="form-field">
        <label>Target object</label>
        <select value={input.target_object_key} onChange={(e) => setInput({ ...input, target_object_key: e.target.value })}>
          <option value="">Select...</option>
          {objectKeys.map((o) => <option key={o.object_key} value={o.object_key}>{o.label}</option>)}
        </select>
      </div>
      <div className="form-field"><label>Match key</label><input value={input.match_key} onChange={(e) => setInput({ ...input, match_key: e.target.value })} placeholder="external_id" required /></div>
      <div className="form-field"><label>Cursor field (optional)</label><input value={input.cursor_field ?? ""} onChange={(e) => setInput({ ...input, cursor_field: e.target.value || null })} placeholder="updated_at" /></div>
      <div className="form-field"><label>Interval (minutes)</label><input type="number" min={1} value={input.interval_minutes} onChange={(e) => setInput({ ...input, interval_minutes: Number(e.target.value) })} /></div>
      <div className="form-field full" style={{ display: "flex", gap: 8 }}>
        <button className="btn btn-primary" type="submit" disabled={create.isPending || !input.external_object_id || !input.target_object_key}>{create.isPending ? "Creating..." : "Create job"}</button>
        <button className="btn btn-secondary" type="button" onClick={onCancel}>Cancel</button>
      </div>
    </form>
  );
}

function JobRunsPanel({ jobId }: { jobId: string }) {
  const runs = useQuery({ queryKey: ["integrationJobRuns", jobId], queryFn: () => api.listIntegrationJobRuns(jobId, 20) });
  const rows = runs.data ?? [];
  if (runs.isLoading) return <p style={{ margin: "8px 0" }}>Loading...</p>;
  if (rows.length === 0) return <p className="empty-state">No runs yet.</p>;
  return (
    <div style={{ padding: "10px 4px" }}>
      <table style={{ marginBottom: 0 }}>
        <thead><tr><th>Started</th><th>Status</th><th>Processed</th><th>Failed</th><th>Error</th></tr></thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.id}>
              <td style={{ fontSize: 12 }}>{new Date(r.started_at).toLocaleString()}</td>
              <td><span className={`badge${r.status === "success" ? " badge-success" : r.status === "failed" ? " badge-danger" : ""}`}>{r.status}</span></td>
              <td>{r.records_processed}</td>
              <td>{r.records_failed}</td>
              <td style={{ fontSize: 12 }}>{r.error_message ?? "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// --- Logs & Monitoring (spec §23) -----------------------------------------------

function LogsTab() {
  const [filters, setFilters] = useState<IntegrationExecutionQuery>({ execution_type: null, status: null, correlation_id: null, limit: 100 });
  const executions = useQuery({ queryKey: ["integrationExecutions", filters], queryFn: () => api.listIntegrationExecutions(filters) });
  const purge = useMutation({ mutationFn: () => api.purgeExpiredIntegrationLogs() });
  const rows = executions.data ?? [];
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Logs & Monitoring</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            One unified log across API calls, webhook deliveries, and import/export runs.
          </p>
        </div>
        <button className="btn btn-secondary" style={{ flexShrink: 0, fontSize: 12 }} onClick={() => purge.mutate()} disabled={purge.isPending}>
          {purge.isPending ? "Purging..." : purge.data !== undefined ? `Purged ${purge.data}` : "Purge expired logs"}
        </button>
      </div>
      <div style={{ display: "flex", gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
        <input placeholder="Execution type" value={filters.execution_type ?? ""} onChange={(e) => setFilters({ ...filters, execution_type: e.target.value || null })} style={{ width: 160 }} />
        <input placeholder="Status" value={filters.status ?? ""} onChange={(e) => setFilters({ ...filters, status: e.target.value || null })} style={{ width: 120 }} />
        <input placeholder="Correlation ID" value={filters.correlation_id ?? ""} onChange={(e) => setFilters({ ...filters, correlation_id: e.target.value || null })} style={{ width: 200 }} />
      </div>
      {executions.isLoading && <p>Loading...</p>}
      {!executions.isLoading && rows.length === 0 && <p className="empty-state">No executions logged yet.</p>}
      {rows.length > 0 && (
        <div style={{ overflowX: "auto" }}>
          <table>
            <thead><tr><th>Type</th><th>Direction</th><th>Status</th><th>Records</th><th>Duration</th><th>When</th></tr></thead>
            <tbody>
              {rows.map((e) => (
                <tr key={e.id}>
                  <td>{e.execution_type}</td>
                  <td>{e.direction}</td>
                  <td><span className={`badge${e.status === "success" ? " badge-success" : e.status === "failed" ? " badge-danger" : ""}`}>{e.status}</span></td>
                  <td style={{ fontSize: 12 }}>R:{e.records_read} W:{e.records_written} S:{e.records_skipped} F:{e.records_failed}</td>
                  <td>{e.duration_ms ? `${e.duration_ms}ms` : "—"}</td>
                  <td style={{ fontSize: 12 }}>{new Date(e.started_at).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// --- Settings (spec §21/22) -----------------------------------------------------

function SettingsTab() {
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: ["integrationSettings"], queryFn: () => api.getIntegrationSettings() });
  const [input, setInput] = useState<IntegrationSettingsUpdate | null>(null);
  const [success, setSuccess] = useState(false);

  const current = input ?? (settings.data ? {
    api_rate_limit_per_minute: settings.data.api_rate_limit_per_minute,
    global_rate_limit_per_minute: settings.data.global_rate_limit_per_minute,
    log_retention_days: settings.data.log_retention_days,
    file_retention_days: settings.data.file_retention_days,
    allow_insecure_connections: settings.data.allow_insecure_connections,
  } : null);

  const save = useMutation({
    mutationFn: () => api.updateIntegrationSettings(current!),
    onSuccess: () => { setSuccess(true); queryClient.invalidateQueries({ queryKey: ["integrationSettings"] }); },
  });

  if (settings.isLoading || !current) return <p>Loading...</p>;

  return (
    <div className="card" style={{ maxWidth: 480 }}>
      <h3 style={{ marginTop: 0 }}>Settings</h3>
      {success && <div className="success-banner">Saved.</div>}
      <form className="form-grid" onSubmit={(e) => { e.preventDefault(); setSuccess(false); save.mutate(); }}>
        <div className="form-field"><label>Per-client API rate limit (per minute)</label><input type="number" min={1} value={current.api_rate_limit_per_minute} onChange={(e) => setInput({ ...current, api_rate_limit_per_minute: Number(e.target.value) })} /></div>
        <div className="form-field"><label>Global API rate limit (per minute)</label><input type="number" min={1} value={current.global_rate_limit_per_minute} onChange={(e) => setInput({ ...current, global_rate_limit_per_minute: Number(e.target.value) })} /></div>
        <div className="form-field"><label>Log retention (days)</label><input type="number" min={1} value={current.log_retention_days} onChange={(e) => setInput({ ...current, log_retention_days: Number(e.target.value) })} /></div>
        <div className="form-field"><label>File retention (days)</label><input type="number" min={1} value={current.file_retention_days} onChange={(e) => setInput({ ...current, file_retention_days: Number(e.target.value) })} /></div>
        <div className="form-field full">
          <label style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <input type="checkbox" checked={current.allow_insecure_connections} onChange={(e) => setInput({ ...current, allow_insecure_connections: e.target.checked })} />
            Allow insecure (non-TLS) outbound connections
          </label>
        </div>
        <div className="form-field full">
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>{save.isPending ? "Saving..." : "Save settings"}</button>
        </div>
      </form>
    </div>
  );
}
