import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "../lib/api";

/**
 * In-app notification center (spec §15/Platform Extensibility) - surfaces
 * `add_notification` workflow actions (ADM-WF-04). Polls for unread
 * notifications on the same interval App.tsx uses for scheduled workflow
 * runs, since both are "things that happened while nobody was looking."
 */
export function NotificationBell() {
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const unread = useQuery({
    queryKey: ["notifications", "unread"],
    queryFn: () => api.listNotifications(true),
    refetchInterval: 60_000,
  });
  const recent = useQuery({
    queryKey: ["notifications", "recent"],
    queryFn: () => api.listNotifications(false),
    enabled: open,
  });

  const markRead = useMutation({
    mutationFn: (id: string) => api.markNotificationRead(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notifications"] }),
  });
  const markAllRead = useMutation({
    mutationFn: () => api.markAllNotificationsRead(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["notifications"] }),
  });

  const unreadCount = unread.data?.length ?? 0;

  return (
    <div style={{ position: "relative" }}>
      <button className="btn" onClick={() => setOpen((v) => !v)} style={{ position: "relative" }}>
        🔔
        {unreadCount > 0 && (
          <span
            style={{
              position: "absolute", top: -4, right: -4, background: "var(--danger, #d64545)", color: "white",
              borderRadius: 999, fontSize: 10, minWidth: 16, height: 16, display: "flex", alignItems: "center",
              justifyContent: "center", padding: "0 4px",
            }}
          >
            {unreadCount}
          </span>
        )}
      </button>
      {open && (
        <div
          className="card"
          style={{
            position: "absolute", right: 0, top: "calc(100% + 4px)", width: 320, maxHeight: 400, overflowY: "auto",
            zIndex: 20, boxShadow: "0 4px 16px rgba(0,0,0,0.15)",
          }}
        >
          <div className="toolbar" style={{ marginBottom: 8 }}>
            <strong style={{ fontSize: 14 }}>Notifications</strong>
            {unreadCount > 0 && (
              <button className="link-button" style={{ fontSize: 12 }} onClick={() => markAllRead.mutate()}>
                Mark all read
              </button>
            )}
          </div>
          {(recent.data ?? []).length === 0 && <p className="empty-state">No notifications yet.</p>}
          {(recent.data ?? []).map((n) => (
            <div
              key={n.id}
              style={{ padding: "6px 0", borderBottom: "1px solid var(--border, #eee)", fontSize: 13, opacity: n.read_at ? 0.6 : 1 }}
            >
              <div>{n.message}</div>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: 2 }}>
                <span style={{ color: "var(--text-muted)", fontSize: 11 }}>{new Date(n.created_at).toLocaleString()}</span>
                {!n.read_at && (
                  <button className="link-button" style={{ fontSize: 11 }} onClick={() => markRead.mutate(n.id)}>
                    Mark read
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
