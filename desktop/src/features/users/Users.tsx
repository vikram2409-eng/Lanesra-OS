import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { ROLES, type NewUser, type UserUpdate } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

export function Users() {
  const [view, setView] = useState<View>({ mode: "list" });
  const queryClient = useQueryClient();
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["users"] });
  }

  if (view.mode === "create") {
    return (
      <CreateUserForm
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  if (view.mode === "edit") {
    return (
      <EditUserForm
        userId={view.id}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Users</h2>
        <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
          + New user
        </button>
      </div>
      {users.isLoading && <p>Loading...</p>}
      {users.data && (
        <table>
          <thead>
            <tr>
              <th>Username</th>
              <th>Display name</th>
              <th>Roles</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {users.data.map((u) => (
              <tr key={u.id}>
                <td>{u.username}</td>
                <td>{u.display_name}</td>
                <td>{u.roles.join(", ") || "—"}</td>
                <td>
                  <span className={`badge${u.is_active ? " badge-success" : " badge-danger"}`}>
                    {u.is_active ? "Active" : "Inactive"}
                  </span>
                </td>
                <td>
                  <button className="btn" onClick={() => setView({ mode: "edit", id: u.id })}>
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function RoleCheckboxes({ selected, onChange }: { selected: string[]; onChange: (roles: string[]) => void }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      {ROLES.map((role) => (
        <label key={role} style={{ display: "flex", gap: 8, alignItems: "center", fontSize: 14 }}>
          <input
            type="checkbox"
            checked={selected.includes(role)}
            onChange={(e) => {
              if (e.target.checked) onChange([...selected, role]);
              else onChange(selected.filter((r) => r !== role));
            }}
          />
          {role}
        </label>
      ))}
    </div>
  );
}

function CreateUserForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<NewUser>({
    username: "",
    display_name: "",
    password: "",
    roles: ["Sales"],
  });
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () => api.createUser(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create the user"),
  });

  return (
    <div>
      <h2>New user</h2>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field">
          <label>Username</label>
          <input value={input.username} onChange={(e) => setInput({ ...input, username: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Display name</label>
          <input
            value={input.display_name}
            onChange={(e) => setInput({ ...input, display_name: e.target.value })}
            required
          />
        </div>
        <div className="form-field">
          <label>Password (min 8 characters)</label>
          <input
            type="password"
            value={input.password}
            onChange={(e) => setInput({ ...input, password: e.target.value })}
            minLength={8}
            required
          />
        </div>
        <div className="form-field">
          <label>Roles</label>
          <RoleCheckboxes selected={input.roles} onChange={(roles) => setInput({ ...input, roles })} />
        </div>
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

function EditUserForm({ userId, onDone, onCancel }: { userId: string; onDone: () => void; onCancel: () => void }) {
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const existing = users.data?.find((u) => u.id === userId);
  const [input, setInput] = useState<UserUpdate | null>(null);
  const [newPassword, setNewPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  if (existing && !input) {
    setInput({ display_name: existing.display_name, roles: existing.roles, is_active: existing.is_active });
  }

  const save = useMutation({
    mutationFn: () => api.updateUser(userId, input as UserUpdate),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not update the user"),
  });

  const resetPassword = useMutation({
    mutationFn: () => api.setUserPassword(userId, { new_password: newPassword }),
    onSuccess: () => setNewPassword(""),
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not reset the password"),
  });

  if (!input || !existing) return <p>Loading...</p>;

  return (
    <div>
      <h2>Edit user — {existing.username}</h2>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field">
          <label>Display name</label>
          <input value={input.display_name} onChange={(e) => setInput({ ...input, display_name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={input.is_active}
              onChange={(e) => setInput({ ...input, is_active: e.target.checked })}
            />
            Active
          </label>
        </div>
        <div className="form-field">
          <label>Roles</label>
          <RoleCheckboxes selected={input.roles} onChange={(roles) => setInput({ ...input, roles })} />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>

      <div className="card" style={{ marginTop: 16, maxWidth: 420 }}>
        <h3 style={{ marginTop: 0 }}>Reset password</h3>
        <form
          style={{ display: "flex", gap: 8, alignItems: "flex-end" }}
          onSubmit={(e) => {
            e.preventDefault();
            resetPassword.mutate();
          }}
        >
          <div className="form-field full">
            <label>New password (min 8 characters)</label>
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              minLength={8}
              required
            />
          </div>
          <button className="btn" type="submit" disabled={resetPassword.isPending}>
            Reset
          </button>
        </form>
      </div>
    </div>
  );
}
