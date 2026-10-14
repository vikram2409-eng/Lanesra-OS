/**
 * PR 3 — UI primitives smoke tests.
 *
 * Covers the PR 1 component library basics that the graph surfaces
 * depend on: Button variants/click, Card slots, and Dialog open/
 * Escape/overlay-close behavior.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "../components/ui/dialog";

describe("Button", () => {
  it("renders children and fires onClick", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Click me</Button>);
    const btn = screen.getByRole("button", { name: "Click me" });
    expect(btn).toHaveClass("ui-btn-default");
    fireEvent.click(btn);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("applies variant and size classes", () => {
    render(
      <Button variant="destructive" size="sm">
        Delete
      </Button>,
    );
    const btn = screen.getByRole("button", { name: "Delete" });
    expect(btn).toHaveClass("ui-btn-destructive");
    expect(btn).toHaveClass("ui-btn-size-sm");
  });

  it("forwards the disabled state", () => {
    render(<Button disabled>Off</Button>);
    expect(screen.getByRole("button", { name: "Off" })).toBeDisabled();
  });
});

describe("Card", () => {
  it("renders title, description, and content slots", () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Card title</CardTitle>
          <CardDescription>Card description</CardDescription>
        </CardHeader>
        <CardContent>Card body</CardContent>
      </Card>,
    );
    expect(screen.getByText("Card title")).toBeInTheDocument();
    expect(screen.getByText("Card description")).toBeInTheDocument();
    expect(screen.getByText("Card body")).toBeInTheDocument();
  });
});

describe("Dialog", () => {
  function Harness({ onOpenChange }: { onOpenChange: (open: boolean) => void }) {
    const [open, setOpen] = useState(true);
    return (
      <Dialog
        open={open}
        onOpenChange={(o) => {
          setOpen(o);
          onOpenChange(o);
        }}
      >
        <DialogContent>
          <DialogTitle>Dialog title</DialogTitle>
          <DialogDescription>Dialog description</DialogDescription>
          <button type="button">Inner button</button>
        </DialogContent>
      </Dialog>
    );
  }

  it("renders content when open and closes on Escape", () => {
    const onOpenChange = vi.fn();
    render(<Harness onOpenChange={onOpenChange} />);
    expect(screen.getByText("Dialog title")).toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("closes on overlay click", () => {
    const onOpenChange = vi.fn();
    render(<Harness onOpenChange={onOpenChange} />);
    // Dialog portals to document.body, so query the document.
    const overlay = document.querySelector(".ui-dialog-overlay");
    expect(overlay).not.toBeNull();
    fireEvent.click(overlay!);
    expect(onOpenChange).toHaveBeenCalledWith(false);
    document.body.innerHTML = "";
  });

  it("renders nothing when closed", () => {
    render(
      <Dialog open={false} onOpenChange={() => {}}>
        <DialogContent>
          <DialogTitle>Hidden</DialogTitle>
        </DialogContent>
      </Dialog>,
    );
    expect(screen.queryByText("Hidden")).toBeNull();
  });
});
