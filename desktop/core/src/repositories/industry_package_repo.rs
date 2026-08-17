//! Raw CRUD for every table migration 0027 added - see that migration's
//! own comment for what each table is for and `industry_package_service`
//! for the orchestration logic (validation, transactional install,
//! dependency resolution) that actually calls these.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::industry_package::{AppInstallRun, AppPackage, InstalledApp, PackageArtifact, RecommendedPermission};

// --- app_packages ----------------------------------------------------------

fn map_package(row: &rusqlite::Row) -> rusqlite::Result<AppPackage> {
    Ok(AppPackage {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        package_id: row.get("package_id")?,
        name: row.get("name")?,
        industry: row.get("industry")?,
        version: row.get("version")?,
        min_lanesra_version: row.get("min_lanesra_version")?,
        manifest_json: row.get("manifest_json")?,
        checksum: row.get("checksum")?,
        source: row.get("source")?,
        imported_at: row.get("imported_at")?,
        imported_by: row.get("imported_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert_package(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    package_id: &str,
    name: &str,
    industry: &str,
    version: &str,
    min_lanesra_version: &str,
    manifest_json: &str,
    checksum: &str,
    source: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<AppPackage> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO app_packages
            (id, workspace_id, package_id, name, industry, version, min_lanesra_version, manifest_json, checksum, source, imported_at, imported_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![id, workspace_id, package_id, name, industry, version, min_lanesra_version, manifest_json, checksum, source, now, actor_user_id],
    )?;
    get_package(conn, id).map(|p| p.expect("just inserted"))
}

pub fn get_package(conn: &Connection, id: &str) -> rusqlite::Result<Option<AppPackage>> {
    conn.query_row("SELECT * FROM app_packages WHERE id = ?1", [id], map_package)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_package_by_version(conn: &Connection, workspace_id: &str, package_id: &str, version: &str) -> rusqlite::Result<Option<AppPackage>> {
    conn.query_row(
        "SELECT * FROM app_packages WHERE workspace_id = ?1 AND package_id = ?2 AND version = ?3",
        (workspace_id, package_id, version),
        map_package,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Every imported version of every package, newest import first - the
/// Admin -> App Catalog list.
pub fn list_packages(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AppPackage>> {
    let mut stmt = conn.prepare("SELECT * FROM app_packages WHERE workspace_id = ?1 ORDER BY imported_at DESC")?;
    let rows = stmt.query_map([workspace_id], map_package)?.collect();
    rows
}

// --- app_dependencies --------------------------------------------------

/// Recorded for future querying (spec 13.2 lists this as its own registry
/// table) - actual dependency resolution during install reads the
/// manifest's own `dependencies` list in memory rather than round-
/// tripping through this table, so there's no corresponding "list" used
/// on the install path.
pub fn insert_dependency(
    conn: &Connection,
    id: &str,
    app_package_id: &str,
    dependency_package_id: &str,
    version_constraint: &str,
    is_required: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO app_dependencies (id, app_package_id, dependency_package_id, version_constraint, is_required)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, app_package_id, dependency_package_id, version_constraint, is_required],
    )?;
    Ok(())
}

// --- installed_apps ------------------------------------------------------

fn map_installed_app(row: &rusqlite::Row) -> rusqlite::Result<InstalledApp> {
    let recommended_json: String = row.get("recommended_permissions_json")?;
    let recommended_permissions: Vec<RecommendedPermission> = serde_json::from_str(&recommended_json).unwrap_or_default();
    Ok(InstalledApp {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        package_id: row.get("package_id")?,
        name: row.get("name")?,
        icon: row.get("icon")?,
        industry: row.get("industry")?,
        description: row.get("description")?,
        installed_version: row.get("installed_version")?,
        status: row.get("status")?,
        app_definition_id: row.get("app_definition_id")?,
        recommended_permissions,
        installed_at: row.get("installed_at")?,
        installed_by: row.get("installed_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        deactivated_at: row.get("deactivated_at")?,
        deactivated_by: row.get("deactivated_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_installed_app(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    package_id: &str,
    name: &str,
    icon: &str,
    industry: &str,
    description: Option<&str>,
    installed_version: &str,
    app_definition_id: Option<&str>,
    recommended_permissions_json: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<InstalledApp> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO installed_apps
            (id, workspace_id, package_id, name, icon, industry, description, installed_version, status,
             app_definition_id, recommended_permissions_json, installed_at, installed_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?10, ?11, ?12, ?11, ?12)",
        rusqlite::params![id, workspace_id, package_id, name, icon, industry, description, installed_version, app_definition_id, recommended_permissions_json, now, actor_user_id],
    )?;
    get_installed_app(conn, id).map(|a| a.expect("just inserted"))
}

pub fn get_installed_app(conn: &Connection, id: &str) -> rusqlite::Result<Option<InstalledApp>> {
    conn.query_row("SELECT * FROM installed_apps WHERE id = ?1", [id], map_installed_app)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_installed_app_by_package(conn: &Connection, workspace_id: &str, package_id: &str) -> rusqlite::Result<Option<InstalledApp>> {
    conn.query_row(
        "SELECT * FROM installed_apps WHERE workspace_id = ?1 AND package_id = ?2",
        (workspace_id, package_id),
        map_installed_app,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_installed_apps(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<InstalledApp>> {
    let mut stmt = conn.prepare("SELECT * FROM installed_apps WHERE workspace_id = ?1 ORDER BY installed_at DESC")?;
    let rows = stmt.query_map([workspace_id], map_installed_app)?.collect();
    rows
}

/// Deactivate (status='deactivated', stamps deactivated_at/by) or
/// reactivate (status='active', clears deactivated_at/by) - the spec's
/// "default removal behavior is Deactivate App", non-destructive either
/// direction.
pub fn set_status(conn: &Connection, id: &str, active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    if active {
        conn.execute(
            "UPDATE installed_apps SET status = 'active', deactivated_at = NULL, deactivated_by = NULL, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
            rusqlite::params![now, actor_user_id, id],
        )?;
    } else {
        conn.execute(
            "UPDATE installed_apps SET status = 'deactivated', deactivated_at = ?1, deactivated_by = ?2, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
            rusqlite::params![now, actor_user_id, id],
        )?;
    }
    Ok(())
}

// --- package_artifacts ---------------------------------------------------

fn map_artifact(row: &rusqlite::Row) -> rusqlite::Result<PackageArtifact> {
    Ok(PackageArtifact {
        id: row.get("id")?,
        installed_app_id: row.get("installed_app_id")?,
        artifact_type: row.get("artifact_type")?,
        metadata_id: row.get("metadata_id")?,
        origin_version: row.get("origin_version")?,
        is_locally_customized: row.get("is_locally_customized")?,
        created_at: row.get("created_at")?,
    })
}

pub fn insert_artifact(
    conn: &Connection,
    id: &str,
    installed_app_id: &str,
    artifact_type: &str,
    metadata_id: &str,
    origin_version: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO package_artifacts (id, installed_app_id, artifact_type, metadata_id, origin_version, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, installed_app_id, artifact_type, metadata_id, origin_version, now_iso()],
    )?;
    Ok(())
}

pub fn list_artifacts(conn: &Connection, installed_app_id: &str) -> rusqlite::Result<Vec<PackageArtifact>> {
    let mut stmt = conn.prepare("SELECT * FROM package_artifacts WHERE installed_app_id = ?1 ORDER BY created_at")?;
    let rows = stmt.query_map([installed_app_id], map_artifact)?.collect();
    rows
}

// --- app_install_runs ----------------------------------------------------

fn map_run(row: &rusqlite::Row) -> rusqlite::Result<AppInstallRun> {
    Ok(AppInstallRun {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        package_id: row.get("package_id")?,
        package_version: row.get("package_version")?,
        action: row.get("action")?,
        status: row.get("status")?,
        started_at: row.get("started_at")?,
        completed_at: row.get("completed_at")?,
        backup_snapshot_path: row.get("backup_snapshot_path")?,
        error_message: row.get("error_message")?,
        actor_user_id: row.get("actor_user_id")?,
    })
}

pub fn start_run(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    package_id: &str,
    package_version: &str,
    action: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO app_install_runs (id, workspace_id, package_id, package_version, action, status, started_at, actor_user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, 'running', ?6, ?7)",
        rusqlite::params![id, workspace_id, package_id, package_version, action, now_iso(), actor_user_id],
    )?;
    Ok(())
}

/// Finalizes a run row - this always runs *outside* the install's own
/// transaction (whether that transaction committed or rolled back), so a
/// failed install still leaves a readable "why it failed" record instead
/// of the failure itself wiping the evidence.
pub fn complete_run(conn: &Connection, id: &str, status: &str, error_message: Option<&str>, backup_snapshot_path: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE app_install_runs SET status = ?1, completed_at = ?2, error_message = ?3, backup_snapshot_path = COALESCE(?4, backup_snapshot_path) WHERE id = ?5",
        rusqlite::params![status, now_iso(), error_message, backup_snapshot_path, id],
    )?;
    Ok(())
}

pub fn list_runs(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AppInstallRun>> {
    let mut stmt = conn.prepare("SELECT * FROM app_install_runs WHERE workspace_id = ?1 ORDER BY started_at DESC")?;
    let rows = stmt.query_map([workspace_id], map_run)?.collect();
    rows
}
