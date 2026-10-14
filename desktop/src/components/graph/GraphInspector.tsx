/**
 * PR 3 — Shared graph inspector panel.
 *
 * Presentational inspector used by the graph surfaces (Object 360 today;
 * the Ontology Graph admin inspector can adopt it when behavior matches).
 * Shows a type-badge header, action buttons, and grouped relationship
 * sections — the same visual contract as the demo's Object 360 inspector.
 */
import { memo, type ReactNode } from "react";

import { cn } from "../ui/cn";
import { Button } from "../ui/button";

export interface InspectorItem {
  key: string;
  label: string;
  /** Colored dot shown before the label. */
  dotColor?: string;
  /** When present the item renders as a button (inspector bullet). */
  onClick?: () => void;
  /** Trailing content (e.g. an arrow). */
  right?: ReactNode;
}

export interface InspectorGroup {
  key: string;
  label: string;
  items: InspectorItem[];
}

export interface InspectorAction {
  label: string;
  onClick: () => void;
  variant?: "default" | "secondary" | "outline" | "ghost";
  disabled?: boolean;
}

export interface GraphInspectorProps {
  title: string;
  subtitle?: string;
  /** Badge dot color (node type color). */
  badgeColor?: string;
  actions?: InspectorAction[];
  groups: InspectorGroup[];
  /** Message when there are no groups. */
  emptyMessage?: string;
  footer?: ReactNode;
  className?: string;
  /** Extra props spread onto the header (e.g. drag handlers). */
  headerProps?: React.HTMLAttributes<HTMLDivElement>;
}

function Dot({ color }: { color: string }) {
  return (
    <span
      aria-hidden="true"
      style={{ display: "inline-block", width: 10, height: 10, borderRadius: "50%", background: color, flexShrink: 0 }}
    />
  );
}

export const GraphInspector = memo(function GraphInspector(props: GraphInspectorProps) {
  const { title, subtitle, badgeColor, actions, groups, emptyMessage, footer, className, headerProps } = props;

  return (
    <div className={cn("graph-inspector", className)}>
      <div className="ontology-inspector-head" {...headerProps}>
        {badgeColor && <Dot color={badgeColor} />}
        <div>
          <strong>{title}</strong>
          {subtitle && <div className="ontology-inspector-subtitle">{subtitle}</div>}
        </div>
      </div>

      {actions && actions.length > 0 && (
        <div className="ontology-inspector-actions">
          {actions.map((a) => (
            <Button
              key={a.label}
              variant={a.variant ?? "secondary"}
              size="sm"
              onClick={a.onClick}
              disabled={a.disabled}
            >
              {a.label}
            </Button>
          ))}
        </div>
      )}

      <div className="ontology-inspector-groups">
        {groups.map((g) => (
          <div key={g.key} className="ontology-inspector-group">
            <div className="ontology-inspector-group-label">{g.label}</div>
            {g.items.map((it) =>
              it.onClick ? (
                <button key={it.key} type="button" className="ontology-inspector-bullet" onClick={it.onClick}>
                  {it.dotColor && <Dot color={it.dotColor} />}
                  <span>{it.label}</span>
                  {it.right}
                </button>
              ) : (
                <div key={it.key} className="ontology-inspector-row" style={{ fontSize: 13 }}>
                  {it.dotColor && <Dot color={it.dotColor} />}
                  <span>{it.label}</span>
                  {it.right}
                </div>
              ),
            )}
          </div>
        ))}
        {groups.length === 0 && emptyMessage && (
          <p style={{ fontSize: 12, color: "var(--text-muted)" }}>{emptyMessage}</p>
        )}
      </div>

      {footer}
    </div>
  );
});
