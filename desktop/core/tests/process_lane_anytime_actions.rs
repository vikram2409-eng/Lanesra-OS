//! Phase 7, WP-0711: notes, attachments and reassign at any point
//! (ADR-0024; REQ-LANE-018).
//!
//! From the runner, at any step: add a note, attach a file, reassign the
//! current step to one person. Each action is audited and shows in the
//! runner's audit trail (AC-1); reassign only names a person who holds the
//! step's role (AC-2); an attachment is visible only through its run
//! (AC-3). Covers:
//!   AC-1  a note lands on the run with its step, an attachment lands with
//!         its bytes on disk, and both appear in the run view's audit trail.
//!   AC-2  reassign to a role holder narrows the step to that person (the
//!         view names them, only they - or an admin - may complete it);
//!         reassign to someone without the role is refused; a later
//!         handoff clears the reassignment.
//!   AC-3  an attachment cannot be read through another run, and is not
//!         listed on another run's view.

use serde_json::{json, Map, Value};

use lanesra_core::models::access_role::AccessRoleInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::graph_run_repo;
use lanesra_core::services::{
    access_role_service, custom_object_service, custom_record_service, process_lane_service,
    user_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Anytime Co".into(),
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

fn make_user(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    username: &str,
    display_name: &str,
) -> String {
    // NewUser.roles are the legacy system roles (Administrator, Manager,
    // Sales, Finance, ReadOnly); a lane step's role is an access role and
    // is assigned separately, below.
    user_service::create(
        conn,
        ws,
        &NewUser {
            username: username.into(),
            display_name: display_name.into(),
            password: "correct horse battery staple".into(),
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
            description: format!("{name} role"),
        },
        Some(admin),
    )
    .unwrap()
    .id
}

/// One object, one record, one role (`role_counter`) held by two users
/// (Casey and Robin) and a third user (Ollie) who holds no role.
struct Fixture {
    record_id: String,
    role_id: String,
    casey: String,
    robin: String,
    ollie: String,
}

fn seed(conn: &rusqlite::Connection, ws: &str, admin: &str) -> Fixture {
    custom_object_service::create_with_key(
        conn,
        ws,
        "batch",
        &CustomObjectDefinitionInput {
            singular_label: "batch".into(),
            plural_label: "batchs".into(),
            icon: "▣".into(),
            prefix: "BAT".into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
    let record_id = custom_record_service::create(
        conn,
        ws,
        &CustomRecordInput {
            object_key: "batch".into(),
            primary_name: "BAT-1".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(admin),
    )
    .unwrap()
    .id;
    let role_id = make_role(conn, ws, admin, "Counter");
    let casey = make_user(conn, ws, admin, "casey", "Casey Counter");
    let robin = make_user(conn, ws, admin, "robin", "Robin Receiver");
    let ollie = make_user(conn, ws, admin, "ollie", "Ollie Outsider");
    access_role_service::assign_to_user(conn, &casey, &role_id, Some(admin)).unwrap();
    access_role_service::assign_to_user(conn, &robin, &role_id, Some(admin)).unwrap();
    Fixture {
        record_id,
        role_id,
        casey,
        robin,
        ollie,
    }
}

/// A one-step form lane: the step waits on `role_counter` and writes one
/// run-data text field, so a submit completes the run.
fn publish_simple_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, role_id: &str) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-simple", "name": "Simple", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": [{"key": "note_text", "type": "string", "required": false}]},
            "stages": [{
                "id": "s1", "name": "Step one", "steps": [{
                    "id": "fill", "name": "Fill", "kind": "form",
                    "role": role_id, "required": true,
                    "fields": [{
                        "key": "note_text", "label": "Note text", "type": "string",
                        "binds_to": "run_data.note_text", "required": true
                    }]
                }]
            }]
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-simple", Some(admin)).unwrap();
}

async fn start_run(conn: &rusqlite::Connection, ws: &str, admin: &str, record_id: &str) -> String {
    process_lane_service::start_lane_run(
        conn,
        ws,
        &master_key(),
        "lane-simple",
        record_id,
        Some(admin),
    )
    .await
    .unwrap()
    .id
}

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(v)))
        .collect()
}

