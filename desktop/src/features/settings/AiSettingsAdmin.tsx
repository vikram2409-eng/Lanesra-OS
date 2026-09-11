import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AiProvider, AiProviderInput, AiSettingsInput } from "../../lib/types";

function apiErrorMessage(err: unknown, fallback: string): string {
  return err instanceof ApiError ? err.message : fallback;
}

const PROVIDERS: { key: string; label: string }[] = [
  { key: "anthropic", label: "Anthropic" },
  { key: "openai_compatible", label: "OpenAI-compatible (OpenAI, Groq, Mistral, Ollama, vLLM, llama.cpp, ...)" },
  { key: "google_gemini", label: "Google Gemini" },
];

type AiSubTab = "llm" | "gateway" | "mcp";
const AI_SUB_TABS: { key: AiSubTab; label: string }[] = [
  { key: "llm", label: "LLM" },
  { key: "gateway", label: "Gateway" },
  { key: "mcp", label: "MCP Server" },
];

/**
 * AI & Agentic Layer. A dedicated category of its own (not folded into
 * Integrations - see Settings.tsx's own comment on why), with two
 * sub-tabs matching the two things this initiative is actually about:
 *
 * - **LLM** (Phase 1, built): the one thing every later agent feature (the
 *   MCP server below, a unified Activity Timeline, and agent actions
 *   built on top - meeting-prep briefings, follow-up capture, record
 *   hygiene, natural-language reporting) needs to exist first. Lanesra
 *   has no SaaS billing surface to meter inference through, so a
 *   workspace supplies its own provider key here - encrypted at rest the
 *   same way any Connection's secret already is (see `ai_service.rs`) -
 *   and Lanesra itself never resells, proxies or bills for it. Nothing in
 *   this product calls an LLM at all until this tab has a working key.
 * - **MCP Server** (Phase 2, built): a `POST /mcp` endpoint
 *   (`server/src/mcp.rs`) exposing the same 7-operation, permission-checked
 *   `api_object_service` dispatcher Integration Hub's REST API already
 *   uses, over the Model Context Protocol - so any MCP-capable agent
 *   (Claude, or any other) reads/writes Lanesra records under the
 *   identical rules a human's UI action already goes through. Reuses the
 *   exact same scoped API-client bearer key the REST API authenticates
 *   with (Integration Hub -> API Access) - there's no separate MCP
 *   credential to provision. Only reachable where a Team Workspace
 *   server is actually running, the same platform boundary API Access's
 *   own copy already states - a pure desktop install has no listening
 *   socket for an external agent to reach either.
 */
export function AiSettingsAdmin() {
  const [tab, setTab] = useState<AiSubTab>("llm");

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <div>
        <h3 style={{ margin: "0 0 4px" }}>LLM &amp; MCP</h3>
        <p style={{ color: "var(--text-muted)", fontSize: 13, margin: 0 }}>
          Bring your own LLM provider key, and (once built) the MCP server that lets an agent read and write this
          workspace's records the same way a person does. Lanesra is self-hosted with no subscription or seat
          charges, so there's no billing surface to meter any of this through.
        </p>
      </div>
      <div className="tab-row">
        {AI_SUB_TABS.map((t) => (
          <button key={t.key} className={`tab${tab === t.key ? " active" : ""}`} onClick={() => setTab(t.key)}>
            {t.label}
          </button>
        ))}
      </div>
      {tab === "llm" && <LlmTab />}
      {tab === "gateway" && <GatewayTab />}
      {tab === "mcp" && <McpTab />}
    </div>
  );
}

