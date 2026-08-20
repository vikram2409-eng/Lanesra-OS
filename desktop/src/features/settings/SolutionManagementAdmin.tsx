import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type {
  AppPackage,
  InstalledApp,
  PackageUpdateDiff,
  Publisher,
  PublisherInput,
  WorkspaceComponent,
  WorkspaceDependency,
} from "../../lib/types";
import { artifactTypeLabel } from "./IndustryPackagesAdmin";

type SolutionTab = "packages" | "components" | "dependencies" | "publishers";

const SOLUTION_TABS: { key: SolutionTab; label: string }[] = [
  { key: "packages", label: "Solution Packages" },
  { key: "components", label: "Components" },
  { key: "dependencies", label: "Dependencies" },
  { key: "publishers", label: "Publishers" },
];

function downloadJson(filename: string, content: string): void {
  const blob = new Blob([content], { type: "application/json;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Solution Management (Solution Packages & Admin IA design spec).
 * Phase 1: a read-only landing screen answering "what's installed, what
 * did it create, and what does it depend on". Phase 2 added a real
 * Publisher registry with enforced namespace validation. Phase 3 (this
 * revision) adds the rest of what a Solution Packages spec means:
 *   - Components now comes from `solution_components` (component-tagging,
 *     migration 0030) instead of only `package_artifacts` - every
 *     hand-built customization is visible here too, not just what an
 *     install created.
 *   - A synthetic "Local Workspace" row in Solution Packages, the
 *     Managed/Unmanaged distinction's Unmanaged half - everything tagged
 *     to the `local` publisher, with a real Export action that builds a
 *     re-importable manifest (no fake `app_packages` row ever created for
 *     it).
 *   - Per-package Releases: every imported version, oldest first - each
 *     `app_packages` row already is an immutable snapshot, so this is
 *     just a new view over existing data.
 *   - Update-with-diff: once a newer version of an installed package has
 *     been imported (via Admin -> App Catalog's existing Review step), an
 *     "Update available" action here previews Added/Modified/Removed
 *     before applying it - replacing the old "reinstalling an installed
 *     package_id is rejected outright" dead end.
 *
 * Still not built, per the plan's forward roadmap: the full 5-layer
 * extension model and a UI for attributing a specific hand-built
 * component to a publisher other than `local` (registering a publisher
 * doesn't yet let you *reassign* an existing component to it).
 */
export function SolutionManagementAdmin() {
  const [tab, setTab] = useState<SolutionTab>("packages");

  const installed = useQuery({ queryKey: ["installedApps"], queryFn: () => api.listInstalledApps() });
  const packages = useQuery({ queryKey: ["industryPackages"], queryFn: () => api.listIndustryPackages() });
  const components = useQuery({ queryKey: ["solutionComponents"], queryFn: () => api.listSolutionComponents() });
  const dependencies = useQuery({ queryKey: ["packageDependencies"], queryFn: () => api.listPackageDependencies() });
  const publishers = useQuery({ queryKey: ["publishers"], queryFn: () => api.listPublishers() });
  const localSummary = useQuery({ queryKey: ["localWorkspaceSummary"], queryFn: () => api.getLocalWorkspaceSummary() });

  return (
    <div style={{ display: "grid", gap: 16 }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Every industry app installed in this workspace, what it created, what it depends on, and who published it -
        plus everything you've built by hand, grouped as your Local Workspace. Install, deactivate or reactivate an
        app from <b>Admin → App Catalog</b>; this is where you see the result, review updates, and export your own
        customizations.
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
          components={components.data ?? []}
          dependencies={dependencies.data ?? []}
          localSummary={localSummary.data ?? null}
          loading={installed.isLoading}
        />
      )}
      {tab === "components" && <ComponentsTab components={components.data ?? []} loading={components.isLoading} />}
      {tab === "dependencies" && <DependenciesTab dependencies={dependencies.data ?? []} loading={dependencies.isLoading} />}
      {tab === "publishers" && <PublishersTab publishers={publishers.data ?? []} packages={packages.data ?? []} loading={publishers.isLoading} />}
    </div>
  );
}

function SolutionPackagesTab({
  installed,
  packages,
  publishers,
  components,
  dependencies,
  localSummary,
  loading,
}: {
  installed: InstalledApp[];
  packages: AppPackage[];
  publishers: Publisher[];
  components: WorkspaceComponent[];
  dependencies: WorkspaceDependency[];
  localSummary: { publisher_id: string; component_count: number; components_by_type: [string, number][] } | null;
  loading: boolean;
}) {
  const queryClient = useQueryClient();
  const publisherById = new Map(publishers.map((p) => [p.id, p]));
  // installed_apps doesn't carry publisher_id itself - only the
  // app_packages row it was installed from does - so join through the
  // (package_id, version) pair the unique index on app_packages already
  // guarantees identifies exactly one row.
  const publisherForApp = (app: InstalledApp): Publisher | undefined => {
    const pkg = packages.find((p) => p.package_id === app.package_id && p.version === app.installed_version);
    return pkg?.publisher_id ? publisherById.get(pkg.publisher_id) : undefined;
  };
  // Any imported version of this app's package_id that isn't the one
  // currently installed is a candidate update - the newest one by import
  // time is what "Update available" offers.
  const updateCandidateFor = (app: InstalledApp): AppPackage | undefined => {
    const others = packages.filter((p) => p.package_id === app.package_id && p.version !== app.installed_version);
    return others.sort((a, b) => (a.imported_at < b.imported_at ? 1 : -1))[0];
  };

  const [releasesOpenFor, setReleasesOpenFor] = useState<string | null>(null);
  const [updateModalPackageId, setUpdateModalPackageId] = useState<string | null>(null);

  const exportLocal = useMutation({
    mutationFn: () => api.exportLocalWorkspace(),
    onSuccess: (json) => downloadJson("local-workspace-export.lanesra.json", json),
  });

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Solution Packages</h3>
      {loading && <p>Loading...</p>}
      {!loading && installed.length === 0 && !localSummary?.component_count && (
        <p className="empty-state">
          Nothing installed yet. Install a reference package from <b>Admin → App Catalog</b> to see it here.
        </p>
      )}
      {(installed.length > 0 || !!localSummary?.component_count) && (
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
              <th></th>
            </tr>
          </thead>
          <tbody>
            {installed.map((app) => {
              const componentCount = components.filter((c) => c.component.installed_app_id === app.id).length;
              const dependencyCount = dependencies.filter((d) => d.package_id === app.package_id).length;
              const publisher = publisherForApp(app);
              const updateCandidate = updateCandidateFor(app);
              const releasesOpen = releasesOpenFor === app.package_id;
              return (
                <>
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
                    <td style={{ display: "flex", gap: 6, whiteSpace: "nowrap" }}>
                      <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => setReleasesOpenFor(releasesOpen ? null : app.package_id)}>
                        {releasesOpen ? "Hide releases" : "Releases"}
                      </button>
                      {updateCandidate && (
                        <button className="btn btn-primary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => setUpdateModalPackageId(updateCandidate.id)}>
                          Update to v{updateCandidate.version}
                        </button>
                      )}
                    </td>
                  </tr>
                  {releasesOpen && (
                    <tr key={`${app.id}-releases`}>
                      <td colSpan={8} style={{ background: "var(--bg-subtle, rgba(0,0,0,0.02))" }}>
                        <ReleasesPanel packageId={app.package_id} installedVersion={app.installed_version} />
                      </td>
                    </tr>
                  )}
                </>
              );
            })}
            {!!localSummary?.component_count && (
              <tr>
                <td>🧩 Local Workspace</td>
                <td>local</td>
                <td>
                  <span className="badge">Unmanaged</span>
                </td>
                <td>—</td>
                <td>—</td>
                <td>{localSummary.component_count}</td>
                <td>—</td>
                <td>
                  <button className="btn btn-secondary" style={{ fontSize: 12, padding: "4px 8px" }} onClick={() => exportLocal.mutate()} disabled={exportLocal.isPending}>
                    {exportLocal.isPending ? "Exporting..." : "Export"}
                  </button>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      )}

      {updateModalPackageId && (
        <UpdateDiffModal
          newAppPackageId={updateModalPackageId}
          onClose={() => setUpdateModalPackageId(null)}
          onApplied={() => {
            setUpdateModalPackageId(null);
            queryClient.invalidateQueries({ queryKey: ["installedApps"] });
            queryClient.invalidateQueries({ queryKey: ["industryPackages"] });
            queryClient.invalidateQueries({ queryKey: ["solutionComponents"] });
            queryClient.invalidateQueries({ queryKey: ["localWorkspaceSummary"] });
          }}
        />
      )}
    </div>
  );
}

/** Every imported version of one package, oldest first - each
 * `app_packages` row is already an immutable per-version snapshot, so
 * this is a real Releases view over existing data, not a new registry. */
function ReleasesPanel({ packageId, installedVersion }: { packageId: string; installedVersion: string }) {
  const versions = useQuery({ queryKey: ["packageVersions", packageId], queryFn: () => api.listPackageVersions(packageId) });
  if (versions.isLoading) return <p style={{ margin: "8px 0" }}>Loading releases...</p>;
  const rows = versions.data ?? [];
  if (rows.length === 0) return <p className="empty-state">No versions recorded.</p>;
  return (
    <div style={{ padding: "10px 4px" }}>
      <table style={{ marginBottom: 0 }}>
        <thead>
          <tr>
            <th>Version</th>
            <th>Imported</th>
            <th>Source</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.id}>
              <td>{r.version}</td>
              <td>{new Date(r.imported_at).toLocaleString()}</td>
              <td>{r.source}</td>
              <td>{r.version === installedVersion ? <span className="badge badge-success">Installed</span> : null}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Update-with-diff's review step: shows what `plan_package_update`
 * reports before `apply_package_update` runs unconditionally. Objects/
 * fields get a real per-key Added/Modified/Removed diff; everything else
 * is a single added-count - see the Rust core's plan_update doc comment
 * for exactly why. */
function UpdateDiffModal({ newAppPackageId, onClose, onApplied }: { newAppPackageId: string; onClose: () => void; onApplied: () => void }) {
  const diff = useQuery({ queryKey: ["packageUpdateDiff", newAppPackageId], queryFn: () => api.planPackageUpdate(newAppPackageId) });
  const apply = useMutation({
    mutationFn: () => api.applyPackageUpdate(newAppPackageId),
    onSuccess: onApplied,
  });

  const kindBadgeClass = (kind: string) => (kind === "added" ? "badge-success" : kind === "removed" ? "badge-danger" : "");

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" style={{ maxWidth: 560 }} onClick={(e) => e.stopPropagation()}>
        <h3 style={{ marginTop: 0 }}>Review update</h3>
        {diff.isLoading && <p>Comparing versions...</p>}
        {diff.isError && <p className="error-banner">{diff.error instanceof ApiError ? diff.error.message : "Could not compute the diff"}</p>}
        {diff.data && <UpdateDiffView diff={diff.data} kindBadgeClass={kindBadgeClass} />}
        {apply.isError && (
          <div className="error-banner" style={{ marginTop: 12 }}>
            {apply.error instanceof ApiError ? apply.error.message : "Could not apply the update"}
          </div>
        )}
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 16 }}>
          <button className="btn btn-secondary" onClick={onClose} disabled={apply.isPending}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={() => apply.mutate()} disabled={!diff.data || apply.isPending}>
            {apply.isPending ? "Applying..." : "Apply update"}
          </button>
        </div>
      </div>
    </div>
  );
}

function UpdateDiffView({ diff, kindBadgeClass }: { diff: PackageUpdateDiff; kindBadgeClass: (kind: string) => string }) {
  const anyEntries = diff.objects.length > 0 || diff.fields.length > 0;
  const anyAddedCounts =
    diff.relationships_added > 0 || diff.business_rules_added > 0 || diff.workflows_added > 0 || diff.screen_layouts_added > 0 || diff.reports_added > 0;
  return (
    <div style={{ display: "grid", gap: 10 }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, margin: 0 }}>
        {diff.package_id}: v{diff.from_version} → v{diff.to_version}
      </p>
      {!anyEntries && !anyAddedCounts && <p className="empty-state">No changes detected - this version is identical to what's installed.</p>}
      {diff.objects.length > 0 && (
        <div>
          <b style={{ fontSize: 13 }}>Objects</b>
          {diff.objects.map((e) => (
            <div key={e.key} style={{ display: "flex", justifyContent: "space-between", fontSize: 13, padding: "2px 0" }}>
              <code>{e.key}</code>
              <span className={`badge ${kindBadgeClass(e.kind)}`}>{e.kind}</span>
            </div>
          ))}
        </div>
      )}
      {diff.fields.length > 0 && (
        <div>
          <b style={{ fontSize: 13 }}>Fields</b>
          {diff.fields.map((e) => (
            <div key={e.key} style={{ display: "flex", justifyContent: "space-between", fontSize: 13, padding: "2px 0" }}>
              <code>{e.key}</code>
              <span className={`badge ${kindBadgeClass(e.kind)}`}>{e.kind}</span>
            </div>
          ))}
        </div>
      )}
      {anyAddedCounts && (
        <div>
          <b style={{ fontSize: 13 }}>Also adds</b>
          <ul style={{ margin: "4px 0 0", paddingLeft: 18, fontSize: 13, color: "var(--text-muted)" }}>
            {diff.relationships_added > 0 && <li>{diff.relationships_added} new relationship{diff.relationships_added === 1 ? "" : "s"}</li>}
            {diff.business_rules_added > 0 && <li>{diff.business_rules_added} new business rule{diff.business_rules_added === 1 ? "" : "s"}</li>}
            {diff.workflows_added > 0 && <li>{diff.workflows_added} new workflow{diff.workflows_added === 1 ? "" : "s"}</li>}
            {diff.screen_layouts_added > 0 && <li>{diff.screen_layouts_added} new screen layout{diff.screen_layouts_added === 1 ? "" : "s"}</li>}
            {diff.reports_added > 0 && <li>{diff.reports_added} new report{diff.reports_added === 1 ? "" : "s"}</li>}
          </ul>
        </div>
      )}
    </div>
  );
}

function ComponentsTab({ components, loading }: { components: WorkspaceComponent[]; loading: boolean }) {
  const [filter, setFilter] = useState("");

  const byType = new Map<string, number>();
  for (const c of components) {
    byType.set(c.component.artifact_type, (byType.get(c.component.artifact_type) ?? 0) + 1);
  }

  const needle = filter.trim().toLowerCase();
  const filtered = needle
    ? components.filter(
        (c) =>
          c.publisher_name.toLowerCase().includes(needle) ||
          (c.installed_app_name ?? "").toLowerCase().includes(needle) ||
          artifactTypeLabel(c.component.artifact_type).toLowerCase().includes(needle) ||
          c.component.metadata_id.toLowerCase().includes(needle),
      )
    : components;

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Components</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Every custom object, field, relationship, business rule, workflow, screen layout and report in this
        workspace - hand-built or installed by an app - and who owns it.
      </p>
      {loading && <p>Loading...</p>}
      {!loading && components.length === 0 && <p className="empty-state">Nothing here yet.</p>}
      {components.length > 0 && (
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
            placeholder="Filter by publisher, app, type or id..."
            style={{ marginBottom: 8, width: "100%", maxWidth: 320 }}
          />
          <table>
            <thead>
              <tr>
                <th>Type</th>
                <th>Publisher</th>
                <th>Source</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((c) => (
                <tr key={c.component.id}>
                  <td>{artifactTypeLabel(c.component.artifact_type)}</td>
                  <td>
                    {c.publisher_name}
                    {c.is_local && (
                      <span className="badge" style={{ marginLeft: 6 }}>
                        Local
                      </span>
                    )}
                  </td>
                  <td>{c.installed_app_name ?? "Hand-built"}</td>
                </tr>
              ))}
              {filtered.length === 0 && (
                <tr>
                  <td colSpan={3} className="empty-state">
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
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 16 }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <h3 style={{ marginTop: 0 }}>Publishers</h3>
          <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
            Who a package's namespace belongs to. Every package_id is expected to be "&lt;publisher-key&gt;.&lt;name&gt;" -
            importing a package under an unregistered key is rejected until its publisher is registered here.
          </p>
        </div>
        <button className="btn btn-primary" style={{ flexShrink: 0 }} onClick={() => setAdding((v) => !v)}>
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
