/**
 * WP-0106 - the Security & sign-in admin screen shows the settings and
 * active sessions, saves an edit, and signs out all sessions.
 *
 * Covers: REQ-PRE-019 (WP-0106 AC-2, AC-4)
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SecurityAdmin } from "../features/settings/SecurityAdmin";
import { api } from "../lib/api";

vi.mock("../lib/api", async (orig) => {
  const actual = await orig<typeof import("../lib/api")>();
  return {
    ...actual,
    api: {
      getSecuritySettings: vi.fn(),
      updateSecuritySettings: vi.fn(),
      listActiveSessions: vi.fn(),
      signOutAllSessions: vi.fn(),
    },
  };
});

const settings = {
  workspace_id: "w",
  lockout_max_failures: 10,
  lockout_minutes: 15,
  session_lifetime_hours: 336,
  session_idle_minutes: 0,
  public_base_url: null,
  updated_at: "",
  updated_by: null,
};

const session = {
  id: "s1",
  user_id: "u1",
  username: "admin",
  display_name: "Admin",
  created_at: "2026-01-01T00:00:00Z",
  last_seen_at: "2026-01-01T00:00:00Z",
  expires_at: "2026-02-01T00:00:00Z",
  current: true,
};

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { retry: false } } }));
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.mocked(api.getSecuritySettings).mockResolvedValue({ ...settings });
  vi.mocked(api.listActiveSessions).mockResolvedValue([{ ...session }]);
  vi.mocked(api.updateSecuritySettings).mockResolvedValue({ ...settings, lockout_max_failures: 5 });
  vi.mocked(api.signOutAllSessions).mockResolvedValue({ ended: 1 });
});

describe("SecurityAdmin", () => {
  it("shows the settings and the active session", async () => {
    render(<Providers><SecurityAdmin /></Providers>);
    await screen.findByText(/10 failures, 15 minute lockout/);
    expect(screen.getByText("This device")).toBeInTheDocument();
  });

  it("saves an edited lockout threshold", async () => {
    render(<Providers><SecurityAdmin /></Providers>);
    await screen.findByText(/10 failures/);
    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    const input = await screen.findByLabelText("Max failed sign-ins before lockout");
    fireEvent.change(input, { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(api.updateSecuritySettings).toHaveBeenCalledWith(
        expect.objectContaining({ lockout_max_failures: 5 }),
      ),
    );
  });

  it("signs out all sessions after confirmation", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    render(<Providers><SecurityAdmin /></Providers>);
    await screen.findByText("This device");
    fireEvent.click(screen.getByRole("button", { name: "Sign out all sessions" }));
    await waitFor(() => expect(api.signOutAllSessions).toHaveBeenCalled());
  });
});
