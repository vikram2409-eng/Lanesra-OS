import { useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { CustomFieldsAdmin } from "./CustomFieldsAdmin";
import type { Workspace, WorkspaceUpdate } from "../../lib/types";

// Caps the logo at 240px on its longest side and re-encodes it as PNG via
// canvas, before it ever leaves the browser - keeps the stored blob well
// under the server's 256KB cap regardless of what the admin uploads, and
// normalizes the mime type so the server only has to accept one format
// from this path (it still validates independently - see FR-BRD-02).
const MAX_LOGO_DIMENSION = 240;

function resizeImageToPngBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error);
    reader.onload = () => {
      const img = new Image();
      img.onerror = () => reject(new Error("Could not read that image file"));
      img.onload = () => {
        const scale = Math.min(1, MAX_LOGO_DIMENSION / Math.max(img.width, img.height));
        const canvas = document.createElement("canvas");
        canvas.width = Math.round(img.width * scale);
        canvas.height = Math.round(img.height * scale);
        const ctx = canvas.getContext("2d");
        if (!ctx) {
          reject(new Error("Could not process that image"));
          return;
        }
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        const dataUrl = canvas.toDataURL("image/png");
        resolve(dataUrl.slice(dataUrl.indexOf(",") + 1));
      };
      img.src = reader.result as string;
    };
    reader.readAsDataURL(file);
  });
}

export function Settings() {
  const queryClient = useQueryClient();
  const workspace = useQuery({ queryKey: ["workspaceStatus"], queryFn: () => api.workspaceStatus() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["workspaceStatus"] });
  }

  if (!workspace.data) return <p>Loading...</p>;

  return (
    <div>
      <h2>Settings</h2>
      <p style={{ color: "var(--text-muted)" }}>
        Business profile and letterhead branding shown on printed quotes, orders and invoices.
      </p>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, alignItems: "start" }}>
        <ProfileForm workspace={workspace.data} onSaved={invalidate} />
        <LogoCard workspace={workspace.data} onChanged={invalidate} />
      </div>
      <div style={{ marginTop: 16 }}>
        <CustomFieldsAdmin />
      </div>
    </div>
  );
}

function ProfileForm({ workspace, onSaved }: { workspace: Workspace; onSaved: () => void }) {
  const [input, setInput] = useState<WorkspaceUpdate>({
    business_name: workspace.business_name,
    legal_name: workspace.legal_name,
    business_address: workspace.business_address,
    currency_code: workspace.currency_code,
    locale: workspace.locale,
    timezone: workspace.timezone,
    default_tax_rate_bp: workspace.default_tax_rate_bp,
  });
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const save = useMutation({
    mutationFn: () => api.updateWorkspace(input),
    onSuccess: () => {
      setError(null);
      setSuccess(true);
      onSaved();
    },
    onError: (err) => {
      setSuccess(false);
      setError(err instanceof ApiError ? err.message : "Could not save the workspace profile");
    },
  });

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Business profile</h3>
      {error && <div className="error-banner">{error}</div>}
      {success && <div className="success-banner">Saved.</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          setSuccess(false);
          save.mutate();
        }}
      >
        <div className="form-field full">
          <label>Business name</label>
          <input
            value={input.business_name}
            onChange={(e) => setInput({ ...input, business_name: e.target.value })}
            required
          />
        </div>
        <div className="form-field full">
          <label>Legal name</label>
          <input
            value={input.legal_name ?? ""}
            onChange={(e) => setInput({ ...input, legal_name: e.target.value || null })}
          />
        </div>
        <div className="form-field full">
          <label>Business address</label>
          <textarea
            value={input.business_address ?? ""}
            onChange={(e) => setInput({ ...input, business_address: e.target.value || null })}
          />
        </div>
        <div className="form-field">
          <label>Currency</label>
          <input
            value={input.currency_code}
            onChange={(e) => setInput({ ...input, currency_code: e.target.value.toUpperCase() })}
            maxLength={3}
            required
          />
        </div>
        <div className="form-field">
          <label>Default tax rate (%)</label>
          <input
            type="number"
            step="0.01"
            min={0}
            value={(input.default_tax_rate_bp / 100).toString()}
            onChange={(e) => setInput({ ...input, default_tax_rate_bp: Math.round(Number(e.target.value) * 100) })}
          />
        </div>
        <div className="form-field">
          <label>Locale</label>
          <input value={input.locale} onChange={(e) => setInput({ ...input, locale: e.target.value })} />
        </div>
        <div className="form-field">
          <label>Timezone</label>
          <input value={input.timezone} onChange={(e) => setInput({ ...input, timezone: e.target.value })} />
        </div>
        <div className="form-field full">
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save profile
          </button>
        </div>
      </form>
    </div>
  );
}

function LogoCard({ workspace, onChanged }: { workspace: Workspace; onChanged: () => void }) {
  const fileInput = useRef<HTMLInputElement>(null);
  const [error, setError] = useState<string | null>(null);

  const setLogo = useMutation({
    mutationFn: async (file: File) => {
      const logo_base64 = await resizeImageToPngBase64(file);
      return api.setWorkspaceLogo({ logo_base64, logo_mime: "image/png" });
    },
    onSuccess: () => {
      setError(null);
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not upload the logo"),
  });

  const clearLogo = useMutation({
    mutationFn: () => api.clearWorkspaceLogo(),
    onSuccess: () => {
      setError(null);
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not remove the logo"),
  });

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Logo</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Shown next to the business name on the print preview for quotes, orders and invoices. PNG or JPEG.
      </p>
      {error && <div className="error-banner">{error}</div>}

      {workspace.logo_base64 && workspace.logo_mime ? (
        <img
          src={`data:${workspace.logo_mime};base64,${workspace.logo_base64}`}
          alt="Business logo"
          style={{ maxWidth: 160, maxHeight: 160, display: "block", marginBottom: 12, borderRadius: 6 }}
        />
      ) : (
        <p className="empty-state">No logo set.</p>
      )}

      <div style={{ display: "flex", gap: 8 }}>
        <button className="btn" onClick={() => fileInput.current?.click()} disabled={setLogo.isPending}>
          {setLogo.isPending ? "Uploading..." : "Upload logo"}
        </button>
        {workspace.logo_base64 && (
          <button className="btn btn-danger" onClick={() => clearLogo.mutate()} disabled={clearLogo.isPending}>
            Remove logo
          </button>
        )}
        <input
          ref={fileInput}
          type="file"
          accept="image/png,image/jpeg"
          hidden
          onChange={(e) => {
            const file = e.target.files?.[0];
            e.target.value = "";
            if (file) setLogo.mutate(file);
          }}
        />
      </div>
    </div>
  );
}
