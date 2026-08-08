//! Whole-workspace backup and restore as a single `.lanesra` file: a zip
//! containing a SQLite snapshot of the live database plus a manifest.json
//! describing what's inside it, so restore can refuse anything it can't
//! safely open before it touches the live data.

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rusqlite::Connection;

use crate::db::{self, migrate};
use crate::domain::ids::{new_uuid, now_iso};
use crate::domain::{AppError, AppResult};
use crate::models::backup::{BackupManifest, BackupPackage};
use crate::repositories::{audit_repo, user_repo, workspace_repo};

const FORMAT_VERSION: u32 = 1;
const MANIFEST_ENTRY: &str = "manifest.json";
const DB_ENTRY: &str = "workspace.sqlite3";

fn zip_err<E: std::fmt::Display>(e: E) -> AppError {
    AppError::Validation(format!("backup package error: {e}"))
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation(
            "Only an Administrator can back up or restore the workspace".into(),
        ));
    }
    Ok(())
}

/// Snapshots the live database via SQLite's online backup API (safe to run
/// against a connection that's actively serving requests) and packages it
/// with a manifest into a `.lanesra` zip, returned base64-encoded so it
/// travels over Tauri IPC / JSON like every other command's response.
pub fn create_backup(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<BackupPackage> {
    require_admin(conn, actor_user_id)?;

    let workspace = workspace_repo::get_current(conn)?
        .ok_or_else(|| AppError::Validation("No workspace has been set up yet".into()))?;

    let snapshot_path = std::env::temp_dir().join(format!("lanesra-backup-{}.sqlite3", new_uuid()));
    {
        let mut dest = Connection::open(&snapshot_path)?;
        let backup = rusqlite::backup::Backup::new(conn, &mut dest).map_err(AppError::from)?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(250), None)
            .map_err(AppError::from)?;
    }
    let db_bytes = std::fs::read(&snapshot_path)
        .map_err(|e| AppError::Validation(format!("could not read backup snapshot: {e}")))?;
    let _ = std::fs::remove_file(&snapshot_path);

    let manifest = BackupManifest {
        format_version: FORMAT_VERSION,
        schema_version: migrate::schema_version(conn)?,
        workspace_name: workspace.business_name.clone(),
        created_at: now_iso(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let manifest_json =
        serde_json::to_vec_pretty(&manifest).map_err(|e| AppError::Validation(format!("could not build manifest: {e}")))?;

    let mut zip_bytes = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut zip_bytes));
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer.start_file(MANIFEST_ENTRY, options).map_err(zip_err)?;
        writer.write_all(&manifest_json).map_err(zip_err)?;
        writer.start_file(DB_ENTRY, options).map_err(zip_err)?;
        writer.write_all(&db_bytes).map_err(zip_err)?;
        writer.finish().map_err(zip_err)?;
    }

    audit_repo::record(
        conn,
        &workspace.id,
        actor_user_id,
        "backup",
        Some("workspace"),
        Some(&workspace.id),
        &format!("Exported a backup of '{}'", workspace.business_name),
        None,
    )?;

    let file_name = format!(
        "{}-{}.lanesra",
        workspace.business_name.to_lowercase().replace(' ', "-"),
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );

    Ok(BackupPackage {
        file_name,
        package_base64: BASE64.encode(&zip_bytes),
        manifest,
    })
}

/// Validates and restores a `.lanesra` package, replacing every byte of the
/// live workspace. `conn_slot` is the connection held by the caller's
/// shared state (behind whatever mutex it's guarded by) - restoring means
/// swapping *that* connection out entirely, not running SQL through it, so
/// this takes `&mut Connection` rather than `&Connection` like every other
/// service function.
///
/// The restored database is staged and fully validated on disk *before*
/// the live connection is touched, so a corrupt or incompatible upload
/// never leaves the workspace half-restored.
pub fn restore_from_package(
    conn_slot: &mut Connection,
    db_path: &Path,
    package_base64: &str,
    actor_user_id: Option<&str>,
) -> AppResult<BackupManifest> {
    require_admin(conn_slot, actor_user_id)?;

    let zip_bytes = BASE64
        .decode(package_base64.trim())
        .map_err(|e| AppError::Validation(format!("Not a valid backup file: {e}")))?;
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes)).map_err(zip_err)?;

    let manifest: BackupManifest = {
        let mut entry = archive
            .by_name(MANIFEST_ENTRY)
            .map_err(|_| AppError::Validation("Not a valid Lanesra OS backup file (missing manifest)".into()))?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf).map_err(zip_err)?;
        serde_json::from_str(&buf).map_err(|e| AppError::Validation(format!("Invalid backup manifest: {e}")))?
    };

    let current_version = migrate::current_schema_version();
    if manifest.schema_version > current_version {
        return Err(AppError::Validation(format!(
            "This backup was made with a newer version of Lanesra OS (schema {}) than this app supports (schema {}). Update the app before restoring it.",
            manifest.schema_version, current_version
        )));
    }

    let db_bytes = {
        let mut entry = archive
            .by_name(DB_ENTRY)
            .map_err(|_| AppError::Validation("Not a valid Lanesra OS backup file (missing database)".into()))?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(zip_err)?;
        buf
    };

    // Stage the restored database next to the live one and open it for
    // real - this both validates it's a genuine, readable Lanesra OS
    // database and brings it up to the current schema via the normal
    // migration path, all before anything live is touched.
    let staged_path = db_path.with_file_name(format!(
        "{}.restoring",
        db_path.file_name().and_then(|n| n.to_str()).unwrap_or("workspace.sqlite3")
    ));
    std::fs::write(&staged_path, &db_bytes)
        .map_err(|e| AppError::Validation(format!("could not stage restored database: {e}")))?;

    let staged_check = db::open_workspace_db(&staged_path).and_then(|staged_conn| {
        staged_conn.query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get::<_, i64>(0))
    });
    if staged_check.is_err() {
        let _ = std::fs::remove_file(&staged_path);
        return Err(AppError::Validation(
            "Backup file does not contain a valid Lanesra OS workspace".into(),
        ));
    }
    // staged_conn from the closure above is dropped by now, releasing its
    // file handle - safe to rename over on every platform including Windows.

    // Close the live connection *before* touching the file on disk: on
    // Windows you cannot rename over a file that's still open, and even on
    // Unix it's cleaner not to have two connections pointed at the same
    // path mid-swap. Assigning through the mutable reference drops the old
    // connection immediately.
    *conn_slot = Connection::open_in_memory()?;

    let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
    let shm_path = PathBuf::from(format!("{}-shm", db_path.display()));
    let _ = std::fs::remove_file(&wal_path);
    let _ = std::fs::remove_file(&shm_path);

    std::fs::rename(&staged_path, db_path).map_err(|e| AppError::Validation(format!("could not finalize restore: {e}")))?;

    let fresh = db::open_workspace_db(db_path)?;
    *conn_slot = fresh;

    audit_repo::record(
        conn_slot,
        &workspace_repo::get_current(conn_slot)?.map(|w| w.id).unwrap_or_default(),
        actor_user_id,
        "restore",
        Some("workspace"),
        None,
        &format!("Restored workspace '{}' from a backup made {}", manifest.workspace_name, manifest.created_at),
        None,
    )?;

    Ok(manifest)
}