fn run_data(conn: &rusqlite::Connection, run_id: &str) -> Value {
    let run = graph_run_repo::get_run(conn, run_id).unwrap().unwrap();
    serde_json::from_str(run.run_data_json.as_deref().unwrap_or("{}")).unwrap()
}

fn status(conn: &rusqlite::Connection, run_id: &str) -> String {
    graph_run_repo::get_run(conn, run_id)
        .unwrap()
        .unwrap()
        .status
}

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "wp0711-test-{}",
        lanesra_core::domain::ids::new_uuid()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// --------------------------------------------------------------------------
// AC-1: a note lands on the run with its step and shows in the audit trail.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-018 (WP-0711 AC-1)
#[tokio::test]
async fn a_note_lands_on_the_run_with_its_step_and_in_the_audit_trail() {
    let (conn, ws, admin) = setup();
    let fx = seed(&conn, &ws, &admin);
    publish_simple_lane(&conn, &ws, &admin, &fx.role_id);
    let run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;
    assert_eq!(status(&conn, &run_id), "waiting_person");

    let note = process_lane_service::add_lane_run_note(
        &conn,
        &ws,
        &run_id,
        "Checked the count against the pick list.",
        Some(&fx.casey),
    )
    .unwrap();
    assert_eq!(note.run_id, run_id);
    assert_eq!(note.step_id.as_deref(), Some("fill"));
    assert_eq!(note.author_user_id.as_deref(), Some(fx.casey.as_str()));
    assert_eq!(note.body, "Checked the count against the pick list.");

    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&admin)).unwrap();
    assert_eq!(view.notes.len(), 1, "the run view lists the note");
    assert_eq!(view.notes[0].id, note.id);
    assert!(
        view.events
            .iter()
            .any(|e| e.event_type == "lane_run_note_added"),
        "the add is in the run's audit trail: {:?}",
        view.events
            .iter()
            .map(|e| &e.event_type)
            .collect::<Vec<_>>()
    );

    // An empty note is refused, not stored as a blank row.
    assert!(
        process_lane_service::add_lane_run_note(&conn, &ws, &run_id, "   ", Some(&fx.casey))
            .is_err()
    );
    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&admin)).unwrap();
    assert_eq!(view.notes.len(), 1, "the refused note was not stored");
}

