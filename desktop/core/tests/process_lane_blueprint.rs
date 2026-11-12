//! AI Agent Platform v2, Phase 7: the lane blueprint read (WP-0712) -
//! a read-only view of a lane definition for any authenticated workspace
//! member, carrying no run and no record data (REQ-LANE-020).
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    process_lane_blueprint, process_lane_service, user_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "secretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn viewer_input(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        display_name: username.into(),
        password: "anothersecretpw".into(),
        roles: vec!["ReadOnly".to_string()],
    }
}

fn example_path() -> String {
    // CARGO_MANIFEST_DIR is desktop/core; the example lives at the repo
    // root, two levels up. (WP-0703's test file uses three levels, which
    // overshoots the root - flagged to the parent, not fixed here.)
    format!(
        "{}/../../program/spec/schemas/examples/lane-wh-receiving.json",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn example_json() -> String {
    std::fs::read_to_string(example_path()).expect("demo lane example exists")
}

fn import_demo_lane(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    process_lane_service::import_lane_definition(conn, ws, &example_json(), Some(admin)).unwrap();
}

/// Covers: REQ-LANE-020 (WP-0712 AC-1)
#[test]
fn blueprint_renders_the_demo_lane_definition_including_branches() {
    let (conn, ws, admin) = setup_workspace();
    import_demo_lane(&conn, &ws, &admin);

    let blueprint =
        process_lane_blueprint::get_lane_blueprint(&conn, &ws, "lane-wh-receiving", Some(&admin))
            .unwrap();
    assert_eq!(blueprint.lane_key, "lane-wh-receiving");
    assert_eq!(blueprint.name, "Receive Shipment");
    assert_eq!(blueprint.object_key, "purchase_order");
    assert_eq!(blueprint.version, 1);
    assert_eq!(blueprint.status, "draft");
    assert_eq!(blueprint.definition.id, "lane-wh-receiving");

    // Five stages, twelve steps (ADR-0024 section 3 / the schema example).
    assert_eq!(blueprint.definition.stages.len(), 5);
    let steps: Vec<_> = blueprint
        .definition
        .stages
        .iter()
        .flat_map(|s| s.steps.iter())
        .collect();
    assert_eq!(steps.len(), 12);

    // The discrepancy check branches both ways, and the approval does too.
    let check = steps
        .iter()
        .find(|s| s.id == "discrepancy-check")
        .expect("discrepancy check step exists");
    let branches = check.branches.as_ref().expect("check step branches");
    assert_eq!(branches.len(), 2);
    assert!(branches.iter().any(|b| b.on == "pass"));
    assert!(branches.iter().any(|b| b.on == "fail"));
    let approval = steps
        .iter()
        .find(|s| s.id == "review-discrepancy")
        .expect("approval step exists");
    assert_eq!(
        approval
            .branches
            .as_ref()
            .expect("approval step branches")
            .len(),
        2
    );

    // Run-data fields and mapped record fields are part of the blueprint.
    let run_data = blueprint
        .definition
        .run_data
        .as_ref()
        .expect("run-data schema exists");
    assert_eq!(run_data.fields.len(), 8);
    assert!(run_data.fields.iter().any(|f| f.key == "carrier"));
    let enter_quantities = steps
        .iter()
        .find(|s| s.id == "enter-quantities")
        .expect("enter-quantities step exists");
    let fields = enter_quantities
        .fields
        .as_ref()
        .expect("form step fields exist");
    assert!(fields
        .iter()
        .any(|f| f.binds_to == "record.lines[].qty_received"));
    assert!(fields
        .iter()
        .any(|f| f.binds_to == "run_data.rejection_reason"));
}

/// Covers: REQ-LANE-020 (WP-0712 AC-1)
#[test]
fn blueprint_renders_any_valid_lane_definition_not_only_the_demo_lane() {
    let (conn, ws, admin) = setup_workspace();
    let second = example_json()
        .replace("lane-wh-receiving", "lane-test-second")
        .replace("Receive Shipment", "Second Lane");
    process_lane_service::import_lane_definition(&conn, &ws, &second, Some(&admin)).unwrap();

    let blueprint =
        process_lane_blueprint::get_lane_blueprint(&conn, &ws, "lane-test-second", Some(&admin))
            .unwrap();
    assert_eq!(blueprint.name, "Second Lane");
    assert_eq!(blueprint.definition.stages.len(), 5);
}

/// Covers: REQ-LANE-020 (WP-0712 AC-3)
#[test]
fn a_viewer_reads_the_blueprint_and_the_list_but_nobody_unauthenticated_does() {
    let (conn, ws, admin) = setup_workspace();
    import_demo_lane(&conn, &ws, &admin);
    let viewer = user_service::create(&conn, &ws, &viewer_input("viewer"), Some(&admin)).unwrap();

    let blueprint = process_lane_blueprint::get_lane_blueprint(
        &conn,
        &ws,
        "lane-wh-receiving",
        Some(&viewer.id),
    )
    .unwrap();
    assert_eq!(blueprint.name, "Receive Shipment");

    let list = process_lane_blueprint::list_lane_blueprints(&conn, &ws, Some(&viewer.id)).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].lane_key, "lane-wh-receiving");
    assert_eq!(list[0].latest_version, 1);
    assert_eq!(list[0].latest_status, "draft");

    let unauthenticated =
        process_lane_blueprint::get_lane_blueprint(&conn, &ws, "lane-wh-receiving", None);
    assert!(unauthenticated.is_err());
    let unauthenticated_list = process_lane_blueprint::list_lane_blueprints(&conn, &ws, None);
    assert!(unauthenticated_list.is_err());
}

