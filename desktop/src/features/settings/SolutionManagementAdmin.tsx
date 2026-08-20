import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AppPackage, InstalledApp, Publisher, PublisherInput, WorkspaceArtifact, WorkspaceDependency } from "../../lib/types";
import { artifactTypeLabel } from "./IndustryPackagesAdmin";

type SolutionTab = "packages" | "components" | "dependencies" | "publishers";

const SOLUTION_TABS: { key: SolutionTab; label: string }[] = [
  { key: "packages", label: "Solution Packages" },
  { key: "components", label: "Components" },
  { key: "dependencies", label: "Dependencies" },
  { key: "publishers", label: "Publishers" },
];

/**
 * Solution Management (Solution Packages & Admin IA design spec).
 * Phase 1: a read-only landing screen answering "what's installed, what
 * did it create, and what does it depend on" - the exact question a
 * reported "can't tell what I've customized beyond what I installed" bug
 * surfaced the need for. Every byte of data on the first three tabs
 * already existed (`app_packages`/`installed_apps`/`package_artifacts`/
 * `app_dependencies`, migration 0027) - this is a new frame on it, not a
 * new registry.
 * Phase 2 adds the fourth tab, Publishers: a real registry (migration
 * 0029) rather than a stub, since every package_id is namespaced by a
 * publisher key and import_package now enforces that namespace is
 * actually registered (see `publisher_service::resolve_for_package_id`).
 *
 * Still not built, per the plan's forward roadmap: a real Managed/
 * Unmanaged distinction (every package row is Managed today - Unmanaged
 * doesn't exist until an admin can package their own customizations),
 * component-tagging (attributing hand-built objects/fields/rules to the
 * auto-seeded `local` publisher), and any write/deploy action beyond
 * registering a publisher (install/deactivate stay on Admin -> App
 * Catalog, the screen that already owns them).
 */
