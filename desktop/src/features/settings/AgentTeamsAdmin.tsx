import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { CONDITION_OPERATORS, WORKFLOW_ACTION_TYPES } from "../../lib/types";
import type {
  AiAgentDefinition,
  AiAgentPipeline,
  ConditionOperator,
  ExecutionGraph,
  ExecutionGraphInput,
  GraphNodeType,
  GraphRun,
  WorkflowActionType,
} from "../../lib/types";
import { useCanvasZoom } from "../../components/visualBuilder/useCanvasZoom";
import { useNodeDrag } from "../../components/visualBuilder/useNodeDrag";
import { useConnectMode } from "../../components/visualBuilder/useConnectMode";
import { VisualBuilderCanvas, VisualBuilderZoomControls, type CanvasEdgeView } from "../../components/visualBuilder/VisualBuilderCanvas";
import {
  NODE_TYPE_META as SHARED_NODE_TYPE_META,
  FULL_PALETTE,
  defaultConfig,
  ACTION_PARAM_HINTS,
  gridPosition,
  nodesFromGraph,
  edgesFromGraph,
  nextNodeKey,
  summarizeNode,
  buildGraphInput as buildInput,
  type EditNode,
  type EditEdge,
} from "../../components/visualBuilder/graphNodeMeta";

// AI Agent Platform v2, Phase 5b (GitHub issue #170, UI half): a genuine
// free-form canvas authoring Phase 3's Execution Graphs - the node
// palette/property-panel/zoom-controls shape the plan called for, reusing
// only the CSS-zoom trick and node-card visual language Workflow
// Automation's own (fixed, linear) canvas already established, since that
// screen has no free node placement, no SVG connectors and no drag-and-
// drop to actually reuse beyond those two things (see this repo's own
// investigation notes on that screen). A graph's nodes/edges are edited
// entirely in `node_key` space (never the server-assigned row `id`) since
// that's what `GraphEdgeInput` itself addresses - only a load from an
// existing graph needs to map `GraphNode.id` back to its own `node_key`.
//
// Deliberately scoped to the canvas + a Runs viewer, not the plan's full
// Canvas/Agents/Routing/Shared Context/Policies/Tests/Versions tab set:
// Agents, Policies and Evaluations already have their own dedicated,
// already-shipped admin screens (AI Agent Foundry's own tabs, the Policy
// Engine panel) - duplicating them inside this page would be a second,
// divergent copy of the same configuration, not a real second surface.
// The Action node's `params_json` stays a raw JSON field with a
// per-action-type example, rather than reimplementing Workflow
// Automation's own rich per-action-type builder a second time here - an
// honest, smaller-but-real simplification, not a fake one.

const NODE_TYPE_META = SHARED_NODE_TYPE_META;
const PALETTE = FULL_PALETTE;

export function AgentTeamsAdmin() {
  const [view, setView] = useState<{ kind: "list" } | { kind: "editor"; graphId: string | null } | { kind: "runs"; graphId: string }>({ kind: "list" });

  if (view.kind === "editor") {
    return <GraphEditor graphId={view.graphId} onBack={() => setView({ kind: "list" })} onOpenRuns={(id) => setView({ kind: "runs", graphId: id })} />;
  }
  if (view.kind === "runs") {
    return <GraphRunsView graphId={view.graphId} onBack={() => setView({ kind: "editor", graphId: view.graphId })} />;
  }
  return <GraphListView onOpen={(id) => setView({ kind: "editor", graphId: id })} onNew={() => setView({ kind: "editor", graphId: null })} />;
}

