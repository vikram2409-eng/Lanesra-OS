import { Fragment, useState } from "react";
import type { ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import { useEffectivePage } from "../lib/useEffectivePage";
import { useIsAdmin } from "../lib/useCurrentUser";
import type { PageNode } from "../lib/types";
import { componentDef, isContainerType } from "../features/settings/pageComponentLibrary";
import { RelatedRecordsCard } from "./RelatedRecordsCard";

/**
 * Screen Builder 2.0 (issue #195, 5b): the live counterpart to
 * `PageBuilderAdmin`'s own schematic canvas - renders a *published* Page
 * against one real record, the way `LayoutDetailFields` already does for
 * the older `screen_layouts` system. Returns `null` whenever nothing is
 * published for this entity type (the common case today), so every
 * caller's own existing detail-view JSX - built before this feature
 * existed - is the fallback with zero changes, satisfying the issue's own
 * "every existing object's detail page must keep working" requirement by
 * construction rather than by a parity test alone (there's still one, in
 * `page_render_wiring.rs`, but the opt-in `null` return is the real
 * guarantee).
 *
 * `fields` is the exact same `Record<string, ReactNode>` map
 * `LayoutDetailFields` already takes - each key's pre-built `.form-field`
 * element, however that entity's detail page already renders a value
 * (money formatting, a linked record, whatever) - a `field` node just
 * places that same element, it never re-derives a value itself. The
 * `recordHeader`/`statusBadge`/`owner`/`recordNumber`/`onEdit`/
 * `onQuickAction` slots are the equivalent for the handful of component
 * types that need something *other* than a field value; each is
 * optional, and a node that needs a slot the caller didn't supply renders
 * a small explanatory placeholder instead of silently vanishing or
 * throwing - see `wire PageRenderer into record detail pages` for which
 * entities currently pass which slots.
 *
 * Honestly scoped for 5b, matching the KPI/Chart config fields' own
 * `pageComponentLibrary.ts` help text: a `kpi` node computes
 * `related_count` for real (an actual count of this record's related
 * records for the chosen relationship); `field_sum`/`field_count` and the
 * `chart` component render a clearly-labeled "not live yet" placeholder
 * rather than a fabricated number - both need a cross-entity field-value
 * aggregation endpoint this pass doesn't build. `agent_action` is real:
 * it resolves the named agent by name and sends it a message with this
 * record's context through the exact same `sendAgentMessage` call the
 * Admin Assistant chat already uses, showing the reply inline.
 */
export function PageRenderer({
  entityType,
  entityId,
  fields,
  recordHeader,
  statusBadge,
  owner,
  recordNumber,
  onEdit,
  onQuickAction,
  onOpenAdminTab,
}: {
  entityType: string;
  entityId: string;
  fields: Record<string, ReactNode>;
  recordHeader?: ReactNode;
  statusBadge?: ReactNode;
  owner?: ReactNode;
  recordNumber?: ReactNode;
  onEdit?: () => void;
  onQuickAction?: (targetStatus: string) => void;
  onOpenAdminTab?: (adminTab: string) => void;
}) {
  const effective = useEffectivePage(entityType);
  const isAdmin = useIsAdmin();
  const related = useQuery({
    queryKey: ["relatedRecords", entityType, entityId],
    queryFn: () => api.listRelatedRecords(entityType, entityId),
  });

  const page = effective.data?.page;
  if (!page || page.root.length === 0) return null;

  const relatedCounts: Record<string, number> = {};
  for (const r of related.data ?? []) {
    relatedCounts[r.relationship_key] = (relatedCounts[r.relationship_key] ?? 0) + 1;
  }

  const slots = { recordHeader, statusBadge, owner, recordNumber, onEdit, onQuickAction, fields, relatedCounts, entityType, entityId };

  return (
    <div>
      {isAdmin && onOpenAdminTab && (
        <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 8 }}>
          <AdminGearMenu onOpenAdminTab={onOpenAdminTab} />
        </div>
      )}
      <PageGrid nodes={page.root} slots={slots} />
    </div>
  );
}

/**
 * Admin Control Center Modernization (issue #197): a runtime surface's own
 * cross-link into the admin screen that configures it, mirroring
 * `CustomObjectsAdmin.tsx`'s cross-link buttons but pointed the other
 * direction - from a *published* Page back to the tools that built it,
 * for an admin viewing the live record the way an end user would.
 */
const ADMIN_GEAR_LINKS: { label: string; tab: string }[] = [
  { label: "Edit Page", tab: "pageBuilder" },
  { label: "Manage Fields", tab: "fields" },
  { label: "Business Rules", tab: "rules" },
  { label: "Workflows", tab: "workflow" },
  { label: "Access", tab: "accessRoles" },
];

