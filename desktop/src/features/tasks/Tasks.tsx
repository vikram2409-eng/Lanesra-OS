import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import type { Prefill } from "../../components/AppShell";
import {
  TASK_PRIORITIES,
  TASK_RELATED_TYPES,
  TASK_STATUSES,
  type CustomFieldValues,
  type Task,
  type TaskInput,
  type TaskRelatedType,
} from "../../lib/types";

type Tab = "today" | "upcoming" | "overdue" | "completed" | "owner" | "related";
type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

function taskExportColumns(ownerName: (id: string | null) => string) {
  return [
    { label: "Number", get: (t: Task) => t.task_number },
    { label: "Title", get: (t: Task) => t.title },
    { label: "Priority", get: (t: Task) => t.priority },
    { label: "Status", get: (t: Task) => t.status },
    { label: "Due date", get: (t: Task) => t.due_date ?? "" },
    { label: "Owner", get: (t: Task) => ownerName(t.owner_user_id) },
    { label: "Related type", get: (t: Task) => t.related_type ?? "" },
    { label: "Description", get: (t: Task) => t.description ?? "" },
  ];
}

const TABS: { tab: Tab; label: string }[] = [
  { tab: "today", label: "Today" },
  { tab: "upcoming", label: "Upcoming" },
  { tab: "overdue", label: "Overdue" },
  { tab: "completed", label: "Completed" },
  { tab: "owner", label: "By Owner" },
  { tab: "related", label: "By Related Record" },
];

function emptyInput(ownerUserId: string | null): TaskInput {
  return {
    title: "",
    description: null,
    owner_user_id: ownerUserId,
    priority: "Normal",
    status: "Not Started",
    due_date: null,
    reminder_at: null,
    related_type: null,
    related_id: null,
  };
}

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

