/**
 * WP-0133 - the Guardrails screen lists guardrails, creates a custom one,
 * and edits a guardrail's configuration (saving a new version).
 *
 * Covers: REQ-GOV-007 (WP-0133)
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { GuardrailsAdmin } from "../features/settings/GuardrailsAdmin";
import { api } from "../lib/api";

vi.mock("../lib/api", async (orig) => {
  const actual = await orig<typeof import("../lib/api")>();
  return {
    ...actual,
    api: {
      listGuardrails: vi.fn(),
      createGuardrail: vi.fn(),
      editGuardrail: vi.fn(),
    },
  };
});

const pii = {
  id: "g1", workspace_id: "w", builtin_key: "pii_leak", name: "No PII leak",
  description: null, checker_type: "pii", is_builtin: true, current_version: 1,
  enabled: true, config_json: "{}", updated_at: "", updated_by: null,
};

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false } } }));
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.mocked(api.listGuardrails).mockResolvedValue([{ ...pii }]);
  vi.mocked(api.createGuardrail).mockResolvedValue({ ...pii, id: "g2", builtin_key: null, is_builtin: false, name: "Custom" });
  vi.mocked(api.editGuardrail).mockResolvedValue({ ...pii, current_version: 2 });
});

describe("GuardrailsAdmin", () => {
  it("lists guardrails with the built-in badge", async () => {
    render(<Providers><GuardrailsAdmin /></Providers>);
    await screen.findByText("No PII leak");
    expect(screen.getByText("Built-in")).toBeInTheDocument();
  });

  it("creates a custom guardrail", async () => {
    render(<Providers><GuardrailsAdmin /></Providers>);
    await screen.findByText("No PII leak");
    fireEvent.click(screen.getByRole("button", { name: "New guardrail" }));
    fireEvent.change(await screen.findByLabelText("Name"), { target: { value: "No profanity" } });
    fireEvent.click(screen.getByRole("button", { name: "Create" }));
    await waitFor(() =>
      expect(api.createGuardrail).toHaveBeenCalledWith(
        expect.objectContaining({ name: "No profanity", checker_type: "rule" }),
      ),
    );
  });

  it("edits a guardrail's configuration", async () => {
    render(<Providers><GuardrailsAdmin /></Providers>);
    await screen.findByText("No PII leak");
    fireEvent.click(screen.getByRole("button", { name: "Edit config" }));
    const ta = await screen.findByLabelText("Configuration for No PII leak");
    fireEvent.change(ta, { target: { value: '{"classes":["email"]}' } });
    fireEvent.click(screen.getByRole("button", { name: "Save new version" }));
    await waitFor(() =>
      expect(api.editGuardrail).toHaveBeenCalledWith(
        "g1",
        expect.objectContaining({ config_json: '{"classes":["email"]}' }),
      ),
    );
  });
});
