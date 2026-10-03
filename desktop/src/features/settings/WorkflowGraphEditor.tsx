import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { builtinFieldsFor } from "../../lib/types";
import type { AiAgentDefinition, AiAgentPipeline, ConditionOperator, ExecutionGraph, GraphNodeType } from "../../lib/types";
import { useCanvasZoom } from "../../components/visualBuilder/useCanvasZoom";
import { useNodeDrag } from "../../components/visualBuilder/useNodeDrag";
import { useConnectMode } from "../../components/visualBuilder/useConnectMode";
import { VisualBuilderCanvas, VisualBuilderZoomControls, type CanvasEdgeView } from "../../components/visualBuilder/VisualBuilderCanvas";
import {
  NODE_TYPE_META,
  FULL_PALETTE,
  defaultConfig,
  nodesFromGraph,
  edgesFromGraph,
  nextNodeKey,
  summarizeNode,
  buildGraphInput,
  type EditNode,
  type EditEdge,
} from "../../components/visualBuilder/graphNodeMeta";
import { NodePropertyPanel, BranchLabelForm, GraphRunsView } from "./AgentTeamsAdmin";

/**
 * Workflow Studio 2.0 (issue #193): the real, editable canvas a workflow
 * gets once an admin upgrades it (`workflow_service::upgrade_to_graph`) -
 * built on the exact same Shared Visual Builder Framework primitives
 * (`useCanvasZoom`/`useNodeDrag`/`useConnectMode`/`VisualBuilderCanvas`)
 * and the same generic node-property-panel `AgentTeamsAdmin.tsx` (issue
 * #170) already proved, imported from there rather than duplicated. What's
 * genuinely new here, scoped to this workflow's own needs: a typed
 * variable reference panel (the triggering record's own fields, since a
 * workflow-sourced graph's trigger node seeds them directly into context -
 * see `graph_runtime_service`'s own "trigger" node doc comment) and a
 * client-side Test Run that walks the in-memory (unsaved) graph shape
 * against a sample record, highlighting the path a real run would take as
 * far as it can be known without actually executing anything - it stops
 * cleanly at the first node whose outgoing edge depends on a real runtime
 * outcome (an Agent/Evaluate Result/Approval/Loop/Parallel Split's actual
 * result), labeled as such rather than guessing.
 */
export function WorkflowGraphEditor({
  graphId,
  entityType,
  agents,
  pipelines,
  onBack,
}: {
  graphId: string;
  entityType: string;
  agents: AiAgentDefinition[];
  pipelines: AiAgentPipeline[];
  onBack: () => void;
}) {
  const [subview, setSubview] = useState<"canvas" | "runs">("canvas");
  if (subview === "runs") {
    return <GraphRunsView graphId={graphId} onBack={() => setSubview("canvas")} />;
  }
  return <GraphCanvas graphId={graphId} entityType={entityType} agents={agents} pipelines={pipelines} onBack={onBack} onOpenRuns={() => setSubview("runs")} />;
}

type SimResult = { visitedKeys: string[]; forkKey: string | null; forkReason: string | null; endReached: boolean; error: string | null };

const LIST_SEPARATOR = "|";

function operatorMatches(operator: ConditionOperator, actual: string, value: string): boolean {
  switch (operator) {
    case "equals": return actual === value;
    case "not_equals": return actual !== value;
    case "contains": return value !== "" && actual.includes(value);
    case "not_contains": return value === "" || !actual.includes(value);
    case "starts_with": return value !== "" && actual.startsWith(value);
    case "ends_with": return value !== "" && actual.endsWith(value);
    case "in_list": return value.split(LIST_SEPARATOR).includes(actual);
    case "not_in_list": return !value.split(LIST_SEPARATOR).includes(actual);
    case "is_empty": return actual === "";
    case "is_not_empty": return actual !== "";
    case "greater_than": { const a = Number(actual), b = Number(value); return actual !== "" && value !== "" && !Number.isNaN(a) && !Number.isNaN(b) && a > b; }
    case "less_than": { const a = Number(actual), b = Number(value); return actual !== "" && value !== "" && !Number.isNaN(a) && !Number.isNaN(b) && a < b; }
    case "on_or_after": return actual !== "" && actual >= value;
    case "on_or_before": return actual !== "" && actual <= value;
    default: return false;
  }
}