export function Tasks({
  currentUserId,
  prefill,
  onPrefillConsumed,
}: {
  currentUserId: string;
  prefill?: Prefill | null;
  onPrefillConsumed?: () => void;
}) {
  const [view, setView] = useState<View>(() =>
    prefill?.companyId || prefill?.contactId ? { mode: "create" } : { mode: "list" },
  );
  const [tab, setTab] = useState<Tab>("today");
  const queryClient = useQueryClient();

  useEffect(() => {
    if (prefill?.companyId || prefill?.contactId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const tasks = useQuery({ queryKey: ["tasks"], queryFn: () => api.listTasks() });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });
  const contacts = useQuery({ queryKey: ["contacts"], queryFn: () => api.listContacts() });
  const opportunities = useQuery({ queryKey: ["opportunities"], queryFn: () => api.listOpportunities() });
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const orders = useQuery({ queryKey: ["orders"], queryFn: () => api.listOrders() });
  const invoices = useQuery({ queryKey: ["invoices"], queryFn: () => api.listInvoices() });
  const contracts = useQuery({ queryKey: ["contracts"], queryFn: () => api.listContracts() });

  const relatedLabel = useMemo(() => {
    const byType: Record<string, Map<string, string>> = {
      Company: new Map((companies.data ?? []).map((c) => [c.id, c.name])),
      Contact: new Map((contacts.data ?? []).map((c) => [c.id, `${c.first_name} ${c.last_name}`])),
      Opportunity: new Map((opportunities.data ?? []).map((o) => [o.id, o.name])),
      Quote: new Map((quotes.data ?? []).map((q) => [q.id, q.quote_number])),
      Order: new Map((orders.data ?? []).map((o) => [o.id, o.order_number])),
      Invoice: new Map((invoices.data ?? []).map((i) => [i.id, i.invoice_number])),
      Contract: new Map((contracts.data ?? []).map((c) => [c.id, c.contract_number])),
    };
    return (task: Task): string | null => {
      if (!task.related_type || !task.related_id) return null;
      const label = byType[task.related_type]?.get(task.related_id);
      return `${task.related_type}: ${label ?? task.related_id}`;
    };
  }, [companies.data, contacts.data, opportunities.data, quotes.data, orders.data, invoices.data, contracts.data]);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["tasks"] });
  }

  if (view.mode !== "list") {
    return (
      <TaskForm
        taskId={view.mode === "edit" ? view.id : undefined}
        currentUserId={currentUserId}
        initialRelated={
          view.mode === "create" && prefill?.contactId
            ? { related_type: "Contact", related_id: prefill.contactId }
            : view.mode === "create" && prefill?.companyId
              ? { related_type: "Company", related_id: prefill.companyId }
              : undefined
        }
        users={users.data ?? []}
        relatedOptions={{
          Company: (companies.data ?? []).map((c) => ({ id: c.id, label: c.name })),
          Contact: (contacts.data ?? []).map((c) => ({ id: c.id, label: `${c.first_name} ${c.last_name}` })),
          Opportunity: (opportunities.data ?? []).map((o) => ({ id: o.id, label: o.name })),
          Quote: (quotes.data ?? []).map((q) => ({ id: q.id, label: q.quote_number })),
          Order: (orders.data ?? []).map((o) => ({ id: o.id, label: o.order_number })),
          Invoice: (invoices.data ?? []).map((i) => ({ id: i.id, label: i.invoice_number })),
          Contract: (contracts.data ?? []).map((c) => ({ id: c.id, label: c.contract_number })),
        }}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  const all = tasks.data ?? [];
  const open = all.filter((t) => t.status !== "Completed" && t.status !== "Cancelled");
  const today = todayIso();
  const ownerName = (id: string | null): string =>
    id ? users.data?.find((u) => u.id === id)?.display_name ?? "Unknown user" : "Unassigned";

  let groups: { label: string; rows: Task[] }[];
  if (tab === "today") groups = [{ label: "Today", rows: open.filter((t) => t.due_date === today) }];
  else if (tab === "upcoming")
    groups = [{ label: "Upcoming", rows: open.filter((t) => !!t.due_date && t.due_date > today) }];
  else if (tab === "overdue")
    groups = [{ label: "Overdue", rows: open.filter((t) => !!t.due_date && t.due_date < today) }];
  else if (tab === "completed") groups = [{ label: "Completed", rows: all.filter((t) => t.status === "Completed") }];
  else if (tab === "related") groups = [{ label: "Related", rows: all.filter((t) => !!t.related_type) }];
  else {
    const byOwner = new Map<string, Task[]>();
    for (const t of all) {
      const key = t.owner_user_id ?? "";
      if (!byOwner.has(key)) byOwner.set(key, []);
      byOwner.get(key)!.push(t);
    }
    groups = Array.from(byOwner.entries())
      .sort(([a], [b]) => ownerName(a || null).localeCompare(ownerName(b || null)))
      .map(([ownerId, rows]) => ({ label: ownerName(ownerId || null), rows }));
  }

  const showGroupLabels = tab === "owner";

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Tasks</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton rows={all} columns={taskExportColumns(ownerName)} filename="tasks.csv" />
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New task
          </button>
        </div>
      </div>

      <div className="tab-row">
        {TABS.map((t) => (
          <button key={t.tab} className={`tab${tab === t.tab ? " active" : ""}`} onClick={() => setTab(t.tab)}>
            {t.label}
          </button>
        ))}
      </div>

      {tasks.isLoading && <p>Loading...</p>}
      {groups.every((g) => g.rows.length === 0) && <p className="empty-state">No tasks here.</p>}
      {groups.map(
        (group) =>
          group.rows.length > 0 && (
            <div key={group.label} style={{ marginBottom: showGroupLabels ? 24 : 0 }}>
              {showGroupLabels && <h3>{group.label}</h3>}
              <table>
                <thead>
                  <tr>
                    <th>Number</th>
                    <th>Title</th>
                    <th>Priority</th>
                    <th>Status</th>
                    <th>Due date</th>
                    {!showGroupLabels && <th>Owner</th>}
                    <th>Related to</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {group.rows.map((t) => (
                    <tr key={t.id}>
                      <td>{t.task_number}</td>
                      <td>{t.title}</td>
                      <td>{t.priority}</td>
                      <td>{t.status}</td>
                      <td>{t.due_date ?? "—"}</td>
                      {!showGroupLabels && <td>{ownerName(t.owner_user_id)}</td>}
                      <td>{relatedLabel(t) ?? "General"}</td>
                      <td>
                        <button className="btn" onClick={() => setView({ mode: "edit", id: t.id })}>
                          Edit
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ),
      )}
    </div>
  );
}

function TaskForm({
  taskId,
  currentUserId,
  initialRelated,
  users,
  relatedOptions,
  onDone,
  onCancel,
}: {
  taskId?: string;
  currentUserId: string;
  initialRelated?: { related_type: TaskRelatedType; related_id: string };
  users: { id: string; display_name: string }[];
  relatedOptions: Record<TaskRelatedType, { id: string; label: string }[]>;
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["task", taskId],
    queryFn: () => api.getTask(taskId as string),
    enabled: !!taskId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", taskId],
    queryFn: () => api.getCustomFieldValues(taskId as string),
    enabled: !!taskId,
  });
  const [input, setInput] = useState<TaskInput>(() => ({
    ...emptyInput(currentUserId),
    ...(initialRelated ? { related_type: initialRelated.related_type, related_id: initialRelated.related_id } : {}),
  }));
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== taskId) {
    const { title, description, owner_user_id, priority, status, due_date, reminder_at, related_type, related_id } =
      existing.data;
    setInput({ title, description, owner_user_id, priority, status, due_date, reminder_at, related_type, related_id });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(taskId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const task = taskId ? await api.updateTask(taskId, input) : await api.createTask(input);
      const ruleMessages = await api.setCustomFieldValues("Task", task.id, customValues);
      showRuleMessages(ruleMessages);
      return task;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the task"),
  });

  const relatedType = input.related_type as TaskRelatedType | null;
  const options = relatedType ? relatedOptions[relatedType] ?? [] : [];

  return (
    <div>
      <h2>{taskId ? "Edit task" : "New task"}</h2>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field full">
          <label>Title</label>
          <input value={input.title} onChange={(e) => setInput({ ...input, title: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Owner</label>
          <select
            value={input.owner_user_id ?? ""}
            onChange={(e) => setInput({ ...input, owner_user_id: e.target.value || null })}
          >
            <option value="">— Unassigned —</option>
            {users.map((u) => (
              <option key={u.id} value={u.id}>
                {u.display_name}
                {u.id === currentUserId ? " (you)" : ""}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Priority</label>
          <select value={input.priority} onChange={(e) => setInput({ ...input, priority: e.target.value })}>
            {TASK_PRIORITIES.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Status</label>
          <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
            {TASK_STATUSES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Due date</label>
          <input type="date" value={input.due_date ?? ""} onChange={(e) => setInput({ ...input, due_date: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Reminder</label>
          <input
            type="datetime-local"
            value={input.reminder_at ?? ""}
            onChange={(e) => setInput({ ...input, reminder_at: e.target.value || null })}
          />
        </div>
        <div className="form-field">
          <label>Relates to</label>
          <select
            value={input.related_type ?? ""}
            onChange={(e) =>
              setInput({ ...input, related_type: e.target.value || null, related_id: null })
            }
          >
            <option value="">General (no relation)</option>
            {TASK_RELATED_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        {relatedType && (
          <div className="form-field">
            <label>{relatedType} record</label>
            <select
              value={input.related_id ?? ""}
              onChange={(e) => setInput({ ...input, related_id: e.target.value || null })}
              required
            >
              <option value="">— Select —</option>
              {options.map((o) => (
                <option key={o.id} value={o.id}>
                  {o.label}
                </option>
              ))}
            </select>
          </div>
        )}
        <div className="form-field full">
          <label>Description</label>
          <textarea
            value={input.description ?? ""}
            onChange={(e) => setInput({ ...input, description: e.target.value || null })}
          />
        </div>
        <CustomFieldsSection entityType="Task" status={input.status} values={customValues} onChange={setCustomValues} />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
