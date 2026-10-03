import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  ROLES,
  builtinFieldsFor,
  entityTypeLabel,
  type NodeLayout,
  type PageDefinition,
  type PageLayout,
  type PageNode,
} from "../../lib/types";
import {
  PAGE_COMPONENT_CATEGORIES,
  PAGE_COMPONENT_LIBRARY,
  componentDef,
  isContainerType,
  type ConfigFieldSchema,
  type PageComponentDef,
} from "./pageComponentLibrary";

function newId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `id-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

// NodeLayout only carries three span tiers (desktop/tablet/mobile) - the
// same "don't add a knob nobody asked to differ" restraint ScreenLayout's
// own 1-3 section-column range already uses - but the issue's own spec
// asks for four concrete preview widths (1440/1024/768/390). 1440 and
// 1024 both preview the desktop tier's spans (how they reflow in a
// narrower frame is still useful signal even with the same span numbers);
// 768 switches to the tablet tier and 390 to the mobile tier. A "Custom"
// width previews whichever tier was last selected.
type Breakpoint = "desktop" | "tablet" | "mobile";
type PreviewPreset = "w1440" | "w1024" | "w768" | "w390";

const PREVIEW_PRESETS: { key: PreviewPreset; width: number; tier: Breakpoint; label: string }[] = [
  { key: "w1440", width: 1440, tier: "desktop", label: "Desktop · 1440" },
  { key: "w1024", width: 1024, tier: "desktop", label: "Laptop · 1024" },
  { key: "w768", width: 768, tier: "tablet", label: "Tablet · 768" },
  { key: "w390", width: 390, tier: "mobile", label: "Mobile · 390" },
];

function effectiveSpan(layout: NodeLayout, breakpoint: Breakpoint): number {
  if (breakpoint === "desktop") return layout.column_span;
  if (breakpoint === "tablet") return layout.tablet_column_span ?? layout.column_span;
  return layout.mobile_column_span ?? layout.tablet_column_span ?? layout.column_span;
}

function newNode(componentType: string): PageNode {
  const def = componentDef(componentType);
  return {
    id: newId(),
    component_type: componentType,
    config: {},
    children: [],
    layout: { column_span: def?.defaultColumnSpan ?? 12, tablet_column_span: null, mobile_column_span: null, order: 0 },
  };
}

// ---- Pure tree helpers - every edit replaces the whole PageDefinition.root
// immutably, the same "mutate a local copy, save() pushes it" pattern
// ScreenLayoutsAdmin's LayoutEditor already uses for its tabs/sections tree.

function findNode(nodes: PageNode[], id: string): PageNode | null {
  for (const n of nodes) {
    if (n.id === id) return n;
    const found = findNode(n.children, id);
    if (found) return found;
  }
  return null;
}

function isDescendantOrSelf(node: PageNode, id: string): boolean {
  if (node.id === id) return true;
  return node.children.some((c) => isDescendantOrSelf(c, id));
}

function removeNode(nodes: PageNode[], id: string): { tree: PageNode[]; removed: PageNode | null } {
  let removed: PageNode | null = null;
  const filtered = nodes.filter((n) => {
    if (n.id === id) {
      removed = n;
      return false;
    }
    return true;
  });
  if (removed) return { tree: filtered, removed };
  const tree = filtered.map((n) => {
    const res = removeNode(n.children, id);
    if (res.removed) removed = res.removed;
    return res.removed !== null ? { ...n, children: res.tree } : n;
  });
  return { tree, removed };
}

function insertNode(nodes: PageNode[], parentId: string | null, index: number, node: PageNode): PageNode[] {
  if (parentId === null) {
    const next = [...nodes];
    next.splice(Math.max(0, Math.min(index, next.length)), 0, node);
    return next;
  }
  return nodes.map((n) => {
    if (n.id === parentId) {
      const next = [...n.children];
      next.splice(Math.max(0, Math.min(index, next.length)), 0, node);
      return { ...n, children: next };
    }
    if (n.children.length === 0) return n;
    return { ...n, children: insertNode(n.children, parentId, index, node) };
  });
}

function updateNode(nodes: PageNode[], id: string, updater: (n: PageNode) => PageNode): PageNode[] {
  return nodes.map((n) => {
    if (n.id === id) return updater(n);
    if (n.children.length === 0) return n;
    return { ...n, children: updateNode(n.children, id, updater) };
  });
}

type DragPayload = { kind: "new"; componentType: string } | { kind: "move"; nodeId: string };

/**
 * Screen Builder 2.0 (issue #195, 5a): a responsive page composer for
 * record detail pages, alongside (not replacing) Screen/App Builder's
 * create/edit form designer above - see page_layout.rs's own doc comment
 * for why this is a second system. Drag components from the palette onto
 * a 12-column grid canvas, nest them inside Section/Grid/Columns/Tabs/
 * Sticky Panel/Field Group containers, resize with the column-span
 * steppers in the Inspector (an exact 1-12 control, not pixel-drag, so
 * a span is always a deliberate integer choice), and preview the result
 * at four breakpoints. Composed pages aren't wired into the live record
 * detail view yet - that's 5b, same split the issue itself describes;
 * this screen is the builder only, same "canvas shows a schematic
 * representation of what's placed, not live record data" scoping every
 * other 2.0-era builder in this app (Workflow Studio, Agent Teams) uses
 * for its own node cards.
 */
export function PageBuilderAdmin() {
  const [entityType, setEntityType] = useState<string>("Company");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const queryClient = useQueryClient();

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

  const relationshipDefs = useQuery({ queryKey: ["relationshipDefinitions", "active"], queryFn: () => api.listRelationshipDefinitions(true) });
  const relatedLists: { key: string; label: string }[] = (relationshipDefs.data ?? [])
    .filter((d) => d.show_related_list && (d.source_entity_type === entityType || d.target_entity_type === entityType))
    .map((d) => ({ key: d.key, label: d.source_entity_type === entityType ? d.forward_label : d.reverse_label }));

  const layouts = useQuery({ queryKey: ["pageLayouts", entityType], queryFn: () => api.listPageLayouts(entityType) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["pageLayouts", entityType] });
  }

  const list = layouts.data ?? [];
  const selected = list.find((l) => l.id === selectedId) ?? list.find((l) => l.is_default) ?? list[0] ?? null;

  const create = useMutation({
    mutationFn: (name: string) => api.createPageLayout({ entity_type: entityType, name }),
    onSuccess: (created) => {
      invalidate();
      setCreating(false);
      setSelectedId(created.id);
    },
  });

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Page Builder</h3>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Compose a record detail page from a 12-column grid of Layout, Record, Data, Actions, Content, Navigation and
        Utility components - drag from the palette, nest inside a Section/Grid/Columns/Tabs/Sticky Panel/Field Group
        container, and resize with the column-span steppers. Anyone whose roles don't match a published page sees
        that object's Default.
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
            + New page
          </button>
        </div>
      )}

      {creating && (
        <NewPageForm
          onDone={(name) => create.mutate(name)}
          onCancel={() => setCreating(false)}
          error={create.error instanceof ApiError ? create.error.message : null}
          pending={create.isPending}
        />
      )}

      {selected && !creating && (
        <PageBuilderEditor
          key={selected.id}
          layout={selected}
          fields={fields}
          relatedLists={relatedLists}
          layoutCount={list.length}
          onChanged={invalidate}
          onDeleted={() => {
            invalidate();
            setSelectedId(null);
          }}
        />
      )}
    </div>
  );
}

function NewPageForm({
  onDone,
  onCancel,
  error,
  pending,
}: {
  onDone: (name: string) => void;
  onCancel: () => void;
  error: string | null;
  pending: boolean;
}) {
  const [name, setName] = useState("New page");
  return (
    <div className="card" style={{ background: "var(--bg-elevated)", marginBottom: 12 }}>
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Page name" style={{ flex: 1 }} />
        <button
          className="btn btn-primary"
          disabled={pending || !name.trim()}
          onClick={() => onDone(name.trim())}
        >
          Create
        </button>
        <button className="btn" onClick={onCancel}>
          Cancel
        </button>
      </div>
      {error && <p style={{ color: "var(--danger)", fontSize: 12 }}>{error}</p>}
    </div>
  );
}

function PageBuilderEditor({
  layout,
  fields,
  relatedLists,
  layoutCount,
  onChanged,
  onDeleted,
}: {
  layout: PageLayout;
  fields: { key: string; label: string }[];
  relatedLists: { key: string; label: string }[];
  layoutCount: number;
  onChanged: () => void;
  onDeleted: () => void;
}) {
  const [name, setName] = useState(layout.name);
  const [roles, setRoles] = useState<string[]>(layout.roles);
  const [page, setPage] = useState<PageDefinition>(layout.draft);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [previewPreset, setPreviewPreset] = useState<PreviewPreset>("w1440");
  const [customWidth, setCustomWidth] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [drag, setDrag] = useState<DragPayload | null>(null);

  // Same "every structural edit auto-saves the draft immediately" rule as
  // Screen/App Builder - see ScreenLayoutsAdmin's own comment on why.
  const update = useMutation({
    mutationFn: (next: { name: string; roles: string[]; draft: PageDefinition }) => api.updatePageLayout(layout.id, next),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this page"),
  });

  function save(nextPage: PageDefinition, nextName = name, nextRoles = roles) {
    setPage(nextPage);
    update.mutate({ name: nextName, roles: nextRoles, draft: nextPage });
  }

  const makeDefault = useMutation({
    mutationFn: () => api.makePageLayoutDefault(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not make this the default"),
  });
  const remove = useMutation({
    mutationFn: () => api.deletePageLayout(layout.id),
    onSuccess: onDeleted,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this page"),
  });
  const publish = useMutation({
    mutationFn: () => api.publishPageLayout(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not publish this page"),
  });
  const unpublish = useMutation({
    mutationFn: () => api.unpublishPageLayout(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not unpublish this page"),
  });
  const revert = useMutation({
    mutationFn: () => api.revertPageLayoutDraft(layout.id),
    onSuccess: (updated) => {
      setPage(updated.draft);
      setSelectedNodeId(null);
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not revert this draft"),
  });

  const hasPublished = layout.published !== null;
  const draftPublishedMatch = hasPublished && JSON.stringify(layout.published) === JSON.stringify(page);

  function handleDrop(parentId: string | null, index: number) {
    if (!drag) return;
    if (drag.kind === "new") {
      save({ root: insertNode(page.root, parentId, index, newNode(drag.componentType)) });
    } else {
      if (parentId !== null) {
        const dragged = findNode(page.root, drag.nodeId);
        if (dragged && isDescendantOrSelf(dragged, parentId)) {
          setDrag(null);
          return; // can't drop a container inside its own descendant
        }
      }
      const { tree: withoutDragged, removed } = removeNode(page.root, drag.nodeId);
      if (!removed) {
        setDrag(null);
        return;
      }
      save({ root: insertNode(withoutDragged, parentId, index, removed) });
    }
    setDrag(null);
  }

  function deleteNode(id: string) {
    const { tree } = removeNode(page.root, id);
    save({ root: tree });
    setSelectedNodeId((cur) => (cur === id ? null : cur));
  }

  function updateNodeConfig(id: string, key: string, value: unknown) {
    save({ root: updateNode(page.root, id, (n) => ({ ...n, config: { ...n.config, [key]: value } })) });
  }

  function updateNodeLayout(id: string, patch: Partial<NodeLayout>) {
    save({ root: updateNode(page.root, id, (n) => ({ ...n, layout: { ...n.layout, ...patch } })) });
  }

  const selectedNode = selectedNodeId ? findNode(page.root, selectedNodeId) : null;
  const activePreset = PREVIEW_PRESETS.find((p) => p.key === previewPreset) ?? PREVIEW_PRESETS[0];
  const breakpoint: Breakpoint = activePreset.tier;
  const previewWidth = customWidth ?? activePreset.width;

  return (
    <div>
      <div style={{ display: "flex", gap: 12, alignItems: "center", flexWrap: "wrap", marginBottom: 12 }}>
        <input value={name} onChange={(e) => setName(e.target.value)} onBlur={() => save(page, name, roles)} style={{ flex: "1 1 220px" }} />
        <span className={`badge${layout.is_default ? " badge-success" : ""}`}>{layout.is_default ? "Default page" : "Not default"}</span>
        {!layout.is_default && (
          <button className="btn" onClick={() => makeDefault.mutate()} disabled={makeDefault.isPending}>
            Make default
          </button>
        )}
        <button
          className="btn"
          onClick={() => {
            if (confirm(`Delete page '${layout.name}'?`)) remove.mutate();
          }}
          disabled={remove.isPending || layoutCount <= 1 || layout.is_default}
          title={layout.is_default ? "The default page can't be deleted" : undefined}
        >
          Delete
        </button>
      </div>

      <div style={{ marginBottom: 12 }}>
        <div style={{ fontWeight: 600, fontSize: 12, color: "var(--text-muted)", marginBottom: 6 }}>Visible to roles (none = everyone)</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
          {ROLES.map((r) => (
            <label key={r} style={{ display: "flex", gap: 6, alignItems: "center", fontSize: 13 }}>
              <input
                type="checkbox"
                checked={roles.includes(r)}
                onChange={(e) => {
                  const next = e.target.checked ? [...roles, r] : roles.filter((x) => x !== r);
                  setRoles(next);
                  save(page, name, next);
                }}
              />
              {r}
            </label>
          ))}
        </div>
      </div>

      <BreakpointToolbar preset={previewPreset} onChange={setPreviewPreset} customWidth={customWidth} onCustomWidthChange={setCustomWidth} />

      <div className="page-builder-layout">
        <Palette />
        <div className="page-builder-canvas-wrap">
          <div className="page-builder-canvas-frame" style={{ width: previewWidth, maxWidth: "100%" }}>
            <Canvas
              nodes={page.root}
              breakpoint={breakpoint}
              selectedId={selectedNodeId}
              onSelect={setSelectedNodeId}
              onDragStartNode={(id) => setDrag({ kind: "move", nodeId: id })}
              dragging={drag}
              onDrop={handleDrop}
              onDelete={deleteNode}
              depth={0}
            />
          </div>
        </div>
        <div className="page-builder-side">
          {selectedNode ? (
            <Inspector
              node={selectedNode}
              breakpoint={breakpoint}
              fields={fields}
              relatedLists={relatedLists}
              onConfigChange={(key, value) => updateNodeConfig(selectedNode.id, key, value)}
              onLayoutChange={(patch) => updateNodeLayout(selectedNode.id, patch)}
            />
          ) : (
            <LayersPanel nodes={page.root} selectedId={selectedNodeId} onSelect={setSelectedNodeId} onDelete={deleteNode} />
          )}
        </div>
      </div>

      {selectedNode && (
        <div style={{ marginTop: 12 }}>
          <LayersPanel nodes={page.root} selectedId={selectedNodeId} onSelect={setSelectedNodeId} onDelete={deleteNode} />
        </div>
      )}

      {error && <p style={{ color: "var(--danger)", fontSize: 12 }}>{error}</p>}

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

function BreakpointToolbar({
  preset,
  onChange,
  customWidth,
  onCustomWidthChange,
}: {
  preset: PreviewPreset;
  onChange: (p: PreviewPreset) => void;
  customWidth: number | null;
  onCustomWidthChange: (w: number | null) => void;
}) {
  return (
    <div style={{ display: "flex", gap: 8, alignItems: "center", margin: "12px 0", flexWrap: "wrap" }}>
      <span style={{ fontSize: 12, color: "var(--text-muted)", fontWeight: 600 }}>Preview:</span>
      {PREVIEW_PRESETS.map((p) => (
        <button
          key={p.key}
          className={`tab${preset === p.key && customWidth === null ? " active" : ""}`}
          onClick={() => {
            onChange(p.key);
            onCustomWidthChange(null);
          }}
        >
          {p.label}
        </button>
      ))}
      <label style={{ display: "flex", gap: 6, alignItems: "center", fontSize: 12 }}>
        Custom width
        <input
          type="number"
          min={320}
          max={1920}
          value={customWidth ?? ""}
          placeholder="px"
          style={{ width: 80 }}
          onChange={(e) => {
            const v = e.target.value ? Number(e.target.value) : null;
            onCustomWidthChange(v);
          }}
        />
      </label>
    </div>
  );
}

function Palette() {
  const [dragType, setDragType] = useState<string | null>(null);
  return (
    <div className="page-builder-palette">
      {PAGE_COMPONENT_CATEGORIES.map((cat) => {
        const items = PAGE_COMPONENT_LIBRARY.filter((c) => c.category === cat.key);
        if (items.length === 0) return null;
        return (
          <div key={cat.key} className="page-builder-palette-group">
            <div className="page-builder-palette-group-title">{cat.label}</div>
            {items.map((c) => (
              <div
                key={c.type}
                className={`page-builder-palette-item${dragType === c.type ? " dragging" : ""}`}
                draggable
                title={c.description}
                onDragStart={(e) => {
                  setDragType(c.type);
                  e.dataTransfer.effectAllowed = "copy";
                }}
                onDragEnd={() => setDragType(null)}
              >
                <span className="page-builder-palette-icon">{c.icon}</span>
                {c.label}
              </div>
            ))}
          </div>
        );
      })}
    </div>
  );
}

function DropZone({ onDrop, active }: { onDrop: () => void; active: boolean }) {
  const [over, setOver] = useState(false);
  if (!active) return null;
  return (
    <div
      className={`page-builder-dropzone${over ? " drop-over" : ""}`}
      onDragOver={(e) => {
        e.preventDefault();
        setOver(true);
      }}
      onDragLeave={() => setOver(false)}
      onDrop={(e) => {
        e.preventDefault();
        setOver(false);
        onDrop();
      }}
    />
  );
}

function Canvas({
  nodes,
  breakpoint,
  selectedId,
  onSelect,
  onDragStartNode,
  dragging,
  onDrop,
  onDelete,
  depth,
  parentId = null,
}: {
  nodes: PageNode[];
  breakpoint: Breakpoint;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onDragStartNode: (id: string) => void;
  dragging: DragPayload | null;
  onDrop: (parentId: string | null, index: number) => void;
  onDelete: (id: string) => void;
  depth: number;
  parentId?: string | null;
}) {
  return (
    <div className="page-builder-grid" style={{ display: "grid", gridTemplateColumns: "repeat(12, 1fr)", gap: 10 }}>
      <DropZone active={dragging !== null} onDrop={() => onDrop(parentId, 0)} />
      {nodes.map((node, i) => (
        <div key={node.id} style={{ gridColumn: `span ${Math.min(12, Math.max(1, effectiveSpan(node.layout, breakpoint)))}` }}>
          <NodeCard
            node={node}
            breakpoint={breakpoint}
            selectedId={selectedId}
            onSelect={onSelect}
            onDragStartNode={onDragStartNode}
            dragging={dragging}
            onDrop={onDrop}
            onDelete={onDelete}
            depth={depth}
          />
          <DropZone active={dragging !== null} onDrop={() => onDrop(parentId, i + 1)} />
        </div>
      ))}
      {nodes.length === 0 && (
        <div style={{ gridColumn: "span 12" }}>
          <DropZone active={dragging !== null} onDrop={() => onDrop(parentId, 0)} />
          {depth === 0 && <p className="page-builder-empty-hint">Drag a component from the palette to start building.</p>}
        </div>
      )}
    </div>
  );
}

function NodeCard({
  node,
  breakpoint,
  selectedId,
  onSelect,
  onDragStartNode,
  dragging,
  onDrop,
  onDelete,
  depth,
}: {
  node: PageNode;
  breakpoint: Breakpoint;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onDragStartNode: (id: string) => void;
  dragging: DragPayload | null;
  onDrop: (parentId: string | null, index: number) => void;
  onDelete: (id: string) => void;
  depth: number;
}) {
  const def = componentDef(node.component_type);
  const isContainer = isContainerType(node.component_type);
  const selected = selectedId === node.id;
  const isBeingDragged = dragging?.kind === "move" && dragging.nodeId === node.id;

  return (
    <div
      className={`page-builder-node${selected ? " selected" : ""}${isBeingDragged ? " dragging" : ""}`}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(node.id);
      }}
    >
      <div className="page-builder-node-head" draggable onDragStart={(e) => { e.stopPropagation(); onDragStartNode(node.id); }}>
        <span className="page-builder-drag-handle">⠿</span>
        <span className="page-builder-node-icon">{def?.icon ?? "◻"}</span>
        <span className="page-builder-node-label">{nodeSummaryLabel(node, def)}</span>
        <button
          className="page-builder-node-delete"
          onClick={(e) => {
            e.stopPropagation();
            onDelete(node.id);
          }}
          aria-label="Delete"
          title="Delete"
        >
          ✕
        </button>
      </div>
      {isContainer && (
        <div className="page-builder-node-body">
          <Canvas
            nodes={node.children}
            breakpoint={breakpoint}
            selectedId={selectedId}
            onSelect={onSelect}
            onDragStartNode={onDragStartNode}
            dragging={dragging}
            onDrop={onDrop}
            onDelete={onDelete}
            depth={depth + 1}
            parentId={node.id}
          />
        </div>
      )}
    </div>
  );
}

function nodeSummaryLabel(node: PageNode, def: PageComponentDef | undefined): string {
  const label = def?.label ?? node.component_type;
  const cfg = node.config as Record<string, unknown>;
  const detail = (cfg.title as string) || (cfg.label as string) || (cfg.field_key as string) || (cfg.metric_label as string) || (cfg.relationship_key as string);
  return detail ? `${label}: ${detail}` : label;
}

function Inspector({
  node,
  breakpoint,
  fields,
  relatedLists,
  onConfigChange,
  onLayoutChange,
}: {
  node: PageNode;
  breakpoint: Breakpoint;
  fields: { key: string; label: string }[];
  relatedLists: { key: string; label: string }[];
  onConfigChange: (key: string, value: unknown) => void;
  onLayoutChange: (patch: Partial<NodeLayout>) => void;
}) {
  const def = componentDef(node.component_type);
  const cfg = node.config as Record<string, unknown>;

  return (
    <div className="page-builder-inspector">
      <div className="page-builder-inspector-head">
        <span>{def?.icon}</span> {def?.label ?? node.component_type}
      </div>
      {def?.description && <p style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 0 }}>{def.description}</p>}

      {def?.configSchema.map((field) => (
        <ConfigFieldInput
          key={field.key}
          schema={field}
          value={cfg[field.key]}
          fields={field.type === "field_picker" ? fields : undefined}
          onChange={(v) => onConfigChange(field.key, v)}
        />
      ))}
      {node.component_type === "related_list" || node.component_type === "table" ? (
        relatedLists.length > 0 && (
          <div className="form-field">
            <label>Or pick a relationship</label>
            <select
              value={(cfg.relationship_key as string) ?? ""}
              onChange={(e) => onConfigChange("relationship_key", e.target.value)}
            >
              <option value="">Choose...</option>
              {relatedLists.map((r) => (
                <option key={r.key} value={r.key}>
                  {r.label}
                </option>
              ))}
            </select>
          </div>
        )
      ) : null}

      <div className="page-builder-inspector-divider" />
      <div className="page-builder-inspector-head">Layout</div>
      <ColumnSpanStepper
        label={`Column span (${breakpoint})`}
        value={effectiveSpan(node.layout, breakpoint)}
        onChange={(v) => {
          if (breakpoint === "desktop") onLayoutChange({ column_span: v });
          else if (breakpoint === "tablet") onLayoutChange({ tablet_column_span: v });
          else onLayoutChange({ mobile_column_span: v });
        }}
      />
      <p style={{ fontSize: 11, color: "var(--text-muted)" }}>
        Set per breakpoint - switch the preview above to Tablet or Mobile to override this component's span there.
        Unset breakpoints inherit the next size up.
      </p>
    </div>
  );
}

function ColumnSpanStepper({ label, value, onChange }: { label: string; value: number; onChange: (v: number) => void }) {
  return (
    <div className="form-field">
      <label>{label}</label>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <button className="btn" disabled={value <= 1} onClick={() => onChange(Math.max(1, value - 1))}>
          −
        </button>
        <span style={{ minWidth: 24, textAlign: "center" }}>{value}</span>
        <button className="btn" disabled={value >= 12} onClick={() => onChange(Math.min(12, value + 1))}>
          +
        </button>
        <span style={{ fontSize: 11, color: "var(--text-muted)" }}>of 12</span>
      </div>
    </div>
  );
}

function ConfigFieldInput({
  schema,
  value,
  fields,
  onChange,
}: {
  schema: ConfigFieldSchema;
  value: unknown;
  fields?: { key: string; label: string }[];
  onChange: (v: unknown) => void;
}) {
  if (schema.type === "field_picker") {
    return (
      <div className="form-field">
        <label>{schema.label}</label>
        <select value={(value as string) ?? ""} onChange={(e) => onChange(e.target.value)}>
          <option value="">Choose a field...</option>
          {(fields ?? []).map((f) => (
            <option key={f.key} value={f.key}>
              {f.label}
            </option>
          ))}
        </select>
        {schema.help && <p style={{ fontSize: 11, color: "var(--text-muted)", margin: "4px 0 0" }}>{schema.help}</p>}
      </div>
    );
  }
  if (schema.type === "select") {
    return (
      <div className="form-field">
        <label>{schema.label}</label>
        <select value={(value as string) ?? (schema.defaultValue as string) ?? ""} onChange={(e) => onChange(e.target.value)}>
          {(schema.options ?? []).map((o) => (
            <option key={o} value={o}>
              {o}
            </option>
          ))}
        </select>
      </div>
    );
  }
  if (schema.type === "textarea") {
    return (
      <div className="form-field">
        <label>{schema.label}</label>
        <textarea rows={3} value={(value as string) ?? ""} placeholder={schema.placeholder} onChange={(e) => onChange(e.target.value)} />
      </div>
    );
  }
  if (schema.type === "boolean") {
    return (
      <label style={{ display: "flex", gap: 6, alignItems: "center", fontSize: 13, margin: "8px 0" }}>
        <input type="checkbox" checked={Boolean(value)} onChange={(e) => onChange(e.target.checked)} />
        {schema.label}
      </label>
    );
  }
  if (schema.type === "number") {
    return (
      <div className="form-field">
        <label>{schema.label}</label>
        <input type="number" value={(value as number) ?? ""} onChange={(e) => onChange(e.target.value ? Number(e.target.value) : null)} />
      </div>
    );
  }
  return (
    <div className="form-field">
      <label>{schema.label}</label>
      <input
        type="text"
        value={(value as string) ?? ""}
        placeholder={schema.placeholder}
        onChange={(e) => onChange(e.target.value)}
      />
      {schema.help && <p style={{ fontSize: 11, color: "var(--text-muted)", margin: "4px 0 0" }}>{schema.help}</p>}
    </div>
  );
}

function LayersPanel({
  nodes,
  selectedId,
  onSelect,
  onDelete,
  depth = 0,
}: {
  nodes: PageNode[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
  depth?: number;
}) {
  if (depth === 0) {
    return (
      <div className="page-builder-layers">
        <div className="page-builder-inspector-head">Layers</div>
        {nodes.length === 0 ? (
          <p style={{ fontSize: 12, color: "var(--text-muted)" }}>Nothing placed yet.</p>
        ) : (
          <LayersPanel nodes={nodes} selectedId={selectedId} onSelect={onSelect} onDelete={onDelete} depth={1} />
        )}
      </div>
    );
  }
  return (
    <>
      {nodes.map((n) => {
        const def = componentDef(n.component_type);
        return (
          <div key={n.id}>
            <div
              className={`page-builder-layer-row${selectedId === n.id ? " selected" : ""}`}
              style={{ paddingLeft: depth * 14 }}
              onClick={() => onSelect(n.id)}
            >
              <span>{def?.icon}</span>
              <span className="page-builder-layer-label">{nodeSummaryLabel(n, def)}</span>
              <button
                className="page-builder-node-delete"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(n.id);
                }}
                aria-label="Delete"
              >
                ✕
              </button>
            </div>
            {n.children.length > 0 && (
              <LayersPanel nodes={n.children} selectedId={selectedId} onSelect={onSelect} onDelete={onDelete} depth={depth + 1} />
            )}
          </div>
        );
      })}
    </>
  );
}
