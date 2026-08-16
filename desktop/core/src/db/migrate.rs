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
    (5, include_str!("migrations/0005_field_rules.sql")),
    (6, include_str!("migrations/0006_workflow_rules.sql")),
    (7, include_str!("migrations/0007_broaden_entity_types.sql")),
    (8, include_str!("migrations/0008_workspace_phone_and_kpi_prefs.sql")),
    (9, include_str!("migrations/0009_numbering_configs.sql")),
    (10, include_str!("migrations/0010_custom_reports.sql")),
    (11, include_str!("migrations/0011_custom_objects.sql")),
    (12, include_str!("migrations/0012_relationships.sql")),
    (13, include_str!("migrations/0013_business_rules.sql")),
    (14, include_str!("migrations/0014_workflow_automation.sql")),
    (15, include_str!("migrations/0015_custom_field_validation.sql")),
    (16, include_str!("migrations/0016_builtin_field_targeting.sql")),
    (17, include_str!("migrations/0017_condition_field_comparison.sql")),
    (18, include_str!("migrations/0018_status_transitions.sql")),
    (19, include_str!("migrations/0019_custom_field_extensibility.sql")),
    (20, include_str!("migrations/0020_condition_groups_and_field_effects.sql")),
    (21, include_str!("migrations/0021_company_contact_extra_fields.sql")),
    (22, include_str!("migrations/0022_rule_version_history.sql")),
    (23, include_str!("migrations/0023_screen_layouts.sql")),
    (24, include_str!("migrations/0024_dashboard_layouts.sql")),
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
        assert_eq!(version, 24);

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
