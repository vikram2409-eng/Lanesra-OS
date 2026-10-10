import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  entityTypeLabel,
  type BusinessGlossaryTerm,
  type BusinessGlossaryTermInput,
  type DataClassification,
  type SemanticMappingInput,
  type SemanticRole,
} from "../../lib/types";

const CLASSIFICATION_LABELS: Record<DataClassification, string> = {
  standard: "Standard",
  sensitive: "Sensitive",
  restricted: "Restricted",
};

const SEMANTIC_ROLES: SemanticRole[] = [
  "customer", "policyholder", "amount", "currency", "quantity", "percentage", "effective_date", "expiration_date",
  "region", "owner", "status", "identifier", "email", "phone",
];

function emptyInput(): BusinessGlossaryTermInput {
  return { name: "", definition: "", owner_user_id: null, synonyms: [], data_classification: "standard" };
}

/**
 * Next-Gen program, Domain A (Intelligence Foundation), FND-02: the
 * Business Glossary - explicit business semantics above raw schema, so a
 * generated report or agent can query consistent meaning instead of
 * inferring it from a field's label. Each term's "Mapped to" panel shows
 * its live `semantic_mappings`, which is exactly what drives the term's
 * own System Graph node edges (see `glossary_service::sync_graph_edges`,
 * Rust) - the Dependency Explorer can trace a term straight back to here.
 */
