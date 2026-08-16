use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::security::SecurityConfig;

/// Shared across every request. A single mutex-guarded connection is enough
/// for the "small team on a LAN" scale this mode targets - SQLite already
/// serializes writers, and request volume at this scale won't contend on
/// the lock in practice. Revisit with a real connection pool if that
/// assumption stops holding.
pub struct ServerState {
    pub conn: Mutex<Connection>,
    /// Needed by restore_backup, which replaces the file on disk out from
    /// under the live connection - every other command only ever needs
    /// `conn`.
    pub db_path: PathBuf,
    /// Self-hosted internet deployment settings (secure cookies, CORS) -
    /// see `security::SecurityConfig`.
    pub security: SecurityConfig,
}

pub type SharedState = Arc<ServerState>;

impl ServerState {
    pub fn new(conn: Connection, db_path: PathBuf, security: SecurityConfig) -> SharedState {
        Arc::new(ServerState { conn: Mutex::new(conn), db_path, security })
    }
}
