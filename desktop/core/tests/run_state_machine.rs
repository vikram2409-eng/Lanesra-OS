//! WP-0110: the run and step schema extensions and the one run-status guard
//! (ADR-0007). The transition table below is written out from ADR-0007 §3 on
//! its own, so the guard is checked against the decision, not against itself.

use lanesra_core::db::migrate::run_migrations;
use lanesra_core::domain::AppError;
use lanesra_core::models::graph_run::{RUN_STATUSES, STOP_REASONS};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::graph_run_repo;
use lanesra_core::services::{graph_runtime_service, run_state, workspace_service};
use rusqlite::Connection;

fn master_key() -> [u8; 32] {
    [83u8; 32]
}

/// ADR-0007 §3, row by row.
fn adr_allows(from: &str, to: &str) -> bool {
    const WAITING_OR_PAUSED: &[&str] = &[
        "planning",
        "paused",
        "waiting_approval",
        "waiting_budget",
        "waiting_model",
        "waiting_scheduled",
    ];
    let from_waiting = WAITING_OR_PAUSED.contains(&from);
    matches!(
        (from, to),
        ("queued", "running")
            | ("queued", "cancelled")
            | ("running", "waiting_approval")
            | ("running", "waiting_budget")
            | ("running", "waiting_model")
            | ("running", "waiting_scheduled")
            | ("running", "paused")
            | ("running", "completed")
            | ("running", "completed_partial")
            | ("running", "failed")
            | ("running", "cancelled")
            | ("waiting_approval", "failed")
    ) || (from_waiting && (to == "queued" || to == "cancelled"))
}

/// A workspace with one published two-node graph, as raw rows.
fn workspace_with_graph() -> (Connection, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Run states".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, _admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    let ts = "2026-10-22T00:00:00Z";
    conn.execute(
        "INSERT INTO ai_execution_graphs (id, workspace_id, name, status, created_at, updated_at)
         VALUES ('g1', ?1, 'Graph', 'published', ?2, ?2)",
        (&workspace.id, ts),
    )
    .unwrap();
    conn.execute_batch(
        "INSERT INTO ai_graph_nodes (id, graph_id, node_key, node_type) VALUES ('n_trigger', 'g1', 'trigger', 'trigger');
         INSERT INTO ai_graph_nodes (id, graph_id, node_key, node_type) VALUES ('n_end', 'g1', 'end', 'end');
         INSERT INTO ai_graph_edges (id, graph_id, from_node_id, to_node_id) VALUES ('e1', 'g1', 'n_trigger', 'n_end');",
    )
    .unwrap();
    (conn, workspace.id)
}

/// Puts a run straight into `status`. Only a test may write status directly.
fn run_in(conn: &Connection, ws: &str, id: &str, status: &str) {
    conn.execute(
        "INSERT INTO ai_runs (id, workspace_id, graph_id, status, current_node_id, pending_approval_id, resume_at, started_at)
         VALUES (?1, ?2, 'g1', ?3, 'n_end', NULL, ?4, '2026-10-22T00:00:00Z')",
        (
            id,
            ws,
            status,
            (status == "waiting_scheduled").then_some("2026-10-22T00:00:00Z"),
        ),
    )
    .unwrap();
}

fn status_of(conn: &Connection, id: &str) -> String {
    conn.query_row("SELECT status FROM ai_runs WHERE id = ?1", [id], |r| {
        r.get(0)
    })
    .unwrap()
}

