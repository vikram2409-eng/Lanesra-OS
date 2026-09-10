import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AiAgentTargetType, AiAgentTriggerType } from "../../lib/types";

// AI & Agentic Layer, Phase 6b: schedule/webhook Triggers for one Agent
// or Pipeline - folded into that target's own row rather than a
// separate top-level screen (the same "lives on the thing it configures"
// convention Integration Jobs' own schedule already follows). Manual
// running is always available via each row's own "Run"/"Chat" button and
// isn't a stored Trigger.
export function AiTriggersPanel({ targetType, targetId }: { targetType: AiAgentTargetType; targetId: string }) {
  const queryClient = useQueryClient();
  const key = ["aiAgentTriggers", targetType, targetId];
  const triggersQuery = useQuery({ queryKey: key, queryFn: () => api.listAiAgentTriggers(targetType, targetId) });
  const triggers = triggersQuery.data ?? [];
  const [triggerType, setTriggerType] = useState<AiAgentTriggerType>("schedule");
  const [intervalMinutes, setIntervalMinutes] = useState(60);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: key });
  }

  const create = useMutation({
    mutationFn: () =>
      api.createAiAgentTrigger({
        target_type: targetType,
        target_id: targetId,
        trigger_type: triggerType,
        interval_minutes: triggerType === "schedule" ? intervalMinutes : null,
      }),
    onSuccess: () => {
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not add this trigger"),
  });
  const toggle = useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) => api.setAiAgentTriggerActive(id, isActive),
    onSuccess: invalidate,
  });
  const remove = useMutation({ mutationFn: (id: string) => api.deleteAiAgentTrigger(id), onSuccess: invalidate });

  return (
    <div style={{ marginTop: 8, fontSize: 13 }}>
      <b>Triggers</b>
      {triggers.length === 0 && <p style={{ color: "var(--text-muted)", margin: "4px 0" }}>None yet - always runnable manually regardless.</p>}
      {triggers.map((t) => (
        <div key={t.id} style={{ display: "flex", gap: 8, alignItems: "center", margin: "4px 0", flexWrap: "wrap" }}>
          <span>{t.trigger_type === "schedule" ? `Every ${t.interval_minutes} min` : "Webhook"}</span>
          <span className={`badge${t.is_active ? " badge-success" : ""}`}>{t.is_active ? "Active" : "Paused"}</span>
          {t.last_run_at && <span style={{ color: "var(--text-muted)" }}>last ran {new Date(t.last_run_at).toLocaleString()}</span>}
          <button className="btn btn-secondary" onClick={() => toggle.mutate({ id: t.id, isActive: !t.is_active })}>
            {t.is_active ? "Pause" : "Resume"}
          </button>
          <button className="btn btn-secondary" onClick={() => remove.mutate(t.id)}>
            Remove
          </button>
        </div>
      ))}
      {error && <div className="error-banner">{error}</div>}
      <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 6, flexWrap: "wrap" }}>
        <select value={triggerType} onChange={(e) => setTriggerType(e.target.value as AiAgentTriggerType)}>
          <option value="schedule">Schedule</option>
          <option value="webhook">Webhook</option>
        </select>
        {triggerType === "schedule" && (
          <label>
            every{" "}
            <input type="number" min={1} style={{ width: 70 }} value={intervalMinutes} onChange={(e) => setIntervalMinutes(Number(e.target.value))} /> min
          </label>
        )}
        <button className="btn btn-secondary" onClick={() => create.mutate()} disabled={create.isPending}>
          + Add trigger
        </button>
      </div>
      {triggerType === "webhook" && (
        <p style={{ color: "var(--text-muted)", marginTop: 4 }}>
          Call it with an API client key (scope <code>agents.trigger</code>): <code>POST /api/v1/{targetType === "agent" ? "agents" : "agent-pipelines"}/{targetId}/trigger</code>, body{" "}
          <code>{`{"input": "..."}`}</code>.
        </p>
      )}
    </div>
  );
}