function GraphListView({ onOpen, onNew }: { onOpen: (id: string) => void; onNew: () => void }) {
  const graphsQuery = useQuery({ queryKey: ["executionGraphs"], queryFn: () => api.listExecutionGraphs() });
  const graphs = graphsQuery.data ?? [];
  const qc = useQueryClient();
  const toggleDisabled = useMutation({
    mutationFn: ({ id, disabled }: { id: string; disabled: boolean }) => api.setExecutionGraphDisabled(id, disabled),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["executionGraphs"] }),
  });

  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
        <h3 style={{ margin: 0 }}>Agent Teams</h3>
        <button className="btn btn-primary" onClick={onNew}>
          + New team
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        A team is an Execution Graph: named agents, tools, conditions and control flow wired into one durable, versioned run. A run checkpoints
        after every node - kill the app mid-run and it resumes exactly where it left off. A Draft graph can be freely edited; Publishing
        validates its shape (reachability, branch completeness, a bounded loop) and makes it immutable from then on.
      </p>
      {graphsQuery.isLoading && <p>Loading...</p>}
      {graphs.length === 0 && !graphsQuery.isLoading && <div className="empty-state">No agent teams yet - click "+ New team" to build one.</div>}
      {graphs.length > 0 && (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Status</th>
                <th>Version</th>
                <th>Nodes</th>
                <th>Source</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {graphs.map((g) => (
                <tr key={g.id}>
                  <td>
                    <button className="link-button" onClick={() => onOpen(g.id)}>
                      {g.name}
                    </button>
                    {g.description && <div style={{ color: "var(--text-muted)", fontSize: 12 }}>{g.description}</div>}
                  </td>
                  <td>
                    <span className={`badge ${g.status === "published" ? "badge-success" : g.status === "disabled" ? "badge-danger" : ""}`}>{g.status}</span>
                  </td>
                  <td>v{g.version}</td>
                  <td>{g.nodes.length}</td>
                  <td>{g.source_kind ? `${g.source_kind} shape` : "authored"}</td>
                  <td style={{ textAlign: "right" }}>
                    {(g.status === "published" || g.status === "disabled") && (
                      <button className="btn" onClick={() => toggleDisabled.mutate({ id: g.id, disabled: g.status !== "disabled" })}>
                        {g.status === "disabled" ? "Enable" : "Disable"}
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function GraphEditor({ graphId, onBack, onOpenRuns }: { graphId: string | null; onBack: () => void; onOpenRuns: (id: string) => void }) {
  const qc = useQueryClient();
  const existingQuery = useQuery({ queryKey: ["executionGraph", graphId], queryFn: () => api.getExecutionGraph(graphId as string), enabled: !!graphId });
  const agentsQuery = useQuery({ queryKey: ["aiAgentsForTeams"], queryFn: () => api.listAiAgents(true) });
  const agents: AiAgentDefinition[] = agentsQuery.data ?? [];
  // Workflow Studio 2.0 (issue #193): the Run Agent Team node's picker.
  const pipelinesQuery = useQuery({ queryKey: ["aiAgentPipelinesForTeams"], queryFn: () => api.listAiAgentPipelines(true) });
  const pipelines: AiAgentPipeline[] = pipelinesQuery.data ?? [];

  const [graphMeta, setGraphMeta] = useState<{ id: string | null; name: string; description: string; status: ExecutionGraph["status"] }>({
    id: null,
    name: "New Agent Team",
    description: "",
    status: "draft",
  });
  const [nodes, setNodes] = useState<EditNode[]>([{ node_key: "trigger_1", node_type: "trigger", config: {}, x: 40, y: 40 }, { node_key: "end_1", node_type: "end", config: {}, x: 40, y: 220 }]);
  const [edges, setEdges] = useState<EditEdge[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loadedFor, setLoadedFor] = useState<string | null>(null);

  useEffect(() => {
    if (existingQuery.data && loadedFor !== existingQuery.data.id) {
      const g = existingQuery.data;
      setGraphMeta({ id: g.id, name: g.name, description: g.description ?? "", status: g.status });
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
    onComplete: completeConnection,
  });

  const createMutation = useMutation({
    mutationFn: (input: ExecutionGraphInput) => api.createExecutionGraph(input),
    onSuccess: (g) => {
      setGraphMeta({ id: g.id, name: g.name, description: g.description ?? "", status: g.status });
      setLoadedFor(g.id);
      qc.invalidateQueries({ queryKey: ["executionGraphs"] });
      setError(null);
    },
    onError: (e) => setError(e instanceof ApiError ? e.message : String(e)),
  });
  const updateMutation = useMutation({
    mutationFn: ({ id, input }: { id: string; input: ExecutionGraphInput }) => api.updateExecutionGraph(id, input),
    onSuccess: (g) => {
      setGraphMeta({ id: g.id, name: g.name, description: g.description ?? "", status: g.status });
      qc.invalidateQueries({ queryKey: ["executionGraphs"] });
      qc.invalidateQueries({ queryKey: ["executionGraph", g.id] });
      setError(null);
    },
    onError: (e) => setError(e instanceof ApiError ? e.message : String(e)),
  });
  const publishMutation = useMutation({
    mutationFn: (id: string) => api.publishExecutionGraph(id),
    onSuccess: (g) => {
      setGraphMeta({ id: g.id, name: g.name, description: g.description ?? "", status: g.status });
      qc.invalidateQueries({ queryKey: ["executionGraphs"] });
      setError(null);
    },
    onError: (e) => setError(e instanceof ApiError ? e.message : String(e)),
  });

  function doSave() {
    const input = buildInput(graphMeta.name.trim() || "Untitled team", graphMeta.description.trim() || null, nodes, edges);
    if (graphMeta.id) updateMutation.mutate({ id: graphMeta.id, input });
    else createMutation.mutate(input);
  }

  function addNode(type: GraphNodeType) {
    const key = nextNodeKey(nodes, type);
    const pos = gridPosition(nodes.length);
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

  function completeConnection(from: string, to: string, label: string | null) {
    setEdges((prev) => [...prev, { from_node_key: from, to_node_key: to, branch_label: label }]);
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

  if (graphId && (existingQuery.isLoading || loadedFor !== graphId)) {
    return (
      <div className="card">
        <button className="link-button" onClick={onBack}>
          ← Agent Teams
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
            ← Agent Teams
          </button>
          {isDraft ? (
            <input
              value={graphMeta.name}
              onChange={(e) => setGraphMeta((m) => ({ ...m, name: e.target.value }))}
              style={{ display: "block", fontSize: 18, fontWeight: 600, margin: "6px 0", width: "100%", maxWidth: 360 }}
            />
          ) : (
            <h3 style={{ margin: "6px 0" }}>{graphMeta.name}</h3>
          )}
          <span className={`badge ${graphMeta.status === "published" ? "badge-success" : graphMeta.status === "disabled" ? "badge-danger" : ""}`}>
            {graphMeta.status} · v{existingQuery.data?.version ?? 1}
          </span>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          {graphMeta.id && (
            <button className="btn" onClick={() => onOpenRuns(graphMeta.id as string)}>
              Runs
            </button>
          )}
          {isDraft && (
            <button className="btn btn-primary" onClick={doSave} disabled={createMutation.isPending || updateMutation.isPending}>
              Save draft
            </button>
          )}
          {isDraft && graphMeta.id && (
            <button className="btn" onClick={() => publishMutation.mutate(graphMeta.id as string)} disabled={publishMutation.isPending}>
              Publish
            </button>
          )}
        </div>
      </div>

      {isDraft && (
        <textarea
          placeholder="What does this team do?"
          value={graphMeta.description}
          rows={2}
          style={{ width: "100%", marginBottom: 10 }}
          onChange={(e) => setGraphMeta((m) => ({ ...m, description: e.target.value }))}
        />
      )}
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
          {graphMeta.status === "published" ? "A published graph is immutable - disable it from the list and build a new team to change its shape." : "This team is disabled."}
        </p>
      )}

      <div className="graph-editor-layout">
        <div>
          {isDraft && (
            <div className="graph-palette">
              {PALETTE.map((t) => (
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
                return (
                  <div
                    key={n.node_key}
                    className={`graph-node ${selected === n.node_key ? "graph-node-selected" : ""}`}
                    style={{ left: n.x, top: n.y, borderColor: selected === n.node_key ? meta.color : undefined }}
                    onPointerDown={(e) => startDrag(e, n)}
                    onClick={() => handleNodeClicked(n.node_key)}
                  >
                    <div className="graph-node-head" style={{ background: `${meta.color}26`, color: meta.color }}>
                      {meta.label}
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

export function BranchLabelForm({ onSubmit }: { onSubmit: (label: string) => void }) {
  const [value, setValue] = useState("");
  return (
    <div style={{ display: "flex", gap: 8 }}>
      <input value={value} onChange={(e) => setValue(e.target.value)} placeholder="e.g. high_priority, or 'default'" />
      <button className="btn btn-primary" disabled={!value.trim()} onClick={() => onSubmit(value.trim())}>
        Add
      </button>
    </div>
  );
}

export function NodePropertyPanel({
  node,
  agents,
  pipelines,
  readOnly,
  onRename,
  onConfigChange,
  onDelete,
}: {
  node: EditNode;
  agents: AiAgentDefinition[];
  pipelines: AiAgentPipeline[];
  readOnly: boolean;
  onRename: (key: string) => void;
  onConfigChange: (config: Record<string, unknown>) => void;
  onDelete: () => void;
}) {
  const meta = NODE_TYPE_META[node.node_type];
  return (
    <div>
      <div className="form-field">
        <label>Node key</label>
        <input defaultValue={node.node_key} key={node.node_key} disabled={readOnly} onBlur={(e) => onRename(e.target.value)} />
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 12 }}>{meta.label} node.</p>

      {node.node_type === "condition" && <ConditionsEditor conditions={(node.config.conditions as any[]) ?? []} matchType={(node.config.match_type as string) ?? "all"} readOnly={readOnly} onChange={(conditions, matchType) => onConfigChange({ ...node.config, conditions, match_type: matchType })} />}

      {node.node_type === "router" && <RouterEditor branches={(node.config.branches as any[]) ?? []} readOnly={readOnly} onChange={(branches) => onConfigChange({ ...node.config, branches })} />}

      {node.node_type === "action" && (
        <>
          <div className="form-field">
            <label>Action type</label>
            <select disabled={readOnly} value={String(node.config.action_type ?? WORKFLOW_ACTION_TYPES[0])} onChange={(e) => onConfigChange({ ...node.config, action_type: e.target.value })}>
              {WORKFLOW_ACTION_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </div>
          <div className="form-field">
            <label>Params (JSON)</label>
            <textarea
              disabled={readOnly}
              rows={4}
              value={String(node.config.params_json ?? "{}")}
              placeholder={ACTION_PARAM_HINTS[node.config.action_type as WorkflowActionType]}
              onChange={(e) => onConfigChange({ ...node.config, params_json: e.target.value })}
            />
            {ACTION_PARAM_HINTS[node.config.action_type as WorkflowActionType] && (
              <small style={{ display: "block", color: "var(--text-muted)", fontSize: 11 }}>e.g. {ACTION_PARAM_HINTS[node.config.action_type as WorkflowActionType]}</small>
            )}
          </div>
        </>
      )}

      {node.node_type === "agent" && (
        <>
          <div className="form-field">
            <label>Agent</label>
            <select disabled={readOnly} value={String(node.config.agent_id ?? "")} onChange={(e) => onConfigChange({ ...node.config, agent_id: e.target.value })}>
              <option value="">Select an agent...</option>
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.icon} {a.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-field">
            <label>Input template</label>
            <textarea
              disabled={readOnly}
              rows={3}
              value={String(node.config.input_template ?? "")}
              placeholder="e.g. Summarize: {{trigger_input}}"
              onChange={(e) => onConfigChange({ ...node.config, input_template: e.target.value })}
            />
            <small style={{ display: "block", color: "var(--text-muted)", fontSize: 11 }}>{"{{trigger_input}} or {{node_key.field}} to reference an earlier node's output"}</small>
          </div>
        </>
      )}

      {node.node_type === "run_agent_team" && (
        <>
          <div className="form-field">
            <label>Agent Team (Pipeline)</label>
            <select disabled={readOnly} value={String(node.config.pipeline_id ?? "")} onChange={(e) => onConfigChange({ ...node.config, pipeline_id: e.target.value })}>
              <option value="">Select an agent team...</option>
              {pipelines.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-field">
            <label>Input template</label>
            <textarea
              disabled={readOnly}
              rows={3}
              value={String(node.config.input_template ?? "")}
              placeholder="e.g. Summarize: {{trigger_input}}"
              onChange={(e) => onConfigChange({ ...node.config, input_template: e.target.value })}
            />
            <small style={{ display: "block", color: "var(--text-muted)", fontSize: 11 }}>
              A pipeline run that itself pauses on an approval isn't supported inside this node - it fails the run with a clear message.
            </small>
          </div>
        </>
      )}

      {node.node_type === "evaluate_result" && (
        <>
          <div className="form-field">
            <label>Node to evaluate</label>
            <input
              disabled={readOnly}
              value={String(node.config.source_node_key ?? "")}
              placeholder="e.g. agent_1, or agent_1.field"
              onChange={(e) => onConfigChange({ ...node.config, source_node_key: e.target.value })}
            />
          </div>
          <div className="form-field">
            <label>Success criteria</label>
            <textarea
              disabled={readOnly}
              rows={3}
              value={String(node.config.success_criteria ?? "")}
              placeholder="What does a passing result look like?"
              onChange={(e) => onConfigChange({ ...node.config, success_criteria: e.target.value })}
            />
            <small style={{ display: "block", color: "var(--text-muted)", fontSize: 11 }}>Graded by the same LLM-as-judge call the Evaluation Harness uses. Connect "pass"/"fail" onward.</small>
          </div>
        </>
      )}

      {node.node_type === "approval" && (
        <div className="form-field">
          <label>Subject type</label>
          <input disabled={readOnly} value={String(node.config.subject_type ?? "")} onChange={(e) => onConfigChange({ ...node.config, subject_type: e.target.value })} />
        </div>
      )}

      {node.node_type === "delay" && (
        <div className="form-field">
          <label>Delay (seconds)</label>
          <input type="number" min={0} disabled={readOnly} value={Number(node.config.delay_seconds ?? 0)} onChange={(e) => onConfigChange({ ...node.config, delay_seconds: Number(e.target.value) })} />
        </div>
      )}

      {node.node_type === "loop" && (
        <div className="form-field">
          <label>Max iterations</label>
          <input type="number" min={1} disabled={readOnly} value={Number(node.config.max_iterations ?? 1)} onChange={(e) => onConfigChange({ ...node.config, max_iterations: Number(e.target.value) })} />
          <small style={{ display: "block", color: "var(--text-muted)", fontSize: 11 }}>Connect this node's "body" edge into the loop's own repeated work, and "exit" onward once done.</small>
        </div>
      )}

      {node.node_type === "join" && (
        <>
          <div className="form-field">
            <label>Mode</label>
            <select disabled={readOnly} value={String(node.config.mode ?? "all")} onChange={(e) => onConfigChange({ ...node.config, mode: e.target.value })}>
              <option value="all">All branches must succeed</option>
              <option value="n_of_m">At least N branches</option>
              <option value="first_successful">First successful</option>
              <option value="timeout_partial">Timeout / partial</option>
            </select>
          </div>
          {node.config.mode === "n_of_m" && (
            <div className="form-field">
              <label>Required count (N)</label>
              <input type="number" min={1} disabled={readOnly} value={Number(node.config.required_count ?? 1)} onChange={(e) => onConfigChange({ ...node.config, required_count: Number(e.target.value) })} />
            </div>
          )}
        </>
      )}

      {node.node_type === "transform" && <TransformEditor set={(node.config.set as Record<string, string>) ?? {}} readOnly={readOnly} onChange={(set) => onConfigChange({ ...node.config, set })} />}

      {!readOnly && (
        <button className="btn btn-danger" style={{ marginTop: 14 }} onClick={onDelete}>
          Delete node
        </button>
      )}
    </div>
  );
}

function ConditionsEditor({
  conditions,
  matchType,
  readOnly,
  onChange,
}: {
  conditions: { field_key: string; operator: string; value: string }[];
  matchType: string;
  readOnly: boolean;
  onChange: (conditions: { field_key: string; operator: string; value: string }[], matchType: string) => void;
}) {
  return (
    <div className="form-field">
      <label>
        Match{" "}
        <select disabled={readOnly} value={matchType} onChange={(e) => onChange(conditions, e.target.value)}>
          <option value="all">All (AND)</option>
          <option value="any">Any (OR)</option>
        </select>
      </label>
      {conditions.map((c, i) => (
        <div key={i} className="graph-condition-row">
          <input
            disabled={readOnly}
            placeholder="trigger_input or node_key.field"
            value={c.field_key}
            onChange={(e) => onChange(conditions.map((x, xi) => (xi === i ? { ...x, field_key: e.target.value } : x)), matchType)}
          />
          <select disabled={readOnly} value={c.operator} onChange={(e) => onChange(conditions.map((x, xi) => (xi === i ? { ...x, operator: e.target.value } : x)), matchType)}>
            {(CONDITION_OPERATORS as readonly string[]).map((op) => (
              <option key={op} value={op}>
                {op}
              </option>
            ))}
          </select>
          <input disabled={readOnly} placeholder="value" value={c.value} onChange={(e) => onChange(conditions.map((x, xi) => (xi === i ? { ...x, value: e.target.value } : x)), matchType)} />
          {!readOnly && (
            <button className="btn" style={{ fontSize: 12, padding: "2px 6px" }} onClick={() => onChange(conditions.filter((_, xi) => xi !== i), matchType)}>
              ×
            </button>
          )}
        </div>
      ))}
      {!readOnly && (
        <button className="btn" style={{ marginTop: 6 }} onClick={() => onChange([...conditions, { field_key: "trigger_input", operator: "equals" as ConditionOperator, value: "" }], matchType)}>
          + Add condition
        </button>
      )}
    </div>
  );
}

function RouterEditor({
  branches,
  readOnly,
  onChange,
}: {
  branches: { branch_label: string; match_type: string; conditions: { field_key: string; operator: string; value: string }[] }[];
  readOnly: boolean;
  onChange: (branches: { branch_label: string; match_type: string; conditions: { field_key: string; operator: string; value: string }[] }[]) => void;
}) {
  return (
    <div className="form-field">
      <label>Branches (evaluated in order - first match wins; connect a "default" edge as a fallback)</label>
      {branches.map((b, i) => (
        <div key={i} className="graph-router-branch">
          <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
            <input
              disabled={readOnly}
              placeholder="branch label"
              value={b.branch_label}
              onChange={(e) => onChange(branches.map((x, xi) => (xi === i ? { ...x, branch_label: e.target.value } : x)))}
            />
            {!readOnly && (
              <button className="btn" style={{ fontSize: 12, padding: "2px 6px" }} onClick={() => onChange(branches.filter((_, xi) => xi !== i))}>
                ×
              </button>
            )}
          </div>
          <ConditionsEditor
            conditions={b.conditions}
            matchType={b.match_type}
            readOnly={readOnly}
            onChange={(conditions, matchType) => onChange(branches.map((x, xi) => (xi === i ? { ...x, conditions, match_type: matchType } : x)))}
          />
        </div>
      ))}
      {!readOnly && (
        <button className="btn" style={{ marginTop: 6 }} onClick={() => onChange([...branches, { branch_label: "", match_type: "all", conditions: [] }])}>
          + Add branch
        </button>
      )}
    </div>
  );
}

function TransformEditor({ set, readOnly, onChange }: { set: Record<string, string>; readOnly: boolean; onChange: (set: Record<string, string>) => void }) {
  const entries = Object.entries(set);
  return (
    <div className="form-field">
      <label>Set (context key → template)</label>
      {entries.map(([k, v], i) => (
        <div key={i} className="graph-condition-row">
          <input
            disabled={readOnly}
            placeholder="key"
            value={k}
            onChange={(e) => {
              const next: Record<string, string> = {};
              entries.forEach(([kk, vv], ii) => {
                next[ii === i ? e.target.value : kk] = vv;
              });
              onChange(next);
            }}
          />
          <input
            disabled={readOnly}
            placeholder="e.g. {{trigger_input}}"
            value={v}
            onChange={(e) => {
              const next = { ...set, [k]: e.target.value };
              onChange(next);
            }}
          />
          {!readOnly && (
            <button
              className="btn"
              style={{ fontSize: 12, padding: "2px 6px" }}
              onClick={() => {
                const next = { ...set };
                delete next[k];
                onChange(next);
              }}
            >
              ×
            </button>
          )}
        </div>
      ))}
      {!readOnly && (
        <button className="btn" style={{ marginTop: 6 }} onClick={() => onChange({ ...set, [`field_${entries.length + 1}`]: "" })}>
          + Add field
        </button>
      )}
    </div>
  );
}

export function GraphRunsView({ graphId, onBack }: { graphId: string; onBack: () => void }) {
  const graphQuery = useQuery({ queryKey: ["executionGraph", graphId], queryFn: () => api.getExecutionGraph(graphId) });
  const runsQuery = useQuery({ queryKey: ["graphRuns", graphId], queryFn: () => api.listGraphRuns(graphId), refetchInterval: 4000 });
  const runs = runsQuery.data ?? [];
  const qc = useQueryClient();
  const [triggerInput, setTriggerInput] = useState("{}");
  const [expanded, setExpanded] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    qc.invalidateQueries({ queryKey: ["graphRuns", graphId] });
  }
  const startMutation = useMutation({
    mutationFn: () => api.startGraphRun(graphId, triggerInput),
    onSuccess: invalidate,
    onError: (e) => setError(e instanceof ApiError ? e.message : String(e)),
  });
  const resumeMutation = useMutation({ mutationFn: (id: string) => api.resumeGraphRun(id), onSuccess: invalidate, onError: (e) => setError(e instanceof ApiError ? e.message : String(e)) });
  const resolveMutation = useMutation({ mutationFn: ({ id, approve }: { id: string; approve: boolean }) => api.resolveGraphRunApproval(id, approve), onSuccess: invalidate, onError: (e) => setError(e instanceof ApiError ? e.message : String(e)) });
  const cancelMutation = useMutation({ mutationFn: (id: string) => api.cancelGraphRun(id), onSuccess: invalidate, onError: (e) => setError(e instanceof ApiError ? e.message : String(e)) });

  const graph = graphQuery.data;
  const canStart = graph?.status === "published";

  return (
    <div className="card">
      <button className="link-button" onClick={onBack}>
        ← {graph?.name ?? "Team"}
      </button>
      <h3>Runs{graph ? `: ${graph.name}` : ""}</h3>
      {error && (
        <div className="error-banner">
          {error} <button className="link-button" onClick={() => setError(null)}>dismiss</button>
        </div>
      )}
      {canStart ? (
        <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
          <textarea style={{ flex: 1 }} rows={2} value={triggerInput} onChange={(e) => setTriggerInput(e.target.value)} placeholder='Trigger input, e.g. {"deal_id":"..."}' />
          <button className="btn btn-primary" onClick={() => startMutation.mutate()} disabled={startMutation.isPending}>
            Start run
          </button>
        </div>
      ) : (
        <p style={{ color: "var(--text-muted)" }}>Publish this team before starting a run.</p>
      )}
      {runs.length === 0 && <div className="empty-state">No runs yet.</div>}
      {runs.map((r: GraphRun) => (
        <div key={r.id} className="graph-run-row">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div>
              <span className={`badge ${["completed"].includes(r.status) ? "badge-success" : ["failed", "cancelled"].includes(r.status) ? "badge-danger" : ""}`}>{r.status}</span>{" "}
              <button className="link-button" onClick={() => setExpanded(expanded === r.id ? null : r.id)}>
                {new Date(r.started_at).toLocaleString()}
              </button>
              <span style={{ color: "var(--text-muted)" }}> · {r.steps_executed} step(s)</span>
            </div>
            <div style={{ display: "flex", gap: 6 }}>
              {r.status === "waiting_approval" && (
                <>
                  <button className="btn" onClick={() => resolveMutation.mutate({ id: r.id, approve: true })}>Approve</button>
                  <button className="btn" onClick={() => resolveMutation.mutate({ id: r.id, approve: false })}>Reject</button>
                </>
              )}
              {(r.status === "paused" || r.status === "waiting_scheduled") && (
                <button className="btn" onClick={() => resumeMutation.mutate(r.id)}>Resume</button>
              )}
              {!["completed", "failed", "cancelled"].includes(r.status) && (
                <button className="btn" onClick={() => cancelMutation.mutate(r.id)}>Cancel</button>
              )}
            </div>
          </div>
          {r.error_message && <div style={{ color: "var(--danger)", fontSize: 12 }}>{r.error_message}</div>}
          {expanded === r.id && (
            <div className="table-wrap" style={{ marginTop: 8 }}>
              <table>
                <thead>
                  <tr>
                    <th>Node</th>
                    <th>Type</th>
                    <th>Status</th>
                    <th>Output</th>
                  </tr>
                </thead>
                <tbody>
                  {r.nodes.map((n) => (
                    <tr key={n.id}>
                      <td>{n.node_key}</td>
                      <td>{n.node_type}</td>
                      <td>{n.status}</td>
                      <td style={{ maxWidth: 320, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{n.error_message ?? n.output_json ?? ""}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
