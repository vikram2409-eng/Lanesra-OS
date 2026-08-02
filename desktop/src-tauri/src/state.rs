use std::sync::Mutex;

use rusqlite::Connection;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub session_user_id: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
            session_user_id: Mutex::new(None),
        }
    }
}