type SimCondition = { field_key: string; operator: ConditionOperator; value: string; group_id?: string | null; compare_field_key?: string | null };

/** Client-side mirror of `domain::conditions::conditions_match` - same
 * one-level OR-grouping, same group-then-top-level match_type evaluation
 * `businessRules.ts`'s own identical helper already established for
 * Business Rules' dry-run mode; a separate copy since it's typed against
 * this graph's own condition config shape, not `BusinessRuleCondition`. */
function conditionsMatch(conditions: SimCondition[], matchType: string, ctx: Record<string, string>): boolean {
  if (conditions.length === 0) return true;
  const units: boolean[] = [];
  const groups = new Map<string, boolean>();
  const groupOrder: string[] = [];
  for (const c of conditions) {
    const comparand = c.compare_field_key ? ctx[c.compare_field_key] ?? "" : c.value;
    const matched = operatorMatches(c.operator, ctx[c.field_key] ?? "", comparand);
    if (c.group_id) {
      const existing = groups.get(c.group_id);
      if (existing === undefined) groupOrder.push(c.group_id);
      groups.set(c.group_id, (existing ?? false) || matched);
    } else {
      units.push(matched);
    }
  }
  for (const g of groupOrder) units.push(groups.get(g) as boolean);
  return matchType === "any" ? units.some(Boolean) : units.every(Boolean);
}

/** Node types whose single outgoing edge is reachable without running
 * anything for real - a dry run walks straight through these. Every other
 * type's outgoing edge depends on a real runtime outcome (an Agent's
 * actual output, an Evaluate Result judge call, a human Approval, a
 * Loop's iteration count, a Parallel Split's branch results) and the
 * simulation stops there instead of guessing. */
function isDeterministicPassthrough(nodeType: GraphNodeType): boolean {
  return nodeType === "trigger" || nodeType === "action" || nodeType === "agent" || nodeType === "run_agent_team" || nodeType === "delay" || nodeType === "transform";
}

function simulatePath(nodes: EditNode[], edges: EditEdge[], trigger_input_json: string): SimResult {
  let ctx: Record<string, string> = {};
  try {
    const parsed = JSON.parse(trigger_input_json || "{}");
    if (parsed && typeof parsed === "object") {
      for (const [k, v] of Object.entries(parsed)) ctx[k] = typeof v === "string" ? v : JSON.stringify(v);
    }
  } catch {
    return { visitedKeys: [], forkKey: null, forkReason: null, endReached: false, error: "Sample trigger input isn't valid JSON" };
  }

  const start = nodes.find((n) => n.node_type === "trigger");
  if (!start) return { visitedKeys: [], forkKey: null, forkReason: null, endReached: false, error: "This graph has no Trigger node" };

  const visited: string[] = [];
  let current: EditNode | undefined = start;
  const maxSteps = 50;
  for (let step = 0; step < maxSteps; step++) {
    if (!current) return { visitedKeys: visited, forkKey: null, forkReason: null, endReached: false, error: "Reached a dead end - an edge points nowhere" };
    visited.push(current.node_key);
    if (current.node_type === "end") {
      return { visitedKeys: visited, forkKey: null, forkReason: null, endReached: true, error: null };
    }

    const outgoing = edges.filter((e) => e.from_node_key === current!.node_key);

    if (current.node_type === "condition") {
      const cfg = current.config as { match_type?: string; conditions?: SimCondition[] };
      const result = conditionsMatch(cfg.conditions ?? [], cfg.match_type ?? "all", ctx);
      const label = result ? "true" : "false";
      const next = outgoing.find((e) => e.branch_label === label);
      if (!next) return { visitedKeys: visited, forkKey: current.node_key, forkReason: `No '${label}' edge out of this Condition node`, endReached: false, error: null };
      current = nodes.find((n) => n.node_key === next.to_node_key);
      continue;
    }

    if (current.node_type === "router") {
      const cfg = current.config as { branches?: { branch_label: string; match_type: string; conditions: SimCondition[] }[] };
      let chosenLabel: string | null = null;
      for (const b of cfg.branches ?? []) {
        if (conditionsMatch(b.conditions ?? [], b.match_type ?? "all", ctx)) {
          chosenLabel = b.branch_label;
          break;
        }
      }
      const label = chosenLabel ?? (outgoing.some((e) => e.branch_label === "default") ? "default" : null);
      if (!label) return { visitedKeys: visited, forkKey: current.node_key, forkReason: "No Switch branch matched and no 'default' edge exists", endReached: false, error: null };
      const next = outgoing.find((e) => e.branch_label === label);
      if (!next) return { visitedKeys: visited, forkKey: current.node_key, forkReason: `No edge labeled '${label}' out of this Switch node`, endReached: false, error: null };
      current = nodes.find((n) => n.node_key === next.to_node_key);
      continue;
    }

    if (isDeterministicPassthrough(current.node_type)) {
      const next = outgoing.find((e) => !e.branch_label);
      current = next ? nodes.find((n) => n.node_key === next.to_node_key) : undefined;
      continue;
    }

    // approval / evaluate_result / loop / parallel_split / join: a real
    // runtime outcome decides the branch from here - stop the dry run
    // cleanly rather than guess which one.
    return { visitedKeys: visited, forkKey: current.node_key, forkReason: `This node's outgoing path depends on a real run (${NODE_TYPE_META[current.node_type].label}) - not simulated here`, endReached: false, error: null };
  }
  return { visitedKeys: visited, forkKey: null, forkReason: null, endReached: false, error: "Stopped after 50 steps - this shape may have an unbounded loop" };
}

