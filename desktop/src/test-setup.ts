/**
 * PR 3 — vitest setup.
 *
 * - jest-dom matchers
 * - ResizeObserver mock (GraphCanvas measures its container; jsdom lacks it)
 * - Pointer capture mocks (canvas drag-pan; jsdom lacks them)
 */
import "@testing-library/jest-dom/vitest";
import "vitest-axe/extend-expect";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";


// jsdom + explicit imports (no vitest globals): wire up RTL auto-cleanup.
afterEach(() => {
  cleanup();
});


class ResizeObserverMock {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
};


(globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = ResizeObserverMock;


if (typeof Element !== "undefined") {
  const proto = Element.prototype as unknown as Record<string, unknown>;
  if (typeof proto.setPointerCapture !== "function") {
    proto.setPointerCapture = () => {};
  }
  if (typeof proto.releasePointerCapture !== "function") {
    proto.releasePointerCapture = () => {};
  }
  if (typeof proto.hasPointerCapture !== "function") {
    proto.hasPointerCapture = () => false;
  }
}