#[test]
fn ac2_the_guard_allows_exactly_the_adr_transitions_for_every_pair() {
    // Covers: REQ-CORE-008 (WP-0110 AC-2)
    let (conn, ws) = workspace_with_graph();
    let mut allowed = 0;
    for (i, from) in RUN_STATUSES.iter().enumerate() {
        for (j, to) in RUN_STATUSES.iter().enumerate() {
            let expected = adr_allows(from, to);
            assert_eq!(run_state::is_legal(from, to), expected, "{from} -> {to}");
            let id = format!("r_{i}_{j}");
            run_in(&conn, &ws, &id, from);
            let result = run_state::transition(&conn, &id, from, to, None);
            assert_eq!(result.is_ok(), expected, "{from} -> {to}: {result:?}");
            let now = status_of(&conn, &id);
            assert_eq!(now, if expected { *to } else { *from }, "{from} -> {to}");
            if expected {
                allowed += 1;
            } else {
                assert!(
                    matches!(result, Err(AppError::Validation(_))),
                    "{from} -> {to}"
                );
            }
        }
    }
    // 2 from queued, 9 from running, 1 approval rejection, 2 from each of
    // the six waiting-or-paused statuses.
    assert_eq!(allowed, 2 + 9 + 1 + 6 * 2);
    assert_eq!(RUN_STATUSES.len() * RUN_STATUSES.len(), 144);
}

#[test]
fn ac2_a_run_that_moved_on_cannot_be_moved_again_from_its_old_status() {
    // Covers: REQ-CORE-008 (WP-0110 AC-2)
    let (conn, ws) = workspace_with_graph();
    run_in(&conn, &ws, "r1", "queued");
    run_state::transition(&conn, "r1", "queued", "running", None).unwrap();
    // A second worker still thinks it is queued.
    let second = run_state::transition(&conn, "r1", "queued", "running", None);
    assert!(matches!(second, Err(AppError::Conflict(_))), "{second:?}");
    let missing = run_state::transition(&conn, "no_such_run", "queued", "running", None);
    assert!(matches!(missing, Err(AppError::Conflict(_))), "{missing:?}");
}

#[test]
fn ac2_stop_reasons_leases_and_finish_times_follow_the_status() {
    // Covers: REQ-CORE-008 (WP-0110 AC-2)
    let (conn, ws) = workspace_with_graph();
    run_in(&conn, &ws, "r1", "queued");
    assert!(matches!(
        run_state::transition(&conn, "r1", "queued", "running", Some("not_a_reason")),
        Err(AppError::Validation(_))
    ));
    assert_eq!(status_of(&conn, "r1"), "queued");

    run_state::transition(&conn, "r1", "queued", "running", None).unwrap();
    let run = graph_run_repo::get_run(&conn, "r1").unwrap().unwrap();
    assert_eq!(
        run.lease_owner.as_deref(),
        Some(run_state::lease_owner().as_str())
    );
    assert!(run.lease_expires_at.is_some());
    assert!(run.finished_at.is_none());

    run_state::transition(
        &conn,
        "r1",
        "running",
        "waiting_approval",
        Some("approval_pending"),
    )
    .unwrap();
    let run = graph_run_repo::get_run(&conn, "r1").unwrap().unwrap();
    assert_eq!(run.stop_reason.as_deref(), Some("approval_pending"));
    assert!(run.lease_owner.is_none() && run.lease_expires_at.is_none());
    assert!(run.finished_at.is_none());

    run_state::transition(
        &conn,
        "r1",
        "waiting_approval",
        "failed",
        Some("approval_rejected"),
    )
    .unwrap();
    let run = graph_run_repo::get_run(&conn, "r1").unwrap().unwrap();
    assert_eq!(run.stop_reason.as_deref(), Some("approval_rejected"));
    assert!(run.finished_at.is_some());

    for reason in STOP_REASONS {
        let id = format!("r_{reason}");
        run_in(&conn, &ws, &id, "running");
        run_state::transition(&conn, &id, "running", "cancelled", Some(reason)).unwrap();
    }
}

