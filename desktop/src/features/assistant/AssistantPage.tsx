import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { ChatPanel } from "../../components/ChatPanel";
import { agentUsableBy } from "../../lib/aiAgents";
import type { User } from "../../lib/types";

// AI & Agentic Layer, Phase 6: the main "Assistant" nav page (any user)
// gains a picker for any active AI Agent Foundry agent this user is
// allowed to use, above the Phase 5 default general-purpose records
// assistant - unchanged otherwise. `agentUsableBy` mirrors
// chat_service::agent_requires_admin client-side so an admin-only agent
// never even shows for a non-admin (the server re-checks regardless).
export function AssistantPage({ user }: { user: User }) {
  const isAdmin = user.roles.includes("Administrator");
  const agentsQuery = useQuery({ queryKey: ["aiAgents", "picker"], queryFn: () => api.listAiAgents(true) });
  const usableAgents = (agentsQuery.data ?? []).filter((a) => agentUsableBy(a, isAdmin));
  const [selectedId, setSelectedId] = useState("");
  const selectedAgent = usableAgents.find((a) => a.id === selectedId);

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12, flexWrap: "wrap", gap: 8 }}>
        <h2 style={{ margin: 0 }}>Assistant</h2>
        {usableAgents.length > 0 && (
          <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13 }}>
            Talking to
            <select value={selectedId} onChange={(e) => setSelectedId(e.target.value)}>
              <option value="">General assistant</option>
              {usableAgents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.icon} {a.name}
                </option>
              ))}
            </select>
          </label>
        )}
      </div>
      {selectedAgent ? <ChatPanel agentId={selectedAgent.id} /> : <ChatPanel mode="records" />}
    </div>
  );
}
