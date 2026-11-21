//! Phase 7, WP-0713: board by stage, "my steps" and lane timing (REQ-LANE-019).
//!
//! The board shows each active run in its stage, filtered to the records the
//! viewer may read; "my steps" lists the signed-in person's waiting lane
//! steps and approvals together, oldest first; timing figures (time per
//! stage, completion rate, rework) are computed from the recorded node
//! timestamps; the ageing view lists active runs by time in their current
//! stage, longest first, also access-filtered. Covers:
//!   AC-1  board shows each active run in its stage, only records the
//!         viewer may read
//!   AC-2  "my steps" lists lane steps and approvals together, oldest first
//!   AC-3  timing figures equal the recorded timestamps (computed here
//!         independently from the node rows)
//!   AC-4  ageing lists active runs by time in current stage, longest
//!         first, only records the viewer may read

use serde_json::{json, Map, Value};

use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::ai_approval::AiApprovalResolution;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::graph_run_repo;
use lanesra_core::services::{
    access_role_service, action_type_service, approval_service, custom_object_service,
    custom_record_service, process_lane_service, user_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Metrics Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpw".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [83u8; 32]
}

fn make_object(conn: &rusqlite::Connection, ws: &str, admin: &str, key: &str, prefix: &str) {
    custom_object_service::create_with_key(
        conn,
        ws,
        key,
        &CustomObjectDefinitionInput {
            singular_label: key.into(),
            plural_label: format!("{key}s"),
            icon: "▣".into(),
            prefix: prefix.into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
}

fn make_record(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    object: &str,
    name: &str,
) -> String {
    custom_record_service::create(
        conn,
        ws,
        &CustomRecordInput {
            object_key: object.into(),
            primary_name: name.into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn make_user(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    username: &str,
    display: &str,
) -> String {
    user_service::create(
        conn,
        ws,
        &NewUser {
            username: username.into(),
            display_name: display.into(),
            password: "password123".into(),
            roles: vec!["Sales".into()],
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn make_role(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    access_role_service::create(
        conn,
        ws,
        &AccessRoleInput {
            name: name.into(),
            description: "".into(),
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn grant_read(
    conn: &rusqlite::Connection,
    admin: &str,
    role_id: &str,
    object_key: &str,
    scope: &str,
) {
    access_role_service::upsert_grant(
        conn,
        role_id,
        &AccessRoleGrantInput {
            object_key: object_key.into(),
            can_create: false,
            can_read: true,
            can_update: false,
            can_delete: false,
            can_assign: false,
            record_scope: scope.into(),
        },
        Some(admin),
    )
    .unwrap();
}

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(v)))
        .collect()
}

/// Three batches, two step roles (Counter / Manager) with one holder each,
/// the `batch.set_status` Action Type the approval step's write needs
/// (publish check P-12), and the two-stage lane: Tally (form) then Review
/// (approval). Returns (records, counter_user, manager_user, counter_role,
/// manager_role).
fn seed(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
) -> (Vec<String>, String, String, String, String) {
    make_object(conn, ws, admin, "batch", "BAT");
    let records = vec![
        make_record(conn, ws, admin, "batch", "BAT-1"),
        make_record(conn, ws, admin, "batch", "BAT-2"),
        make_record(conn, ws, admin, "batch", "BAT-3"),
    ];

    let counter_role = make_role(conn, ws, admin, "Counter");
    let manager_role = make_role(conn, ws, admin, "Manager");
    let counter = make_user(conn, ws, admin, "casey", "Casey Counter");
    let manager = make_user(conn, ws, admin, "robin", "Robin Manager");
    access_role_service::assign_to_user(conn, &counter, &counter_role, Some(admin)).unwrap();
    access_role_service::assign_to_user(conn, &manager, &manager_role, Some(admin)).unwrap();

    action_type_service::create(
        conn,
        ws,
        &ActionTypeDefinitionInput {
            name: "batch.set_status".into(),
            description: "Mark a batch approved".into(),
            target_object_type: "batch".into(),
            required_capability: "update".into(),
            parameters: vec![ActionParameterSpec {
                key: "status".into(),
                label: "Status".into(),
                field_type: "text".into(),
                required: true,
                default_value: None,
            }],
            effect: "set_status".into(),
            effect_config_json: json!({"target_status": "APPROVED"}).to_string(),
        },
        Some(admin),
    )
    .unwrap();

    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-metrics", "name": "Metrics", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": [{"key": "counted", "type": "integer", "required": true}]},
            "stages": [
                {"id": "stage-tally", "name": "Tally", "steps": [{
                    "id": "tally", "name": "Tally", "kind": "form",
                    "role": counter_role, "required": true,
                    "fields": [{
                        "key": "counted", "label": "Counted", "type": "integer",
                        "binds_to": "run_data.counted", "required": true
                    }]
                }]},
                {"id": "stage-review", "name": "Review", "steps": [{
                    "id": "review", "name": "Review", "kind": "approval",
                    "role": manager_role, "required": true,
                    "writes": [{"action_type": "batch.set_status", "inputs": {"status": "APPROVED"}}],
                    "branches": [
                        {"on": "approved", "end": {"outcome": "completed"}},
                        {"on": "rejected", "end": {"outcome": "held"}}
                    ]
                }]}
            ]
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-metrics", Some(admin)).unwrap();
    (records, counter, manager, counter_role, manager_role)
}

/// Three runs on the seeded lane: BAT-1 waits on the Tally form step,
/// BAT-2 is completed (tally submitted, review approved), BAT-3 waits on
/// the Review approval. Returns (run1, run2, run3).
async fn three_runs(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    records: &[String],
    counter: &str,
) -> (String, String, String) {
    let run1 = process_lane_service::start_lane_run(
        conn,
        ws,
        &master_key(),
        "lane-metrics",
        &records[0],
        Some(admin),
    )
    .await
    .unwrap();
    let run2 = process_lane_service::start_lane_run(
        conn,
        ws,
        &master_key(),
        "lane-metrics",
        &records[1],
        Some(admin),
    )
    .await
    .unwrap();
    let run3 = process_lane_service::start_lane_run(
        conn,
        ws,
        &master_key(),
        "lane-metrics",
        &records[2],
        Some(admin),
    )
    .await
    .unwrap();

    // BAT-2: tally, then approve the review -> completed. Deciding goes
    // through the admin: `approval_service::decide` requires an admin
    // actor (the step-role gate is the inbox's, not the engine's).
    process_lane_service::submit_person_step(
        conn,
        ws,
        &master_key(),
        &run2.id,
        &values(&[("counted", "12")]),
        Some(counter),
    )
    .await
    .unwrap();
    let run2 = graph_run_repo::get_run(conn, &run2.id).unwrap().unwrap();
    let approval = run2.pending_approval_id.expect("review approval");
    approval_service::decide(
        conn,
        &approval,
        ws,
        &master_key(),
        &AiApprovalResolution {
            approve: true,
            resolution_notes: None,
        },
        Some(admin),
    )
    .await
    .unwrap();

    // BAT-3: tally only, so it waits on the review approval.
    process_lane_service::submit_person_step(
        conn,
        ws,
        &master_key(),
        &run3.id,
        &values(&[("counted", "7")]),
        Some(counter),
    )
    .await
    .unwrap();
    (run1.id, run2.id, run3.id)
}

fn status(conn: &rusqlite::Connection, run_id: &str) -> String {
    graph_run_repo::get_run(conn, run_id)
        .unwrap()
        .unwrap()
        .status
}

/// Stage durations computed independently from the node rows, under the
/// same rule the service uses but derived here from the raw rows: a stage's
/// entered time is the earliest started_at among its nodes; it is left
/// when the next stage (in entered order) is entered, or when the run
/// finishes if it is the last stage entered.
fn independent_stage_seconds(conn: &rusqlite::Connection, run_id: &str) -> Vec<(String, f64)> {
    use chrono::DateTime;
    use lanesra_core::models::graph_run::is_terminal_status;
    use lanesra_core::repositories::execution_graph_repo;
    let run = graph_run_repo::get_run(conn, run_id).unwrap().unwrap();
    // A row's stage comes from its own column when set (human steps), else
    // from the compiled graph node's config — the same resolution the
    // service uses, re-derived here from the raw rows and graph.
    let graph = execution_graph_repo::get(conn, &run.graph_id)
        .unwrap()
        .unwrap();
    let mut stages: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for node in &graph.nodes {
        let config: Value = serde_json::from_str(&node.config_json).unwrap_or(Value::Null);
        if let Some(stage) = config.get("lane_stage_id").and_then(|v| v.as_str()) {
            stages.insert(node.id.clone(), stage.to_string());
        }
    }
    let mut entered: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for node in graph_run_repo::list_run_nodes(conn, run_id).unwrap() {
        let Some(stage) = node
            .stage_id
            .clone()
            .or_else(|| stages.get(&node.node_id).cloned())
        else {
            continue;
        };
        entered
            .entry(stage)
            .and_modify(|e| {
                if node.started_at < *e {
                    *e = node.started_at.clone();
                }
            })
            .or_insert_with(|| node.started_at.clone());
    }
    let mut ordered: Vec<(&String, &String)> = entered.iter().collect();
    ordered.sort_by(|a, b| a.1.cmp(b.1));
    let run_finished = if is_terminal_status(&run.status) {
        run.finished_at.clone()
    } else {
        None
    };
    let mut out = Vec::new();
    for (i, (stage, entered_at)) in ordered.iter().enumerate() {
        let completed = ordered
            .get(i + 1)
            .map(|(_, next)| (*next).clone())
            .or_else(|| run_finished.clone());
        let Some(completed_at) = completed else {
            continue;
        };
        let start = DateTime::parse_from_rfc3339(entered_at).unwrap();
        let end = DateTime::parse_from_rfc3339(&completed_at).unwrap();
        out.push((
            (*stage).clone(),
            (end - start).num_milliseconds() as f64 / 1000.0,
        ));
    }
    out
}

// --------------------------------------------------------------------------
// AC-1: the board shows each active run in its stage, only records the
// viewer may read.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-019 (WP-0713 AC-1)
#[tokio::test]
async fn board_shows_each_active_run_in_its_stage() {
    let (conn, ws, admin) = setup();
    let (records, counter, _manager, _, _) = seed(&conn, &ws, &admin);
    let (run1, run2, run3) = three_runs(&conn, &ws, &admin, &records, &counter).await;
    assert_eq!(status(&conn, &run2), "completed");

    let board =
        process_lane_service::list_lane_board(&conn, &ws, "lane-metrics", Some(&admin)).unwrap();
    assert_eq!(board.lane_id, "lane-metrics");
    // Two stages, in definition order.
    assert_eq!(board.stages.len(), 2);
    assert_eq!(board.stages[0].stage_id, "stage-tally");
    assert_eq!(board.stages[1].stage_id, "stage-review");
    // The completed run is not on the board; the two active runs are, each
    // in its current stage.
    let tally_runs: Vec<&str> = board.stages[0]
        .runs
        .iter()
        .map(|r| r.run_id.as_str())
        .collect();
    let review_runs: Vec<&str> = board.stages[1]
        .runs
        .iter()
        .map(|r| r.run_id.as_str())
        .collect();
    assert_eq!(tally_runs, vec![run1.as_str()]);
    assert_eq!(review_runs, vec![run3.as_str()]);
    assert_eq!(
        board.stages[0].runs[0].anchor_record_id,
        Some(records[0].clone())
    );
    assert_eq!(
        board.stages[1].runs[0].anchor_record_id,
        Some(records[2].clone())
    );
}

// Covers: REQ-LANE-019 (WP-0713 AC-1)
#[tokio::test]
async fn board_hides_runs_on_records_the_viewer_may_not_read() {
    let (conn, ws, admin) = setup();
    let (records, counter, _manager, _, _) = seed(&conn, &ws, &admin);
    let (_, _, _) = three_runs(&conn, &ws, &admin, &records, &counter).await;

    // A reader with an ORGANIZATION Read grant on batch sees both active runs.
    let reader_role = make_role(&conn, &ws, &admin, "Batch reader");
    grant_read(&conn, &admin, &reader_role, "batch", "ORGANIZATION");
    let reader = make_user(&conn, &ws, &admin, "reader", "Reader");
    access_role_service::assign_to_user(&conn, &reader, &reader_role, Some(&admin)).unwrap();
    let board =
        process_lane_service::list_lane_board(&conn, &ws, "lane-metrics", Some(&reader)).unwrap();
    assert_eq!(board.stages.iter().map(|s| s.runs.len()).sum::<usize>(), 2);

    // A reader whose Read grant is OWNER-scoped sees nothing: the batch
    // records have no owner, so no record falls in their scope. The user
    // must first lose the Standard User role `user_service::create`
    // assigns eagerly at creation (its default grants ordinary Read, and
    // the best scope across roles would win).
    let owner_role = make_role(&conn, &ws, &admin, "Batch owner-reader");
    grant_read(&conn, &admin, &owner_role, "batch", "OWNER");
    let owner_reader = make_user(&conn, &ws, &admin, "owner-reader", "Owner Reader");
    let standard =
        lanesra_core::repositories::access_role_repo::get_by_name(&conn, &ws, "Standard User")
            .unwrap()
            .expect("Standard User role");
    access_role_service::remove_from_user(&conn, &owner_reader, &standard.id, Some(&admin))
        .unwrap();
    access_role_service::assign_to_user(&conn, &owner_reader, &owner_role, Some(&admin)).unwrap();
    let board =
        process_lane_service::list_lane_board(&conn, &ws, "lane-metrics", Some(&owner_reader))
            .unwrap();
    assert_eq!(board.stages.iter().map(|s| s.runs.len()).sum::<usize>(), 0);
}

// --------------------------------------------------------------------------
// AC-2: "my steps" lists lane steps and approvals together, oldest first.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-019 (WP-0713 AC-2)
#[tokio::test]
async fn my_steps_lists_steps_and_approvals_together_oldest_first() {
    let (conn, ws, admin) = setup();
    let (records, counter, manager, _, _) = seed(&conn, &ws, &admin);
    let (run1, _, run3) = three_runs(&conn, &ws, &admin, &records, &counter).await;

    // The counter sees the waiting Tally step (run1) and nothing else.
    let mine = process_lane_service::list_my_steps(&conn, &ws, Some(&counter)).unwrap();
    assert_eq!(mine.items.len(), 1);
    assert_eq!(mine.items[0].kind, "step");
    assert_eq!(mine.items[0].run_id, run1);
    assert_eq!(mine.items[0].step_id, "tally");

    // The manager sees the waiting Review approval (run3) and nothing else.
    let mine = process_lane_service::list_my_steps(&conn, &ws, Some(&manager)).unwrap();
    assert_eq!(mine.items.len(), 1);
    assert_eq!(mine.items[0].kind, "approval");
    assert_eq!(mine.items[0].run_id, run3);
    assert_eq!(mine.items[0].step_id, "review");
    assert!(mine.items[0].approval_id.is_some());

    // The admin may act on both, oldest first (run1 waited before run3).
    let mine = process_lane_service::list_my_steps(&conn, &ws, Some(&admin)).unwrap();
    assert_eq!(mine.items.len(), 2);
    assert_eq!(mine.items[0].run_id, run1);
    assert_eq!(mine.items[1].run_id, run3);
    assert!(
        mine.items[0].waiting_since <= mine.items[1].waiting_since,
        "oldest first: {} then {}",
        mine.items[0].waiting_since,
        mine.items[1].waiting_since
    );
}

// --------------------------------------------------------------------------
// AC-3: timing figures equal the recorded timestamps.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-019 (WP-0713 AC-3)
#[tokio::test]
async fn timing_figures_equal_the_recorded_timestamps() {
    let (conn, ws, admin) = setup();
    let (records, counter, _manager, _, _) = seed(&conn, &ws, &admin);
    let (_, run2, run3) = three_runs(&conn, &ws, &admin, &records, &counter).await;

    let timing = process_lane_service::lane_timing(&conn, &ws, "lane-metrics").unwrap();
    assert_eq!(timing.total_runs, 3);
    assert_eq!(timing.completed_runs, 1);
    assert!((timing.completion_rate - 1.0 / 3.0).abs() < 1e-9);
    assert_eq!(timing.rework_steps, 0);

    // The stage durations, computed independently from each run's node
    // rows, must match the service's per-stage figures. Both the
    // completed run and the run waiting on review have left stage-tally,
    // so that stage's count is 2; only the completed run has left
    // stage-review.
    let mut expected: std::collections::BTreeMap<String, Vec<f64>> =
        std::collections::BTreeMap::new();
    for run_id in [&run2, &run3] {
        for (stage_id, seconds) in independent_stage_seconds(&conn, run_id) {
            expected.entry(stage_id).or_default().push(seconds);
        }
    }
    assert_eq!(expected.len(), 2, "both stages have durations");
    for (stage_id, seconds) in expected {
        let reported = timing
            .stages
            .iter()
            .find(|s| s.stage_id == stage_id)
            .expect("stage in timing");
        assert_eq!(reported.runs_completed, seconds.len() as i64);
        let avg = reported.avg_seconds.expect("avg for a completed stage");
        let mean = seconds.iter().sum::<f64>() / seconds.len() as f64;
        assert!(
            (avg - mean).abs() < 0.05,
            "stage {stage_id}: service {avg} vs independent {mean}"
        );
    }
}

// --------------------------------------------------------------------------
// AC-4: ageing lists active runs by time in current stage, longest first,
// only records the viewer may read.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-019 (WP-0713 AC-4)
#[tokio::test]
async fn ageing_lists_active_runs_longest_in_stage_first() {
    let (conn, ws, admin) = setup();
    let (records, counter, _manager, _, _) = seed(&conn, &ws, &admin);
    let (run1, _, run3) = three_runs(&conn, &ws, &admin, &records, &counter).await;

    let ageing =
        process_lane_service::list_lane_ageing(&conn, &ws, "lane-metrics", Some(&admin)).unwrap();
    // Only the two active runs; run1 entered its stage before run3.
    assert_eq!(ageing.len(), 2);
    assert_eq!(ageing[0].run_id, run1);
    assert_eq!(ageing[1].run_id, run3);
    assert!(
        ageing[0].seconds_in_stage >= ageing[1].seconds_in_stage,
        "longest first: {} then {}",
        ageing[0].seconds_in_stage,
        ageing[1].seconds_in_stage
    );

    // A reader whose Read grant is OWNER-scoped sees nothing: the batch
    // records have no owner. (Lose the eager Standard User role first, as
    // above.)
    let owner_role = make_role(&conn, &ws, &admin, "Batch owner-reader");
    grant_read(&conn, &admin, &owner_role, "batch", "OWNER");
    let owner_reader = make_user(&conn, &ws, &admin, "owner-reader", "Owner Reader");
    let standard =
        lanesra_core::repositories::access_role_repo::get_by_name(&conn, &ws, "Standard User")
            .unwrap()
            .expect("Standard User role");
    access_role_service::remove_from_user(&conn, &owner_reader, &standard.id, Some(&admin))
        .unwrap();
    access_role_service::assign_to_user(&conn, &owner_reader, &owner_role, Some(&admin)).unwrap();
    let ageing =
        process_lane_service::list_lane_ageing(&conn, &ws, "lane-metrics", Some(&owner_reader))
            .unwrap();
    assert!(ageing.is_empty());
}
