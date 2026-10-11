import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { ObjectTypeSummary } from "../../lib/types";

/**
 * Next-Gen program, Ontology Layer epic (issues #340-#345): the
 * Object Type Registry (#341) and Link Types (#342) - a single read-only
 * view over every Object Type (built-in and Custom Object alike) and the
 * Link Types it participates in. Not a second definition store: picking
 * an object type here just surfaces what Fields/Relationships/Business
 * Rules/Workflows already have for it, one click away via "Open in...".
 * The fuller Ontology Manager (Action Types, Agent Bindings) lands as a
 * later phase of this same epic (#343-#345) and will extend this screen
 * rather than replace it.
 */
export function ObjectTypesAdmin({ onOpenTab }: { onOpenTab?: (adminTab: string) => void }) {
  const [selected, setSelected] = useState<string | null>(null);

  const objectTypes = useQuery({ queryKey: ["ontologyObjectTypes"], queryFn: () => api.listObjectTypes() });
  const detail = useQuery({
    queryKey: ["ontologyObjectTypeDetail", selected],
    queryFn: () => api.getObjectTypeDetail(selected!),
    enabled: !!selected,
  });

  return (
    <div>
      <h3>Object Types</h3>
      <p style={{ color: "var(--text-muted)", maxWidth: 720 }}>
        Every object in the system - built-in and Custom Object alike - as one typed registry: its identity (label, icon,
        color), its plain-English description where one is mapped in the Business Glossary, and the Link Types it
        participates in.
      </p>

      <div style={{ display: "flex", gap: 20, alignItems: "flex-start" }}>
        <div className="card" style={{ flex: "0 0 320px", padding: 0 }}>
          {objectTypes.isLoading && <p style={{ padding: 16 }}>Loading...</p>}
          <table className="data-table">
            <tbody>
              {(objectTypes.data ?? []).map((o: ObjectTypeSummary) => (
                <tr
                  key={o.key}
                  style={{ cursor: "pointer", background: selected === o.key ? "rgba(127, 127, 127, 0.06)" : undefined }}
                  onClick={() => setSelected(o.key)}
                >
                  <td style={{ width: 28, textAlign: "center" }}>
                    <span style={{ color: `var(--chart-${o.color_index})` }}>{o.icon}</span>
                  </td>
                  <td>
                    <strong>{o.label_plural}</strong>
                    {o.is_custom && <span className="badge" style={{ marginLeft: 8 }}>Custom</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="card" style={{ flex: 1, minWidth: 0 }}>
          {!selected && <p style={{ color: "var(--text-muted)" }}>Select an object type to see its details.</p>}
          {selected && detail.isLoading && <p>Loading...</p>}
          {selected && detail.data && (
            <>
              <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 4 }}>
                <span style={{ fontSize: 24, color: `var(--chart-${detail.data.object_type.color_index})` }}>{detail.data.object_type.icon}</span>
                <h4 style={{ margin: 0 }}>{detail.data.object_type.label_singular}</h4>
                {detail.data.object_type.is_custom && <span className="badge">Custom Object</span>}
              </div>
              <p style={{ color: "var(--text-muted)" }}>
                {detail.data.object_type.description ?? "No Business Glossary mapping yet for this object."}
              </p>

              {onOpenTab && (
                <div className="toolbar" style={{ marginBottom: 16 }}>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("fields")}>
                    Open in Fields
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("relationships")}>
                    Open in Relationships
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("rules")}>
                    Open in Business Rules
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("workflow")}>
                    Open in Workflow Automation
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("dependencyExplorer")}>
                    Open in Dependency Explorer
                  </button>
                </div>
              )}

              <h4>Link Types ({detail.data.link_types.length})</h4>
              {detail.data.link_types.length === 0 && <p style={{ color: "var(--text-muted)" }}>No relationships involve this object type yet.</p>}
              {detail.data.link_types.length > 0 && (
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Inverse name</th>
                      <th>Linked to</th>
                      <th>Type</th>
                    </tr>
                  </thead>
                  <tbody>
                    {detail.data.link_types.map((l) => (
                      <tr key={`${l.relationship_id}-${l.from_object_type}`}>
                        <td>{l.name}</td>
                        <td>{l.inverse_name}</td>
                        <td>{l.to_object_type ?? "Polymorphic (varies per record)"}</td>
                        <td>{l.relationship_type}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