// --------------------------------------------------------------------------
// AC-1/AC-3: an attachment lands with its bytes and is visible only
// through its own run.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-018 (WP-0711 AC-1, AC-3)
#[tokio::test]
async fn an_attachment_lands_with_its_bytes_and_is_visible_only_through_its_run() {
    let (conn, ws, admin) = setup();
    let fx = seed(&conn, &ws, &admin);
    publish_simple_lane(&conn, &ws, &admin, &fx.role_id);
    let run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;
    let dir = temp_dir();

    let bytes = b"count sheet, line 1: 48 counted";
    let attachment = process_lane_service::attach_lane_run_file(
        &conn,
        &ws,
        &dir,
        &run_id,
        "count-sheet.txt",
        Some("text/plain"),
        "Y291bnQgc2hlZXQsIGxpbmUgMTogNDggY291bnRlZA==",
        Some(&fx.casey),
    )
    .unwrap();
    assert_eq!(attachment.run_id, run_id);
    assert_eq!(attachment.file_name, "count-sheet.txt");
    assert_eq!(attachment.size_bytes, Some(bytes.len() as i64));
    assert!(
        dir.join(&attachment.relative_path).exists(),
        "the bytes are on disk under the data dir"
    );

    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&admin)).unwrap();
    assert_eq!(
        view.attachments.len(),
        1,
        "the run view lists the attachment"
    );
    assert!(
        view.events
            .iter()
            .any(|e| e.event_type == "lane_run_attachment_added"),
        "the attach is in the run's audit trail"
    );

    // The bytes come back through the run-scoped read path.
    let (read_back, read_bytes) = process_lane_service::read_lane_run_attachment(
        &conn,
        &ws,
        &dir,
        &run_id,
        &attachment.id,
        Some(&admin),
    )
    .unwrap();
    assert_eq!(read_back.id, attachment.id);
    assert_eq!(read_bytes, bytes);

    // A bad file name and empty content are refused, not stored.
    assert!(process_lane_service::attach_lane_run_file(
        &conn,
        &ws,
        &dir,
        &run_id,
        "../escape.txt",
        None,
        "eA==",
        Some(&fx.casey),
    )
    .is_err());
    assert!(process_lane_service::attach_lane_run_file(
        &conn,
        &ws,
        &dir,
        &run_id,
        "empty.txt",
        None,
        "",
        Some(&fx.casey),
    )
    .is_err());
    // Not valid base64 at all.
    assert!(process_lane_service::attach_lane_run_file(
        &conn,
        &ws,
        &dir,
        &run_id,
        "bad.txt",
        None,
        "!!!",
        Some(&fx.casey),
    )
    .is_err());
    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&admin)).unwrap();
    assert_eq!(
        view.attachments.len(),
        1,
        "the refused files were not stored"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

// Covers: REQ-LANE-018 (WP-0711 AC-3)
#[tokio::test]
async fn an_attachment_cannot_be_read_through_another_run() {
    let (conn, ws, admin) = setup();
    let fx = seed(&conn, &ws, &admin);
    publish_simple_lane(&conn, &ws, &admin, &fx.role_id);
    let run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;
    let dir = temp_dir();
    let attachment = process_lane_service::attach_lane_run_file(
        &conn,
        &ws,
        &dir,
        &run_id,
        "count-sheet.txt",
        Some("text/plain"),
        "Y291bnQgc2hlZXQ=",
        Some(&fx.casey),
    )
    .unwrap();

    // Complete the run, then start a second run on the same record: the
    // first run's attachment must not be listed on the second run's view,
    // and its bytes must not be readable through the second run's id.
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("note_text", "done")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(status(&conn, &run_id), "completed");
    let second_run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;
    assert_ne!(second_run_id, run_id);

    let view =
        process_lane_service::get_run_view(&conn, &ws, &second_run_id, Some(&admin)).unwrap();
    assert!(
        view.attachments.is_empty(),
        "the second run's view lists no attachment from the first run"
    );
    assert!(process_lane_service::read_lane_run_attachment(
        &conn,
        &ws,
        &dir,
        &second_run_id,
        &attachment.id,
        Some(&admin),
    )
    .is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}

// --------------------------------------------------------------------------
// AC-2: reassign narrows the step to one person who holds the role.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-018 (WP-0711 AC-2)
#[tokio::test]
async fn reassign_narrows_the_step_to_one_person_who_holds_the_role() {
    let (conn, ws, admin) = setup();
    let fx = seed(&conn, &ws, &admin);
    publish_simple_lane(&conn, &ws, &admin, &fx.role_id);
    let run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;

    // The candidates are exactly the role holders - not the outsider, and
    // not the admin (who can act but does not hold the role).
    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&admin)).unwrap();
    let mut candidate_ids: Vec<&str> = view
        .reassign_candidates
        .iter()
        .map(|c| c.user_id.as_str())
        .collect();
    candidate_ids.sort();
    let mut expected = vec![fx.casey.as_str(), fx.robin.as_str()];
    expected.sort();
    assert_eq!(candidate_ids, expected);

    // Reassigning to someone without the role is refused.
    assert!(process_lane_service::reassign_lane_step(
        &conn,
        &ws,
        &run_id,
        &fx.ollie,
        "Ollie should cover",
        Some(&admin),
    )
    .is_err());

    // The admin reassigns the step to Casey.
    let reassigned = process_lane_service::reassign_lane_step(
        &conn,
        &ws,
        &run_id,
        &fx.casey,
        "Casey is on shift",
        Some(&admin),
    )
    .unwrap();
    assert_eq!(reassigned.status, "waiting_person", "the run keeps waiting");
    assert_eq!(
        run_data(&conn, &run_id)["__assignees"]["fill"],
        json!(fx.casey),
        "the assignee is stored in run data"
    );

    // The view now names Casey as who the step waits on, and only Casey
    // (or an admin) may complete it.
    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&fx.casey)).unwrap();
    assert_eq!(
        view.current.as_ref().unwrap().waiting_on.as_deref(),
        Some("Casey Counter")
    );
    assert!(view.current.as_ref().unwrap().can_complete);
    let view = process_lane_service::get_run_view(&conn, &ws, &run_id, Some(&fx.robin)).unwrap();
    assert!(!view.current.as_ref().unwrap().can_complete);
    assert!(
        view.events
            .iter()
            .any(|e| e.event_type == "lane_step_reassigned"),
        "the reassign is in the run's audit trail"
    );

    // Robin - a role holder, but not the assignee - cannot complete it.
    assert!(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("note_text", "robin tried")]),
        Some(&fx.robin),
    )
    .await
    .is_err());
    assert_eq!(status(&conn, &run_id), "waiting_person");

    // Robin cannot reassign it away either; only the assignee or an admin.
    assert!(process_lane_service::reassign_lane_step(
        &conn,
        &ws,
        &run_id,
        &fx.robin,
        "taking over",
        Some(&fx.robin),
    )
    .is_err());

    // Casey completes it, and the run finishes.
    let done = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("note_text", "done by Casey")]),
        Some(&fx.casey),
    )
    .await
    .unwrap();
    assert_eq!(done.status, "completed");
}

