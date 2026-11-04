//! WP-0106: the one-time first-run setup token.
//!
//! On a network-reachable server, the first-run wizard would otherwise let
//! whoever reaches a fresh install first create the workspace and its
//! Administrator (RAID I-26). At startup, when no workspace exists yet, the
//! server generates a setup token, logs it and writes it to the data
//! directory (`setup-token`, owner-readable only); the HTTP first-run route
//! then requires it, and it is deleted once setup succeeds.
//!
//! Enforcement is keyed on the token file's presence, which `ensure` makes a
//! startup invariant whenever there is no workspace. The desktop app (and
//! tests that never run server startup) have no token file and are not
//! gated — desktop first-run is trusted because the process is local.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use lanesra_core::domain::errors::ErrorClass;
use lanesra_core::domain::ids::new_uuid;
use lanesra_core::domain::{AppError, AppResult};

const FILE: &str = "setup-token";

pub fn path(db_path: &Path) -> PathBuf {
    db_path.with_file_name(FILE)
}

fn generate() -> String {
    format!(
        "{}{}",
        new_uuid().replace('-', ""),
        new_uuid().replace('-', "")
    )
}

/// Startup hook: ensures the invariant "no workspace ⇒ a token file exists".
/// Returns the token to log when one is active, or `None` when a workspace
/// already exists (in which case any stale token file is removed).
pub fn ensure(db_path: &Path, conn: &Connection) -> std::io::Result<Option<String>> {
    let p = path(db_path);
    let has_workspace = lanesra_core::repositories::workspace_repo::get_current(conn)
        .ok()
        .flatten()
        .is_some();
    if has_workspace {
        let _ = std::fs::remove_file(&p);
        return Ok(None);
    }
    if let Ok(existing) = std::fs::read_to_string(&p) {
        let t = existing.trim().to_string();
        if !t.is_empty() {
            return Ok(Some(t));
        }
    }
    let token = generate();
    write_private(&p, &token)?;
    Ok(Some(token))
}

#[cfg(unix)]
fn write_private(p: &Path, token: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(p)?;
    f.write_all(token.as_bytes())
}

#[cfg(not(unix))]
fn write_private(p: &Path, token: &str) -> std::io::Result<()> {
    std::fs::write(p, token)
}

/// Checks the token presented to the HTTP first-run route. `Ok(true)` means
/// a token file exists and `provided` matched — the caller calls [`consume`]
/// after setup succeeds. `Ok(false)` means no token file exists, so the
/// route is not gated here (desktop/tests). `Err` (policy) means a token is
/// required and `provided` was missing or wrong.
pub fn verify(db_path: &Path, provided: Option<&str>) -> AppResult<bool> {
    let expected = match std::fs::read_to_string(path(db_path)) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return Ok(false),
    };
    if expected.is_empty() {
        return Ok(false);
    }
    match provided {
        Some(got) if constant_time_eq(got.trim().as_bytes(), expected.as_bytes()) => Ok(true),
        _ => Err(AppError::Classified {
            class: ErrorClass::Policy,
            code: "policy.setup_token_required",
            message: "This server needs its one-time setup token to create the first workspace. \
                      It was printed to the server log and written to the data directory as 'setup-token'."
                .into(),
            retry_after_ms: None,
            http_status: None,
        }),
    }
}

/// Removes the token file after a successful first-run setup.
pub fn consume(db_path: &Path) {
    let _ = std::fs::remove_file(path(db_path));
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}
