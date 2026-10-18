/**
 * Accessibility tests for UI primitives (WP-0014, REQ-PRE-011).
 *
 * Uses axe-core directly against rendered components.
 * Covers: REQ-PRE-011 (WP-0014 AC-2)
 */
import { render } from "@testing-library/react";
import axe from "axe-core";
import { describe, expect, it } from "vitest";

import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";

async function checkA11y(container: HTMLElement) {
  const results = await axe.run(container);
  return results.violations;
}

describe("UI primitives accessibility", () => {
  it("Badge has no axe violations", async () => {
    const { container } = render(<Badge>Active</Badge>);
    expect(await checkA11y(container)).toEqual([]);
  });

  it("Button has no axe violations", async () => {
    const { container } = render(<Button>Save</Button>);
    expect(await checkA11y(container)).toEqual([]);
  });

  it("flags a planted violation (image without alt)", async () => {
    // This test proves the axe integration catches violations: it renders
    // deliberately inaccessible markup and expects axe to report it.
    // Covers: REQ-PRE-011 (WP-0014 AC-2) — planted violation check.
    const { container } = render(
      <div>
        {/* eslint-disable-next-line jsx-a11y/alt-text */}
        <img src="x.png" />
      </div>,
    );
    const violations = await checkA11y(container);
    expect(violations.length).toBeGreaterThan(0);
    expect(violations.some((v) => v.id === "image-alt")).toBe(true);
  });
});
