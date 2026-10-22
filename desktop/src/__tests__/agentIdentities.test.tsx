/**
 * WP-0109 — the Agent Identities screen and the identity picker on triggers.
 *
 * Both appear only while the `agents.identities_ui` flag is on. The picker
 * sends the chosen identity (or none) for that trigger; the screen issues a
 * credential and shows its token once.
 */
// Covers: REQ-GOV-003 (WP-0109)
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AgentIdentitiesAdmin } from "../features/settings/AgentIdentitiesAdmin";
import { AiTriggersPanel } from "../features/settings/AiTriggersPanel";
import { api } from "../lib/api";
import type { AgentIdentity, AiAgentTrigger, FeatureFlagList } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: {
    listFeatureFlags: vi.fn(),
    listAiAgentTriggers: vi.fn(),
    createAiAgentTrigger: vi.fn(),
    setAiAgentTriggerActive: vi.fn(),
    deleteAiAgentTrigger: vi.fn(),
    setAiAgentTriggerIdentity: vi.fn(),
    listAgentIdentities: vi.fn(),
    listUsers: vi.fn(),
    listAccessRoles: vi.fn(),
    setAgentIdentityActive: vi.fn(),
    deleteAgentIdentity: vi.fn(),
    listAgentCredentials: vi.fn(),
    issueAgentCredential: vi.fn(),
    rotateAgentCredential: vi.fn(),
    revokeAgentCredential: vi.fn(),
  },
  ApiError: class ApiError extends Error {},
}));

const mocked = api as unknown as Record<string, ReturnType<typeof vi.fn>>;

function flags(enabled: boolean): FeatureFlagList {
  return {
    developer_mode: true,
    flags: [
      {
        key: "agents.identities_ui",
        description: "Agent Identities screen and trigger identity picker",
        work_package: "WP-0109",
        stage: "building",
        enabled,
        overridden: enabled,
        can_toggle: true,
      },
    ],
  };
}

const identity: AgentIdentity = {
  id: "ai_1",
  workspace_id: "ws",
  name: "Nightly reports",
  description: null,
  owner_user_id: "u_admin",
  is_administrator: false,
  access_role_ids: [],
  status: "active",
  created_at: "2026-10-22T00:00:00Z",
  created_by: "u_admin",
  updated_at: "2026-10-22T00:00:00Z",
  updated_by: "u_admin",
};

const trigger: AiAgentTrigger = {
  id: "t_1",
  workspace_id: "ws",
  target_type: "agent",
  target_id: "a_1",
  trigger_type: "schedule",
  interval_minutes: 60,
  is_active: true,
  last_run_at: null,
  created_at: "2026-10-22T00:00:00Z",
  created_by: "u_admin",
  agent_identity_id: null,
};

function wrap(children: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={client}>{children}</QueryClientProvider>);
}

beforeEach(() => {
  vi.clearAllMocks();
  mocked.listAiAgentTriggers.mockResolvedValue([trigger]);
  mocked.listAgentIdentities.mockResolvedValue([identity]);
  mocked.listUsers.mockResolvedValue([
    { id: "u_admin", workspace_id: "ws", username: "admin", display_name: "Admin", is_active: true, roles: [], created_at: "", updated_at: "" },
  ]);
  mocked.listAccessRoles.mockResolvedValue([]);
  mocked.listAgentCredentials.mockResolvedValue([]);
});

describe("trigger identity picker", () => {
  it("is hidden while the flag is off", async () => {
    mocked.listFeatureFlags.mockResolvedValue(flags(false));
    wrap(<AiTriggersPanel targetType="agent" targetId="a_1" />);
    await screen.findByText("Every 60 min");
    expect(screen.queryByLabelText("Runs as Agent Identity")).toBeNull();
    expect(mocked.listAgentIdentities).not.toHaveBeenCalled();
  });

  it("sets and clears the identity a trigger runs as", async () => {
    mocked.listFeatureFlags.mockResolvedValue(flags(true));
    mocked.setAiAgentTriggerIdentity.mockResolvedValue({ ...trigger, agent_identity_id: "ai_1" });
    wrap(<AiTriggersPanel targetType="agent" targetId="a_1" />);
    const picker = await screen.findByLabelText("Runs as Agent Identity");
    await screen.findByRole("option", { name: "Nightly reports" });
    fireEvent.change(picker, { target: { value: "ai_1" } });
    await waitFor(() => expect(mocked.setAiAgentTriggerIdentity).toHaveBeenCalledWith("t_1", "ai_1"));
    fireEvent.change(picker, { target: { value: "" } });
    await waitFor(() => expect(mocked.setAiAgentTriggerIdentity).toHaveBeenCalledWith("t_1", null));
  });
});

describe("Agent Identities screen", () => {
  it("lists identities and shows an issued token once", async () => {
    mocked.issueAgentCredential.mockResolvedValue({
      credential: {
        id: "c_1",
        workspace_id: "ws",
        agent_identity_id: "ai_1",
        label: "ci",
        expires_at: "2026-11-22T00:00:00Z",
        revoked_at: null,
        revoked_by: null,
        last_used_at: null,
        created_at: "2026-10-22T00:00:00Z",
        created_by: "u_admin",
      },
      token: "shown-once-value",
    });
    wrap(<AgentIdentitiesAdmin />);
    await screen.findByText("Nightly reports");
    fireEvent.click(screen.getByRole("button", { name: "Credentials" }));
    fireEvent.change(await screen.findByLabelText("Credential label"), { target: { value: "ci" } });
    fireEvent.click(screen.getByRole("button", { name: "+ Issue credential" }));
    await waitFor(() => expect(mocked.issueAgentCredential).toHaveBeenCalledWith("ai_1", "ci", null));
    expect(await screen.findByText("shown-once-value")).toBeTruthy();
  });
});
