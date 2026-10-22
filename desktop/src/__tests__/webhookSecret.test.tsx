/**
 * WP-0183 - the webhook signing secret is shown once, at creation and at
 * rotation, and is never recoverable from the list.
 *
 * Covers: REQ-INT-002 (WP-0183 AC-2, AC-3)
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { IntegrationHubAdmin } from "../features/settings/IntegrationHubAdmin";
import { api } from "../lib/api";

vi.mock("../lib/api", async (orig) => {
  const actual = await orig<typeof import("../lib/api")>();
  return {
    ...actual,
    api: {
      listWebhooks: vi.fn(),
      listConnections: vi.fn(),
      createWebhook: vi.fn(),
      rotateWebhookSecret: vi.fn(),
    },
  };
});

const webhook = {
  id: "wh1", workspace_id: "w", name: "Orders feed", connection_id: "c1", endpoint_url: null,
  event_types: ["record.created"], object_scope: null, filter_json: null, payload_version: "v1",
  has_secret: true, retry_policy_json: "{}", status: "active",
  created_at: "", created_by: null, updated_at: "", updated_by: null,
};

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false } } }));
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

async function openWebhooksTab() {
  render(<Providers><IntegrationHubAdmin onOpenHelp={() => {}} /></Providers>);
  fireEvent.click(screen.getByRole("button", { name: /^Webhooks/ }));
  await screen.findByText("Orders feed");
}

describe("webhook signing secret", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listWebhooks).mockResolvedValue([webhook] as never);
    vi.mocked(api.listConnections).mockResolvedValue([{ id: "c1", name: "Primary" }] as never);
  });

  it("shows the signing secret once after a webhook is created (AC-2)", async () => {
    vi.mocked(api.createWebhook).mockResolvedValue({ ...webhook, id: "wh2", signing_secret: "s3cr3t-created" } as never);
    await openWebhooksTab();
    fireEvent.click(screen.getByRole("button", { name: "+ New webhook" }));
    const nameInput = screen.getByText("Name", { selector: "label" }).parentElement!.querySelector("input")!;
    fireEvent.change(nameInput, { target: { value: "New hook" } });
    fireEvent.click(screen.getByLabelText("record.created"));
    fireEvent.click(screen.getByRole("button", { name: "Create webhook" }));

    expect(await screen.findByText("s3cr3t-created")).toBeInTheDocument();
    expect(screen.getByText(/never shown again/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(screen.queryByText("s3cr3t-created")).not.toBeInTheDocument();
  });

  it("rotating asks first, then shows the new secret once (AC-3)", async () => {
    vi.mocked(api.rotateWebhookSecret).mockResolvedValue("s3cr3t-rotated");
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    await openWebhooksTab();

    fireEvent.click(screen.getByRole("button", { name: "Rotate secret" }));
    expect(api.rotateWebhookSecret).not.toHaveBeenCalled();

    confirmSpy.mockReturnValue(true);
    fireEvent.click(screen.getByRole("button", { name: "Rotate secret" }));
    await waitFor(() => expect(api.rotateWebhookSecret).toHaveBeenCalledWith("wh1"));
    expect(await screen.findByText("s3cr3t-rotated")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(screen.queryByText("s3cr3t-rotated")).not.toBeInTheDocument();
    confirmSpy.mockRestore();
  });
});
