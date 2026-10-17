/**
 * Accessibility tests for UI primitives (WP-0014, REQ-PRE-011).
 *
 * Uses vitest-axe to run axe-core against rendered components.
 * Covers: REQ-PRE-011 (WP-0014 AC-2)
 */
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { axe } from "vitest-axe";

import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";

describe("UI primitives accessibility", () => {
  it("Badge has no axe violations", async () => {
    const { container } = render(<Badge>Active</Badge>);
    expect(await axe(container)).toHaveNoViolations();
  });

  it("Button has no axe violations", async () => {
    const { container } = render(<Button>Save</Button>);
    expect(await axe(container)).toHaveNoViolations();
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
    const results = await axe(container);
    expect(results.violations.length).toBeGreaterThan(0);
    expect(results.violations.some((v) => v.id === "image-alt")).toBe(true);
  });
});
