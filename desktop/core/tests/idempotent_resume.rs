//! Phase 1, WP-0141: idempotency keys and side-effect-safe resume
//! (REQ-TOOL-005, REQ-GOV-023; ADR-0007 §5).
//!
//!   AC-1  a write applied once is not repeated when its step runs again
//!         (the resume-after-crash case: re-running the node finds the key)
//!   AC-2  a retry after a transient failure writes once, not twice
//!   AC-3  two requests for the same step attempt write once; the second
//!         gets the first result
//!   AC-4  interrupted_run_policy: always_pause pauses an interrupted run,
//!         resume_if_safe resumes it; only an admin may set it
//!   AC-5  the tick resumes due runs and recovers interrupted runs with no UI
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::json;

use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    custom_object_service, execution_graph_service, graph_runtime_service, idempotency_service,
    run_policy_service, user_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Idem Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "correct horse battery staple".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [83u8; 32]
}

fn make_widget_object(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    custom_object_service::create_with_key(
        conn,
        ws,
        "widget",
        &CustomObjectDefinitionInput {
            singular_label: "Widget".into(),
            plural_label: "Widgets".into(),
            icon: "▣".into(),
            prefix: "WID".into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
}

fn widget_count(conn: &rusqlite::Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM custom_records WHERE object_key = 'widget'",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

/// Starts and completes a trivial run so its id is a real `ai_runs` row - the
/// ledger's `run_id` is a foreign key, so `run_once` needs a run that exists.
async fn a_real_run_id(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    let input = ExecutionGraphInput {
        name: "Noop".into(),
        description: None,
        nodes: vec![
            node("trigger", "trigger", json!({})),
            node("end", "end", json!({})),
        ],
        edges: vec![edge("trigger", "end", None)],
    };
    let graph = execution_graph_service::create(conn, ws, &input, Some(admin)).unwrap();
    let graph = execution_graph_service::publish(conn, &graph.id, ws, Some(admin)).unwrap();
    let run = graph_runtime_service::start_run(
        conn,
        ws,
        &master_key(),
        &graph.id,
        "x",
        Some(admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    run.id
}

/// A closure that creates one widget and returns its id - a genuinely
/// non-idempotent write (every call makes a new row), so "applied once" is
/// observable by counting rows.
fn create_widget(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    name: &str,
) -> lanesra_core::domain::AppResult<serde_json::Value> {
    let rec = lanesra_core::services::custom_record_service::create(
        conn,
        ws,
        &CustomRecordInput {
            object_key: "widget".into(),
            primary_name: name.into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(admin),
    )?;
    Ok(json!(rec.id))
}

// --------------------------------------------------------------------------
// AC-1 + AC-3: the same key applies its write once; the second call gets the
// first result and makes no second row.
// --------------------------------------------------------------------------

// Covers: REQ-TOOL-005 (WP-0141 AC-1), REQ-GOV-023 (WP-0141 AC-3)
#[tokio::test]
async fn the_same_step_attempt_writes_exactly_once() {
    let (conn, ws, admin) = setup();
    make_widget_object(&conn, &ws, &admin);
    let run_id = a_real_run_id(&conn, &ws, &admin).await;

    let first = idempotency_service::run_once(&conn, &ws, &run_id, "node-a", 1, "", || {
        create_widget(&conn, &ws, &admin, "only one")
    })
    .unwrap();
    assert_eq!(widget_count(&conn), 1);

    // Re-running the same attempt (a resume after a crash, or a double
    // submit) makes no second row and returns the first result.
    let again = idempotency_service::run_once(&conn, &ws, &run_id, "node-a", 1, "", || {
        create_widget(&conn, &ws, &admin, "should not be created")
    })
    .unwrap();
    assert_eq!(widget_count(&conn), 1, "no duplicate record");
    assert_eq!(first, again, "the second call replays the first result");

    // A different attempt of the same node is a different key and writes
    // again - a genuine retry of a new attempt is not suppressed.
    idempotency_service::run_once(&conn, &ws, &run_id, "node-a", 2, "", || {
        create_widget(&conn, &ws, &admin, "attempt two")
    })
    .unwrap();
    assert_eq!(widget_count(&conn), 2);
}

// --------------------------------------------------------------------------
// AC-2: a transient failure leaves no trace; the retry writes once.
// --------------------------------------------------------------------------

// Covers: REQ-TOOL-005 (WP-0141 AC-2)
#[tokio::test]
async fn a_transient_failure_does_not_write_and_the_retry_writes_once() {
    let (conn, ws, admin) = setup();
    make_widget_object(&conn, &ws, &admin);
    let run_id = a_real_run_id(&conn, &ws, &admin).await;

    // The write fails transiently - nothing is recorded, nothing is written.
    let err = idempotency_service::run_once(&conn, &ws, &run_id, "node-b", 1, "", || {
        Err(lanesra_core::domain::AppError::Validation(
            "transient".into(),
        ))
    });
    assert!(err.is_err());
    assert_eq!(widget_count(&conn), 0);

    // The retry of the same attempt succeeds and writes exactly once.
    idempotency_service::run_once(&conn, &ws, &run_id, "node-b", 1, "", || {
        create_widget(&conn, &ws, &admin, "after retry")
    })
    .unwrap();
    assert_eq!(widget_count(&conn), 1);

    // A further retry now finds the key and does not write again.
    idempotency_service::run_once(&conn, &ws, &run_id, "node-b", 1, "", || {
        create_widget(&conn, &ws, &admin, "must not happen")
    })
    .unwrap();
    assert_eq!(widget_count(&conn), 1);
}

// --------------------------------------------------------------------------
// AC-4 + AC-5: the interrupted-run policy and the recovery sweep.
// --------------------------------------------------------------------------

fn node(key: &str, node_type: &str, config: serde_json::Value) -> GraphNodeInput {
    GraphNodeInput {
        node_key: key.into(),
        node_type: node_type.into(),
        config_json: config.to_string(),
        position_x: None,
        position_y: None,
        sort_order: 0,
    }
}
fn edge(from: &str, to: &str, label: Option<&str>) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.into(),
        to_node_key: to.into(),
        branch_label: label.map(String::from),
        sort_order: 0,
    }
}

/// Publishes a trigger -> approval -> end graph and starts a run, which
/// pauses at the approval. Then forges that run into the "crashed while
/// running" state: status `running`, lease long expired. Returns the run id.
async fn interrupted_run(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    let input = ExecutionGraphInput {
        name: "Approve then end".into(),
        description: None,
        nodes: vec![
            node("trigger", "trigger", json!({})),
            node("appr", "approval", json!({})),
            node("end", "end", json!({})),
        ],
        edges: vec![
            edge("trigger", "appr", None),
            edge("appr", "end", Some("approved")),
            edge("appr", "end", Some("rejected")),
        ],
    };
    let graph = execution_graph_service::create(conn, ws, &input, Some(admin)).unwrap();
    let graph = execution_graph_service::publish(conn, &graph.id, ws, Some(admin)).unwrap();
    let run = graph_runtime_service::start_run(
        conn,
        ws,
        &master_key(),
        &graph.id,
        "go",
        Some(admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_approval");
    // Forge a crashed worker: the run is `running` with a lease in the past.
    conn.execute(
        "UPDATE ai_runs SET status = 'running', lease_owner = 'dead', lease_expires_at = '2000-01-01T00:00:00+00:00' WHERE id = ?1",
        [&run.id],
    )
    .unwrap();
    run.id
}

// Covers: REQ-GOV-023 (WP-0141 AC-4)
#[tokio::test]
async fn always_pause_pauses_an_interrupted_run_and_resume_if_safe_resumes_it() {
    // always_pause: the interrupted run is paused, not resumed.
    let (conn, ws, admin) = setup();
    run_policy_service::set(&conn, &ws, run_policy_service::ALWAYS_PAUSE, Some(&admin)).unwrap();
    let run_id = interrupted_run(&conn, &ws, &admin).await;
    let handled = graph_runtime_service::recover_interrupted_runs(&conn, &ws, &master_key())
        .await
        .unwrap();
    assert_eq!(handled, 1);
    let run = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run_id)
        .unwrap()
        .unwrap();
    assert_eq!(run.status, "paused");

    // resume_if_safe (the default): the interrupted run is resumed - it
    // re-enters its approval node and pauses there again.
    let (conn, ws, admin) = setup();
    assert_eq!(
        run_policy_service::get(&conn, &ws).unwrap(),
        run_policy_service::RESUME_IF_SAFE
    );
    let run_id = interrupted_run(&conn, &ws, &admin).await;
    let handled = graph_runtime_service::recover_interrupted_runs(&conn, &ws, &master_key())
        .await
        .unwrap();
    assert_eq!(handled, 1);
    let run = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run_id)
        .unwrap()
        .unwrap();
    assert_eq!(run.status, "waiting_approval", "resumed and re-paused");
}

// Covers: REQ-GOV-023 (WP-0141 AC-4)
#[test]
fn only_an_admin_may_set_the_policy_and_the_value_is_validated() {
    let (conn, ws, admin) = setup();
    // A non-admin cannot set it.
    let member = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "member".into(),
            display_name: "Member".into(),
            password: "member-password".into(),
            roles: vec!["Sales".into()],
        },
        Some(&admin),
    )
    .unwrap();
    assert!(run_policy_service::set(
        &conn,
        &ws,
        run_policy_service::ALWAYS_PAUSE,
        Some(&member.id)
    )
    .is_err());
    // An unknown value is refused.
    assert!(run_policy_service::set(&conn, &ws, "whenever", Some(&admin)).is_err());
    // The admin sets a valid value and it is read back.
    run_policy_service::set(&conn, &ws, run_policy_service::ALWAYS_PAUSE, Some(&admin)).unwrap();
    assert_eq!(
        run_policy_service::get(&conn, &ws).unwrap(),
        run_policy_service::ALWAYS_PAUSE
    );
    // The change is audited.
    let audited: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE event_type = 'run_policy'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(audited >= 1);
}

// Covers: REQ-GOV-023 (WP-0141 AC-5)
#[tokio::test]
async fn the_tick_recovers_an_interrupted_run_with_no_ui() {
    let (conn, ws, admin) = setup();
    let run_id = interrupted_run(&conn, &ws, &admin).await;
    // The tick alone (what the server scheduler and desktop poll call) moves
    // the run again, with nothing else driving it.
    let moved = graph_runtime_service::tick(&conn, &ws, &master_key())
        .await
        .unwrap();
    assert!(moved >= 1);
    let run = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run_id)
        .unwrap()
        .unwrap();
    assert_ne!(run.status, "running", "the interrupted run was recovered");
}
