/**
 * Guided start for a new lane (WP-0722, REQ-LANE-024).
 * Covers: REQ-LANE-024 (WP-0722 AC-2).
 *
 * The dialog asks name, object and start point, then hands the choice to
 * its caller — it fetches nothing itself (the parent passes the workspace
 * objects), so it renders bare, with no QueryClient or api mock.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { NewLaneStart } from "./NewLaneStart";

const OBJECTS = [
  { key: "purchase_order", plural_label: "Purchase Orders" },
  { key: "cycle_count", plural_label: "Cycle Counts" },
];

function renderStart(props: Partial<Parameters<typeof NewLaneStart>[0]> = {}) {
  const onCreate = vi.fn();
  const onCancel = vi.fn();
  render(
    <NewLaneStart
      objects={OBJECTS}
      onCancel={onCancel}
      onCreate={onCreate}
      {...props}
    />,
  );
  return { onCreate, onCancel };
}

describe("NewLaneStart", () => {
  it("AC-2: offers blank / describe / template, with template disabled — no templates yet", () => {
    renderStart();
    expect(screen.getByText("Start blank — an empty lane on this object")).toBeVisible();
    expect(screen.getByText(/Describe it — the Admin Assistant drafts the lane/)).toBeVisible();
    const template = screen.getByRole("radio", { name: /Start from a template/ });
    expect(template).toBeDisabled();
    expect(screen.getByText(/no templates yet \(templates arrive with WP-0717\)/)).toBeVisible();
    // Describe's textarea stays hidden until describe is picked.
    expect(screen.queryByLabelText("Process description")).not.toBeInTheDocument();
  });

  it("AC-2: the object arrives pre-filled when the start comes from an object's page", () => {
    renderStart({ initialObjectKey: "cycle_count" });
    expect(screen.getByLabelText("Lane object")).toHaveValue("cycle_count");
  });

  it("AC-2: blank lands in the builder with name and object applied; an empty name becomes Untitled lane", () => {
    const { onCreate } = renderStart({ initialObjectKey: "purchase_order" });
    fireEvent.click(screen.getByRole("button", { name: "Create lane" }));
    expect(onCreate).toHaveBeenCalledWith({
      name: "Untitled lane",
      objectKey: "purchase_order",
      start: "blank",
      description: "",
    });
  });

  it("AC-2: a typed name is carried through", () => {
    const { onCreate } = renderStart({ initialObjectKey: "purchase_order" });
    fireEvent.change(screen.getByLabelText("Lane name"), { target: { value: "Count the bins" } });
    fireEvent.click(screen.getByRole("button", { name: "Create lane" }));
    expect(onCreate).toHaveBeenCalledWith(
      expect.objectContaining({ name: "Count the bins", start: "blank" }),
    );
  });

  it("AC-2: describe shows the description field and hands it over", () => {
    const { onCreate } = renderStart({ initialObjectKey: "purchase_order" });
    fireEvent.click(screen.getByRole("radio", { name: /Describe it/ }));
    fireEvent.change(screen.getByLabelText("Process description"), {
      target: { value: "Receive a purchase order. Verify the delivery." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create lane" }));
    expect(onCreate).toHaveBeenCalledWith({
      name: "Untitled lane",
      objectKey: "purchase_order",
      start: "describe",
      description: "Receive a purchase order. Verify the delivery.",
    });
  });

  it("AC-2: describe with an empty description is refused in the dialog, not sent", () => {
    const { onCreate } = renderStart({ initialObjectKey: "purchase_order" });
    fireEvent.click(screen.getByRole("radio", { name: /Describe it/ }));
    fireEvent.click(screen.getByRole("button", { name: "Create lane" }));
    expect(onCreate).not.toHaveBeenCalled();
    expect(screen.getByRole("alert")).toHaveTextContent(/empty description drafts nothing/);
  });

  it("AC-2: with no objects the create button is disabled and says why", () => {
    renderStart({ objects: [] });
    expect(screen.getByText(/No objects yet — create a Custom Object first/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Create lane" })).toBeDisabled();
  });

  it("AC-2: cancel closes the dialog without creating", () => {
    const { onCancel, onCreate } = renderStart();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("AC-2: a describe failure from the caller stays visible in the dialog", () => {
    renderStart({ error: "Drafting failed — no model configured." });
    expect(screen.getByRole("alert")).toHaveTextContent(/no model configured/);
  });
});
