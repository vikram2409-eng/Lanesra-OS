import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  entityTypeLabel,
  type MetricAggregation,
  type MetricDefinition,
  type MetricDefinitionInput,
} from "../../lib/types";

const AGGREGATIONS: MetricAggregation[] = ["sum", "avg", "count", "count_distinct", "min", "max"];

function emptyInput(entityTypes: string[]): MetricDefinitionInput {
  return { name: "", description: null, source_entity_type: entityTypes[0] ?? "Company", source_field_key: null, aggregation: "sum", grain: null, filters_json: "{}", time_logic: null };
}

/**
 * Next-Gen program, Domain A (Intelligence Foundation), FND-02: Metric
 * definitions - a declarative description of what a metric means
 * (aggregation/source/grain/filters/time logic), not a working formula
 * evaluator. Domain D's later Derived Metrics work computes against this
 * metadata; this screen only defines and versions it. A metric's own
 * System Graph node traces straight back to its source object/field and
 * glossary term (see `metric_service::sync_graph_node`, Rust) - the
 * Dependency Explorer is this screen's own "where does this number come
 * from" answer.
 */
export function MetricDefinitionsAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [versionsId, setVersionsId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const metrics = useQuery({ queryKey: ["metricDefinitions", "all"], queryFn: () => api.listMetricDefinitions(false) });
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const glossaryTerms = useQuery({ queryKey: ["glossaryTerms", "active"], queryFn: () => api.listGlossaryTerms(true) });
  const entityTypes = [...CUSTOM_FIELD_ENTITY_TYPES, ...(customObjects.data?.map((o) => o.key) ?? [])];
  const labelFor = (t: string) => customObjects.data?.find((o) => o.key === t)?.plural_label ?? entityTypeLabel(t);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["metricDefinitions"] });
  }

  const editing = metrics.data?.find((m) => m.id === editingId) ?? null;
  const deactivate = useMutation({ mutationFn: (id: string) => api.deactivateMetricDefinition(id), onSuccess: invalidate });
  const remove = useMutation({ mutationFn: (id: string) => api.deleteMetricDefinition(id), onSuccess: invalidate });

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Metric Definitions</h3>
        <button className="btn btn-primary" onClick={() => { setCreating((v) => !v); setEditingId(null); }}>
          + New metric
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Declare what a metric means - its source, aggregation, grain and filters - so it has one traceable definition
        instead of being redefined ad hoc in every report. No computation happens here yet; this is the metadata layer
        later analytics builds on.
      </p>

      {creating && (
        <MetricForm entityTypes={entityTypes} labelFor={labelFor} glossaryTerms={glossaryTerms.data ?? []} onDone={() => { invalidate(); setCreating(false); }} onCancel={() => setCreating(false)} />
      )}
      {editing && (
        <MetricForm metric={editing} entityTypes={entityTypes} labelFor={labelFor} glossaryTerms={glossaryTerms.data ?? []} onDone={() => { invalidate(); setEditingId(null); }} onCancel={() => setEditingId(null)} />
      )}

      <div className="table-wrap">
        <table className="table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Source</th>
              <th>Aggregation</th>
              <th>Version</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {(metrics.data ?? []).map((m) => (
              <>
                <tr key={m.id}>
                  <td><b>{m.name}</b>{m.description && <><br /><small className="muted">{m.description}</small></>}</td>
                  <td>{labelFor(m.source_entity_type)}{m.source_field_key ? `.${m.source_field_key}` : ""}</td>
                  <td>{m.aggregation}</td>
                  <td>v{m.version}</td>
                  <td>{m.is_active ? <span className="badge badge-success">Active</span> : <span className="badge">Inactive</span>}</td>
                  <td>
                    <div className="actions">
                      <button className="icon-btn" onClick={() => setVersionsId(versionsId === m.id ? null : m.id)}>Versions</button>
                      <button className="icon-btn" onClick={() => { setEditingId(m.id); setCreating(false); }}>Edit</button>
                      {m.is_active && <button className="icon-btn" onClick={() => deactivate.mutate(m.id)}>Deactivate</button>}
                      <button className="icon-btn" onClick={() => { if (confirm(`Delete metric '${m.name}'?`)) remove.mutate(m.id); }}>Delete</button>
                    </div>
                  </td>
                </tr>
                {versionsId === m.id && (
                  <tr key={`${m.id}-versions`}>
                    <td colSpan={6}><VersionHistory metricId={m.id} /></td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
        {!metrics.data?.length && <div className="empty">No metric definitions yet</div>}
      </div>
    </div>
  );
}

function MetricForm({
  metric,
  entityTypes,
  labelFor,
  glossaryTerms,
  onDone,
  onCancel,
}: {
  metric?: MetricDefinition;
  entityTypes: string[];
  labelFor: (t: string) => string;
  glossaryTerms: { id: string; name: string }[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<MetricDefinitionInput>(
    metric
      ? {
          name: metric.name, description: metric.description, source_entity_type: metric.source_entity_type, source_field_key: metric.source_field_key,
          aggregation: metric.aggregation, grain: metric.grain, filters_json: metric.filters_json, time_logic: metric.time_logic,
          owner_user_id: metric.owner_user_id, glossary_term_id: metric.glossary_term_id,
          effective_start_date: metric.effective_start_date, effective_end_date: metric.effective_end_date,
        }
      : emptyInput(entityTypes),
  );
  const [error, setError] = useState<string | null>(null);
  const fields = useQuery({ queryKey: ["customFieldDefinitions", input.source_entity_type], queryFn: () => api.listCustomFieldDefinitions(input.source_entity_type, true) });

  const save = useMutation({
    mutationFn: () => (metric ? api.updateMetricDefinition(metric.id, input) : api.createMetricDefinition(input)),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this metric"),
  });

  return (
    <div className="panel" style={{ marginBottom: 16 }}>
      {error && <div className="error-banner">{error}</div>}
      <div className="form-grid">
        <div className="form-field">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} />
        </div>
        <div className="form-field">
          <label>Aggregation</label>
          <select value={input.aggregation} onChange={(e) => setInput({ ...input, aggregation: e.target.value as MetricAggregation })}>
            {AGGREGATIONS.map((a) => <option key={a} value={a}>{a}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Source object</label>
          <select value={input.source_entity_type} onChange={(e) => setInput({ ...input, source_entity_type: e.target.value, source_field_key: null })}>
            {entityTypes.map((t) => <option key={t} value={t}>{labelFor(t)}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Source field</label>
          <select value={input.source_field_key ?? ""} onChange={(e) => setInput({ ...input, source_field_key: e.target.value || null })}>
            <option value="">Record count / whole object</option>
            {(fields.data ?? []).map((f) => <option key={f.id} value={f.key}>{f.label}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Grain</label>
          <input placeholder="e.g. monthly, by region" value={input.grain ?? ""} onChange={(e) => setInput({ ...input, grain: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Time logic</label>
          <input placeholder="e.g. trailing 30 days" value={input.time_logic ?? ""} onChange={(e) => setInput({ ...input, time_logic: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Glossary term</label>
          <select value={input.glossary_term_id ?? ""} onChange={(e) => setInput({ ...input, glossary_term_id: e.target.value || null })}>
            <option value="">None</option>
            {glossaryTerms.map((t) => <option key={t.id} value={t.id}>{t.name}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Effective start date</label>
          <input type="date" value={input.effective_start_date ?? ""} onChange={(e) => setInput({ ...input, effective_start_date: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Effective end date</label>
          <input type="date" value={input.effective_end_date ?? ""} onChange={(e) => setInput({ ...input, effective_end_date: e.target.value || null })} />
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Description</label>
          <textarea rows={2} value={input.description ?? ""} onChange={(e) => setInput({ ...input, description: e.target.value || null })} />
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Filters (JSON)</label>
          <textarea rows={2} value={input.filters_json ?? "{}"} onChange={(e) => setInput({ ...input, filters_json: e.target.value })} />
        </div>
      </div>
      <div className="actions">
        <button className="btn btn-primary" onClick={() => save.mutate()} disabled={save.isPending}>{metric ? "Save" : "Create"}</button>
        <button className="btn" onClick={onCancel}>Cancel</button>
      </div>
    </div>
  );
}

function VersionHistory({ metricId }: { metricId: string }) {
  const versions = useQuery({ queryKey: ["metricVersions", metricId], queryFn: () => api.listMetricVersions(metricId) });
  if (!versions.data?.length) return <p className="muted" style={{ fontSize: 12, padding: "8px 0" }}>No saved versions yet - versions are recorded on each edit.</p>;
  return (
    <div style={{ padding: "8px 0" }}>
      {versions.data.map((v) => (
        <div key={v.id} style={{ marginBottom: 6, fontSize: 13 }}>
          <b>{new Date(v.saved_at).toLocaleString()}</b> — {v.snapshot.aggregation} of {v.snapshot.source_entity_type}
          {v.snapshot.source_field_key ? `.${v.snapshot.source_field_key}` : ""}, grain: {v.snapshot.grain ?? "—"}
        </div>
      ))}
    </div>
  );
}