function GraphCanvas({
  graphId,
  entityType,
  agents,
  pipelines,
  onBack,
  onOpenRuns,
}: {
  graphId: string;
  entityType: string;
  agents: AiAgentDefinition[];
  pipelines: AiAgentPipeline[];
  onBack: () => void;
  onOpenRuns: () => void;
}) {
  const qc = useQueryClient();
  const existingQuery = useQuery({ queryKey: ["executionGraph", graphId], queryFn: () => api.getExecutionGraph(graphId) });

  const [graphMeta, setGraphMeta] = useState<{ name: string; description: string; status: ExecutionGraph["status"]; version: number }>({
    name: "", description: "", status: "draft", version: 1,
  });
  const [nodes, setNodes] = useState<EditNode[]>([]);
  const [edges, setEdges] = useState<EditEdge[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loadedFor, setLoadedFor] = useState<string | null>(null);
  const [showVariables, setShowVariables] = useState(false);
  const [testOpen, setTestOpen] = useState(false);
  const [sampleInput, setSampleInput] = useState('{\n  "status": ""\n}');
  const [simResult, setSimResult] = useState<SimResult | null>(null);

  useEffect(() => {
    if (existingQuery.data && loadedFor !== existingQuery.data.id) {
      const g = existingQuery.data;
      setGraphMeta({ name: g.name, description: g.description ?? "", status: g.status, version: g.version });
      setNodes(nodesFromGraph(g));
      setEdges(edgesFromGraph(g));
      setLoadedFor(g.id);
    }
  }, [existingQuery.data, loadedFor]);

  const isDraft = graphMeta.status === "draft";
  const readOnly = !isDraft;

  const { zoom, zoomIn, zoomOut } = useCanvasZoom();
  const { startDrag } = useNodeDrag<EditNode>(setNodes, zoom, readOnly, (n) => n.node_key);
  const { connectFrom, pendingChoice, startConnecting, cancelConnecting, resolveChoice, targetClicked } = useConnectMode({
    edges: edges.map((e) => ({ from: e.from_node_key, to: e.to_node_key, label: e.branch_label })),
    ruleFor: (nodeKey) => {
      const n = nodes.find((x) => x.node_key === nodeKey);
      return n ? NODE_TYPE_META[n.node_type].rule : { kind: "none" };
    },
    onError: setError,
    onComplete: (from, to, label) => setEdges((prev) => [...prev, { from_node_key: from, to_node_key: to, branch_label: label }]),
  });

  const updateMutation = useMutation({
    mutationFn: () => api.updateExecutionGraph(graphId, buildGraphInput(graphMeta.name.trim() || "Untitled", graphMeta.description.trim() || null, nodes, edges)),
    onSuccess: (g) => {
      setGraphMeta({ name: g.name, description: g.description ?? "", status: g.status, version: g.version });
      qc.invalidateQueries({ queryKey: ["executionGraph", graphId] });
      qc.invalidateQueries({ queryKey: ["workflowRules"] });
      setError(null);
    },
    onError: (e) => setError(e instanceof ApiError ? e.message : String(e)),
  });
  const publishMutation = useMutation({
    mutationFn: () => api.publishExecutionGraph(graphId),
    onSuccess: (g) => {
      setGraphMeta({ name: g.name, description: g.description ?? "", status: g.status, version: g.version });
      qc.invalidateQueries({ queryKey: ["executionGraph", graphId] });
      qc.invalidateQueries({ queryKey: ["workflowRules"] });
      setError(null);
    },
    onError: (e) => setError(e instanceof ApiError ? e.message : String(e)),
  });

  function addNode(type: GraphNodeType) {
    const key = nextNodeKey(nodes, type);
    const pos = { x: 40 + (nodes.length % 4) * 260, y: 40 + Math.floor(nodes.length / 4) * 160 };
    setNodes((prev) => [...prev, { node_key: key, node_type: type, config: defaultConfig(type), x: pos.x, y: pos.y }]);
    setSelected(key);
  }
  function deleteNode(key: string) {
    setNodes((prev) => prev.filter((n) => n.node_key !== key));
    setEdges((prev) => prev.filter((e) => e.from_node_key !== key && e.to_node_key !== key));
    if (selected === key) setSelected(null);
  }
  function renameNode(oldKey: string, newKey: string) {
    const trimmed = newKey.trim();
    if (!trimmed || trimmed === oldKey) return;
    if (nodes.some((n) => n.node_key === trimmed)) {
      setError(`A node named '${trimmed}' already exists`);
      return;
    }
    setNodes((prev) => prev.map((n) => (n.node_key === oldKey ? { ...n, node_key: trimmed } : n)));
    setEdges((prev) => prev.map((e) => ({ from_node_key: e.from_node_key === oldKey ? trimmed : e.from_node_key, to_node_key: e.to_node_key === oldKey ? trimmed : e.to_node_key, branch_label: e.branch_label })));
    setSelected(trimmed);
  }
  function updateNodeConfig(key: string, config: Record<string, unknown>) {
    setNodes((prev) => prev.map((n) => (n.node_key === key ? { ...n, config } : n)));
  }
  function handleNodeClicked(key: string) {
    if (readOnly) {
      setSelected(key);
      return;
    }
    if (connectFrom && targetClicked(key)) return;
    setSelected(key);
  }

  const canvasWidth = Math.max(1400, ...nodes.map((n) => n.x + 260));
  const canvasHeight = Math.max(900, ...nodes.map((n) => n.y + 160));
  function centerOf(n: EditNode) {
    return { x: n.x + 110, y: n.y + 30 };
  }
  const selectedNode = nodes.find((n) => n.node_key === selected) ?? null;

  const variableFields = [...builtinFieldsFor(entityType).filter((f) => f.actionable), { key: "trigger_input", label: "Raw trigger input" }];

  function runTest() {
    setSimResult(simulatePath(nodes, edges, sampleInput));
  }

  if (existingQuery.isLoading || loadedFor !== graphId) {
    return (
      <div className="card">
        <button className="link-button" onClick={onBack}>
          ← Workflow Automation
        </button>
        <p>Loading...</p>
      </div>
    );
  }

  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
        <div style={{ flex: 1, minWidth: 220 }}>
          <button className="link-button" onClick={onBack}>
            ← Workflow Automation
          </button>
          <h3 style={{ margin: "6px 0" }}>{graphMeta.name}</h3>
          <span className={`badge ${graphMeta.status === "published" ? "badge-success" : graphMeta.status === "disabled" ? "badge-danger" : ""}`}>
            {graphMeta.status} · v{graphMeta.version}
          </span>
        </div>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          <button className="btn" onClick={onOpenRuns}>
            Runs
          </button>
          <button className="btn" onClick={() => setShowVariables((v) => !v)}>
            {showVariables ? "Hide" : "Show"} variables
          </button>
          <button className="btn" onClick={() => setTestOpen((v) => !v)}>
            {testOpen ? "Close test" : "Test run"}
          </button>
          {isDraft && (
            <button className="btn btn-primary" onClick={() => updateMutation.mutate()} disabled={updateMutation.isPending}>
              Save draft
            </button>
          )}
          {isDraft && (
            <button className="btn" onClick={() => publishMutation.mutate()} disabled={publishMutation.isPending}>
              Publish
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="error-banner" style={{ marginBottom: 10 }}>
          {error}
          <button className="link-button" style={{ marginLeft: 10 }} onClick={() => setError(null)}>
            dismiss
          </button>
        </div>
      )}
      {!isDraft && (
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
          {graphMeta.status === "published" ? "A published graph is immutable - this is how your workflow actually runs now." : "This graph is disabled."}
        </p>
      )}
      {isDraft && (
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
          This workflow stopped firing when it was upgraded, until this graph is published - add the Switch/Loop/Parallel/Join/Run Agent Team/Evaluate Result
          nodes you need, then Publish to resume it on the new engine.
        </p>
      )}

      {showVariables && (
        <div className="card" style={{ marginBottom: 10, background: "var(--surface-sidebar, var(--bg-muted))" }}>
          <strong style={{ fontSize: 13 }}>Available variables</strong>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "4px 0 8px" }}>
            Click to copy a token, then paste it into an Agent's input template, a Transform's value, or a Condition/Switch branch's field.
          </p>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {variableFields.map((f) => (
              <button
                key={f.key}
                className="btn"
                style={{ fontSize: 12, padding: "2px 8px" }}
                onClick={() => navigator.clipboard?.writeText(`{{${f.key}}}`).catch(() => {})}
                title={`Copy {{${f.key}}}`}
              >
                {`{{${f.key}}}`} <span style={{ color: "var(--text-muted)" }}>{f.label}</span>
              </button>
            ))}
            {nodes.filter((n) => n.node_key !== "trigger_1" && n.node_type !== "end").map((n) => (
              <button
                key={n.node_key}
                className="btn"
                style={{ fontSize: 12, padding: "2px 8px", borderColor: NODE_TYPE_META[n.node_type].color }}
                onClick={() => navigator.clipboard?.writeText(`{{${n.node_key}}}`).catch(() => {})}
                title={`Copy {{${n.node_key}}} (this node's own output)`}
              >
                {`{{${n.node_key}}}`} <span style={{ color: "var(--text-muted)" }}>{NODE_TYPE_META[n.node_type].label} output</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {testOpen && (
        <div className="card" style={{ marginBottom: 10 }}>
          <strong style={{ fontSize: 13 }}>Test run (dry run)</strong>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "4px 0 8px" }}>
            Walks this graph's current (unsaved) shape against a sample trigger input - nothing is executed for real. Stops at the first node whose branch
            depends on a real run (an Agent's actual output, an Approval, Evaluate Result, Loop or Parallel Split).
          </p>
          <div style={{ display: "flex", gap: 8 }}>
            <textarea style={{ flex: 1 }} rows={3} value={sampleInput} onChange={(e) => setSampleInput(e.target.value)} />
            <button className="btn btn-primary" onClick={runTest}>
              Run test
            </button>
          </div>
          {simResult?.error && <div className="error-banner" style={{ marginTop: 8 }}>{simResult.error}</div>}
          {simResult && !simResult.error && (
            <p style={{ fontSize: 12, marginTop: 8 }}>
              Path: {simResult.visitedKeys.join(" → ")}
              {simResult.endReached && " → End (reached)"}
              {simResult.forkKey && <span style={{ color: "var(--warning, #b45309)" }}> — {simResult.forkReason}</span>}
            </p>
          )}
        </div>
      )}

      <div className="graph-editor-layout">
        <div>
          {isDraft && (
            <div className="graph-palette">
              {FULL_PALETTE.map((t) => (
                <button key={t} className="btn" style={{ borderColor: NODE_TYPE_META[t].color }} onClick={() => addNode(t)}>
                  + {NODE_TYPE_META[t].label}
                </button>
              ))}
            </div>
          )}
          {connectFrom && (
            <div className="graph-connect-banner">
              Connecting from <strong>{connectFrom}</strong> - click a target node, or press Esc to cancel.
            </div>
          )}
          {!connectFrom && isDraft && nodes.length > 0 && edges.length === 0 && (
            <div className="graph-hint-banner">
              Nodes on the canvas aren't connected until you link them: click the <strong>→</strong> button on a node, then
              click the node it should lead to.
            </div>
          )}
          <div className="graph-canvas-wrap">
            <VisualBuilderCanvas<EditNode>
              nodes={nodes}
              zoom={zoom}
              canvasWidth={canvasWidth}
              canvasHeight={canvasHeight}
              editable={isDraft}
              onEdgeDelete={(key) => setEdges((prev) => prev.filter((_, xi) => String(xi) !== key))}
              edges={edges.reduce<CanvasEdgeView[]>((acc, e, i) => {
                const from = nodes.find((n) => n.node_key === e.from_node_key);
                const to = nodes.find((n) => n.node_key === e.to_node_key);
                if (!from || !to) return acc;
                const a = centerOf(from);
                const b = centerOf(to);
                acc.push({ key: String(i), fromX: a.x, fromY: a.y, toX: b.x, toY: b.y, label: e.branch_label });
                return acc;
              }, [])}
              renderNode={(n) => {
                const meta = NODE_TYPE_META[n.node_type];
                const visited = simResult?.visitedKeys.includes(n.node_key) ?? false;
                const isFork = simResult?.forkKey === n.node_key;
                return (
                  <div
                    key={n.node_key}
                    className={`graph-node ${selected === n.node_key ? "graph-node-selected" : ""} ${visited ? "graph-node-sim-visited" : ""} ${isFork ? "graph-node-sim-fork" : ""}`}
                    style={{ left: n.x, top: n.y, borderColor: selected === n.node_key ? meta.color : isFork ? "var(--warning, #b45309)" : visited ? "#16a34a" : undefined }}
                    onPointerDown={(e) => startDrag(e, n)}
                    onClick={() => handleNodeClicked(n.node_key)}
                  >
                    <div className="graph-node-head" style={{ background: `${meta.color}26`, color: meta.color }}>
                      {meta.label}
                      {visited && <span title="Reached in last test run"> ✓</span>}
                      {isFork && <span title="Stopped here - real run decides"> ⚠</span>}
                    </div>
                    <div className="graph-node-body">
                      <strong>{n.node_key}</strong>
                      <small>{summarizeNode(n)}</small>
                    </div>
                    {meta.rule.kind !== "none" && isDraft && (
                      <button
                        className="graph-node-connect"
                        title="Connect to another node"
                        onPointerDown={(e) => e.stopPropagation()}
                        onClick={(e) => {
                          e.stopPropagation();
                          startConnecting(n.node_key);
                        }}
                      >
                        →
                      </button>
                    )}
                  </div>
                );
              }}
            />
            <VisualBuilderZoomControls onZoomIn={zoomIn} onZoomOut={zoomOut} />
          </div>
        </div>

        <div className="graph-property-panel">
          {!selectedNode && <p style={{ color: "var(--text-muted)" }}>Select a node to edit its configuration.</p>}
          {selectedNode && (
            <NodePropertyPanel
              node={selectedNode}
              agents={agents}
              pipelines={pipelines}
              readOnly={readOnly}
              onRename={(newKey) => renameNode(selectedNode.node_key, newKey)}
              onConfigChange={(config) => updateNodeConfig(selectedNode.node_key, config)}
              onDelete={() => deleteNode(selectedNode.node_key)}
            />
          )}
        </div>
      </div>

      {pendingChoice && (
        <div className="modal-overlay" onClick={cancelConnecting}>
          <div className="modal" style={{ maxWidth: 360 }} onClick={(e) => e.stopPropagation()}>
            <h4 style={{ marginTop: 0 }}>{pendingChoice.options.length > 0 ? "Which branch?" : "Branch label"}</h4>
            {pendingChoice.options.length > 0 ? (
              <div style={{ display: "flex", gap: 8 }}>
                {pendingChoice.options.map((opt) => (
                  <button key={opt} className="btn btn-primary" onClick={() => resolveChoice(opt)}>
                    {opt}
                  </button>
                ))}
              </div>
            ) : (
              <BranchLabelForm onSubmit={(label) => resolveChoice(label)} />
            )}
            <button className="btn" style={{ marginTop: 10 }} onClick={cancelConnecting}>
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
