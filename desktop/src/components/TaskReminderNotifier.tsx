import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";

const NOTIFIED_KEY = "lanesra.notifiedTaskReminders";

function loadNotified(): Set<string> {
  try {
    const raw = localStorage.getItem(NOTIFIED_KEY);
    return raw ? new Set(JSON.parse(raw) as string[]) : new Set();
  } catch {
    return new Set();
  }
}

function saveNotified(ids: Set<string>) {
  try {
    localStorage.setItem(NOTIFIED_KEY, JSON.stringify([...ids].slice(-500)));
  } catch {
    // best-effort only
  }
}

/**
 * FR-TSK-06: "Support Windows notifications when enabled." Uses the
 * standard Web Notification API rather than a native Tauri plugin -
 * WebView2 (the Windows webview Tauri v2 uses) surfaces it as a real
 * Windows toast notification, so no extra native dependency is needed;
 * the same code also works unmodified in Team Workspace's plain browser
 * tab. Polls every minute for tasks whose reminder has just come due and
 * haven't already been notified this session (tracked in localStorage so
 * a reload/relaunch doesn't repeat one that already fired).
 */
export function TaskReminderNotifier({ currentUserId }: { currentUserId: string }) {
  const tasks = useQuery({ queryKey: ["tasks"], queryFn: () => api.listTasks(), refetchInterval: 60_000 });
  const notified = useRef<Set<string>>(loadNotified());
  const permissionRequested = useRef(false);

  useEffect(() => {
    if (typeof Notification === "undefined" || permissionRequested.current) return;
    permissionRequested.current = true;
    if (Notification.permission === "default") {
      Notification.requestPermission().catch(() => {});
    }
  }, []);

  useEffect(() => {
    if (typeof Notification === "undefined" || Notification.permission !== "granted") return;
    if (!tasks.data) return;
    const now = Date.now();
    for (const task of tasks.data) {
      if (!task.reminder_at || task.owner_user_id !== currentUserId) continue;
      if (task.status === "Completed" || task.status === "Cancelled" || task.archived_at) continue;
      if (notified.current.has(task.id)) continue;
      const dueAt = new Date(task.reminder_at).getTime();
      if (Number.isNaN(dueAt) || dueAt > now) continue;
      new Notification("Lanesra OS - Task reminder", { body: task.title, tag: task.id });
      notified.current.add(task.id);
      saveNotified(notified.current);
    }
  }, [tasks.data, currentUserId]);

  return null;
}
