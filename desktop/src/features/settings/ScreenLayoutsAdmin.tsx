import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  ROLES,
  builtinFieldsFor,
  entityTypeLabel,
  type LayoutSection,
  type LayoutTab,
  type LayoutTabs,
  type ScreenLayout,
} from "../../lib/types";

const SECTION_COLUMN_CHOICES = [1, 2, 3] as const;

function newId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `id-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function emptySection(): LayoutSection {
  return { id: newId(), title: "Section", columns: 2, fields: [] };
}
function emptyTab(): LayoutTab {
  return { id: newId(), title: "New tab", sections: [emptySection()] };
}

/**
 * Screen/App Builder Phase 1-2: lets an Administrator design the
 * create/edit form for any built-in or custom object - named layouts
 * made of tabs of field sections, assigned to roles. Each section lays
 * its fields out in its own 1-3 column grid (Phase 2), with any field
 * optionally spanning the section's full width. Edits are all against a
 * layout's draft; nothing changes on a live form until Publish (see
 * screen_layout_service's doc comments in the Rust core for the full
 * draft/published/role-resolution model - this screen is a thin editor
 * over it). Mirrors the online demo's Admin > Screen layouts builder,
 * built the same week - see that commit for the shared design.
 *
 * Every entity always has at least one layout: the Default, auto-created
 * server-side the first time this screen (or resolve_effective_layout)
 * looks at an entity type with none yet. Exactly one layout per entity is
 * ever the default, enforced by a DB-level unique index - it's the
 * fallback for any signed-in user whose roles no other published layout
 * claims.
 */
export function ScreenLayoutsAdmin() {
  const [entityType, setEntityType] = useState<string>("Company");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  // Same entity-tab set as Custom fields: the 9 built-in entities plus
  // every active custom object, each carrying its own layouts.
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTabs: { key: string; label: string }[] = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((t) => ({ key: t as string, label: entityTypeLabel(t) })),
    ...(customObjects.data ?? []).map((o) => ({ key: o.key, label: o.plural_label })),
  ];

  const customFields = useQuery({
    queryKey: ["customFieldDefinitions", entityType, "active"],
    queryFn: () => api.listCustomFieldDefinitions(entityType, true),
  });
  const fields: { key: string; label: string }[] = [
    ...builtinFieldsFor(entityType).map((f) => ({ key: f.key, label: f.label })),
    ...(customFields.data ?? []).map((f) => ({ key: f.key, label: f.label })),
  ];
  const fieldLabel = (key: string) => fields.find((f) => f.key === key)?.label ?? key;

  const layouts = useQuery({ queryKey: ["screenLayouts", entityType], queryFn: () => api.listScreenLayouts(entityType) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["screenLayouts", entityType] });
  }

  const list = layouts.data ?? [];
  const selected = list.find((l) => l.id === selectedId) ?? list.find((l) => l.is_default) ?? list[0] ?? null;
  const previewLayout = list.find((l) => l.id === previewId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Screen layouts</h3>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Design the create/edit form for any object - tabs of field sections, assigned to roles. Anyone whose roles
        don't match a published layout sees that object's Default.
      </p>

      <div className="tab-row">
        {entityTabs.map((t) => (
          <button
            key={t.key}
            className={`tab${entityType === t.key ? " active" : ""}`}
            onClick={() => {
              setEntityType(t.key);
              setSelectedId(null);
              setCreating(false);
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {layouts.isLoading && <p>Loading...</p>}

      {list.length > 0 && (
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", margin: "12px 0" }}>
          {list.map((l) => (
            <button
              key={l.id}
              className={`tab${selected?.id === l.id ? " active" : ""}`}
              onClick={() => {
                setSelectedId(l.id);
                setCreating(false);
              }}
            >
              {l.name}
              {l.is_default ? " · Default" : ""}
            </button>
          ))}
          <button className="btn" onClick={() => setCreating((v) => !v)}>
            + New layout
          </button>
        </div>
      )}

      {creating && (
        <NewLayoutForm
          entityType={entityType}
          initialFields={fields.map((f) => f.key)}
          onDone={(created) => {
            invalidate();
            setCreating(false);
            setSelectedId(created.id);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {selected && !creating && (
        <LayoutEditor
          key={selected.id}
          layout={selected}
          fields={fields}
          fieldLabel={fieldLabel}
          layoutCount={list.length}
          onChanged={invalidate}
          onDeleted={() => {
            invalidate();
            setSelectedId(null);
          }}
          onPreview={() => setPreviewId(selected.id)}
        />
      )}

      {previewLayout && <LayoutPreviewModal layout={previewLayout} fieldLabel={fieldLabel} onClose={() => setPreviewId(null)} />}
    </div>
  );
}

function NewLayoutForm({
  entityType,
  initialFields,
  onDone,
  onCancel,
}: {
  entityType: string;
  initialFields: string[];
  onDone: (created: ScreenLayout) => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createScreenLayout({ entity_type: entityType, name, initial_fields: initialFields }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this layout"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          create.mutate();
        }}
      >
        <div className="form-field full">
          <label>Layout name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Sales layout" required autoFocus />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create layout
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function LayoutEditor({
  layout,
  fields,
  fieldLabel,
  layoutCount,
  onChanged,
  onDeleted,
  onPreview,
}: {
  layout: ScreenLayout;
  fields: { key: string; label: string }[];
  fieldLabel: (key: string) => string;
  layoutCount: number;
  onChanged: () => void;
  onDeleted: () => void;
  onPreview: () => void;
}) {
  const [name, setName] = useState(layout.name);
  const [roles, setRoles] = useState<string[]>(layout.roles);
  const [tabs, setTabs] = useState<LayoutTabs>(layout.draft);
  const [activeTabIdx, setActiveTabIdx] = useState(0);
  const [error, setError] = useState<string | null>(null);

  // Every structural edit (add/rename/delete tab or section, place/move/
  // remove a field) saves immediately - there's no separate "save" step
  // for the draft, only for Publish. That keeps this screen from ever
  // holding admin work that silently vanishes on navigating away, while
  // the draft/published split still protects the live form from
  // half-finished changes.
  const update = useMutation({
    mutationFn: (next: { name: string; roles: string[]; draft: LayoutTabs }) => api.updateScreenLayout(layout.id, next),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this layout"),
  });

  function save(nextTabs: LayoutTabs, nextName = name, nextRoles = roles) {
    setTabs(nextTabs);
    update.mutate({ name: nextName, roles: nextRoles, draft: nextTabs });
  }

  const makeDefault = useMutation({
    mutationFn: () => api.makeScreenLayoutDefault(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not make this the default"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteScreenLayout(layout.id),
    onSuccess: onDeleted,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this layout"),
  });

  const publish = useMutation({
    mutationFn: () => api.publishScreenLayout(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not publish this layout"),
  });

  const unpublish = useMutation({
    mutationFn: () => api.unpublishScreenLayout(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not unpublish this layout"),
  });

  const revert = useMutation({
    mutationFn: () => api.revertScreenLayoutDraft(layout.id),
    onSuccess: (updated) => {
      setTabs(updated.draft);
      setActiveTabIdx(0);
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not revert this draft"),
  });

  const activeTabIdxClamped = Math.min(activeTabIdx, tabs.tabs.length - 1);
  const activeTab = tabs.tabs[activeTabIdxClamped];

  function addTab() {
    const next: LayoutTabs = { tabs: [...tabs.tabs, emptyTab()] };
    save(next);
    setActiveTabIdx(next.tabs.length - 1);
  }

  function renameTab(idx: number, title: string) {
    save({ tabs: tabs.tabs.map((t, i) => (i === idx ? { ...t, title } : t)) });
  }

  function deleteTab(idx: number) {
    if (tabs.tabs.length <= 1) return;
    const next: LayoutTabs = { tabs: tabs.tabs.filter((_, i) => i !== idx) };
    save(next);
    setActiveTabIdx((cur) => Math.max(0, Math.min(cur, next.tabs.length - 1)));
  }

  function addSection(tabIdx: number) {
    save({ tabs: tabs.tabs.map((t, i) => (i === tabIdx ? { ...t, sections: [...t.sections, emptySection()] } : t)) });
  }

  function setSectionColumns(tabIdx: number, sectionIdx: number, columns: number) {
    save({
      tabs: tabs.tabs.map((t, i) =>
        i !== tabIdx ? t : { ...t, sections: t.sections.map((s, si) => (si === sectionIdx ? { ...s, columns } : s)) },
      ),
    });
  }

  function renameSection(tabIdx: number, sectionIdx: number, title: string) {
    save({
      tabs: tabs.tabs.map((t, i) =>
        i !== tabIdx ? t : { ...t, sections: t.sections.map((s, si) => (si === sectionIdx ? { ...s, title } : s)) },
      ),
    });
  }

  function deleteSection(tabIdx: number, sectionIdx: number) {
    const tab = tabs.tabs[tabIdx];
    if (!tab || tab.sections.length <= 1) return;
    save({
      tabs: tabs.tabs.map((t, i) => (i !== tabIdx ? t : { ...t, sections: t.sections.filter((_, si) => si !== sectionIdx) })),
    });
  }

  // A field only ever lives in one place across the whole layout, so
  // placing it strips it from every section on every tab first - picking
  // a field that's already elsewhere in the layout moves it here instead
  // of duplicating it (including across tabs). New placements default to
  // a single column - an admin who wants it full-width flips that after.
  function addField(tabIdx: number, sectionIdx: number, key: string) {
    const stripped = tabs.tabs.map((t) => ({
      ...t,
      sections: t.sections.map((s) => ({ ...s, fields: s.fields.filter((f) => f.key !== key) })),
    }));
    stripped[tabIdx] = {
      ...stripped[tabIdx],
      sections: stripped[tabIdx].sections.map((s, si) =>
        si === sectionIdx ? { ...s, fields: [...s.fields, { key, full_width: false }] } : s,
      ),
    };
    save({ tabs: stripped });
  }

  function removeField(tabIdx: number, sectionIdx: number, key: string) {
    save({
      tabs: tabs.tabs.map((t, i) =>
        i !== tabIdx
          ? t
          : {
              ...t,
              sections: t.sections.map((s, si) => (si === sectionIdx ? { ...s, fields: s.fields.filter((f) => f.key !== key) } : s)),
            },
      ),
    });
  }

  function moveField(tabIdx: number, sectionIdx: number, key: string, direction: -1 | 1) {
    save({
      tabs: tabs.tabs.map((t, i) => {
        if (i !== tabIdx) return t;
        return {
          ...t,
          sections: t.sections.map((s, si) => {
            if (si !== sectionIdx) return s;
            const idx = s.fields.findIndex((f) => f.key === key);
            const swapWith = idx + direction;
            if (idx < 0 || swapWith < 0 || swapWith >= s.fields.length) return s;
            const nextFields = [...s.fields];
            [nextFields[idx], nextFields[swapWith]] = [nextFields[swapWith], nextFields[idx]];
            return { ...s, fields: nextFields };
          }),
        };
      }),
    });
  }

  function toggleFieldFullWidth(tabIdx: number, sectionIdx: number, key: string) {
    save({
      tabs: tabs.tabs.map((t, i) =>
        i !== tabIdx
          ? t
          : {
              ...t,
              sections: t.sections.map((s, si) =>
                si !== sectionIdx
                  ? s
                  : { ...s, fields: s.fields.map((f) => (f.key === key ? { ...f, full_width: !f.full_width } : f)) },
              ),
            },
      ),
    });
  }

  const placedKeys = new Set(tabs.tabs.flatMap((t) => t.sections.flatMap((s) => s.fields.map((f) => f.key))));
  // "Available" for a section's picker means "not already on this tab" -
  // a field placed on another tab still shows up here (labeled as such),
  // so picking it is how you move a field between tabs.
  const availableForActiveTab = activeTab
    ? fields.filter((f) => !activeTab.sections.some((s) => s.fields.some((sf) => sf.key === f.key)))
    : [];
  const unplacedCount = fields.length - placedKeys.size;

  const hasPublished = layout.published !== null;
  const draftPublishedMatch = hasPublished && JSON.stringify(tabs) === JSON.stringify(layout.published);

  return (
    <div>
      {error && <div className="error-banner">{error}</div>}

      <div className="card" style={{ background: "var(--surface-2, transparent)", marginBottom: 16 }}>
        <div className="form-grid">
          <div className="form-field">
            <label>Layout name</label>
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={() => {
                if (name.trim() && name !== layout.name) save(tabs, name, roles);
              }}
            />
          </div>
          <div className="form-field">
            <label title="Whichever roles a signed-in user has, the first published layout that claims one of them wins - the Default is the fallback for everyone else.">
              Roles
            </label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 12 }}>
              {ROLES.map((role) => (
                <label key={role} style={{ display: "flex", gap: 6, alignItems: "center", fontSize: 13 }}>
                  <input
                    type="checkbox"
                    checked={roles.includes(role)}
                    onChange={(e) => {
                      const nextRoles = e.target.checked ? [...roles, role] : roles.filter((r) => r !== role);
                      setRoles(nextRoles);
                      save(tabs, name, nextRoles);
                    }}
                  />
                  {role}
                </label>
              ))}
            </div>
          </div>
          <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            <span className={`badge${layout.is_default ? " badge-success" : ""}`}>
              {layout.is_default ? "Default layout" : "Not default"}
            </span>
            <span className={`badge${hasPublished ? " badge-success" : ""}`}>{hasPublished ? "Published" : "Never published"}</span>
            {hasPublished && !draftPublishedMatch && <span className="badge badge-warning">Unpublished changes</span>}
            <div style={{ flex: 1 }} />
            {!layout.is_default && (
              <button className="btn" onClick={() => makeDefault.mutate()} disabled={makeDefault.isPending}>
                Make default
              </button>
            )}
            <button className="btn" onClick={onPreview}>
              Preview
            </button>
            <button
              className="btn btn-danger"
              onClick={() => {
                if (confirm(`Delete layout '${layout.name}'?`)) remove.mutate();
              }}
              disabled={remove.isPending || layoutCount <= 1 || layout.is_default}
              title={
                layout.is_default
                  ? "The default layout can't be deleted"
                  : layoutCount <= 1
                    ? "An object needs at least one layout"
                    : undefined
              }
            >
              Delete layout
            </button>
          </div>
        </div>
      </div>

      <div className="tab-row">
        {tabs.tabs.map((t, i) => (
          <button key={t.id} className={`tab${i === activeTabIdxClamped ? " active" : ""}`} onClick={() => setActiveTabIdx(i)}>
            {t.title}
          </button>
        ))}
        <button className="btn" onClick={addTab}>
          + Add tab
        </button>
      </div>

      {activeTab && (
        <div className="card" style={{ background: "var(--surface-2, transparent)" }}>
          <div className="form-grid" style={{ marginBottom: 8 }}>
            <div className="form-field">
              <label>Tab title</label>
              <input value={activeTab.title} onChange={(e) => renameTab(activeTabIdxClamped, e.target.value)} />
            </div>
            <div className="form-field" style={{ justifyContent: "flex-end", flexDirection: "row", gap: 8 }}>
              <button className="btn" onClick={() => deleteTab(activeTabIdxClamped)} disabled={tabs.tabs.length <= 1}>
                Delete this tab
              </button>
            </div>
          </div>

          {activeTab.sections.map((section, si) => (
            <div key={section.id} className="card" style={{ marginBottom: 12 }}>
              <div className="toolbar">
                <input
                  value={section.title}
                  onChange={(e) => renameSection(activeTabIdxClamped, si, e.target.value)}
                  style={{ fontWeight: 600, border: "none", background: "transparent", fontSize: 14, padding: 0 }}
                />
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 12, color: "var(--text-muted)" }}>
                    Columns
                    {SECTION_COLUMN_CHOICES.map((n) => (
                      <button
                        key={n}
                        type="button"
                        className={`tab${section.columns === n ? " active" : ""}`}
                        style={{ padding: "2px 10px" }}
                        onClick={() => setSectionColumns(activeTabIdxClamped, si, n)}
                      >
                        {n}
                      </button>
                    ))}
                  </div>
                  <button className="btn" onClick={() => deleteSection(activeTabIdxClamped, si)} disabled={activeTab.sections.length <= 1}>
                    Remove section
                  </button>
                </div>
              </div>
              <div style={{ display: "flex", flexWrap: "wrap", gap: 8, margin: "8px 0" }}>
                {section.fields.length === 0 && <span className="empty-state">No fields yet.</span>}
                {section.fields.map((f, fi) => (
                  <span key={f.key} className="badge" style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
                    {fieldLabel(f.key)}
                    <button
                      className="link-button"
                      onClick={() => toggleFieldFullWidth(activeTabIdxClamped, si, f.key)}
                      title={f.full_width ? "Full width - click to shrink to one column" : "One column - click to span the full section"}
                      disabled={section.columns === 1}
                    >
                      {f.full_width ? "⭤ full" : "⭤ 1 col"}
                    </button>
                    <button
                      className="link-button"
                      onClick={() => moveField(activeTabIdxClamped, si, f.key, -1)}
                      disabled={fi === 0}
                      title="Move earlier"
                    >
                      ↑
                    </button>
                    <button
                      className="link-button"
                      onClick={() => moveField(activeTabIdxClamped, si, f.key, 1)}
                      disabled={fi === section.fields.length - 1}
                      title="Move later"
                    >
                      ↓
                    </button>
                    <button className="link-button" onClick={() => removeField(activeTabIdxClamped, si, f.key)} title="Remove from layout">
                      ×
                    </button>
                  </span>
                ))}
              </div>
              {availableForActiveTab.length > 0 && (
                <select
                  value=""
                  onChange={(e) => {
                    if (e.target.value) addField(activeTabIdxClamped, si, e.target.value);
                  }}
                >
                  <option value="">+ Add field to this section...</option>
                  {availableForActiveTab.map((f) => (
                    <option key={f.key} value={f.key}>
                      {f.label}
                      {placedKeys.has(f.key) ? " (currently on another tab)" : ""}
                    </option>
                  ))}
                </select>
              )}
            </div>
          ))}

          <button className="btn" onClick={() => addSection(activeTabIdxClamped)}>
            + Add section
          </button>
        </div>
      )}

      {unplacedCount > 0 && (
        <p style={{ color: "var(--text-muted)", fontSize: 12 }}>
          {unplacedCount} field(s) not on this layout yet - add them from the field picker on whichever tab they belong on.
        </p>
      )}

      <div className="toolbar" style={{ marginTop: 16 }}>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" onClick={() => publish.mutate()} disabled={publish.isPending || draftPublishedMatch}>
            Publish
          </button>
          {hasPublished && (
            <button className="btn" onClick={() => unpublish.mutate()} disabled={unpublish.isPending}>
              Unpublish
            </button>
          )}
          {hasPublished && !draftPublishedMatch && (
            <button className="btn" onClick={() => revert.mutate()} disabled={revert.isPending}>
              Revert draft to published
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function LayoutPreviewModal({
  layout,
  fieldLabel,
  onClose,
}: {
  layout: ScreenLayout;
  fieldLabel: (key: string) => string;
  onClose: () => void;
}) {
  const [tabIdx, setTabIdx] = useState(0);
  const tabs = layout.draft.tabs;
  const activeIdx = Math.min(tabIdx, tabs.length - 1);
  const active = tabs[activeIdx];

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div className="card" style={{ width: 520, maxHeight: "80vh", overflowY: "auto" }} onClick={(e) => e.stopPropagation()}>
        <div className="toolbar">
          <h3 style={{ margin: 0 }}>Preview - {layout.name}</h3>
          <button className="btn" onClick={onClose}>
            Close
          </button>
        </div>
        <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: 0 }}>
          Shows this layout's draft, as it will appear once published. Fields are read-only here.
        </p>
        {tabs.length > 1 && (
          <div className="tab-row">
            {tabs.map((t, i) => (
              <button key={t.id} className={`tab${i === activeIdx ? " active" : ""}`} onClick={() => setTabIdx(i)}>
                {t.title}
              </button>
            ))}
          </div>
        )}
        {active &&
          active.sections.map((s) => (
            <div key={s.id} style={{ marginBottom: 16 }}>
              <h4 style={{ marginBottom: 8 }}>{s.title}</h4>
              {s.fields.length === 0 && <span className="empty-state">No fields in this section.</span>}
              <div className="form-grid" style={{ gridTemplateColumns: `repeat(${s.columns}, 1fr)` }}>
                {s.fields.map((f) => (
                  <div className={`form-field${f.full_width ? " full" : ""}`} key={f.key}>
                    <label>{fieldLabel(f.key)}</label>
                    <input disabled placeholder={fieldLabel(f.key)} />
                  </div>
                ))}
              </div>
            </div>
          ))}
      </div>
    </div>
  );
}
