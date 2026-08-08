import { useState } from "react";
import { useMutation } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { ChangeOwnPassword, User } from "../../lib/types";

const EMPTY: ChangeOwnPassword = { current_password: "", new_password: "" };

export function Account({ user }: { user: User }) {
  const [input, setInput] = useState<ChangeOwnPassword>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const change = useMutation({
    mutationFn: () => api.changeMyPassword(input),
    onSuccess: () => {
      setInput(EMPTY);
      setError(null);
      setSuccess(true);
    },
    onError: (err) => {
      setSuccess(false);
      setError(err instanceof ApiError ? err.message : "Could not change the password");
    },
  });

  return (
    <div>
      <h2>My account</h2>
      <p style={{ color: "var(--text-muted)" }}>
        Signed in as <strong>{user.display_name}</strong> ({user.username}) · {user.roles.join(", ")}
      </p>

      <div className="card" style={{ marginTop: 16, maxWidth: 420 }}>
        <h3 style={{ marginTop: 0 }}>Change my password</h3>
        {error && <div className="error-banner">{error}</div>}
        {success && <div className="success-banner">Password changed.</div>}
        <form
          className="form-grid"
          onSubmit={(e) => {
            e.preventDefault();
            setSuccess(false);
            change.mutate();
          }}
        >
          <div className="form-field full">
            <label>Current password</label>
            <input
              type="password"
              value={input.current_password}
              onChange={(e) => setInput({ ...input, current_password: e.target.value })}
              required
            />
          </div>
          <div className="form-field full">
            <label>New password (min 8 characters)</label>
            <input
              type="password"
              value={input.new_password}
              onChange={(e) => setInput({ ...input, new_password: e.target.value })}
              minLength={8}
              required
            />
          </div>
          <div className="form-field full">
            <button className="btn btn-primary" type="submit" disabled={change.isPending}>
              Change password
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
