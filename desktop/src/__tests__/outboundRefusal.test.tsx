/**
 * WP-0191 - when the Team Workspace outbound guard refuses a connection's
 * address, the admin sees the server's message, naming the address and the
 * operator setting, where they typed the URL.
 *
 * Covers: REQ-PRE-019 (WP-0191 AC-3)
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { IntegrationHubAdmin } from "../features/settings/IntegrationHubAdmin";
import { api, ApiError } from "../lib/api";

vi.mock("../lib/api", async (orig) => {
  const actual = await orig<typeof import("../lib/api")>();
  return {
    ...actual,
    api: {
      listConnections: vi.fn(),
      createConnection: vi.fn(),
    },
  };
});

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false } } }));
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const refusal =
  "localhost resolves to 127.0.0.1 is on the server's internal network; the server operator must list it in LANESRA_OUTBOUND_ALLOW before Team Workspace can reach it";

describe("outbound refusals on save", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listConnections).mockResolvedValue([] as never);
  });

  it("shows the refused address and the setting in the connection form", async () => {
    vi.mocked(api.createConnection).mockRejectedValue(new ApiError({ kind: "validation", message: refusal }));
    render(<Providers><IntegrationHubAdmin onOpenHelp={() => {}} /></Providers>);
    fireEvent.click(screen.getByRole("button", { name: /^Connections/ }));
    fireEvent.click(await screen.findByRole("button", { name: "+ New connection" }));
    const field = (label: string) => screen.getByText(label, { selector: "label" }).parentElement!.querySelector("input")!;
    fireEvent.change(field("Name"), { target: { value: "Local Ollama" } });
    fireEvent.change(field("Base URL / host"), { target: { value: "http://localhost:11434" } });
    fireEvent.click(screen.getByRole("button", { name: "Create connection" }));

    expect(await screen.findByText(/LANESRA_OUTBOUND_ALLOW/)).toBeInTheDocument();
    expect(screen.getByText(/127\.0\.0\.1/)).toBeInTheDocument();
  });
});
