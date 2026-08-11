import { useEffect, useRef, useState } from "react";

import { api, ApiError } from "../lib/api";
import type { User } from "../lib/types";

// Session inactivity auto-lock (Phase E polish). Not admin-configurable
// yet - a fixed 15-minute idle window, applied the same way for every
// user. Locking re-authenticates against the *current* user's password
// (via the same `login` command the sign-in screen uses) rather than a
// separate "unlock" concept, so there's no new session/credential model
// to build - it just re-proves the same person is still at the keyboard.
const IDLE_TIMEOUT_MS = 15 * 60_000;
const ACTIVITY_EVENTS = ["mousedown", "mousemove", "keydown", "scroll", "touchstart"] as const;

export function SessionLock({ user, children }: { user: User; children: React.ReactNode }) {
  const [locked, setLocked] = useState(false);
  const lastActivity = useRef(Date.now());

  useEffect(() => {
    const markActive = () => {
      lastActivity.current = Date.now();
    };
    ACTIVITY_EVENTS.forEach((evt) => window.addEventListener(evt, markActive, { passive: true }));

    const interval = setInterval(() => {
      if (!locked && Date.now() - lastActivity.current >= IDLE_TIMEOUT_MS) {
        setLocked(true);
      }
    }, 15_000);

    return () => {
      ACTIVITY_EVENTS.forEach((evt) => window.removeEventListener(evt, markActive));
      clearInterval(interval);
    };
  }, [locked]);

  if (!locked) return <>{children}</>;

  return (
    <UnlockScreen
      username={user.username}
      onUnlocked={() => {
        lastActivity.current = Date.now();
        setLocked(false);
      }}
    />
  );
}

function UnlockScreen({ username, onUnlocked }: { username: string; onUnlocked: () => void }) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await api.login({ username, password });
      setPassword("");
      onUnlocked();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Incorrect password");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div
      style={{
        position: "fixed", inset: 0, background: "var(--bg, #1a1d29)", display: "flex", alignItems: "center",
        justifyContent: "center", zIndex: 1000,
      }}
    >
      <div className="card" style={{ width: 320 }}>
        <h2 style={{ marginTop: 0 }}>🔒 Session locked</h2>
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
          Signed in as <strong>{username}</strong>. Enter your password to continue.
        </p>
        {error && <div className="error-banner">{error}</div>}
        <form onSubmit={handleSubmit} className="form-grid">
          <div className="form-field full">
            <label>Password</label>
            <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} autoFocus required />
          </div>
          <div className="form-field full">
            <button className="btn btn-primary" type="submit" disabled={busy} style={{ width: "100%" }}>
              Unlock
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
