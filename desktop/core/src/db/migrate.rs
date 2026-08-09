use chrono::Utc;
use rusqlite::Connection;

/// Ordered list of (version, sql). Add new migrations by appending here and
/// creating a new file under `migrations/`; never edit an already-released
/// migration in place.
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("migrations/0001_init.sql")),
    (2, include_str!("migrations/0002_web_sessions.sql")),
    (3, include_str!("migrations/0003_workspace_branding.sql")),
    (4, include_str!("migrations/0004_custom_fields.sql")),
];

/// The newest schema version this build knows about - used to reject
/// restoring a backup that was made by a newer version of the app than
/// this one (an older app can't safely open a schema it's never seen).
pub fn current_schema_version() -> i64 {
    MIGRATIONS.last().map(|(v, _)| *v).unwrap_or(0)
}

/// The schema version actually applied to `conn`.
pub fn schema_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
}

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current_version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    for (version, sql) in MIGRATIONS {
        if *version <= current_version {
            continue;
        }
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            (version, Utc::now().to_rfc3339()),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_all_migrations_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();

        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // must not fail or double-apply

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, 4);

        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'companies'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);
    }
}
