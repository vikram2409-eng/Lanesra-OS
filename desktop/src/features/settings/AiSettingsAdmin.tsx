import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AiSettingsInput } from "../../lib/types";

function apiErrorMessage(err: unknown, fallback: string): string {
  return err instanceof ApiError ? err.message : fallback;
}

const PROVIDERS: { key: string; label: string }[] = [
  { key: "anthropic", label: "Anthropic" },
  { key: "openai_compatible", label: "OpenAI-compatible (OpenAI, or a self-hosted server)" },
];

/**
 * AI & Agentic Layer, Phase 1: the one thing every later agent feature
 * (an MCP server over the existing generic object API, a unified
 * Activity Timeline, and agent actions built on top - meeting-prep
 * briefings, follow-up capture, record hygiene, natural-language
 * reporting) needs to exist first. Lanesra has no SaaS billing surface to
 * meter inference through, so a workspace supplies its own provider key
 * here - encrypted at rest the same way any Connection's secret already
 * is (see `ai_service.rs`) - and Lanesra itself never resells, proxies or
 * bills for it. Nothing in this product calls an LLM at all until this
 * screen has a working key.
 */
export function AiSettingsAdmin() {
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
      <h3 style={{ marginTop: 0 }}>AI</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Bring your own LLM provider key. Lanesra is self-hosted with no subscription or seat charges, so there's no
        billing surface to meter AI usage through - the cost and the relationship are between this workspace and
        whichever provider you configure here. No key configured means no AI feature in this product does anything;
        nothing here calls out anywhere until you set this up.
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
