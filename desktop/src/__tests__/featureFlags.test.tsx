/**
 * WP-0010 — workspace feature flags in the UI.
 *
 * A flagged entry point renders only when its flag is on, and stays hidden
 * while the flag list is loading or has failed to load. The admin screen
 * lists flags and sends overrides.
 */
// Covers: REQ-PRE-006 (WP-0010 AC-1, AC-3)
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { FeatureFlagsAdmin } from "../features/settings/FeatureFlagsAdmin";
import { api } from "../lib/api";
import type { FeatureFlagList } from "../lib/types";
import { useFeature } from "../lib/useFeature";

vi.mock("../lib/api", () => ({
  api: {
    listFeatureFlags: vi.fn(),
    setFeatureFlagOverride: vi.fn(),
  },
  ApiError: class ApiError extends Error {},
}));

function list(enabled: boolean, overrides: Partial<FeatureFlagList> = {}): FeatureFlagList {
  return {
    developer_mode: true,
    flags: [
      {
        key: "playground.shell",
        description: "Agent Playground: build, chat and trace panes",
        work_package: "WP-0203",
        stage: "building",
        enabled,
        overridden: enabled,
        can_toggle: true,
      },
    ],
    ...overrides,
  };
}

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false } } }));
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function PlaygroundEntry() {
  const on = useFeature("playground.shell");
  return <div>{on ? <button>Open Playground</button> : <span>no playground</span>}</div>;
}

describe("useFeature", () => {
  beforeEach(() => vi.clearAllMocks());

  it("hides the entry point when the flag is off", async () => {
    vi.mocked(api.listFeatureFlags).mockResolvedValue(list(false));
    render(<Providers><PlaygroundEntry /></Providers>);
    await waitFor(() => expect(api.listFeatureFlags).toHaveBeenCalled());
    expect(await screen.findByText("no playground")).toBeInTheDocument();
    expect(screen.queryByText("Open Playground")).not.toBeInTheDocument();
  });

  it("shows the entry point when the flag is on", async () => {
    vi.mocked(api.listFeatureFlags).mockResolvedValue(list(true));
    render(<Providers><PlaygroundEntry /></Providers>);
    expect(await screen.findByText("Open Playground")).toBeInTheDocument();
  });

  it("stays hidden when the flag list fails to load", async () => {
    vi.mocked(api.listFeatureFlags).mockRejectedValue(new Error("offline"));
    render(<Providers><PlaygroundEntry /></Providers>);
    await waitFor(() => expect(api.listFeatureFlags).toHaveBeenCalled());
    expect(screen.getByText("no playground")).toBeInTheDocument();
  });

  it("is false for a key the registry does not know", async () => {
    vi.mocked(api.listFeatureFlags).mockResolvedValue(list(true));
    function Unknown() {
      return <span>{useFeature("no_such.flag") ? "on" : "off"}</span>;
    }
    render(<Providers><Unknown /></Providers>);
    await waitFor(() => expect(api.listFeatureFlags).toHaveBeenCalled());
    expect(screen.getByText("off")).toBeInTheDocument();
  });
});

describe("FeatureFlagsAdmin", () => {
  beforeEach(() => vi.clearAllMocks());

  it("lists flags and sends an override when toggled", async () => {
    vi.mocked(api.listFeatureFlags).mockResolvedValue(list(false));
    vi.mocked(api.setFeatureFlagOverride).mockResolvedValue(list(true));
    render(<Providers><FeatureFlagsAdmin /></Providers>);
    const box = await screen.findByLabelText("Turn playground.shell on");
    expect(screen.getByText(/Developer mode is on/)).toBeInTheDocument();
    fireEvent.click(box);
    await waitFor(() =>
      expect(api.setFeatureFlagOverride).toHaveBeenCalledWith({ key: "playground.shell", enabled: true }),
    );
    expect(await screen.findByText("Reset to default")).toBeInTheDocument();
  });

  it("says so when nothing can be changed", async () => {
    vi.mocked(api.listFeatureFlags).mockResolvedValue({ developer_mode: false, flags: [] });
    render(<Providers><FeatureFlagsAdmin /></Providers>);
    expect(await screen.findByText("No features are waiting to be turned on.")).toBeInTheDocument();
  });
});
