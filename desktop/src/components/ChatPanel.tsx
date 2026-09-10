import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../lib/api";
import type { ChatMessage, ChatMode } from "../lib/types";

// AI & Agentic Layer, Phase 5: the reusable chat UI for both the
// records-mode assistant (any user - look up and act on records) and the
// admin-mode assistant (Administrator only - configure the admin surface).
// Same component either way; `mode` picks which tool set and system prompt
// `chat_service` uses server-side, and which (user, mode) conversation this
// screen reads/appends to.
//
// Phase 6 adds a third addressing shape, `{agentId}` - chatting with one
// specific named AI Agent from the Foundry instead of a fixed persona.
// `resolveTarget` below is the one place that distinguishes the two.
//
// A tool call round trip is rendered as a small muted "used <tool>" line,
// expandable to the raw JSON result - not as raw provider JSON inline -
// so a multi-round exchange still reads like a conversation.

/** Best-effort tool name extraction from an assistant message's raw
 * `tool_calls` - shaped differently per provider (`ai_service::
 * complete_with_tools`'s own doc comment): Anthropic's is the response's
 * `content` block array (`{type:"tool_use", id, name, ...}` among plain
 * text blocks); an OpenAI-compatible provider's is `message.tool_calls`
 * itself (`{id, function:{name, ...}}`). Read structurally instead of
 * branching on provider, since either shape can show up here. */
function toolCallEntries(toolCalls: unknown): { id: string; name: string }[] {
  if (!Array.isArray(toolCalls)) return [];
  const entries: { id: string; name: string }[] = [];
  for (const item of toolCalls) {
    if (!item || typeof item !== "object") continue;
    const obj = item as Record<string, unknown>;
    const id = typeof obj.id === "string" ? obj.id : undefined;
    const name =
      typeof obj.name === "string" ? obj.name : typeof (obj.function as Record<string, unknown> | undefined)?.name === "string" ? ((obj.function as Record<string, unknown>).name as string) : undefined;
    if (id && name) entries.push({ id, name });
  }
  return entries;
}

function ChatBubble({ role, children }: { role: "user" | "assistant"; children: React.ReactNode }) {
  return (
    <div style={{ display: "flex", justifyContent: role === "user" ? "flex-end" : "flex-start", marginBottom: 8 }}>
      <div
        style={{
          maxWidth: "80%",
          padding: "8px 12px",
          borderRadius: 10,
          background: role === "user" ? "var(--accent)" : "var(--surface-2, rgba(127,127,127,0.15))",
          color: role === "user" ? "var(--accent-contrast, #fff)" : "inherit",
          whiteSpace: "pre-wrap",
          fontSize: 14,
        }}
      >
        {children}
      </div>
    </div>
  );
}

function ToolCallLine({ name, result }: { name: string; result?: string }) {
  return (
    <div style={{ margin: "2px 0 8px", fontSize: 12, color: "var(--text-muted)" }}>
      {result ? (
        <details>
          <summary style={{ cursor: "pointer" }}>used {name}</summary>
          <pre style={{ whiteSpace: "pre-wrap", fontSize: 11, margin: "4px 0 0" }}>{result}</pre>
        </details>
      ) : (
        <span>using {name}...</span>
      )}
    </div>
  );
}

export type ChatTarget = { mode: ChatMode } | { agentId: string };

export function ChatPanel(target: ChatTarget) {
  const queryClient = useQueryClient();
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const isAgent = "agentId" in target;
  const mode: ChatMode | "agent" = isAgent ? "agent" : target.mode;
  const historyKey = isAgent ? ["chatHistory", "agent", target.agentId] : ["chatHistory", target.mode];
  const history = useQuery({
    queryKey: historyKey,
    queryFn: () => (isAgent ? api.getAgentChatHistory(target.agentId) : api.getChatHistory(target.mode)),
  });
  const messages = history.data ?? [];

  const send = useMutation({
    mutationFn: (message: string) => (isAgent ? api.sendAgentMessage(target.agentId, message) : api.sendChatMessage(target.mode, message)),
    onSuccess: (appended) => {
      setError(null);
      setText("");
      queryClient.setQueryData<ChatMessage[]>(historyKey, (existing) => [...(existing ?? []), ...appended]);
    },
    onError: (err) => {
      setError(err instanceof ApiError ? err.message : "Could not send that message");
    },
  });

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [messages.length, send.isPending]);

  // Tool results are keyed by `tool_call_id`, matched back to the name the
  // preceding assistant message's `tool_calls` requested - build that
  // lookup once per render rather than re-scanning per tool message.
  const toolNameById = new Map<string, string>();
  for (const m of messages) {
    for (const entry of toolCallEntries(m.tool_calls)) toolNameById.set(entry.id, entry.name);
  }

  return (
    <div className="card" style={{ display: "flex", flexDirection: "column", height: "70vh" }}>
      <div ref={scrollRef} style={{ flex: 1, overflowY: "auto", padding: 4 }}>
        {history.isLoading && <p style={{ color: "var(--text-muted)" }}>Loading...</p>}
        {!history.isLoading && messages.length === 0 && (
          <p className="empty-state">
            {mode === "admin"
              ? "Ask for help building a business rule, a workflow, an integration, or anything else in the admin surface - it can create these for real, same as the forms."
              : mode === "agent"
                ? "Say hello - this agent has its own persona, actions and memory, configured in AI Agent Foundry."
                : "Ask about your records, or ask it to create or update one - it can look things up and take action."}
          </p>
        )}
        {messages.map((m) => {
          if (m.role === "user") {
            return (
              <ChatBubble key={m.id} role="user">
                {m.content}
              </ChatBubble>
            );
          }
          if (m.role === "assistant") {
            const calls = toolCallEntries(m.tool_calls);
            if (m.content) {
              return (
                <ChatBubble key={m.id} role="assistant">
                  {m.content}
                </ChatBubble>
              );
            }
            if (calls.length > 0) {
              return (
                <div key={m.id}>
                  {calls.map((c) => (
                    <ToolCallLine key={c.id} name={c.name} />
                  ))}
                </div>
              );
            }
            return null;
          }
          // role === "tool"
          const name = (m.tool_call_id && toolNameById.get(m.tool_call_id)) || "a tool";
          return <ToolCallLine key={m.id} name={name} result={m.content ?? undefined} />;
        })}
        {send.isPending && (
          <div style={{ fontSize: 13, color: "var(--text-muted)" }}>Thinking...</div>
        )}
      </div>

      {error && <div className="error-banner">{error}</div>}

      <form
        style={{ display: "flex", gap: 8, marginTop: 8 }}
        onSubmit={(e) => {
          e.preventDefault();
          if (text.trim() && !send.isPending) send.mutate(text);
        }}
      >
        <input
          style={{ flex: 1 }}
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={mode === "admin" ? "e.g. create a business rule that..." : mode === "agent" ? "Message this agent..." : "e.g. find the company named..."}
          disabled={send.isPending}
        />
        <button className="btn btn-primary" type="submit" disabled={send.isPending || !text.trim()}>
          {send.isPending ? "Sending..." : "Send"}
        </button>
      </form>
    </div>
  );
}
