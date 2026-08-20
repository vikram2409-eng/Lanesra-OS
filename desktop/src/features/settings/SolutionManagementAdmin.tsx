import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { InstalledApp, WorkspaceArtifact, WorkspaceDependency } from "../../lib/types";
import { artifactTypeLabel } from "./IndustryPackagesAdmin";

type SolutionTab = "packages" | "components" | "dependencies";

const SOLUTION_TABS: { key: SolutionTab; label: string }[] = [
  { key: "packages", label: "Solution Packages" },
  { key: "components", label: "Components" },
  { key: "dependencies", label: "Dependencies" },
];

/**
 * Solution Management (Solution Packages & Admin IA design spec, Phase
 * 1): a read-only landing screen answering "what's installed, what did
 * it create, and what does it depend on" - the exact question a reported
 * "can't tell what I've customized beyond what I installed" bug
 * surfaced the need for. Every byte of data here already existed
 * (`app_packages`/`installed_apps`/`package_artifacts`/`app_dependencies`,
 * migration 0027) - this is a new frame on it, not a new registry.
 *
 * Deliberately not built yet, per the Phase 1 plan: a Publishers tab (no
 * real Publisher entity exists until Phase 2), a Managed/Unmanaged
 * distinction (every row is Managed today - Unmanaged doesn't exist
 * until an admin can package their own customizations), and any
 * write/deploy action (install/deactivate stay on Admin -> App Catalog,
 * the screen that already owns them - this is a browsing surface, not a
 * second copy of that control surface).
 */
export function SolutionManagementAdmin() {
  const [tab, setTab] = useState<SolutionTab>("packages");

  const installed = useQuery({ queryKey: ["installedApps"], queryFn: () => api.listInstalledApps() });
  const artifacts = useQuery({ queryKey: ["packageArtifactsForWorkspace"], queryFn: () => api.listPackageArtifactsForWorkspace() });
  const dependencies = useQuery({ queryKey: ["packageDependencies"], queryFn: () => api.listPackageDependencies() });

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Every industry app installed in this workspace, what it created, and what it depends on - read-only. Install,
        deactivate or reactivate an app from <b>Admin → App Catalog</b>; this is where you see the result.
      </p>

      <div className="tab-row">
        {SOLUTION_TABS.map((t) => (
          <button key={t.key} className={`tab${tab === t.key ? " active" : ""}`} onClick={() => setTab(t.key)}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === "packages" && (
        <SolutionPackagesTab installed={installed.data ?? []} artifacts={artifacts.data ?? []} dependencies={dependencies.data ?? []} loading={installed.isLoading} />
      )}
      {tab === "components" && <ComponentsTab artifacts={artifacts.data ?? []} loading={artifacts.isLoading} />}
      {tab === "dependencies" && <DependenciesTab dependencies={dependencies.data ?? []} loading={dependencies.isLoading} />}
    </div>
  );
}

function SolutionPackagesTab({
  installed,
  artifacts,
  dependencies,
  loading,
}: {
  installed: InstalledApp[];
  artifacts: WorkspaceArtifact[];
  dependencies: WorkspaceDependency[];
  loading: boolean;
}) {
  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Solution Packages</h3>
      {loading && <p>Loading...</p>}
      {!loading && installed.length === 0 && (
        <p className="empty-state">
          Nothing installed yet. Install a reference package from <b>Admin → App Catalog</b> to see it here.
        </p>
      )}
      {installed.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Version</th>
              <th>Status</th>
              <th>Components</th>
              <th>Dependencies</th>
            </tr>
          </thead>
          <tbody>
            {installed.map((app) => {
              const componentCount = artifacts.filter((a) => a.artifact.installed_app_id === app.id).length;
              const dependencyCount = dependencies.filter((d) => d.package_id === app.package_id).length;
              return (
                <tr key={app.id}>
                  <td>
                    {app.icon} {app.name}
                  </td>
                  <td>
                    <span className="badge">Managed</span>
                  </td>
                  <td>{app.installed_version}</td>
                  <td>
                    <span className={`badge${app.status === "active" ? " badge-success" : ""}`}>{app.status}</span>
                  </td>
                  <td>{componentCount}</td>
                  <td>{dependencyCount}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ComponentsTab({ artifacts, loading }: { artifacts: WorkspaceArtifact[]; loading: boolean }) {
  const [filter, setFilter] = useState("");

  const byType = new Map<string, number>();
  for (const a of artifacts) {
    byType.set(a.artifact.artifact_type, (byType.get(a.artifact.artifact_type) ?? 0) + 1);
  }

  const needle = filter.trim().toLowerCase();
  const filtered = needle
    ? artifacts.filter(
        (a) =>
          a.installed_app_name.toLowerCase().includes(needle) ||
          artifactTypeLabel(a.artifact.artifact_type).toLowerCase().includes(needle) ||
          a.artifact.metadata_id.toLowerCase().includes(needle),
      )
    : artifacts;

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Components</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Every record any installed app created, across the whole workspace - what have you actually got beyond what
        you installed.
      </p>
      {loading && <p>Loading...</p>}
      {!loading && artifacts.length === 0 && <p className="empty-state">Nothing installed yet.</p>}
      {artifacts.length > 0 && (
        <>
          <div style={{ display: "flex", gap: 12, flexWrap: "wrap", marginBottom: 12 }}>
            {[...byType.entries()].map(([type, count]) => (
              <span key={type} className="badge">
                {count} {artifactTypeLabel(type)}
                {count === 1 ? "" : "s"}
              </span>
            ))}
          </div>
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter by app, type or id..."
            style={{ marginBottom: 8, width: "100%", maxWidth: 320 }}
          />
          <table>
            <thead>
              <tr>
                <th>Type</th>
                <th>Installed app</th>
                <th>Created by version</th>
                <th>Customized locally</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((a) => (
                <tr key={a.artifact.id}>
                  <td>{artifactTypeLabel(a.artifact.artifact_type)}</td>
                  <td>{a.installed_app_name}</td>
                  <td>{a.artifact.origin_version}</td>
                  <td>{a.artifact.is_locally_customized ? <span className="badge">Yes</span> : "—"}</td>
                </tr>
              ))}
              {filtered.length === 0 && (
                <tr>
                  <td colSpan={4} className="empty-state">
                    No components match "{filter}".
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </>
      )}
    </div>
  );
}

function DependenciesTab({ dependencies, loading }: { dependencies: WorkspaceDependency[]; loading: boolean }) {
  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Dependencies</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Every dependency declared by a package imported into this workspace, and whether it's currently satisfied.
      </p>
      {loading && <p>Loading...</p>}
      {!loading && dependencies.length === 0 && <p className="empty-state">No imported package declares a dependency.</p>}
      {dependencies.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Package</th>
              <th>Depends on</th>
              <th>Version</th>
              <th>Required</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {dependencies.map((d) => (
              <tr key={d.dependency.id}>
                <td>
                  {d.package_name} <span style={{ color: "var(--text-muted)" }}>v{d.package_version}</span>
                </td>
                <td>{d.dependency.dependency_package_id}</td>
                <td>{d.dependency.version_constraint}</td>
                <td>{d.dependency.is_required ? "Required" : "Optional"}</td>
                <td>
                  <span className={`badge${d.is_satisfied ? " badge-success" : ""}`}>{d.is_satisfied ? "Satisfied" : "Unsatisfied"}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