export function BusinessGlossaryAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [mappingPanelId, setMappingPanelId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const terms = useQuery({ queryKey: ["glossaryTerms", "all"], queryFn: () => api.listGlossaryTerms(false) });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["glossaryTerms"] });
  }

  const editing = terms.data?.find((t) => t.id === editingId) ?? null;

  const deactivate = useMutation({
    mutationFn: (id: string) => api.deactivateGlossaryTerm(id),
    onSuccess: invalidate,
  });

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Business Glossary</h3>
        <button className="btn btn-primary" onClick={() => { setCreating((v) => !v); setEditingId(null); }}>
          + New term
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Define business terms once - name, definition, synonyms, data classification - then map them onto the objects
        and fields they describe. Agents and generated analytics query this instead of guessing from a field's label.
      </p>

      {creating && (
        <GlossaryTermForm
          users={users.data ?? []}
          onDone={() => { invalidate(); setCreating(false); }}
          onCancel={() => setCreating(false)}
        />
      )}
      {editing && (
        <GlossaryTermForm
          term={editing}
          users={users.data ?? []}
          onDone={() => { invalidate(); setEditingId(null); }}
          onCancel={() => setEditingId(null)}
        />
      )}

      <div className="table-wrap">
        <table className="table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Definition</th>
              <th>Classification</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {(terms.data ?? []).map((t) => (
              <>
                <tr key={t.id}>
                  <td><b>{t.name}</b>{t.synonyms.length > 0 && <><br /><small className="muted">aka {t.synonyms.join(", ")}</small></>}</td>
                  <td style={{ maxWidth: 360 }}>{t.definition}</td>
                  <td>{CLASSIFICATION_LABELS[t.data_classification]}</td>
                  <td>{t.is_active ? <span className="badge badge-success">Active</span> : <span className="badge">Inactive</span>}</td>
                  <td>
                    <div className="actions">
                      <button className="icon-btn" onClick={() => setMappingPanelId(mappingPanelId === t.id ? null : t.id)}>Mapped to</button>
                      <button className="icon-btn" onClick={() => { setEditingId(t.id); setCreating(false); }}>Edit</button>
                      {t.is_active && (
                        <button className="icon-btn" onClick={() => deactivate.mutate(t.id)}>Deactivate</button>
                      )}
                    </div>
                  </td>
                </tr>
                {mappingPanelId === t.id && (
                  <tr key={`${t.id}-mappings`}>
                    <td colSpan={5}>
                      <MappingsPanel term={t} />
                    </td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
        {!terms.data?.length && <div className="empty">No glossary terms yet</div>}
      </div>
    </div>
  );
}

function GlossaryTermForm({
  term,
  users,
  onDone,
  onCancel,
}: {
  term?: BusinessGlossaryTerm;
  users: { id: string; display_name: string }[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<BusinessGlossaryTermInput>(
    term
      ? { name: term.name, definition: term.definition, owner_user_id: term.owner_user_id, synonyms: term.synonyms, data_classification: term.data_classification }
      : emptyInput(),
  );
  const [synonymsText, setSynonymsText] = useState((term?.synonyms ?? []).join(", "));
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () => (term ? api.updateGlossaryTerm(term.id, input) : api.createGlossaryTerm(input)),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this term"),
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
          <label>Data classification</label>
          <select value={input.data_classification} onChange={(e) => setInput({ ...input, data_classification: e.target.value as DataClassification })}>
            {(Object.keys(CLASSIFICATION_LABELS) as DataClassification[]).map((c) => (
              <option key={c} value={c}>{CLASSIFICATION_LABELS[c]}</option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Owner</label>
          <select value={input.owner_user_id ?? ""} onChange={(e) => setInput({ ...input, owner_user_id: e.target.value || null })}>
            <option value="">Unassigned</option>
            {users.map((u) => <option key={u.id} value={u.id}>{u.display_name}</option>)}
          </select>
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Definition</label>
          <textarea rows={3} value={input.definition} onChange={(e) => setInput({ ...input, definition: e.target.value })} />
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Synonyms (comma-separated)</label>
          <input
            value={synonymsText}
            onChange={(e) => {
              setSynonymsText(e.target.value);
              setInput({ ...input, synonyms: e.target.value.split(",").map((s) => s.trim()).filter(Boolean) });
            }}
          />
        </div>
      </div>
      <div className="actions">
        <button className="btn btn-primary" onClick={() => save.mutate()} disabled={save.isPending}>{term ? "Save" : "Create"}</button>
        <button className="btn" onClick={onCancel}>Cancel</button>
      </div>
    </div>
  );
}

function MappingsPanel({ term }: { term: BusinessGlossaryTerm }) {
  const queryClient = useQueryClient();
  const mappings = useQuery({ queryKey: ["semanticMappings", "term", term.id], queryFn: () => api.listSemanticMappingsForTerm(term.id) });
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTypes = [...CUSTOM_FIELD_ENTITY_TYPES, ...(customObjects.data?.map((o) => o.key) ?? [])];
  const labelFor = (t: string) => customObjects.data?.find((o) => o.key === t)?.plural_label ?? entityTypeLabel(t);

  const [entityType, setEntityType] = useState(entityTypes[0] ?? "Company");
  const [fieldKey, setFieldKey] = useState("");
  const [semanticRole, setSemanticRole] = useState<SemanticRole | "">("");
  const [error, setError] = useState<string | null>(null);

  const fields = useQuery({ queryKey: ["customFieldDefinitions", entityType], queryFn: () => api.listCustomFieldDefinitions(entityType, true) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["semanticMappings"] });
    queryClient.invalidateQueries({ queryKey: ["systemNode"] });
  }

  const create = useMutation({
    mutationFn: () => {
      const input: SemanticMappingInput = { entity_type: entityType, field_key: fieldKey || null, glossary_term_id: term.id, semantic_role: semanticRole || null };
      return api.createSemanticMapping(input);
    },
    onSuccess: () => { invalidate(); setFieldKey(""); setSemanticRole(""); },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not add this mapping"),
  });

  const remove = useMutation({ mutationFn: (id: string) => api.deleteSemanticMapping(id), onSuccess: invalidate });

  return (
    <div style={{ padding: "8px 0" }}>
      <p className="muted" style={{ fontSize: 12, margin: "0 0 8px" }}>
        Where "{term.name}" applies - an object, or a specific field on it.
      </p>
      {error && <div className="error-banner">{error}</div>}
      {(mappings.data ?? []).map((m) => (
        <div key={m.id} style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 4 }}>
          <span>{labelFor(m.entity_type)}{m.field_key ? `.${m.field_key}` : ""}{m.semantic_role ? ` — role: ${m.semantic_role}` : ""}</span>
          <button className="icon-btn" onClick={() => remove.mutate(m.id)}>Remove</button>
        </div>
      ))}
      {!mappings.data?.length && <p className="muted" style={{ fontSize: 12 }}>Not mapped to anything yet.</p>}

      <div style={{ display: "flex", gap: 8, marginTop: 8, flexWrap: "wrap" }}>
        <select value={entityType} onChange={(e) => { setEntityType(e.target.value); setFieldKey(""); }}>
          {entityTypes.map((t) => <option key={t} value={t}>{labelFor(t)}</option>)}
        </select>
        <select value={fieldKey} onChange={(e) => setFieldKey(e.target.value)}>
          <option value="">Whole object</option>
          {(fields.data ?? []).map((f) => <option key={f.id} value={f.key}>{f.label}</option>)}
        </select>
        <select value={semanticRole} onChange={(e) => setSemanticRole(e.target.value as SemanticRole | "")}>
          <option value="">No semantic role</option>
          {SEMANTIC_ROLES.map((r) => <option key={r} value={r}>{r}</option>)}
        </select>
        <button className="btn" onClick={() => create.mutate()} disabled={create.isPending}>+ Add mapping</button>
      </div>
    </div>
  );
}