export function SolutionManagementAdmin() {
  const [tab, setTab] = useState<SolutionTab>("packages");

  const installed = useQuery({ queryKey: ["installedApps"], queryFn: () => api.listInstalledApps() });
  const packages = useQuery({ queryKey: ["industryPackages"], queryFn: () => api.listIndustryPackages() });
  const artifacts = useQuery({ queryKey: ["packageArtifactsForWorkspace"], queryFn: () => api.listPackageArtifactsForWorkspace() });
  const dependencies = useQuery({ queryKey: ["packageDependencies"], queryFn: () => api.listPackageDependencies() });
  const publishers = useQuery({ queryKey: ["publishers"], queryFn: () => api.listPublishers() });

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Every industry app installed in this workspace, what it created, what it depends on, and who published it -
        read-only. Install, deactivate or reactivate an app from <b>Admin → App Catalog</b>; this is where you see
        the result.
      </p>

      <div className="tab-row">
        {SOLUTION_TABS.map((t) => (
          <button key={t.key} className={`tab${tab === t.key ? " active" : ""}`} onClick={() => setTab(t.key)}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === "packages" && (
        <SolutionPackagesTab
          installed={installed.data ?? []}
          packages={packages.data ?? []}
          publishers={publishers.data ?? []}
          artifacts={artifacts.data ?? []}
          dependencies={dependencies.data ?? []}
          loading={installed.isLoading}
        />
      )}
      {tab === "components" && <ComponentsTab artifacts={artifacts.data ?? []} loading={artifacts.isLoading} />}
      {tab === "dependencies" && <DependenciesTab dependencies={dependencies.data ?? []} loading={dependencies.isLoading} />}
      {tab === "publishers" && <PublishersTab publishers={publishers.data ?? []} packages={packages.data ?? []} loading={publishers.isLoading} />}
    </div>
  );
}

function SolutionPackagesTab({
  installed,
  packages,
  publishers,
  artifacts,
  dependencies,
  loading,
}: {
  installed: InstalledApp[];
  packages: AppPackage[];
  publishers: Publisher[];
  artifacts: WorkspaceArtifact[];
  dependencies: WorkspaceDependency[];
  loading: boolean;
}) {
  const publisherById = new Map(publishers.map((p) => [p.id, p]));
  // installed_apps doesn't carry publisher_id itself - only the
  // app_packages row it was installed from does - so join through the
  // (package_id, version) pair the unique index on app_packages already
  // guarantees identifies exactly one row.
  const publisherForApp = (app: InstalledApp): Publisher | undefined => {
    const pkg = packages.find((p) => p.package_id === app.package_id && p.version === app.installed_version);
    return pkg?.publisher_id ? publisherById.get(pkg.publisher_id) : undefined;
  };

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
              <th>Publisher</th>
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
              const publisher = publisherForApp(app);
              return (
                <tr key={app.id}>
                  <td>
                    {app.icon} {app.name}
                  </td>
                  <td>{publisher ? publisher.name : "—"}</td>
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

function PublishersTab({ publishers, packages, loading }: { publishers: Publisher[]; packages: AppPackage[]; loading: boolean }) {
  const queryClient = useQueryClient();
  const [adding, setAdding] = useState(false);
  const [input, setInput] = useState<PublisherInput>({ key: "", name: "", description: null });
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createPublisher({ ...input, key: input.key.trim().toLowerCase() }),
    onSuccess: () => {
      setError(null);
      setAdding(false);
      setInput({ key: "", name: "", description: null });
      queryClient.invalidateQueries({ queryKey: ["publishers"] });
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not register that publisher"),
  });

  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div>
          <h3 style={{ marginTop: 0 }}>Publishers</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Who a package's namespace belongs to. Every package_id is expected to be "&lt;publisher-key&gt;.&lt;name&gt;" -
            importing a package under an unregistered key is rejected until its publisher is registered here.
          </p>
        </div>
        <button className="btn btn-primary" onClick={() => setAdding((v) => !v)}>
          {adding ? "Cancel" : "+ Register publisher"}
        </button>
      </div>

      {adding && (
        <form
          className="form-grid"
          style={{ marginBottom: 16 }}
          onSubmit={(e) => {
            e.preventDefault();
            create.mutate();
          }}
        >
          {error && (
            <div className="error-banner" style={{ gridColumn: "1 / -1" }}>
              {error}
            </div>
          )}
          <div className="form-field">
            <label>Key</label>
            <input
              value={input.key}
              onChange={(e) => setInput({ ...input, key: e.target.value })}
              placeholder="acme"
              required
            />
          </div>
          <div className="form-field">
            <label>Name</label>
            <input
              value={input.name}
              onChange={(e) => setInput({ ...input, name: e.target.value })}
              placeholder="Acme Corp"
              required
            />
          </div>
          <div className="form-field full">
            <label>Description (optional)</label>
            <input
              value={input.description ?? ""}
              onChange={(e) => setInput({ ...input, description: e.target.value || null })}
            />
          </div>
          <div className="form-field full">
            <button className="btn btn-primary" type="submit" disabled={create.isPending}>
              {create.isPending ? "Registering..." : "Register publisher"}
            </button>
          </div>
        </form>
      )}

      {loading && <p>Loading...</p>}
      {!loading && publishers.length === 0 && <p className="empty-state">No publishers yet.</p>}
      {publishers.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Key</th>
              <th>Name</th>
              <th>Description</th>
              <th>Packages</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {publishers.map((p) => {
              const packageCount = packages.filter((pkg) => pkg.publisher_id === p.id).length;
              return (
                <tr key={p.id}>
                  <td>
                    <code>{p.key}</code>
                  </td>
                  <td>{p.name}</td>
                  <td style={{ color: "var(--text-muted)" }}>{p.description ?? "—"}</td>
                  <td>{packageCount}</td>
                  <td>
                    {p.is_official && <span className="badge">Official</span>}
                    {p.is_local && <span className="badge">Local</span>}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
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
