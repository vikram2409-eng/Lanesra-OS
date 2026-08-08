use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub session_user_id: Mutex<Option<String>>,
    /// Needed by restore_backup, which has to replace the file on disk out
    /// from under the live connection - every other command only ever
    /// needs `conn`.
    pub db_path: PathBuf,
}

impl AppState {
    pub fn new(conn: Connection, db_path: PathBuf) -> Self {
        Self {
            conn: Mutex::new(conn),
            session_user_id: Mutex::new(None),
            db_path,
        }
    }
}