// Covers: REQ-LANE-018 (WP-0711 AC-2)
#[tokio::test]
async fn a_later_handoff_clears_the_reassignment() {
    let (conn, ws, admin) = setup();
    let fx = seed(&conn, &ws, &admin);
    publish_simple_lane(&conn, &ws, &admin, &fx.role_id);
    let run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;

    process_lane_service::reassign_lane_step(
        &conn,
        &ws,
        &run_id,
        &fx.casey,
        "Casey is on shift",
        Some(&admin),
    )
    .unwrap();
    assert_eq!(
        run_data(&conn, &run_id)["__assignees"]["fill"],
        json!(fx.casey)
    );

    // Hand the step to a different role: the person-level assignment for
    // the step is cleared, so the new role's holders can act.
    let other_role = make_role(&conn, &ws, &admin, "Checker");
    process_lane_service::handoff_step(
        &conn,
        &ws,
        &run_id,
        &other_role,
        "checking needs a second pair of eyes",
        Some(&admin),
    )
    .unwrap();
    assert!(
        run_data(&conn, &run_id)
            .get("__assignees")
            .and_then(|a| a.get("fill"))
            .is_none(),
        "the handoff cleared the step's assignee"
    );
    assert_eq!(
        run_data(&conn, &run_id)["__handoffs"]["fill"],
        json!(other_role)
    );
}

// --------------------------------------------------------------------------
// AC-1: a finished run takes no more notes or attachments.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-018 (WP-0711 AC-1)
#[tokio::test]
async fn a_finished_run_takes_no_more_notes_or_attachments() {
    let (conn, ws, admin) = setup();
    let fx = seed(&conn, &ws, &admin);
    publish_simple_lane(&conn, &ws, &admin, &fx.role_id);
    let run_id = start_run(&conn, &ws, &admin, &fx.record_id).await;
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("note_text", "done")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(status(&conn, &run_id), "completed");

    assert!(process_lane_service::add_lane_run_note(
        &conn,
        &ws,
        &run_id,
        "too late",
        Some(&fx.casey),
    )
    .is_err());
    let dir = temp_dir();
    assert!(process_lane_service::attach_lane_run_file(
        &conn,
        &ws,
        &dir,
        &run_id,
        "late.txt",
        None,
        "bGF0ZQ==",
        Some(&fx.casey),
    )
    .is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}