function LlmTab() {
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: ["aiSettings"], queryFn: () => api.getAiSettings() });
  const [input, setInput] = useState<AiSettingsInput | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);

  const current = input ?? (settings.data ? { provider: settings.data.provider, base_url: settings.data.base_url, model: settings.data.model, api_key: null } : null);

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["aiSettings"] });

  const save = useMutation({
    mutationFn: () => api.saveAiSettings({ ...current!, api_key: apiKeyDraft || null }),
    onSuccess: () => {
      setApiKeyDraft("");
      setTestResult(null);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(apiErrorMessage(err, "Could not save AI settings")),
  });

  const test = useMutation({
    mutationFn: () => api.testAiKey(),
    onSuccess: (result) => {
      setTestResult(result.ok ? `Key valid (${result.latency_ms}ms) - ${result.message}` : `Failed: ${result.message}`);
      invalidate();
    },
    onError: (err) => setTestResult(apiErrorMessage(err, "Test failed")),
  });

  if (settings.isLoading || !current) return <p>Loading...</p>;

  return (
    <div className="card" style={{ maxWidth: 560 }}>
      <h3 style={{ marginTop: 0 }}>LLM</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        No key configured means no AI feature in this product does anything; nothing here calls out anywhere until
        you set this up. The cost and the relationship are between this workspace and whichever provider you
        configure below - Lanesra never sees or resells it.
      </p>
      {error && <div className="error-banner">{error}</div>}

      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field">
          <label>Provider</label>
          <select value={current.provider} onChange={(e) => setInput({ ...current, provider: e.target.value })}>
            {PROVIDERS.map((p) => (
              <option key={p.key} value={p.key}>
                {p.label}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Model</label>
          <input
            value={current.model}
            onChange={(e) => setInput({ ...current, model: e.target.value })}
            placeholder={current.provider === "anthropic" ? "claude-haiku-4-5-20251001 (default if left blank)" : "e.g. gpt-4o, llama3.1"}
          />
        </div>
        {(current.provider === "openai_compatible" || !!current.base_url) && (
          <div className="form-field full">
            <label>{current.provider === "openai_compatible" ? "Base URL (required)" : "Base URL override (optional)"}</label>
            <input
              value={current.base_url ?? ""}
              onChange={(e) => setInput({ ...current, base_url: e.target.value || null })}
              placeholder={current.provider === "openai_compatible" ? "http://localhost:11434/v1" : "https://api.anthropic.com"}
            />
          </div>
        )}
        <div className="form-field full">
          <label>{settings.data?.has_key ? "API key (leave blank to keep the current one)" : "API key"}</label>
          <input type="password" value={apiKeyDraft} onChange={(e) => setApiKeyDraft(e.target.value)} placeholder={settings.data?.has_key ? "Stored - unchanged unless you enter a new one" : "sk-..."} />
        </div>
        <div className="form-field full" style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            {save.isPending ? "Saving..." : "Save"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={() => test.mutate()} disabled={test.isPending || !settings.data?.has_key}>
            {test.isPending ? "Testing..." : "Test key"}
          </button>
          {settings.data && (
            <span className={`badge${settings.data.status === "connected" ? " badge-success" : settings.data.status === "failed" ? " badge-danger" : ""}`}>{settings.data.status}</span>
          )}
        </div>
      </form>

      {testResult && <p style={{ fontSize: 13, marginTop: 8, color: testResult.startsWith("Failed") ? "var(--danger, #dc2626)" : "var(--success, #059669)" }}>{testResult}</p>}
      {!settings.data?.has_key && <p className="empty-state" style={{ marginTop: 12 }}>No key configured yet - save one above, then Test key to prove it works.</p>}
    </div>
  );
}

function emptyProviderInput(): AiProviderInput {
  return { name: "", provider: "anthropic", base_url: null, model: "", api_key: null };
}

/**
 * Phase 7a: the Unified AI Gateway's admin surface - named provider
 * connections an agent's Model Routing (AiAgentsAdmin.tsx -> Routing) can
 * pick per tier, the System-tier daily token budget, and a small health
 * view of recent failover events (a run that didn't end up served by its
 * primary tier - see `ai_gateway_service`'s own doc comment).
 */
function GatewayTab() {
  const queryClient = useQueryClient();
  const providersQuery = useQuery({ queryKey: ["aiProviders"], queryFn: () => api.listAiProviders(false) });
  const usageQuery = useQuery({ queryKey: ["aiTokenUsageSummary"], queryFn: () => api.getAiTokenUsageSummary() });
  const failoverQuery = useQuery({ queryKey: ["aiGatewayFailoverEvents"], queryFn: () => api.listAiGatewayFailoverEvents(25) });
  const settingsQuery = useQuery({ queryKey: ["aiSettings"], queryFn: () => api.getAiSettings() });
  const providers = providersQuery.data ?? [];

  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<AiProvider | null>(null);
  const [budgetDraft, setBudgetDraft] = useState<string | null>(null);
  const [otlpDraft, setOtlpDraft] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);

  function invalidateProviders() {
    queryClient.invalidateQueries({ queryKey: ["aiProviders"] });
  }

  const create = useMutation({
    mutationFn: (input: AiProviderInput) => api.createAiProvider(input),
    onSuccess: () => {
      setCreating(false);
      setError(null);
      invalidateProviders();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this provider"),
  });
  const update = useMutation({
    mutationFn: ({ id, input }: { id: string; input: AiProviderInput }) => api.updateAiProvider(id, input),
    onSuccess: () => {
      setEditing(null);
      setError(null);
      invalidateProviders();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this provider"),
  });
  const toggleActive = useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) => api.setAiProviderActive(id, isActive),
    onSuccess: invalidateProviders,
  });
  const test = useMutation({
    mutationFn: (id: string) => api.testAiProviderKey(id),
    onSuccess: (result, id) => setTestResults((prev) => ({ ...prev, [id]: result.ok ? `Valid (${result.latency_ms}ms)` : `Failed: ${result.message}` })),
    onError: (err, id) => setTestResults((prev) => ({ ...prev, [id]: apiErrorMessage(err, "Test failed") })),
  });
  const saveBudget = useMutation({
    mutationFn: (daily_token_budget: number | null) => api.setAiDailyTokenBudget({ daily_token_budget }),
    onSuccess: () => {
      setBudgetDraft(null);
      queryClient.invalidateQueries({ queryKey: ["aiTokenUsageSummary"] });
    },
  });
  const saveOtlpEndpoint = useMutation({
    mutationFn: (otlp_endpoint: string | null) => api.setAiOtlpEndpoint({ otlp_endpoint }),
    onSuccess: () => {
      setOtlpDraft(null);
      queryClient.invalidateQueries({ queryKey: ["aiSettings"] });
    },
  });

  const budget = usageQuery.data?.daily_token_budget ?? null;
  const budgetValue = budgetDraft !== null ? budgetDraft : budget === null ? "" : String(budget);
  const otlpValue = otlpDraft !== null ? otlpDraft : settingsQuery.data?.otlp_endpoint ?? "";

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <div className="card" style={{ maxWidth: 560 }}>
        <h3 style={{ marginTop: 0 }}>System token budget</h3>
        <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
          The top of the Gateway's System → Agent → User token-budget hierarchy - applies to every agent run in this
          workspace, on top of whatever daily budget an individual agent's own Routing settings might add.
        </p>
        {usageQuery.data && (
          <p style={{ fontSize: 13 }}>
            Used today: <b>{usageQuery.data.today_input_tokens + usageQuery.data.today_output_tokens}</b> tokens
          </p>
        )}
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <input
            type="number"
            min={1}
            style={{ maxWidth: 160 }}
            value={budgetValue}
            placeholder="Unlimited"
            onChange={(e) => setBudgetDraft(e.target.value)}
          />
          <button
            className="btn btn-secondary"
            disabled={saveBudget.isPending}
            onClick={() => saveBudget.mutate(budgetValue === "" ? null : Number(budgetValue))}
          >
            {saveBudget.isPending ? "Saving..." : "Save"}
          </button>
        </div>
      </div>

      <div className="card" style={{ maxWidth: 560 }}>
        <h3 style={{ marginTop: 0 }}>Observability</h3>
        <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
          An optional OTLP collector endpoint - each agent/pipeline run's trace (real per-step timing, no LLM output) can be pushed here on demand
          from that run's own history, or exported as OTLP JSON with no endpoint configured at all.
        </p>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <input style={{ flex: 1 }} placeholder="https://your-collector:4318/v1/traces" value={otlpValue} onChange={(e) => setOtlpDraft(e.target.value)} />
          <button className="btn btn-secondary" disabled={saveOtlpEndpoint.isPending} onClick={() => saveOtlpEndpoint.mutate(otlpValue.trim() === "" ? null : otlpValue)}>
            {saveOtlpEndpoint.isPending ? "Saving..." : "Save"}
          </button>
        </div>
      </div>

      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
          <div>
            <h3 style={{ margin: 0 }}>AI providers</h3>
            <p style={{ color: "var(--text-muted)", fontSize: 13, margin: "4px 0 0" }}>
              Named provider connections, beyond the single LLM tab default above - an agent's Model Routing (AI
              Agents → Routing) picks one of these per tier (primary/fallback/local air-gapped).
            </p>
          </div>
          <button className="btn btn-primary" onClick={() => setCreating(true)}>
            + New provider
          </button>
        </div>
        {error && <div className="error-banner">{error}</div>}
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Provider</th>
                <th>Model</th>
                <th>Key</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {providers.map((p) => (
                <tr key={p.id}>
                  <td>{p.name}</td>
                  <td>{p.provider}</td>
                  <td>{p.model}</td>
                  <td>{p.has_key ? "Configured" : "—"}</td>
                  <td>
                    <span className={`badge${p.is_active ? " badge-success" : ""}`}>{p.is_active ? "Active" : "Inactive"}</span>
                  </td>
                  <td>
                    <div style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
                      <button className="btn btn-secondary" onClick={() => setEditing(p)}>
                        Edit
                      </button>
                      <button className="btn btn-secondary" onClick={() => test.mutate(p.id)} disabled={!p.has_key || test.isPending}>
                        Test
                      </button>
                      <button className="btn btn-secondary" onClick={() => toggleActive.mutate({ id: p.id, isActive: !p.is_active })}>
                        {p.is_active ? "Deactivate" : "Reactivate"}
                      </button>
                      {testResults[p.id] && <span style={{ fontSize: 12, color: "var(--text-muted)" }}>{testResults[p.id]}</span>}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {providers.length === 0 && <div className="empty-state">No named providers yet - agents without one use the plain LLM tab default.</div>}
        </div>
      </div>

      {(creating || editing) && (
        <AiProviderForm
          initial={editing ?? undefined}
          onCancel={() => {
            setCreating(false);
            setEditing(null);
          }}
          onSubmit={(input) => (editing ? update.mutate({ id: editing.id, input }) : create.mutate(input))}
          pending={create.isPending || update.isPending}
        />
      )}

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Gateway health</h3>
        <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
          Recent dispatches that didn't end up served by an agent's primary tier - a real failover, or forced
          air-gapped routing because the outbound payload matched a sensitive-data class.
        </p>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Agent</th>
                <th>Served by</th>
                <th>Reason</th>
                <th>When</th>
              </tr>
            </thead>
            <tbody>
              {(failoverQuery.data ?? []).map((ev) => (
                <tr key={ev.id}>
                  <td>{ev.agent_name}</td>
                  <td>
                    <span className="badge">{ev.served_by}</span>
                  </td>
                  <td style={{ fontSize: 13 }}>{ev.reason}</td>
                  <td style={{ fontSize: 12, color: "var(--text-muted)" }}>{ev.created_at}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {(failoverQuery.data ?? []).length === 0 && <div className="empty-state">No failovers recorded yet.</div>}
        </div>
      </div>
    </div>
  );
}

function AiProviderForm({
  initial,
  onCancel,
  onSubmit,
  pending,
}: {
  initial?: AiProvider;
  onCancel: () => void;
  onSubmit: (input: AiProviderInput) => void;
  pending: boolean;
}) {
  const [input, setInput] = useState<AiProviderInput>(
    initial ? { name: initial.name, provider: initial.provider, base_url: initial.base_url, model: initial.model, api_key: null } : emptyProviderInput(),
  );

  return (
    <div className="card" style={{ maxWidth: 560 }}>
      <h3 style={{ marginTop: 0 }}>{initial ? `Edit ${initial.name}` : "New provider"}</h3>
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          onSubmit(input);
        }}
      >
        <div className="form-field">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} placeholder="e.g. Self-hosted Ollama" required />
        </div>
        <div className="form-field">
          <label>Provider</label>
          <select value={input.provider} onChange={(e) => setInput({ ...input, provider: e.target.value })}>
            {PROVIDERS.map((p) => (
              <option key={p.key} value={p.key}>
                {p.label}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Model</label>
          <input value={input.model} onChange={(e) => setInput({ ...input, model: e.target.value })} placeholder="e.g. gpt-4o, llama3.1, gemini-1.5-pro" />
        </div>
        <div className="form-field full">
          <label>{input.provider === "openai_compatible" ? "Base URL (required)" : "Base URL override (optional)"}</label>
          <input
            value={input.base_url ?? ""}
            onChange={(e) => setInput({ ...input, base_url: e.target.value || null })}
            placeholder={input.provider === "openai_compatible" ? "http://localhost:11434/v1" : ""}
          />
        </div>
        <div className="form-field full">
          <label>{initial?.has_key ? "API key (leave blank to keep the current one)" : "API key"}</label>
          <input
            type="password"
            value={input.api_key ?? ""}
            onChange={(e) => setInput({ ...input, api_key: e.target.value || null })}
            placeholder={initial?.has_key ? "Stored - unchanged unless you enter a new one" : "sk-..."}
          />
        </div>
        <div className="form-field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={pending}>
            {pending ? "Saving..." : initial ? "Save provider" : "Create provider"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

const MCP_TOOLS: { name: string; description: string }[] = [
  { name: "list_objects", description: "List every built-in and custom object this workspace exposes" },
  { name: "get_object_metadata", description: "An object's label and custom field definitions" },
  { name: "list_records", description: "List records for an object, paginated, with optional filter/sort" },
  { name: "get_record", description: "Get a single record by id" },
  { name: "create_record", description: "Create a record (Company, Contact, Product, Task, or a custom object)" },
  { name: "update_record", description: "Update a record's fields by id" },
  { name: "archive_record", description: "Archive (soft-delete) a record by id" },
];

function McpTab() {
  const mcpUrl = `${window.location.origin}/mcp`;
  return (
    <div className="card" style={{ maxWidth: 560 }}>
      <h3 style={{ marginTop: 0 }}>MCP Server</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        A single <code>POST /mcp</code> endpoint exposing 7 tools over the Model Context Protocol - the same
        generic, permission-checked object dispatcher the <code>/api/v1</code> REST API already wraps, so an
        MCP-capable agent reads and writes records under the identical rules a human's UI action already goes
        through. Only reachable where a Team Workspace server is actually running - a pure desktop install has no
        listening socket to receive external calls on, the same boundary Integration Hub → API Access already
        states.
      </p>

      <div className="form-field full">
        <label>Endpoint</label>
        <input readOnly value={mcpUrl} onFocus={(e) => e.currentTarget.select()} />
      </div>

      <p style={{ fontSize: 13 }}>
        Point an MCP client at the endpoint above with an{" "}
        <b>
          <code>Authorization: Bearer &#123;client_id&#125;.&#123;secret&#125;</code>
        </b>{" "}
        header - the same API client credential the REST API already uses. Issue or reuse one from{" "}
        <b>Integration Hub → API Access</b> with the scopes each tool below needs (<code>metadata.read</code> for
        the two read-only lookups, <code>objects.read</code> to list/get records, <code>objects.write</code> to
        create/update/archive). No separate MCP credential exists.
      </p>

      <table>
        <thead>
          <tr>
            <th>Tool</th>
            <th>What it does</th>
          </tr>
        </thead>
        <tbody>
          {MCP_TOOLS.map((t) => (
            <tr key={t.name}>
              <td><code style={{ fontSize: 12 }}>{t.name}</code></td>
              <td style={{ fontSize: 13 }}>{t.description}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <p className="empty-state" style={{ marginTop: 12 }}>
        The paired <b>CLI</b> (<code>lanesra</code>, in this repo's <code>desktop/cli</code>) is a separate, simpler
        way to script the same REST API from a shell - build it with <code>cargo build --release -p lanesra-cli</code>
        and point it at this same base URL and a client key. It's a plain wrapper, not an MCP client itself.
      </p>
    </div>
  );
}
