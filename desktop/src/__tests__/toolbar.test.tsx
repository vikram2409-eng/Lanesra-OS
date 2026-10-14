/**
 * PR 3 — GraphToolbar regression tests.
 *
 * Verifies the compact 8-group toolbar: all groups render, zoom/fit/
 * reset callbacks fire, the zoom readout tracks state, depth changes
 * propagate, impact/lineage/fields toggles fire, find-path mode toggles,
 * and search input/selects behave.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GraphToolbar, type GraphToolbarProps } from "../components/graph/GraphToolbar";

function baseProps(overrides: Partial<GraphToolbarProps> = {}): GraphToolbarProps {
  return {
    nodeType: "custom_object",
    onNodeTypeChange: vi.fn(),
    componentId: "companies",
    onComponentChange: vi.fn(),
    componentOptions: [{ id: "companies", label: "Companies" }],
    searchQuery: "",
    onSearchQueryChange: vi.fn(),
    searchResults: [],
    onSearchSelect: vi.fn(),
    pathMode: false,
    onTogglePathMode: vi.fn(),
    zoom: 1,
    onZoomIn: vi.fn(),
    onZoomOut: vi.fn(),
    onFit: vi.fn(),
    onReset: vi.fn(),
    depth: "1",
    onDepthChange: vi.fn(),
    showImpact: false,
    onToggleImpact: vi.fn(),
    showLineage: false,
    onToggleLineage: vi.fn(),
    showFields: false,
    onToggleFields: vi.fn(),
    ...overrides,
  };
}

describe("GraphToolbar", () => {
  it("renders all 8 tool groups", () => {
    const { container } = render(<GraphToolbar {...baseProps()} />);
    expect(container.querySelectorAll(".og-toolbar-card .tool-group")).toHaveLength(8);
  });

  it("fires zoom callbacks and shows the zoom readout", () => {
    const props = baseProps({ zoom: 1.5 });
    const { container } = render(<GraphToolbar {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Zoom in" }));
    expect(props.onZoomIn).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Zoom out" }));
    expect(props.onZoomOut).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Fit" }));
    expect(props.onFit).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(props.onReset).toHaveBeenCalledTimes(1);
    expect(container.querySelector("[data-zoom-readout]")?.textContent).toBe("150%");
  });

  it("propagates depth changes", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    fireEvent.change(screen.getByLabelText("Depth"), { target: { value: "3" } });
    expect(props.onDepthChange).toHaveBeenCalledWith("3");
  });

  it("toggles find-path mode", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Find path" }));
    expect(props.onTogglePathMode).toHaveBeenCalledTimes(1);
  });

  it("toggles impact / lineage chips", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    const impact = screen.getByRole("button", { name: "Impact" });
    expect(impact).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(impact);
    expect(props.onToggleImpact).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Lineage" }));
    expect(props.onToggleLineage).toHaveBeenCalledTimes(1);
  });

  it("toggles the fields checkbox", () => {
    const props = baseProps();
    const { container } = render(<GraphToolbar {...props} />);
    const checkbox = container.querySelector('input[type="checkbox"]') as HTMLInputElement;
    expect(checkbox).not.toBeNull();
    fireEvent.click(checkbox);
    expect(props.onToggleFields).toHaveBeenCalledTimes(1);
  });

  it("forwards search input", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    fireEvent.change(screen.getByLabelText("Search components"), { target: { value: "comp" } });
    expect(props.onSearchQueryChange).toHaveBeenCalledWith("comp");
  });

  it("fires simulate and copy-link when provided", () => {
    const props = baseProps({ onSimulate: vi.fn(), onCopyLink: vi.fn() });
    render(<GraphToolbar {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Simulate change" }));
    expect(props.onSimulate).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Copy link" }));
    expect(props.onCopyLink).toHaveBeenCalledTimes(1);
  });

  it("shows the path banner with clear action when a path is set", () => {
    const props = baseProps({ pathFromLabel: "A", pathToLabel: "B", onClearPath: vi.fn() });
    render(<GraphToolbar {...props} />);
    expect(screen.getByText(/A → B/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Clear path" }));
    expect(props.onClearPath).toHaveBeenCalledTimes(1);
  });
});
