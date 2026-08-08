use serde::{Deserialize, Serialize};

/// Stored inside every `.lanesra` package alongside the database snapshot,
/// so restore can validate compatibility before touching the live data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    /// Version of the `.lanesra` package format itself (not the DB schema).
    /// Bump this only if the zip's layout changes.
    pub format_version: u32,
    pub schema_version: i64,
    pub workspace_name: String,
    pub created_at: String,
    pub app_version: String,
}

/// Returned to the frontend so it can name the downloaded file sensibly and
/// show what it just backed up.
#[derive(Debug, Clone, Serialize)]
pub struct BackupPackage {
    pub file_name: String,
    pub package_base64: String,
    pub manifest: BackupManifest,
}
