/**
 * PR 2 — Desktop Ontology Graph: shared graph engine.
 *
 * One module used by BOTH the Ontology Graph (Dependency Explorer) and
 * Object 360 (PR 3), mirroring the demo's shared `graphRender`.
 */
export {
  SYSTEM_NODE_TYPE_COLORS,
  SYSTEM_NODE_TYPE_LABELS,
  GRAPH_CLUSTER_ORDER,
  graphNodeKey,
  isAccessEdge,
  type GraphNode,
  type GraphEdge,
  type GraphModel,
} from "./graphTypes";
export { useGraphModel, type GraphDepth, type UseGraphModelOptions, type UseGraphModelResult } from "./useGraphModel";
export { layoutGraphPositions, type GraphLayoutOptions, type GraphPosition } from "./useGraphLayout";
export { useHoverDimming, type HoverDimming } from "./useHoverDimming";
export { GraphCanvas, type GraphCanvasProps } from "./GraphCanvas";
export { PillNode, type PillNodeProps } from "./PillNode";
export { ClusterBoundary, type ClusterBoundaryProps } from "./ClusterBoundary";
export { GraphToolbar, type GraphToolbarProps, type ComponentOption } from "./GraphToolbar";
export { GraphLegend, type GraphLegendProps } from "./GraphLegend";
export { Object360Graph, type Object360GraphProps, type Object360GraphData, type Object360RecordRef, type Object360InspectTarget, entityTypeToSystemType, entityTypeColor, recordNodeKey } from "./Object360Graph";
export { GraphInspector, type GraphInspectorProps, type InspectorGroup, type InspectorItem, type InspectorAction } from "./GraphInspector";
