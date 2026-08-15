import { useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { Users } from "../users/Users";
import { CustomObjectsAdmin } from "./CustomObjectsAdmin";
import { RelationshipsAdmin } from "./RelationshipsAdmin";
import { CustomFieldsAdmin } from "./CustomFieldsAdmin";
import { BusinessRulesAdmin } from "./BusinessRulesAdmin";
import { WorkflowAutomationAdmin } from "./WorkflowAutomationAdmin";
import { StatusTransitionsAdmin } from "./StatusTransitionsAdmin";
import { NumberingAdmin } from "./NumberingAdmin";
import { DashboardKpiAdmin } from "./DashboardKpiAdmin";
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

type AdminTab = "users" | "profile" | "objects" | "relationships" | "fields" | "rules" | "workflow" | "transitions" | "numbering" | "kpis";

const ADMIN_TABS: { key: AdminTab; label: string }[] = [
  { key: "users", label: "Users" },
  { key: "profile", label: "Business profile" },
  { key: "objects", label: "Custom Objects" },
  { key: "relationships", label: "Relationships" },
  { key: "fields", label: "Custom fields" },
  { key: "rules", label: "Business rules" },
  { key: "workflow", label: "Workflow automation" },
  { key: "transitions", label: "Status transitions" },
  { key: "numbering", label: "Numbering" },
  { key: "kpis", label: "Dashboard KPIs" },
];

function tabLabel(key: AdminTab): string {
  return ADMIN_TABS.find((t) => t.key === key)?.label ?? key;
}

// Groups the same 10 tabs above into named categories for the landing page
// below - purely a presentation grouping, the tab keys and their screens
// are unchanged. Desktop has no Screen layouts or Integrations tab (both
// are demo-first capabilities that don't exist here), so its Customization
// category has one fewer item than the online demo's equivalent.
const ADMIN_CATEGORIES: { key: string; label: string; icon: string; note: string; items: AdminTab[] }[] = [
  { key: "workspace", label: "Workspace", icon: "⚙", note: "How the workspace looks and is identified", items: ["profile", "numbering", "kpis"] },
  { key: "access", label: "Access", icon: "👤", note: "Who can sign in and what they can do", items: ["users"] },
  { key: "customization", label: "Customization", icon: "🧩", note: "Extend the data model without code", items: ["objects", "relationships", "fields"] },
  { key: "automation", label: "Automation", icon: "⚡", note: "Rules and workflows that run themselves", items: ["rules", "workflow", "transitions"] },
];

/**
 * The Admin panel - every administrator-facing capability lives here
 * under one nav item instead of scattered across the sidebar (Users used
 * to be its own top-level entry): user accounts and roles, business
 * profile/branding, custom fields, business rules, workflow automation,
 * ID/number formats, and which Dashboard KPIs show. Each tab is its own
 * previously-standalone screen, unchanged internally.
 *
 * Landing on Admin shows a categorized home (Workspace/Access/
 * Customization/Automation) rather than jumping straight into a tab -
 * clicking a category item opens that tool directly, with a breadcrumb
 * back to the landing page. Re-entering Admin (this component remounting)
 * always starts on the landing page, the same as Setup always reopening
 * Setup Home in Salesforce.
 */
export function AdminPanel() {
  const [view, setView] = useState<"landing" | "tool">("landing");
  const [tab, setTab] = useState<AdminTab>("users");
  const queryClient = useQueryClient();
  const workspace = useQuery({ queryKey: ["workspaceStatus"], queryFn: () => api.workspaceStatus() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["workspaceStatus"] });
  }

  function openTab(key: AdminTab) {
    setTab(key);
    setView("tool");
  }

  if (view === "landing") {
    return (
      <div>
        <h2>Admin</h2>
        <p style={{ color: "var(--text-muted)" }}>
          Users and access, business branding, and the admin-configurable layer on top of the fixed schema: custom
          objects, relationships, custom fields, business rules, workflow automation, status transitions, number
          formats and Dashboard KPIs.
        </p>
        <div className="admin-landing-grid">
          {ADMIN_CATEGORIES.map((cat) => (
            <div key={cat.key} className="admin-cat-card">
              <div className="admin-cat-head">
                <span className="admin-cat-icon">{cat.icon}</span>
                <div>
                  <h3>{cat.label}</h3>
                  <p>{cat.note}</p>
                </div>
              </div>
              <div className="admin-cat-items">
                {cat.items.map((key) => (
                  <button key={key} className="admin-cat-item" onClick={() => openTab(key)}>
                    {tabLabel(key)}
                    <span className="admin-cat-arrow">→</span>
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="admin-breadcrumb">
        <button className="link-button" onClick={() => setView("landing")}>
          Admin
        </button>
        <span> / </span>
        <span>{tabLabel(tab)}</span>
      </div>
      <h2>{tabLabel(tab)}</h2>

      {tab === "users" && <Users />}

      {tab === "profile" && workspace.data && (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, alignItems: "start" }}>
          <ProfileForm workspace={workspace.data} onSaved={invalidate} />
          <LogoCard workspace={workspace.data} onChanged={invalidate} />
        </div>
      )}
      {tab === "profile" && !workspace.data && <p>Loading...</p>}

      {tab === "objects" && <CustomObjectsAdmin />}
      {tab === "relationships" && <RelationshipsAdmin />}
      {tab === "fields" && <CustomFieldsAdmin />}
      {tab === "rules" && <BusinessRulesAdmin />}
      {tab === "workflow" && <WorkflowAutomationAdmin />}
      {tab === "transitions" && <StatusTransitionsAdmin />}
      {tab === "numbering" && <NumberingAdmin />}
      {tab === "kpis" && <DashboardKpiAdmin />}
    </div>
  );
}

function ProfileForm({ workspace, onSaved }: { workspace: Workspace; onSaved: () => void }) {
  const [input, setInput] = useState<WorkspaceUpdate>({
    business_name: workspace.business_name,
    legal_name: workspace.legal_name,
    business_address: workspace.business_address,
    phone: workspace.phone,
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
        <div className="form-field full">
          <label>Phone</label>
          <input value={input.phone ?? ""} onChange={(e) => setInput({ ...input, phone: e.target.value || null })} />
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
