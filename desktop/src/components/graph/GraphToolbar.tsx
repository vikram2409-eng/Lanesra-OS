/**
 * PR 2 — Desktop Ontology Graph: compact 8-group toolbar.
 *
 * React port of the demo's `og-toolbar-card` (app.js toolbar template +
 * styles.css). All controls in one card, 1–2 slim rows on desktop via
 * flex-wrap; stacks on mobile.
 *
 * Groups: Type | Component | Search | Find path | Zoom | Depth |
 * Impact/Lineage | Fields/Simulate/Copy link
 */
import { memo, useState } from "react";

import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Select } from "../ui/select";
import { Checkbox } from "../ui/checkbox";
import { Label } from "../ui/label";
import { SYSTEM_NODE_TYPE_LABELS } from "./graphTypes";
import { GRAPH_CLUSTER_ORDER } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";
import type { GraphDepth } from "./useGraphModel";

export interface ComponentOption {
  id: string;
  label: string;
}

export interface GraphToolbarProps {
  // Type + Component
  nodeType: SystemNodeType;
  onNodeTypeChange: (t: SystemNodeType) => void;
  componentId: string;
  onComponentChange: (id: string) => void;
  componentOptions: ComponentOption[];
  // Search
  searchQuery: string;
  onSearchQueryChange: (q: string) => void;
  searchResults: ComponentOption[];
  onSearchSelect: (opt: ComponentOption) => void;
  // Find path
  pathMode: boolean;
  onTogglePathMode: () => void;
  pathFromLabel?: string | null;
  pathToLabel?: string | null;
  onClearPath?: () => void;
  // Zoom
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFit: () => void;
  onReset: () => void;
  // Depth
  depth: GraphDepth;
  onDepthChange: (d: GraphDepth) => void;
  // Impact / Lineage
  showImpact: boolean;
  onToggleImpact: () => void;
  showLineage: boolean;
  onToggleLineage: () => void;
  // Fields / Simulate / Copy link
  showFields: boolean;
  onToggleFields: () => void;
  onSimulate?: () => void;
  onCopyLink?: () => void;
}

export const GraphToolbar = memo(function GraphToolbar(props: GraphToolbarProps) {
  const {
    nodeType, onNodeTypeChange,
    componentId, onComponentChange, componentOptions,
    searchQuery, onSearchQueryChange, searchResults, onSearchSelect,
    pathMode, onTogglePathMode, pathFromLabel, pathToLabel, onClearPath,
    zoom, onZoomIn, onZoomOut, onFit, onReset,
    depth, onDepthChange,
    showImpact, onToggleImpact, showLineage, onToggleLineage,
    showFields, onToggleFields, onSimulate, onCopyLink,
  } = props;

  const [searchFocused, setSearchFocused] = useState(false);

  return (
    <div className="og-toolbar-card" role="toolbar" aria-label="Graph tools">
      {/* Type */}
      <div className="tool-group">
        <Label className="tool-inline">
          Type{" "}
          <Select
            value={nodeType}
            onValueChange={(v) => onNodeTypeChange(v as SystemNodeType)}
            aria-label="Node type"
            options={GRAPH_CLUSTER_ORDER.map((t) => ({ value: t, label: SYSTEM_NODE_TYPE_LABELS[t] }))}
          />
        </Label>
      </div>

      {/* Component */}
      <div className="tool-group">
        <Label className="tool-inline">
          Component{" "}
          <Select
            value={componentId}
            onValueChange={(v) => onComponentChange(v)}
            aria-label="Component"
            placeholder="Select…"
            options={componentOptions.map((o) => ({ value: o.id, label: o.label }))}
          />
        </Label>
      </div>

      {/* Search */}
      <div className="tool-group">
        <div className="search-wrap" style={{ position: "relative", minWidth: 200 }}>
          <Input
            type="search"
            value={searchQuery}
            onChange={(e) => onSearchQueryChange(e.target.value)}
            onFocus={() => setSearchFocused(true)}
            onBlur={() => setTimeout(() => setSearchFocused(false), 150)}
            placeholder="Search components…"
            aria-label="Search components"
          />
          {searchFocused && searchResults.length > 0 && (
            <div className="ontology-search-results">
              {searchResults.map((r) => (
                <button key={r.id} type="button" onClick={() => onSearchSelect(r)}>
                  {r.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Find path */}
      <div className="tool-group">
        <Button
          variant={pathMode ? "default" : "secondary"}
          size="sm"
          onClick={onTogglePathMode}
          title="Click two nodes on the canvas to trace the shortest path between them"
        >
          Find path
        </Button>
        {(pathFromLabel || pathToLabel) && (
          <span className="path-fields" style={{ fontSize: 12, color: "var(--muted,#64748b)" }}>
            {pathFromLabel ?? "…"} → {pathToLabel ?? "…"}
            {onClearPath && (
              <Button variant="ghost" size="sm" onClick={onClearPath} aria-label="Clear path">
                ✕
              </Button>
            )}
          </span>
        )}
      </div>

      {/* Zoom */}
      <div className="tool-group">
        <div className="zoom-row" role="group" aria-label="Viewport" style={{ display: "flex", gap: 4 }}>
          <Button variant="outline" size="sm" onClick={onZoomOut} title="Zoom out" aria-label="Zoom out">
            −
          </Button>
          <Button variant="outline" size="sm" onClick={onFit} title="Fit graph to viewport">
            Fit
          </Button>
          <Button variant="outline" size="sm" onClick={onZoomIn} title="Zoom in" aria-label="Zoom in">
            +
          </Button>
          <Button variant="outline" size="sm" onClick={onReset} title="Reset zoom, pan, path and simulation">
            Reset
          </Button>
        </div>
        <span className="zoom-readout" data-zoom-readout>
          {Math.round(zoom * 100)}%
        </span>
      </div>

      {/* Depth */}
      <div className="tool-group">
        <Label className="tool-inline">
          Depth{" "}
          <Select
            value={depth}
            onValueChange={(v) => onDepthChange(v as GraphDepth)}
            aria-label="Depth"
            options={(["1", "2", "3", "all"] as GraphDepth[]).map((d) => ({
              value: d,
              label: d === "all" ? "All" : d,
            }))}
          />
        </Label>
      </div>

      {/* Impact / Lineage */}
      <div className="tool-group">
        <button
          type="button"
          className={`toggle-chip${showImpact ? " active" : ""}`}
          onClick={onToggleImpact}
          aria-pressed={showImpact}
        >
          Impact
        </button>
        <button
          type="button"
          className={`toggle-chip${showLineage ? " active" : ""}`}
          onClick={onToggleLineage}
          aria-pressed={showLineage}
        >
          Lineage
        </button>
      </div>

      {/* Fields / Simulate / Copy link */}
      <div className="tool-group">
        <Label className="ontology-fields-toggle">
          <Checkbox checked={showFields} onCheckedChange={() => onToggleFields()} />
          Fields
        </Label>
        {onSimulate && (
          <Button variant="default" size="sm" onClick={onSimulate} title="Simulate a change">
            Simulate change
          </Button>
        )}
        {onCopyLink && (
          <Button variant="secondary" size="sm" onClick={onCopyLink} title="Copy a shareable link to this view">
            Copy link
          </Button>
        )}
      </div>
    </div>
  );
});