function AdminGearMenu({ onOpenAdminTab }: { onOpenAdminTab: (adminTab: string) => void }) {
  const [open, setOpen] = useState(false);
  return (
    <div style={{ position: "relative" }}>
      <button className="btn" title="Admin: jump to this page's configuration" onClick={() => setOpen((v) => !v)}>
        ⚙
      </button>
      {open && (
        <div
          className="card"
          style={{ position: "absolute", right: 0, top: "calc(100% + 4px)", width: 180, zIndex: 20, boxShadow: "0 4px 16px rgba(0,0,0,0.15)" }}
          onMouseLeave={() => setOpen(false)}
        >
          {ADMIN_GEAR_LINKS.map((l) => (
            <button
              key={l.tab}
              className="link-button"
              style={{ display: "block", width: "100%", textAlign: "left", padding: "4px 0" }}
              onClick={() => {
                setOpen(false);
                onOpenAdminTab(l.tab);
              }}
            >
              {l.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

type Slots = {
  recordHeader?: ReactNode;
  statusBadge?: ReactNode;
  owner?: ReactNode;
  recordNumber?: ReactNode;
  onEdit?: () => void;
  onQuickAction?: (targetStatus: string) => void;
  fields: Record<string, ReactNode>;
  relatedCounts: Record<string, number>;
  entityType: string;
  entityId: string;
};

function PageGrid({ nodes, slots }: { nodes: PageNode[]; slots: Slots }) {
  return (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(12, 1fr)", gap: 12 }}>
      {nodes.map((n) => (
        <div key={n.id} style={{ gridColumn: `span ${Math.min(12, Math.max(1, n.layout.column_span))}` }}>
          <PageNodeView node={n} slots={slots} />
        </div>
      ))}
    </div>
  );
}

function Placeholder({ text }: { text: string }) {
  return <p style={{ fontSize: 11, color: "var(--text-muted)", fontStyle: "italic", margin: 0 }}>{text}</p>;
}

function PageNodeView({ node, slots }: { node: PageNode; slots: Slots }) {
  const def = componentDef(node.component_type);
  const cfg = node.config as Record<string, unknown>;

  if (isContainerType(node.component_type)) {
    switch (node.component_type) {
      case "section":
        return (
          <div className="card">
            {typeof cfg.title === "string" && cfg.title && <h4 style={{ marginTop: 0 }}>{cfg.title}</h4>}
            <PageGrid nodes={node.children} slots={slots} />
          </div>
        );
      case "field_group":
        return (
          <div className="form-grid full" style={{ maxWidth: "none" }}>
            {typeof cfg.title === "string" && cfg.title && (
              <div className="form-field full" style={{ marginBottom: -4 }}>
                <strong style={{ fontSize: 13 }}>{cfg.title}</strong>
              </div>
            )}
            <PageGrid nodes={node.children} slots={slots} />
          </div>
        );
      case "tabs":
        return <TabsNode node={node} slots={slots} />;
      case "sticky_panel":
        return (
          <div style={{ position: "sticky", [cfg.position === "bottom" ? "bottom" : "top"]: 0, zIndex: 1, background: "var(--bg-elevated, #fff)" }}>
            <PageGrid nodes={node.children} slots={slots} />
          </div>
        );
      default:
        // "grid" and "columns" are both a plain nested grid - the ratio/
        // column-count config only mattered for the *builder's* own
        // schematic preview, not for live rendering, where each child's
        // own column_span already says exactly how wide it is.
        return <PageGrid nodes={node.children} slots={slots} />;
    }
  }

  switch (node.component_type) {
    case "divider":
      return <hr style={{ border: "none", borderTop: "1px solid var(--line)" }} />;
    case "spacer":
      return <div style={{ height: cfg.height === "large" ? 32 : cfg.height === "small" ? 8 : 16 }} />;
    case "field": {
      const key = typeof cfg.field_key === "string" ? cfg.field_key : "";
      return <Fragment>{slots.fields[key] ?? <Placeholder text={key ? `Field "${key}" isn't available here.` : "No field chosen."} />}</Fragment>;
    }
    case "record_header":
      return slots.recordHeader ?? <Placeholder text="Record header isn't available on this page." />;
    case "status_badge":
      return slots.statusBadge ?? <Placeholder text="Status isn't available on this page." />;
    case "owner":
      return slots.owner ?? <Placeholder text="Owner isn't available on this page." />;
    case "record_number":
      return slots.recordNumber ?? <Placeholder text="Record number isn't available on this page." />;
    case "related_list":
    case "table": {
      const key = typeof cfg.relationship_key === "string" && cfg.relationship_key ? cfg.relationship_key : undefined;
      return <RelatedRecordsCard entityType={slots.entityType} entityId={slots.entityId} only={key ? [key] : undefined} />;
    }
    case "kpi": {
      const label = typeof cfg.metric_label === "string" && cfg.metric_label ? cfg.metric_label : def?.label ?? "KPI";
      if (cfg.source === "related_count" && typeof cfg.relationship_key === "string") {
        const count = slots.relatedCounts[cfg.relationship_key] ?? 0;
        return (
          <div className="card" style={{ textAlign: "center" }}>
            <div style={{ fontSize: 24, fontWeight: 700 }}>{count}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{label}</div>
          </div>
        );
      }
      return (
        <div className="card" style={{ textAlign: "center" }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "var(--text-muted)" }}>—</div>
          <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{label}</div>
          <Placeholder text="Field-based KPIs aren't wired to live data yet." />
        </div>
      );
    }
    case "chart": {
      const label = typeof cfg.metric_label === "string" && cfg.metric_label ? cfg.metric_label : "Chart";
      return (
        <div className="card">
          <div style={{ fontSize: 12, fontWeight: 600 }}>{label}</div>
          <Placeholder text="Chart rendering isn't wired to live data yet." />
        </div>
      );
    }
    case "button":
    case "quick_action": {
      const label = typeof cfg.label === "string" && cfg.label ? cfg.label : def?.label ?? "Action";
      if (node.component_type === "quick_action") {
        const target = typeof cfg.target_status === "string" ? cfg.target_status : "";
        return (
          <button className="btn" disabled={!slots.onQuickAction || !target} title={slots.onQuickAction ? undefined : "Not wired to a live action on this page yet"} onClick={() => target && slots.onQuickAction?.(target)}>
            {label}
          </button>
        );
      }
      const action = cfg.action;
      return (
        <button
          className={`btn${cfg.style === "primary" ? " btn-primary" : ""}`}
          disabled={action === "edit" ? !slots.onEdit : true}
          title={action === "navigate" ? "Navigation targets aren't wired yet" : undefined}
          onClick={() => {
            if (action === "edit") slots.onEdit?.();
          }}
        >
          {label}
        </button>
      );
    }
    case "command_bar": {
      const labels = typeof cfg.buttonLabels === "string" ? cfg.buttonLabels.split(",").map((s) => s.trim()).filter(Boolean) : [];
      return (
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          {labels.map((l) => (
            <button key={l} className="btn" disabled title="Not wired to a live action on this page yet">
              {l}
            </button>
          ))}
        </div>
      );
    }
    case "agent_action":
      return <AgentActionNode entityType={slots.entityType} entityId={slots.entityId} cfg={cfg} />;
    case "rich_text":
      return <p style={{ whiteSpace: "pre-wrap" }}>{typeof cfg.text === "string" ? cfg.text : ""}</p>;
    case "anchor_link": {
      const targetId = typeof cfg.target_section_id === "string" ? cfg.target_section_id : "";
      const label = typeof cfg.label === "string" && cfg.label ? cfg.label : "Jump to section";
      return (
        <a href={targetId ? `#${targetId}` : undefined} onClick={(e) => { if (!targetId) e.preventDefault(); }}>
          {label}
        </a>
      );
    }
    case "note": {
      const tone = cfg.tone === "warning" ? "badge-warning" : "badge-success";
      return (
        <div className={`card`}>
          <span className={`badge ${tone}`} style={{ marginBottom: 6 }}>
            {cfg.tone === "warning" ? "Warning" : "Note"}
          </span>
          <p style={{ margin: 0 }}>{typeof cfg.text === "string" ? cfg.text : ""}</p>
        </div>
      );
    }
    default:
      return null;
  }
}

function TabsNode({ node, slots }: { node: PageNode; slots: Slots }) {
  const labels = typeof (node.config as Record<string, unknown>).tabLabels === "string"
    ? ((node.config as Record<string, unknown>).tabLabels as string).split(",").map((s) => s.trim()).filter(Boolean)
    : [];
  const [active, setActive] = useState(0);
  const idx = Math.min(active, Math.max(0, node.children.length - 1));
  return (
    <div>
      {node.children.length > 1 && (
        <div className="tab-row">
          {node.children.map((_, i) => (
            <button key={i} type="button" className={`tab${i === idx ? " active" : ""}`} onClick={() => setActive(i)}>
              {labels[i] ?? `Tab ${i + 1}`}
            </button>
          ))}
        </div>
      )}
      {node.children[idx] && <PageGrid nodes={[node.children[idx]]} slots={slots} />}
    </div>
  );
}

function AgentActionNode({ entityType, entityId, cfg }: { entityType: string; entityId: string; cfg: Record<string, unknown> }) {
  const [reply, setReply] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const label = typeof cfg.label === "string" && cfg.label ? cfg.label : "Run agent";
  const agentName = typeof cfg.agent_name === "string" ? cfg.agent_name.trim() : "";

  async function run() {
    if (!agentName) return;
    setPending(true);
    setError(null);
    try {
      const agents = await api.listAiAgents(true);
      const agent = agents.find((a) => a.name.toLowerCase() === agentName.toLowerCase());
      if (!agent) {
        setError(`No active agent named "${agentName}"`);
        return;
      }
      const messages = await api.sendAgentMessage(agent.id, `Regarding this ${entityType} record (id ${entityId}): please help.`);
      const lastAssistant = [...messages].reverse().find((m) => m.role === "assistant");
      setReply(lastAssistant?.content ?? "(no reply)");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not reach the agent");
    } finally {
      setPending(false);
    }
  }

  return (
    <div>
      <button className="btn" disabled={!agentName || pending} title={agentName ? undefined : "No agent name set"} onClick={run}>
        {pending ? "Running…" : label}
      </button>
      {error && <p style={{ color: "var(--danger)", fontSize: 12 }}>{error}</p>}
      {reply && <p style={{ fontSize: 13, marginTop: 6, whiteSpace: "pre-wrap" }}>{reply}</p>}
    </div>
  );
}
