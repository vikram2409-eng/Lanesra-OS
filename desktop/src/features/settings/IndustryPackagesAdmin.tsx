import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AppPackage, InstalledApp, InstalledAppDetail } from "../../lib/types";
import { PackageDetailsPanel } from "./PackageDetails";

/**
 * Industry Data Model foundations (roadmap "Industry Data Model"): the
 * Admin -> App Catalog screen the dev spec describes as "Admin -> Apps &
 * Industry Models -> App Catalog -> Review -> Validate -> Install" - a
 * way to import a package manifest, review its "Details" (see
 * PackageDetailsPanel below) before committing, install it, and see/
 * deactivate what's installed. Every primitive an install actually
 * touches (Custom Objects, Business Rules, Workflow Automation, Screen/
 * App Builder, Dashboards, Reports, Numbering, App Builder) already has
 * its own Admin screen; this is purely the import/review/install/
 * deactivate control surface on top, mirroring the Rust core's
 * `industry_package_service`.
 */
export function IndustryPackagesAdmin() {
  const [manifestJson, setManifestJson] = useState("");
  const [importError, setImportError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [detailId, setDetailId] = useState<string | null>(null);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const packages = useQuery({ queryKey: ["industryPackages"], queryFn: () => api.listIndustryPackages() });
  const installed = useQuery({ queryKey: ["installedApps"], queryFn: () => api.listInstalledApps() });
  const detail = useQuery({
    queryKey: ["installedAppDetail", detailId],
    queryFn: () => api.getInstalledAppDetail(detailId as string),
    enabled: detailId !== null,
  });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["industryPackages"] });
    queryClient.invalidateQueries({ queryKey: ["installedApps"] });
    // A just-installed app may publish an App Builder AppDefinition -
    // keep the sidebar App Switcher (and Apps admin tab) in sync the same
    // way AppsAdmin's own mutations do.
    queryClient.invalidateQueries({ queryKey: ["accessibleApps"] });
    queryClient.invalidateQueries({ queryKey: ["apps"] });
  }

  const loadStarter = useMutation({
    mutationFn: (key: string) => api.getReferencePackageManifest(key),
    onSuccess: (manifestJsonText) => {
      setImportError(null);
      setManifestJson(manifestJsonText);
    },
    onError: (err) => setImportError(err instanceof ApiError ? err.message : "Could not load that starter package"),
  });

  const importPackage = useMutation({
    mutationFn: () => api.importIndustryPackage({ manifest_json: manifestJson }),
    onSuccess: () => {
      setImportError(null);
      setManifestJson("");
      invalidate();
    },
    onError: (err) => setImportError(err instanceof ApiError ? err.message : "Could not import that package"),
  });

  const install = useMutation({
    mutationFn: (appPackageId: string) => api.installIndustryPackage(appPackageId),
    onSuccess: () => {
      setActionError(null);
      invalidate();
    },
    onError: (err) => setActionError(err instanceof ApiError ? err.message : "Install failed"),
  });

  const deactivate = useMutation({
    mutationFn: (id: string) => api.deactivateInstalledApp(id),
    onSuccess: () => {
      setActionError(null);
      invalidate();
    },
    onError: (err) => setActionError(err instanceof ApiError ? err.message : "Could not deactivate"),
  });

  const reactivate = useMutation({
    mutationFn: (id: string) => api.reactivateInstalledApp(id),
    onSuccess: () => {
      setActionError(null);
      invalidate();
    },
    onError: (err) => setActionError(err instanceof ApiError ? err.message : "Could not reactivate"),
  });

  const packageList = packages.data ?? [];
  const installedList = installed.data ?? [];
  const installedPackageIds = new Set(installedList.map((a) => a.package_id));

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <div className="card">
        <h3 style={{ marginTop: 0 }}>Import a package</h3>
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
          Paste a Lanesra industry app package manifest (JSON), or load a bundled starter below to review first.
          Importing only adds it to this workspace's local catalog for review - nothing is created until you
          install it.
        </p>
        {importError && <div className="error-banner">{importError}</div>}
        <div style={{ marginBottom: 8, display: "flex", gap: 8 }}>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("field_service")}>
            {loadStarter.isPending ? "Loading..." : "Load Field Service starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("property_management")}>
            {loadStarter.isPending ? "Loading..." : "Load Property Management starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("construction")}>
            {loadStarter.isPending ? "Loading..." : "Load Construction & Contractors starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("professional_services")}>
            {loadStarter.isPending ? "Loading..." : "Load Professional Services starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("practice_admin")}>
            {loadStarter.isPending ? "Loading..." : "Load Practice Administration starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("recruitment")}>
            {loadStarter.isPending ? "Loading..." : "Load Recruitment & Staffing starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("real_estate")}>
            {loadStarter.isPending ? "Loading..." : "Load Real Estate Brokerage starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("legal_practice")}>
            {loadStarter.isPending ? "Loading..." : "Load Legal Practice starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("nonprofit_association")}>
            {loadStarter.isPending ? "Loading..." : "Load Nonprofit & Association starter"}
          </button>
          <button className="btn" disabled={loadStarter.isPending} onClick={() => loadStarter.mutate("auto_service")}>
            {loadStarter.isPending ? "Loading..." : "Load Auto Repair & Service Garage starter"}
          </button>
        </div>
        <textarea
          value={manifestJson}
          onChange={(e) => setManifestJson(e.target.value)}
          rows={8}
          placeholder='{"format_version": 1, "package_id": "lanesra.field_service", ...}'
          style={{ width: "100%", fontFamily: "monospace", fontSize: 12 }}
        />
        <div style={{ marginTop: 8 }}>
          <button
            className="btn btn-primary"
            disabled={!manifestJson.trim() || importPackage.isPending}
            onClick={() => importPackage.mutate()}
          >
            {importPackage.isPending ? "Importing..." : "Import"}
          </button>
        </div>
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Imported packages</h3>
        {actionError && <div className="error-banner">{actionError}</div>}
        {packages.isLoading && <p>Loading...</p>}
        {!packages.isLoading && packageList.length === 0 && <p className="empty-state">Nothing imported yet.</p>}
        {packageList.length > 0 && (
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Industry</th>
                <th>Version</th>
                <th>Min Lanesra</th>
                <th>Imported</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {packageList.map((p: AppPackage) => (
                <>
                  <tr key={p.id}>
                    <td>
                      <button className="link-button" onClick={() => setPreviewId(previewId === p.id ? null : p.id)}>
                        {p.name}
                      </button>
                    </td>
                    <td>{p.industry}</td>
                    <td>{p.version}</td>
                    <td>{p.min_lanesra_version}</td>
                    <td>{new Date(p.imported_at).toLocaleString()}</td>
                    <td style={{ display: "flex", gap: 6 }}>
                      <button className="btn" onClick={() => setPreviewId(previewId === p.id ? null : p.id)}>
                        {previewId === p.id ? "Hide details" : "Details"}
                      </button>
                      {installedPackageIds.has(p.package_id) ? (
                        <span className="badge">Installed</span>
                      ) : (
                        <button className="btn btn-primary" disabled={install.isPending} onClick={() => install.mutate(p.id)}>
                          {install.isPending ? "Installing..." : "Install"}
                        </button>
                      )}
                    </td>
                  </tr>
                  {previewId === p.id && (
                    <tr>
                      <td colSpan={6}>
                        <PackageDetailsPanel manifestJson={p.manifest_json} />
                        {!installedPackageIds.has(p.package_id) && (
                          <button className="btn btn-primary" disabled={install.isPending} onClick={() => install.mutate(p.id)}>
                            {install.isPending ? "Installing..." : "Install this package"}
                          </button>
                        )}
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Installed apps</h3>
        {installed.isLoading && <p>Loading...</p>}
        {!installed.isLoading && installedList.length === 0 && <p className="empty-state">Nothing installed yet.</p>}
        {installedList.length > 0 && (
          <table>
            <thead>
              <tr>
                <th>App</th>
                <th>Industry</th>
                <th>Version</th>
                <th>Status</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {installedList.map((a: InstalledApp) => (
                <>
                  <tr key={a.id}>
                    <td>
                      <button className="link-button" onClick={() => setDetailId(detailId === a.id ? null : a.id)}>
                        {a.icon} {a.name}
                      </button>
                    </td>
                    <td>{a.industry}</td>
                    <td>{a.installed_version}</td>
                    <td>
                      <span className={`badge${a.status === "active" ? " badge-success" : ""}`}>{a.status}</span>
                    </td>
                    <td>
                      {a.status === "active" ? (
                        <button className="btn btn-danger" disabled={deactivate.isPending} onClick={() => deactivate.mutate(a.id)}>
                          Deactivate
                        </button>
                      ) : (
                        <button className="btn" disabled={reactivate.isPending} onClick={() => reactivate.mutate(a.id)}>
                          Reactivate
                        </button>
                      )}
                    </td>
                  </tr>
                  {detailId === a.id && detail.data && (
                    <tr>
                      <td colSpan={5}>
                        <InstalledAppDetailPanel detail={detail.data} />
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

/** Recommended permissions are informational only (spec: "always reviewed
 * by administrator before activation") - shown here as a checklist to
 * apply by hand from the Users/Apps screens, never auto-granted by
 * install itself. Artifacts are grouped by type so an admin can see at a
 * glance what an install actually created, and jump to the matching Admin
 * screen (Custom Objects, Business Rules, ...) to review it in place. */
function InstalledAppDetailPanel({ detail }: { detail: InstalledAppDetail }) {
  const byType = new Map<string, number>();
  for (const artifact of detail.artifacts) {
    byType.set(artifact.artifact_type, (byType.get(artifact.artifact_type) ?? 0) + 1);
  }

  return (
    <div style={{ padding: "8px 0" }}>
      {detail.app.description && <p style={{ color: "var(--text-muted)", fontSize: 13 }}>{detail.app.description}</p>}

      <h4 style={{ marginBottom: 4 }}>What this install created ({detail.artifacts.length})</h4>
      {detail.artifacts.length === 0 ? (
        <p className="empty-state">Nothing recorded.</p>
      ) : (
        <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13 }}>
          {[...byType.entries()].map(([type, count]) => (
            <li key={type}>
              {count} {artifactTypeLabel(type)}
              {count === 1 ? "" : "s"}
            </li>
          ))}
        </ul>
      )}

      {detail.app.recommended_permissions.length > 0 && (
        <>
          <h4 style={{ marginBottom: 4, marginTop: 12 }}>Recommended permissions (review and apply manually)</h4>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13 }}>
            {detail.app.recommended_permissions.map((p, i) => (
              <li key={i}>
                {p.role}: {p.level}
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}

/** Shared with SolutionManagementAdmin.tsx's Components tab - same
 * artifact_type vocabulary, same display labels. */
export function artifactTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    custom_object: "custom object",
    custom_field: "custom field",
    relationship_definition: "relationship",
    business_rule: "business rule",
    workflow_definition: "workflow",
    screen_layout: "screen layout",
    dashboard_layout: "dashboard",
    custom_report: "report",
    numbering_override: "numbering override",
    custom_record: "seed record",
  };
  return labels[type] ?? type;
}
