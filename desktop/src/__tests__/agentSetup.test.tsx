/**
 * WP-0160 - an agent's whole configuration is grouped under one "Set up"
 * panel instead of ten separate per-row modal buttons.
 *
 * Stage 1a grouped the ten per-row modals into the panel. Stage 1b splits the
 * single "Agent" tab into the plain-language groups (Identity / Job / Tools &
 * actions / Data access) and unifies the Job save, so instructions, tone and
 * the persona (role/goal/background/reasoning mode) save in one action - the
 * Persona -> Job merge (Gate-0 Q1).
 *
 * Covers: REQ-SETUP-001 (WP-0160)
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AgentSetupPanel } from "../features/settings/AiAgentsAdmin";
import { api } from "../lib/api";
import type { AiAgentDefinition } from "../lib/types";

vi.mock("../lib/api", async (orig) => {
  const actual = await orig<typeof import("../lib/api")>();
  return {
    ...actual,
    api: {
      ...actual.api,
      getAiAgentPolicy: vi.fn(),
      listUsers: vi.fn(),
      updateAiAgent: vi.fn(),
      setAiAgentPersona: vi.fn(),
    },
  };
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
  vi.mocked(api.listUsers).mockResolvedValue([]);
  vi.mocked(api.updateAiAgent).mockResolvedValue(agent);
  vi.mocked(api.setAiAgentPersona).mockResolvedValue(agent);
});

describe("AgentSetupPanel (WP-0160)", () => {
  it("groups the whole agent setup under one panel with the nine plain-language group tabs", () => {
    renderPanel();
    expect(screen.getByRole("heading", { name: /set up/i })).toBeInTheDocument();
    for (const label of [
      "Identity",
      "Job",
      "Knowledge & memory",
      "Tools & actions",
      "Data access",
      "Safety",
      "Model & cost",
      "Test & publish",
      "Run & triggers",
    ]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
  });

  it("puts instructions, tone and the persona together under one Job group (the Persona -> Job merge)", () => {
    renderPanel();
    fireEvent.click(screen.getByRole("button", { name: "Job" }));
    expect(screen.getByRole("button", { name: "Save job" })).toBeInTheDocument();
    expect(screen.getByText("Purpose / instructions")).toBeInTheDocument();
    expect(screen.getByText("Role")).toBeInTheDocument();
    expect(screen.getByText("Goal")).toBeInTheDocument();
    expect(screen.getByText("Background")).toBeInTheDocument();
  });

  it("saves the Job group through update_ai_agent and set_ai_agent_persona in one action", async () => {
    renderPanel();
    fireEvent.click(screen.getByRole("button", { name: "Job" }));
    fireEvent.click(screen.getByRole("button", { name: "Save job" }));
    await waitFor(() => expect(api.updateAiAgent).toHaveBeenCalledWith("a1", expect.objectContaining({ system_prompt: "You chase invoices." })));
    await waitFor(() => expect(api.setAiAgentPersona).toHaveBeenCalledWith("a1", expect.objectContaining({ role: "", goal: "", background: "", reasoning_mode: "direct" })));
  });

  it("gives Tools & actions its own group with a tool checklist and save", () => {
    renderPanel();
    fireEvent.click(screen.getByRole("button", { name: "Tools & actions" }));
    expect(screen.getByRole("button", { name: "Save tools & actions" })).toBeInTheDocument();
    expect(screen.getByText("What this agent can do")).toBeInTheDocument();
  });
});
