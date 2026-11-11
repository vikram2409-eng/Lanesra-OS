/**
 * WP-0160 Stage 1a - an agent's whole configuration is grouped under one
 * "Set up" panel instead of ten separate per-row modal buttons, with Persona
 * folded into the agent's job (the Persona -> Job merge).
 *
 * Covers: REQ-SETUP-001 (WP-0160)
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AgentSetupPanel } from "../features/settings/AiAgentsAdmin";
import { api } from "../lib/api";
import type { AiAgentDefinition } from "../lib/types";

vi.mock("../lib/api", async (orig) => {
  const actual = await orig<typeof import("../lib/api")>();
  return { ...actual, api: { ...actual.api, getAiAgentPolicy: vi.fn() } };
});

const agent = {
  id: "a1",
  workspace_id: "w",
  name: "Invoice chaser",
  description: null,
  icon: "🤖",
  system_prompt: "You chase invoices.",
  action_names: [],
  delegate_agent_ids: [],
  skill_ids: [],
  object_type_scopes: [],
  action_type_ids: [],
  is_active: true,
  role: "",
  goal: "",
  background: "",
  reasoning_mode: "direct",
  model_routing: null,
  acts_as_user_id: null,
  current_version_id: null,
  created_at: "",
  created_by: null,
  updated_at: "",
  updated_by: null,
} as unknown as AiAgentDefinition;

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false } } }));
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function renderPanel() {
  render(
    <Providers>
      <AgentSetupPanel
        agent={agent}
        agents={[agent]}
        skills={[]}
        providers={[]}
        connectorTools={[]}
        mcpTools={[]}
        onClose={() => {}}
        onOpenHelp={() => {}}
      />
    </Providers>,
  );
}

beforeEach(() => {
  vi.mocked(api.getAiAgentPolicy).mockResolvedValue(null);
});

describe("AgentSetupPanel (WP-0160 Stage 1a)", () => {
  it("groups the whole agent setup under one panel with the eight group tabs", () => {
    renderPanel();
    expect(screen.getByRole("heading", { name: /set up/i })).toBeInTheDocument();
    for (const label of [
      "Agent",
      "Persona",
      "Knowledge & memory",
      "Safety",
      "Model & cost",
      "Identity",
      "Test & publish",
      "Run & triggers",
    ]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
  });

  it("keeps Persona inside the agent's setup (the Persona -> Job merge)", () => {
    renderPanel();
    fireEvent.click(screen.getByRole("button", { name: "Persona" }));
    expect(screen.getByRole("button", { name: "Save persona" })).toBeInTheDocument();
    expect(screen.getByText("Role")).toBeInTheDocument();
  });
});
