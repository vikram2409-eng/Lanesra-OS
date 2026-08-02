import { useState } from "react";

import { api, ApiError } from "../../lib/api";
import type { User, Workspace } from "../../lib/types";

export function FirstRun({ onComplete }: { onComplete: (workspace: Workspace, user: User) => void }) {
  const [businessName, setBusinessName] = useState("");
  const [currencyCode, setCurrencyCode] = useState("USD");
  const [adminUsername, setAdminUsername] = useState("admin");
  const [adminDisplayName, setAdminDisplayName] = useState("");
  const [adminPassword, setAdminPassword] = useState("");
  const [loadSampleData, setLoadSampleData] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const [workspace, user] = await api.firstRunSetup({
        business_name: businessName,
        legal_name: null,
        currency_code: currencyCode,
        locale: "en-US",
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone ?? "UTC",
        default_tax_rate_bp: 0,
        admin_username: adminUsername,
        admin_display_name: adminDisplayName || adminUsername,
        admin_password: adminPassword,
        load_sample_data: loadSampleData,
      });
      onComplete(workspace, user);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Could not set up the workspace");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="centered-screen">
      <div className="card auth-card">
        <h1>Welcome to Lanesra OS</h1>
        <p>Set up your local workspace. Everything stays on this computer.</p>
        {error && <div className="error-banner">{error}</div>}
        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <div className="form-field">
            <label>Business name</label>
            <input value={businessName} onChange={(e) => setBusinessName(e.target.value)} required />
          </div>
          <div className="form-field">
            <label>Currency</label>
            <input
              value={currencyCode}
              onChange={(e) => setCurrencyCode(e.target.value.toUpperCase())}
              maxLength={3}
              required
            />
          </div>
          <div className="form-field">
            <label>Administrator username</label>
            <input value={adminUsername} onChange={(e) => setAdminUsername(e.target.value)} required />
          </div>
          <div className="form-field">
            <label>Administrator display name</label>
            <input value={adminDisplayName} onChange={(e) => setAdminDisplayName(e.target.value)} />
          </div>
          <div className="form-field">
            <label>Administrator password (min 8 characters)</label>
            <input
              type="password"
              value={adminPassword}
              onChange={(e) => setAdminPassword(e.target.value)}
              minLength={8}
              required
            />
          </div>
          <label style={{ fontSize: 13, display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={loadSampleData}
              onChange={(e) => setLoadSampleData(e.target.checked)}
            />
            Start with sample data
          </label>
          <button className="btn btn-primary" type="submit" disabled={submitting}>
            {submitting ? "Setting up..." : "Create workspace"}
          </button>
        </form>
      </div>
    </div>
  );
}
