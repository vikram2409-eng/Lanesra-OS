import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { ActionCapability, ObjectTypeScope } from "../../lib/types";

export const ONTOLOGY_CAPABILITIES: ActionCapability[] = ["create", "read", "update", "delete", "assign"];

/**
 * Ontology Layer epic, issue #345: the Ontology-scope picker (object type +
 * capability checkboxes, plus an Action Type multi-select) extracted out of
 * the Agent Foundry's inline agent form so the standalone Agent Bindings
 * screen can reuse the exact same component - one implementation, so the
 * two surfaces can never drift on what a scope edit actually does.
 */
export function AgentOntologyScopeEditor({
  objectTypeScopes,
  actionTypeIds,
  onToggleObjectTypeScope,
  onToggleScopeCapability,
  onToggleActionType,
}: {
  objectTypeScopes: ObjectTypeScope[];
  actionTypeIds: string[];
  onToggleObjectTypeScope: (objectType: string) => void;
  onToggleScopeCapability: (objectType: string, capability: ActionCapability) => void;
  onToggleActionType: (id: string) => void;
}) {
  const objectTypesQuery = useQuery({ queryKey: ["ontologyObjectTypes"], queryFn: () => api.listObjectTypes() });
  const actionTypesQuery = useQuery({ queryKey: ["actionTypes", "all", "all"], queryFn: () => api.listActionTypes(undefined, true) });

  return (
    <>
      <div className="field full">
        <label>Ontology scope</label>
        <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "2px 0 6px" }}>
          Empty means unscoped - this agent's Record tools above work against any object type, exactly as before this
          feature. Add an object type to restrict this agent to it, and which capabilities it may exercise there.
        </p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 10, marginBottom: 8 }}>
          {(objectTypesQuery.data ?? []).map((o) => {
            const scope = objectTypeScopes.find((s) => s.object_type === o.key);
            return (
              <label key={o.key} style={{ fontSize: 13 }}>
                <input type="checkbox" checked={!!scope} onChange={() => onToggleObjectTypeScope(o.key)} /> {o.label_plural}
              </label>
            );
          })}
        </div>
        {objectTypeScopes.map((scope) => (
          <div key={scope.object_type} style={{ display: "flex", gap: 10, alignItems: "center", fontSize: 12, marginBottom: 4 }}>
            <span style={{ width: 140 }}>{scope.object_type}:</span>
            {ONTOLOGY_CAPABILITIES.map((c) => (
              <label key={c} style={{ display: "flex", gap: 4, alignItems: "center" }}>
                <input type="checkbox" checked={scope.capabilities.includes(c)} onChange={() => onToggleScopeCapability(scope.object_type, c)} /> {c}
              </label>
            ))}
          </div>
        ))}
      </div>
      {(actionTypesQuery.data ?? []).length > 0 && (
        <div className="field full">
          <label>Action Types this agent may invoke</label>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "2px 0 6px" }}>
            Grants the new `invoke_action` tool, scoped to exactly the Action Types checked here - each still enforces
            its own required capability independent of the Ontology scope above.
          </p>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
            {(actionTypesQuery.data ?? []).map((a) => (
              <label key={a.id} style={{ fontSize: 13 }}>
                <input type="checkbox" checked={actionTypeIds.includes(a.id)} onChange={() => onToggleActionType(a.id)} /> {a.name} ({a.target_object_type})
              </label>
            ))}
          </div>
        </div>
      )}
    </>
  );
}
