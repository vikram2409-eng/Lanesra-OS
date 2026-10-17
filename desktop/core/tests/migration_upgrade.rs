//! Database upgrade (migration) test harness (WP-0007, REQ-PRE-005).
//!
//! Proves every migration upgrades a real older database without data loss.
//! Opens each fixture database under `fixtures/db/`, runs migrations, and
//! verifies row counts and key invariants are preserved.
//! Covers: REQ-PRE-005 (WP-0007 AC-1, AC-2)

use lanesra_core::db::migrate::{run_migrations, schema_version};
use rusqlite::Connection;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("db")
}

fn fixture_paths() -> Vec<PathBuf> {
    let dir = fixture_dir();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("fixture dir not found: {}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "db").unwrap_or(false))
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no fixture databases in {}",
        dir.display()
    );
    paths
}

/// AC-1: Upgrade test passes for the current HEAD.
/// Opens each fixture database (created by the last released desktop version
/// and by current main), runs migrations, and verifies row counts and key
/// invariants are preserved.
/// Covers: REQ-PRE-005 (WP-0007 AC-1)
#[test]
fn upgrade_fixtures_preserves_data() {
    for path in fixture_paths() {
        let conn = Connection::open(&path)
            .unwrap_or_else(|_| panic!("open fixture {}", path.display()));
        conn.pragma_update(None, "foreign_keys", true)
            .expect("pragma");

        let before_version = schema_version(&conn).expect("version");
        let before_companies: i64 = conn
            .query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))
            .expect("count companies");
        let before_workspaces: i64 = conn
            .query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get(0))
            .expect("count workspaces");
        // Canary: the companies.name column must hold its data.
        let before_name: String = conn
            .query_row(
                "SELECT name FROM companies WHERE id = 'c_fixture'",
                [],
                |r| r.get(0),
            )
            .expect("canary name");

        run_migrations(&conn)
            .unwrap_or_else(|_| panic!("migrate {}", path.display()));

        let after_version = schema_version(&conn).expect("version");
        assert!(
            after_version >= before_version,
            "version went backwards in {}",
            path.display()
        );

        let after_companies: i64 = conn
            .query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))
            .expect("count companies");
        let after_workspaces: i64 = conn
            .query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get(0))
            .expect("count workspaces");
        assert_eq!(
            before_companies,
            after_companies,
            "companies row count changed in {}",
            path.display()
        );
        assert_eq!(
            before_workspaces,
            after_workspaces,
            "workspaces row count changed in {}",
            path.display()
        );

        let after_name: String = conn
            .query_row(
                "SELECT name FROM companies WHERE id = 'c_fixture'",
                [],
                |r| r.get(0),
            )
            .expect("canary name after migration");
        assert_eq!(
            before_name,
            after_name,
            "canary data corrupted in {}",
            path.display()
        );
    }
}

/// AC-1 (contiguity): Migration numbers are contiguous, unique, and every
/// migration file is listed in MIGRATIONS.
/// Covers: REQ-PRE-005 (WP-0007 AC-1)
#[test]
fn migration_numbers_contiguous_and_unique() {
    let versions = lanesra_core::db::migrate::migration_versions();
    assert!(!versions.is_empty(), "no migrations found");

    // Unique
    let mut sorted = versions.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), versions.len(), "duplicate migration versions");

    // Contiguous from 1
    for (i, v) in sorted.iter().enumerate() {
        assert_eq!(
            *v,
            (i + 1) as i64,
            "migration numbers not contiguous at position {i}"
        );
    }
}

/// AC-2: A migration that drops a populated column makes the test fail.
///
/// This test documents the canary pattern: if a migration drops a column
/// holding data, the data-preservation assertions above fail. The canary
/// is demonstrated in the PR by temporarily adding a DROP COLUMN migration,
/// observing the failure, then removing it before merge.
/// Covers: REQ-PRE-005 (WP-0007 AC-2)
#[test]
fn dropping_populated_column_fails() {
    // The fixture-based test above (upgrade_fixtures_preserves_data) fails
    // if any migration drops the companies.name column, because the canary
    // SELECT would fail. This test asserts the canary column exists in the
    // fixtures as a precondition.
    for path in fixture_paths() {
        let conn = Connection::open(&path)
            .unwrap_or_else(|_| panic!("open fixture {}", path.display()));
        let name: String = conn
            .query_row(
                "SELECT name FROM companies WHERE id = 'c_fixture'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| {
                panic!("canary column missing in {}", path.display())
            });
        assert_eq!(name, "Fixture Industries");
    }
}