#[tokio::test]
async fn ac2_the_runtime_moves_runs_only_through_the_guard() {
    // Covers: REQ-CORE-008 (WP-0110 AC-2)
    let (conn, ws) = workspace_with_graph();
    let run = graph_runtime_service::start_run(
        &conn,
        &ws,
        &master_key(),
        "g1",
        "{}",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed");
    assert_eq!(run.stop_reason.as_deref(), Some("end_reached"));
    assert!(run.lease_owner.is_none());
    assert!(run.finished_at.is_some());
    let admin: String = conn
        .query_row("SELECT id FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();
    let again = graph_runtime_service::cancel_run(&conn, &ws, &run.id, Some(&admin)).await;
    assert!(again.is_err(), "a completed run cannot be cancelled");
}

fn upgraded_runs_fixture() -> Connection {
    let src =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/db/0082_runs.db");
    let copy = std::env::temp_dir().join(format!(
        "lanesra-wp0110-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::copy(&src, &copy).unwrap();
    let conn = Connection::open(&copy).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    run_migrations(&conn).unwrap();
    conn
}

#[test]
fn ac1_the_upgrade_adds_the_fields_and_parks_retired_statuses() {
    // Covers: REQ-CORE-008, REQ-API-001 (WP-0110 AC-1)
    let conn = upgraded_runs_fixture();
    for (table, column) in [
        ("ai_runs", "stop_reason"),
        ("ai_runs", "parent_run_id"),
        ("ai_runs", "fork_of_step_id"),
        ("ai_runs", "budget_json"),
        ("ai_runs", "cost_usd"),
        ("ai_runs", "lease_owner"),
        ("ai_runs", "lease_expires_at"),
        ("ai_run_nodes", "tokens_in"),
        ("ai_run_nodes", "tokens_out"),
        ("ai_run_nodes", "cost_usd"),
        ("ai_run_nodes", "context_receipt_json"),
        ("ai_run_nodes", "guardrail_verdicts_json"),
    ] {
        let found: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"),
                [column],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(found, 1, "{table}.{column}");
    }
    for stray in ["r_waiting_tool", "r_waiting_agent"] {
        let run = graph_run_repo::get_run(&conn, stray).unwrap().unwrap();
        assert_eq!(run.status, "paused", "{stray}");
        assert_eq!(run.stop_reason.as_deref(), Some("interrupted"), "{stray}");
    }
    let bad: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ai_runs WHERE status NOT IN ('queued','planning','running','paused','waiting_approval','waiting_budget','waiting_model','waiting_scheduled','completed','completed_partial','failed','cancelled')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(bad, 0);
}

#[tokio::test]
async fn ac3_existing_run_history_still_reads_and_resumes() {
    // Covers: REQ-CORE-008 (WP-0110 AC-3)
    let conn = upgraded_runs_fixture();
    let done = graph_run_repo::get_run(&conn, "r_completed")
        .unwrap()
        .unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(done.trigger_input, r#"{"topic":"fixture"}"#);
    assert_eq!(done.context_json, r#"{"trigger":{"ok":true}}"#);
    assert_eq!(done.nodes.len(), 1);
    assert_eq!(done.nodes[0].output_json.as_deref(), Some(r#"{"ok":true}"#));
    assert!(done.nodes[0].tokens_in.is_none());
    assert!(done.stop_reason.is_none());
    let failed = graph_run_repo::get_run(&conn, "r_failed").unwrap().unwrap();
    assert_eq!(
        failed.error_message.as_deref(),
        Some("provider unavailable")
    );
    assert_eq!(
        graph_run_repo::list_runs_for_graph(&conn, "g_fixture", 10)
            .unwrap()
            .len(),
        5
    );

    // The due delayed run finishes through the guard after the upgrade.
    let resumed = graph_runtime_service::resume_due_delays(&conn, "ws_fixture", &master_key())
        .await
        .unwrap();
    assert_eq!(resumed, 1);
    let scheduled = graph_run_repo::get_run(&conn, "r_scheduled")
        .unwrap()
        .unwrap();
    assert_eq!(scheduled.status, "completed");
    assert_eq!(scheduled.stop_reason.as_deref(), Some("end_reached"));

    // A run parked by the upgrade can be resumed by a person.
    let run = graph_runtime_service::resume_run(
        &conn,
        "ws_fixture",
        &master_key(),
        "r_waiting_tool",
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed");
}
