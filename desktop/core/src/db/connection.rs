use rusqlite::Connection;
use std::path::Path;

use super::migrate;

/// Opens (creating if necessary) the workspace SQLite database at `path`,
/// enables foreign key enforcement (BR-016) and WAL journaling, and runs
/// any pending migrations.
pub fn open_workspace_db<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrate::run_migrations(&conn)?;
    Ok(conn)
}

/// Opens an in-memory database with migrations applied. Used by tests and
/// can be used for ephemeral previews.
pub fn open_in_memory_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", true)?;
    migrate::run_migrations(&conn)?;
    Ok(conn)
}