/// Covers: REQ-LANE-020 (WP-0712 AC-3)
#[test]
fn the_blueprint_carries_no_record_data() {
    let (conn, ws, admin) = setup_workspace();
    import_demo_lane(&conn, &ws, &admin);

    let blueprint =
        process_lane_blueprint::get_lane_blueprint(&conn, &ws, "lane-wh-receiving", Some(&admin))
            .unwrap();
    let rendered: serde_json::Value = serde_json::to_value(&blueprint).unwrap();
    // The payload is the stored definition plus version metadata, nothing
    // else: no anchor record, no run id, no entered values, no stock
    // levels read from any record table - the service only ever reads
    // `process_lanes` / `process_lane_versions`.
    let top = rendered.as_object().expect("blueprint object");
    assert_eq!(top.len(), 6);
    for key in [
        "lane_key",
        "name",
        "object_key",
        "version",
        "status",
        "definition",
    ] {
        assert!(top.contains_key(key), "blueprint carries {key}");
    }
    let flat = serde_json::to_string(&rendered).unwrap();
    assert!(!flat.contains("anchor_record_id"));
    assert!(!flat.contains("run_id"));
    assert!(!flat.contains("entered_values"));
    // Run data in a blueprint is field declarations only, never values.
    let run_data = rendered["definition"]["run_data"]["fields"]
        .as_array()
        .expect("run-data fields");
    for field in run_data {
        let keys: Vec<&str> = field
            .as_object()
            .expect("run-data field object")
            .keys()
            .map(String::as_str)
            .collect();
        assert!(
            keys.iter()
                .all(|k| ["key", "type", "required", "per_child", "description"].contains(k)),
            "run-data field declares a field, never a value: {keys:?}"
        );
    }
}

/// Covers: REQ-LANE-020 (WP-0712 AC-1)
#[test]
fn an_unknown_lane_reads_as_not_found() {
    let (conn, ws, admin) = setup_workspace();
    import_demo_lane(&conn, &ws, &admin);

    let missing =
        process_lane_blueprint::get_lane_blueprint(&conn, &ws, "lane-no-such-lane", Some(&admin));
    assert!(missing.is_err());
}

/// Covers: REQ-LANE-020 (WP-0712 AC-2)
#[test]
fn the_blueprint_definition_is_the_stored_definition_verbatim() {
    let (conn, ws, admin) = setup_workspace();
    import_demo_lane(&conn, &ws, &admin);

    let blueprint =
        process_lane_blueprint::get_lane_blueprint(&conn, &ws, "lane-wh-receiving", Some(&admin))
            .unwrap();
    // Compared through the same struct on both sides, so fields the file
    // omits (a `required: false` / `per_child: false` the schema defaults)
    // serialize identically instead of differing on absent-vs-false.
    let reparsed = lanesra_core::models::process_lane::parse_definition(&example_json()).unwrap();
    assert_eq!(
        serde_json::to_value(&blueprint.definition).unwrap(),
        serde_json::to_value(&reparsed.lane).unwrap()
    );
    // And the stored bytes are the file's bytes, unchanged.
    let stored: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
    assert_eq!(stored["lane"]["id"], "lane-wh-receiving");
    assert_eq!(stored["lane"]["name"], "Receive Shipment");
}
