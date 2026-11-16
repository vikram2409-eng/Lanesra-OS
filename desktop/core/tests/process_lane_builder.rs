//! Phase 7, WP-0718: the visual lane builder's core/service layer
//! (REQ-LANE-024) - canonical round-trip (AC-1), live publish checks on
//! an unsaved draft (AC-2), the per-step reads/writes map and lane-level
//! plumbing table, the Action-Type picker catalogue, the builder dry run,
//! and the builder save path. The builder reuses the WP-0714 checks and
//! dry run, so its reports must match `check_publish` / `dry_run_lane`
//! on the same stored lane (AC-4: the same service both editions call).
//!
//! Covers: REQ-LANE-024 (WP-0718 AC-1), REQ-LANE-024 (WP-0718 AC-2),
//! REQ-LANE-024 (WP-0718 AC-4)

use serde_json::{json, Map, Value};

use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    action_type_service, custom_field_service, custom_object_service, custom_record_service,
    process_lane_builder, process_lane_publish, process_lane_service, relationship_service,
    workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Builder Co".into(),
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

fn field_input(label: &str, field_type: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: String::new(),
        label: label.into(),
        field_type: field_type.into(),
        options: vec![],
        required: false,
        show_in_list: false,
        sort_order: 0,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable: false,
        is_filterable: false,
        is_reportable: false,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}

fn create_object(conn: &rusqlite::Connection, ws: &str, admin: &str, key: &str) {
    let (singular, plural) = (key.to_string(), format!("{key}s"));
    custom_object_service::create_with_key(
        conn,
        ws,
        key,
        &CustomObjectDefinitionInput {
            singular_label: singular,
            plural_label: plural,
            icon: "▣".into(),
            prefix: "XX".into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
}

fn create_field(conn: &rusqlite::Connection, ws: &str, admin: &str, entity: &str, key: &str) {
    let mut input = field_input(key, "text");
    input.entity_type = entity.into();
    custom_field_service::create_definition_with_key(conn, ws, key, &input, Some(admin)).unwrap();
}

fn create_record(
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

fn set_values(
    conn: &rusqlite::Connection,
    object: &str,
    record: &str,
    admin: &str,
    pairs: &[(&str, &str)],
) {
    let mut values = lanesra_core::models::custom_field::CustomFieldValues::new();
    for (k, v) in pairs {
        values.insert((*k).into(), (*v).into());
    }
    custom_field_service::set_entity_values(conn, object, record, &values, Some(admin)).unwrap();
}

fn get_values(conn: &rusqlite::Connection, record: &str) -> Map<String, Value> {
    let values = custom_field_service::get_entity_values(conn, record).unwrap();
    values
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect()
}

fn example_lane(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../program/spec/schemas/examples/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|_| panic!("demo lane example {name} exists"))
}

struct WarehouseSeed {
    po: String,
    po_lines: Vec<String>,
}

fn seed_warehouse(conn: &rusqlite::Connection, ws: &str, admin: &str) -> WarehouseSeed {
    for object in [
        "purchase_order",
        "purchase_order_line",
        "sales_order",
        "sales_order_line",
        "cycle_count",
        "cycle_count_line",
        "reorder_rule",
    ] {
        create_object(conn, ws, admin, object);
    }
    create_field(conn, ws, admin, "purchase_order", "status");
    create_field(conn, ws, admin, "purchase_order", "expected_date");
    create_field(conn, ws, admin, "purchase_order", "supplier_id");
    create_field(conn, ws, admin, "purchase_order_line", "qty_ordered");
    create_field(conn, ws, admin, "purchase_order_line", "qty_received");
    create_field(conn, ws, admin, "purchase_order_line", "qty_rejected");
    create_field(conn, ws, admin, "purchase_order_line", "product_id");
    create_field(conn, ws, admin, "sales_order", "status");
    create_field(conn, ws, admin, "sales_order", "carrier");
    create_field(conn, ws, admin, "sales_order", "ship_method");
    create_field(conn, ws, admin, "sales_order_line", "qty_picked");
    create_field(conn, ws, admin, "sales_order_line", "qty_reserved");
    create_field(conn, ws, admin, "sales_order_line", "qty_damaged");
    create_field(conn, ws, admin, "sales_order_line", "qty_packed");
    create_field(conn, ws, admin, "sales_order_line", "product_id");
    create_field(conn, ws, admin, "cycle_count", "status");
    create_field(conn, ws, admin, "cycle_count_line", "counted_qty");
    create_field(conn, ws, admin, "cycle_count_line", "variance");
    create_field(conn, ws, admin, "cycle_count_line", "on_hand");
    create_field(conn, ws, admin, "cycle_count_line", "product_id");
    create_field(conn, ws, admin, "cycle_count_line", "location_id");
    create_field(conn, ws, admin, "reorder_rule", "status");
    create_field(conn, ws, admin, "reorder_rule", "triggered");
    create_field(conn, ws, admin, "reorder_rule", "product_id");

    for (source, target) in [
        ("purchase_order", "purchase_order_line"),
        ("sales_order", "sales_order_line"),
        ("cycle_count", "cycle_count_line"),
    ] {
        relationship_service::create(
            conn,
            ws,
            &RelationshipDefinitionInput {
                source_entity_type: source.into(),
                target_entity_type: target.into(),
                target_is_polymorphic: false,
                relationship_type: "many_to_many".into(),
                forward_label: "lines".into(),
                reverse_label: source.into(),
                is_required: false,
                show_related_list: true,
                delete_behavior: "restrict".into(),
                sort_order: 0,
            },
            Some(admin),
        )
        .unwrap();
    }

    let po = create_record(conn, ws, admin, "purchase_order", "PO-2026-001");
    set_values(
        conn,
        "purchase_order",
        &po,
        admin,
        &[("status", "OPEN"), ("expected_date", "2026-11-20")],
    );
    let mut po_lines = Vec::new();
    for (name, qty, product) in [
        ("PO-2026-001-L1", "40", "HG-1001"),
        ("PO-2026-001-L2", "30", "HG-1002"),
    ] {
        let line = create_record(conn, ws, admin, "purchase_order_line", name);
        set_values(
            conn,
            "purchase_order_line",
            &line,
            admin,
            &[
                ("qty_ordered", qty),
                ("qty_received", "0"),
                ("qty_rejected", "0"),
                ("product_id", product),
            ],
        );
        po_lines.push(line);
    }
    let so = create_record(conn, ws, admin, "sales_order", "SO-2026-002");
    set_values(
        conn,
        "sales_order",
        &so,
        admin,
        &[
            ("status", "ALLOCATED"),
            ("carrier", "FastFreight"),
            ("ship_method", "GROUND"),
        ],
    );
    let cc = create_record(conn, ws, admin, "cycle_count", "CC-2026-001");
    set_values(conn, "cycle_count", &cc, admin, &[("status", "COUNTING")]);
    let mut cc_lines = Vec::new();
    for (i, (on_hand, variance)) in [
        ("10", "-2"),
        ("20", "0"),
        ("30", "1"),
        ("40", "-2"),
        ("50", "0"),
    ]
    .into_iter()
    .enumerate()
    {
        let line = create_record(
            conn,
            ws,
            admin,
            "cycle_count_line",
            &format!("CC-2026-001-L{}", i + 1),
        );
        set_values(
            conn,
            "cycle_count_line",
            &line,
            admin,
            &[
                ("on_hand", on_hand),
                ("variance", variance),
                ("product_id", "HG-1001"),
                ("location_id", "A-01-03"),
            ],
        );
        cc_lines.push(line);
    }
    let rr = create_record(conn, ws, admin, "reorder_rule", "RR-2026-001");
    set_values(
        conn,
        "reorder_rule",
        &rr,
        admin,
        &[
            ("status", "TRIGGERED"),
            ("triggered", "true"),
            ("product_id", "HG-1001"),
        ],
    );

    for (source, target, lines) in [
        ("purchase_order", "purchase_order_line", &po_lines),
        ("cycle_count", "cycle_count_line", &cc_lines),
    ] {
        let anchor = if source == "purchase_order" { &po } else { &cc };
        let defs = relationship_service::list(conn, ws, true).unwrap();
        let def = defs
            .iter()
            .find(|d| d.source_entity_type == source && d.target_entity_type == target)
            .expect("lines relationship exists");
        for line in lines {
            relationship_service::link(
                conn,
                ws,
                &def.id,
                source,
                anchor,
                target,
                line,
                Some(admin),
            )
            .unwrap();
        }
    }

    WarehouseSeed { po, po_lines }
}

fn import_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, definition: &str) {
    process_lane_service::import_lane_definition(conn, ws, definition, Some(admin)).unwrap();
}

fn fixtures(
    run_data: Value,
    per_child: Value,
    approvals: Value,
) -> process_lane_publish::DryRunFixtures {
    serde_json::from_value(json!({
        "run_data": run_data,
        "per_child": per_child,
        "approvals": approvals,
        "form_values": {},
    }))
    .unwrap()
}

fn count_records(conn: &rusqlite::Connection, ws: &str, object: &str) -> usize {
    custom_record_service::list(conn, ws, object).unwrap().len()
}

fn adopt_set_tracking(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    action_type_service::create(
        conn,
        ws,
        &ActionTypeDefinitionInput {
            name: "Set Tracking".into(),
            description: String::new(),
            target_object_type: "widget".into(),
            required_capability: "update".into(),
            parameters: vec![ActionParameterSpec {
                key: "val".into(),
                label: "Value".into(),
                field_type: "text".into(),
                required: true,
                default_value: None,
            }],
            effect: "set_fields".into(),
            effect_config_json: json!({"field_sources": {"tracking": "val"}}).to_string(),
        },
        Some(admin),
    )
    .unwrap();
}

fn seed_widget_with_tracking_action(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    create_object(conn, ws, admin, "widget");
    create_field(conn, ws, admin, "widget", "tracking");
    adopt_set_tracking(conn, ws, admin);
    create_record(conn, ws, admin, "widget", "Widget one")
}

fn widget_lane_json(id: &str, writes: Value, version: i64) -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": "Widget Lane", "object_key": "widget",
            "version": version, "status": "draft",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true},
                {"key": "count", "type": "integer"}
            ]},
            "stages": [
                {"id": "capture", "name": "Capture", "steps": [
                    {"id": "enter", "name": "Enter", "kind": "form", "role": "r",
                     "required": true,
                     "fields": [{"key": "carrier", "label": "Carrier", "type": "string",
                                 "binds_to": "run_data.carrier", "required": true}]},
                    {"id": "count-check", "name": "Count check", "kind": "check",
                     "check": {"scope": "anchor", "message": "count must be positive",
                               "condition": {"field": "$run_data.count", "operator": "greater_than", "value": 0}},
                     "branches": [
                        {"on": "pass", "target": {"stage": "post", "step": "post-tracking"}},
                        {"on": "fail", "end": {"outcome": "held"}}]}
                ]},
                {"id": "post", "name": "Post", "steps": [
                    {"id": "post-tracking", "name": "Post tracking", "kind": "record", "role": "r",
                     "required": true, "writes": writes}
                ]}
            ]
        }
    })
    .to_string()
}

fn step_count(definition_json: &str) -> usize {
    let value: Value = serde_json::from_str(definition_json).unwrap();
    value["lane"]["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["steps"].as_array().unwrap().len())
        .sum()
}

fn report_ids(report: &process_lane_publish::PublishReport, id: &str, status: &str) -> usize {
    report
        .checks
        .iter()
        .filter(|c| c.id == id && c.status == status)
        .count()
}

// --------------------------------------------------------------------------
// AC-1: canonical round-trip.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn round_trip_all_five_example_lanes_are_canonical_and_stable() {
    for name in [
        "lane-wh-receiving",
        "lane-wh-pickpack",
        "lane-wh-count",
        "lane-wh-replenish",
        "lane-wh-supplier-check",
    ] {
        let original = example_lane(name);
        let canonical = process_lane_builder::canonical_lane_json(&original).unwrap();
        // Canonicalising twice is a fixed point.
        let again = process_lane_builder::canonical_lane_json(&canonical).unwrap();
        assert_eq!(canonical, again, "{name}: canonical form must be stable");
        // Byte-identical modulo key order: the parsed values are equal.
        let original_value: Value = serde_json::from_str(&original).unwrap();
        let canonical_value: Value = serde_json::from_str(&canonical).unwrap();
        assert_eq!(
            original_value, canonical_value,
            "{name}: round-trip must not add, drop or change a field"
        );
        // The canonical bytes are key-sorted: the top-level keys read
        // format_version before lane.
        assert!(
            canonical.find("\"format_version\"").unwrap() < canonical.find("\"lane\"").unwrap(),
            "{name}: canonical JSON sorts keys"
        );
    }
    // The Receiving lane as it is now: 13 steps, including the WP-0716
    // child step 1.3 verify-supplier.
    let receiving = example_lane("lane-wh-receiving");
    assert_eq!(
        step_count(&receiving),
        13,
        "the Receiving lane has 13 steps"
    );
    let canonical = process_lane_builder::canonical_lane_json(&receiving).unwrap();
    let value: Value = serde_json::from_str(&canonical).unwrap();
    let child = value["lane"]["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["steps"].as_array().unwrap().clone())
        .find(|s| s["id"] == "verify-supplier")
        .expect("the child step survives the round-trip");
    assert_eq!(child["child_lane"]["lane_id"], "lane-wh-supplier-check");
    assert_eq!(child["child_lane"]["anchor_record"], "$record.supplier_id");
    assert_eq!(step_count(&canonical), 13);
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn round_trip_renamed_step() {
    let original = example_lane("lane-wh-receiving");
    let mut value: Value = serde_json::from_str(&original).unwrap();
    let steps = value["lane"]["stages"][0]["steps"].as_array_mut().unwrap();
    steps[0]["name"] = json!("Verify the supplier, renamed");
    let renamed = value.to_string();
    let canonical = process_lane_builder::canonical_lane_json(&renamed).unwrap();
    let back: Value = serde_json::from_str(&canonical).unwrap();
    assert_eq!(
        back["lane"]["stages"][0]["steps"][0]["name"],
        "Verify the supplier, renamed"
    );
    assert_eq!(step_count(&canonical), 13);
    // Everything else is untouched.
    let original_value: Value = serde_json::from_str(&original).unwrap();
    let mut expected = original_value.clone();
    expected["lane"]["stages"][0]["steps"][0]["name"] = json!("Verify the supplier, renamed");
    assert_eq!(back, expected);
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn round_trip_changed_branch_target() {
    let original = example_lane("lane-wh-pickpack");
    let mut value: Value = serde_json::from_str(&original).unwrap();
    // The pick check's fail branch goes to the hold-review stage; retarget
    // it at the record-hold step in the same stage.
    let steps = value["lane"]["stages"][0]["steps"].as_array_mut().unwrap();
    let check = steps.iter_mut().find(|s| s["id"] == "pick-check").unwrap();
    let branches = check["branches"].as_array_mut().unwrap();
    let fail = branches.iter_mut().find(|b| b["on"] == "fail").unwrap();
    fail["target"] = json!({"stage": "hold-review", "step": "record-hold"});
    let retargeted = value.to_string();
    let canonical = process_lane_builder::canonical_lane_json(&retargeted).unwrap();
    let back: Value = serde_json::from_str(&canonical).unwrap();
    let steps = back["lane"]["stages"][0]["steps"].as_array().unwrap();
    let check = steps.iter().find(|s| s["id"] == "pick-check").unwrap();
    let fail = check["branches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["on"] == "fail")
        .unwrap();
    assert_eq!(fail["target"]["step"], "record-hold");
    assert_eq!(step_count(&canonical), 13);
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn canonical_rejects_invalid_lane_json() {
    assert!(process_lane_builder::canonical_lane_json("not json").is_err());
    // A lane whose branch target names no step fails validation.
    let mut value: Value = serde_json::from_str(&example_lane("lane-wh-count")).unwrap();
    let steps = value["lane"]["stages"][0]["steps"].as_array_mut().unwrap();
    // Count has no branches; break the lane id instead.
    let _ = steps;
    value["lane"]["id"] = json!("not-a-lane-id");
    assert!(process_lane_builder::canonical_lane_json(&value.to_string()).is_err());
}

// --------------------------------------------------------------------------
// AC-2: live publish checks on the unsaved draft.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-024 (WP-0718 AC-2)
#[test]
fn builder_check_returns_every_p01_to_p16_entry() {
    let (conn, ws, admin) = setup();
    let _seed = seed_warehouse(&conn, &ws, &admin);
    let report = process_lane_builder::builder_check(
        &conn,
        &ws,
        &example_lane("lane-wh-receiving"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(report.lane_key, "lane-wh-receiving");
    for n in 1..=16 {
        let id = format!("P-{n:02}");
        assert!(
            report.checks.iter().any(|c| c.id == id),
            "the builder check must report {id}: {:?}",
            report.checks.iter().map(|c| &c.id).collect::<Vec<_>>()
        );
    }
    // P-16 passes for the draft being edited: the definition IS the draft,
    // and saving it creates the draft version.
    assert_eq!(report_ids(&report, "P-16", "pass"), 1);
    assert!(report.refused, "the Receiving lane refuses on P-12");
    assert!(report_ids(&report, "P-12", "refuse") > 0);
}

// Covers: REQ-LANE-024 (WP-0718 AC-2)
#[test]
fn builder_check_matches_check_publish_on_the_same_stored_lane() {
    let (conn, ws, admin) = setup();
    let _seed = seed_warehouse(&conn, &ws, &admin);
    for name in [
        "lane-wh-receiving",
        "lane-wh-pickpack",
        "lane-wh-count",
        "lane-wh-replenish",
    ] {
        let definition = example_lane(name);
        import_lane(&conn, &ws, &admin, &definition);
        let stored = process_lane_publish::check_publish(&conn, &ws, name, Some(&admin)).unwrap();
        let builder =
            process_lane_builder::builder_check(&conn, &ws, &definition, Some(&admin)).unwrap();
        assert_eq!(
            stored.refused, builder.refused,
            "{name}: builder and publish must agree on refusal"
        );
        for n in 1..=16 {
            let id = format!("P-{n:02}");
            let stored_refuse = report_ids(&stored, &id, "refuse");
            let builder_refuse = report_ids(&builder, &id, "refuse");
            assert_eq!(
                stored_refuse, builder_refuse,
                "{name}: {id} refuse counts must match (stored {stored_refuse}, builder {builder_refuse})"
            );
            // Every P-12 refusal names the same provisional Action Type.
            for entry in stored
                .checks
                .iter()
                .filter(|c| c.id == id && c.status == "refuse")
            {
                assert!(
                    builder
                        .checks
                        .iter()
                        .any(|c| c.id == id && c.status == "refuse" && c.reason == entry.reason),
                    "{name}: builder must carry the same {id} reason: {:?}",
                    entry.reason
                );
            }
        }
    }
}

// Covers: REQ-LANE-024 (WP-0718 AC-2)
#[test]
fn builder_check_on_an_unsaved_edited_lane_reports_the_edit() {
    let (conn, ws, admin) = setup();
    let _seed = seed_warehouse(&conn, &ws, &admin);
    // Break the Receiving lane's child anchor: an undeclared record field.
    let mut value: Value = serde_json::from_str(&example_lane("lane-wh-receiving")).unwrap();
    let steps = value["lane"]["stages"][0]["steps"].as_array_mut().unwrap();
    let child = steps
        .iter_mut()
        .find(|s| s["id"] == "verify-supplier")
        .unwrap();
    child["child_lane"]["anchor_record"] = json!("$record.no_such_field");
    let report =
        process_lane_builder::builder_check(&conn, &ws, &value.to_string(), Some(&admin)).unwrap();
    assert!(report.refused);
    assert!(
        report.checks.iter().any(|c| c.id == "P-11"
            && c.status == "refuse"
            && c.reason.as_deref().unwrap_or("").contains("no_such_field")),
        "the edited anchor must fail P-11: {:?}",
        report.checks
    );
}

// --------------------------------------------------------------------------
// AC-4: reads/writes map and plumbing table.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-024 (WP-0718 AC-4)
#[test]
fn reads_writes_maps_damage_approved_between_approval_and_check() {
    let report =
        process_lane_builder::builder_reads_writes(&example_lane("lane-wh-pickpack")).unwrap();
    assert_eq!(report.lane_key, "lane-wh-pickpack");
    let approval = report
        .steps
        .iter()
        .find(|s| s.step_id == "review-short-pick")
        .expect("the damage approval is in the map");
    assert!(
        approval
            .run_data_writes
            .contains(&"damage_approved".to_string()),
        "the approval writes damage_approved: {:?}",
        approval.run_data_writes
    );
    assert!(
        approval
            .writes
            .iter()
            .any(|w| w.action_type == "run_data.set"),
        "the approval's run_data.set write is listed: {:?}",
        approval.writes
    );
    let check = report
        .steps
        .iter()
        .find(|s| s.step_id == "pick-check")
        .expect("the pick check is in the map");
    assert!(
        check
            .reads
            .iter()
            .any(|r| r.reference == "$run_data.damage_approved" && r.context == "check"),
        "the pick check reads damage_approved: {:?}",
        check.reads
    );
    // The lane-level plumbing table connects them.
    let row = report
        .plumbing
        .iter()
        .find(|r| r.field == "damage_approved")
        .expect("damage_approved has a plumbing row");
    let write_cell = row
        .steps
        .iter()
        .find(|c| c.step_id == "review-short-pick")
        .expect("the approval cell exists");
    assert_eq!(write_cell.access, "write");
    let read_cell = row
        .steps
        .iter()
        .find(|c| c.step_id == "pick-check")
        .expect("the check cell exists");
    assert_eq!(read_cell.access, "read");
}

// Covers: REQ-LANE-024 (WP-0718 AC-4)
#[test]
fn reads_writes_includes_the_child_lane_anchor_ref() {
    let report =
        process_lane_builder::builder_reads_writes(&example_lane("lane-wh-receiving")).unwrap();
    let step = report
        .steps
        .iter()
        .find(|s| s.step_id == "verify-supplier")
        .expect("the child step is in the map");
    assert!(
        step.reads
            .iter()
            .any(|r| r.reference == "$record.supplier_id" && r.context == "child lane anchor"),
        "the child step reads its anchor: {:?}",
        step.reads
    );
}

// Covers: REQ-LANE-024 (WP-0718 AC-4)
#[test]
fn reads_writes_includes_binds_to_and_write_inputs() {
    let report =
        process_lane_builder::builder_reads_writes(&example_lane("lane-wh-pickpack")).unwrap();
    let pick = report
        .steps
        .iter()
        .find(|s| s.step_id == "pick-remaining-lines")
        .expect("the pick step is in the map");
    assert!(
        pick.reads
            .iter()
            .any(|r| r.reference == "$record.lines[].qty_picked"),
        "a binds_to path is a read: {:?}",
        pick.reads
    );
    assert!(
        pick.run_data_writes.contains(&"damage_reason".to_string()),
        "a form field bound to run_data.damage_reason writes it: {:?}",
        pick.run_data_writes
    );
    // The damage_reason plumbing row shows the pick step as both.
    let row = report
        .plumbing
        .iter()
        .find(|r| r.field == "damage_reason")
        .expect("damage_reason has a plumbing row");
    let cell = row
        .steps
        .iter()
        .find(|c| c.step_id == "pick-remaining-lines")
        .expect("the pick cell exists");
    assert_eq!(cell.access, "both");
    // A record step's write inputs are reads.
    let post = report
        .steps
        .iter()
        .find(|s| s.step_id == "post-pick-moves")
        .expect("the post-pick step is in the map");
    assert!(
        post.reads.iter().any(|r| r.reference == "$item.qty_picked"),
        "write inputs are reads: {:?}",
        post.reads
    );
    assert!(
        post.writes
            .iter()
            .any(|w| w.action_type == "warehouse.post_stock_move"),
        "the StockMove write is listed: {:?}",
        post.writes
    );
}

// --------------------------------------------------------------------------
// AC-4: the Action-Type picker catalogue.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-024 (WP-0718 AC-4)
#[test]
fn builder_action_types_flags_provisional_ids_and_lists_adopted_ones() {
    let (conn, ws, admin) = setup();
    let _seed = seed_warehouse(&conn, &ws, &admin);
    for name in [
        "lane-wh-receiving",
        "lane-wh-pickpack",
        "lane-wh-count",
        "lane-wh-replenish",
    ] {
        import_lane(&conn, &ws, &admin, &example_lane(name));
    }
    let all = process_lane_builder::builder_action_types(&conn, &ws, None, Some(&admin)).unwrap();
    // Every provisional id the warehouse lanes use is listed, flagged,
    // and never publishable.
    for provisional in [
        "purchase_order.set_status",
        "warehouse.post_stock_move",
        "run_data.set",
        "sales_order.ship",
        "purchase_order.create",
    ] {
        let entry = all
            .iter()
            .find(|e| e.name == provisional)
            .unwrap_or_else(|| {
                panic!("provisional Action Type '{provisional}' must be listed: {all:?}")
            });
        assert!(entry.provisional, "{provisional} must be provisional");
        assert!(!entry.adopted, "{provisional} must not be adopted");
    }
    // Adopted types list as adopted, not provisional.
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let widget =
        process_lane_builder::builder_action_types(&conn, &ws, Some("widget"), Some(&admin))
            .unwrap();
    let adopted = widget
        .iter()
        .find(|e| e.name == "Set Tracking")
        .expect("the adopted Set Tracking type is listed");
    assert!(adopted.adopted);
    assert!(!adopted.provisional);
    assert_eq!(adopted.target_object_type, "widget");
    // Filtering by object hides the warehouse provisional ids.
    assert!(
        widget.iter().all(|e| e.target_object_type == "widget"),
        "an object filter scopes the catalogue: {widget:?}"
    );
}

// --------------------------------------------------------------------------
// AC-2/AC-4: the builder dry run.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-024 (WP-0718 AC-2)
#[test]
fn builder_dry_run_matches_stored_dry_run_and_posts_nothing() {
    let (conn, ws, admin) = setup();
    let seed = seed_warehouse(&conn, &ws, &admin);
    let definition = example_lane("lane-wh-receiving");
    import_lane(&conn, &ws, &admin, &definition);
    let before: Vec<usize> = [
        "purchase_order",
        "purchase_order_line",
        "sales_order",
        "cycle_count",
        "cycle_count_line",
        "reorder_rule",
    ]
    .iter()
    .map(|o| count_records(&conn, &ws, o))
    .collect();
    let po_before = get_values(&conn, &seed.po);

    let mut per_child = Map::new();
    for (i, line) in seed.po_lines.iter().enumerate() {
        let qty = ["40", "30"][i];
        per_child.insert(
            line.clone(),
            json!({
                "delivery_note_qty": qty,
                "qty_received": qty,
                "qty_rejected": "0",
                "putaway_bin": "A-01-03",
            }),
        );
    }
    let fixtures = fixtures(
        json!({"carrier": "FastFreight", "delivery_note_number": "DN-8812",
               "received_date": "2026-11-20", "staging_bin": "RCV-01"}),
        Value::Object(per_child),
        json!({}),
    );
    let stored = process_lane_publish::dry_run_lane(
        &conn,
        &ws,
        "lane-wh-receiving",
        &seed.po,
        &fixtures,
        Some("completed"),
        Some(&admin),
    )
    .unwrap();
    let builder = process_lane_builder::builder_dry_run(
        &conn,
        &ws,
        &definition,
        &seed.po,
        &fixtures,
        Some("completed"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(builder.lane_key, stored.lane_key);
    assert_eq!(builder.lane_version, stored.lane_version);
    assert!(builder.would_start);
    assert!(builder.completed);
    assert_eq!(builder.end_outcome.as_deref(), Some("completed"));
    assert!(builder.matches_expected);
    assert_eq!(builder.stages_visited, stored.stages_visited);
    assert_eq!(builder.writes.len(), stored.writes.len());
    assert_eq!(
        builder
            .writes
            .iter()
            .filter(|w| w.action_type == "warehouse.post_stock_move")
            .count(),
        4,
        "receipt + putaway per line"
    );
    assert!(builder.unevaluated.is_empty());
    assert!(builder.zero_quantity_flags.is_empty());

    let after: Vec<usize> = [
        "purchase_order",
        "purchase_order_line",
        "sales_order",
        "cycle_count",
        "cycle_count_line",
        "reorder_rule",
    ]
    .iter()
    .map(|o| count_records(&conn, &ws, o))
    .collect();
    assert_eq!(before, after, "a builder dry run creates no records");
    assert_eq!(
        get_values(&conn, &seed.po),
        po_before,
        "the PO is untouched"
    );
}

// Covers: REQ-LANE-024 (WP-0718 AC-2)
#[test]
fn builder_dry_run_with_a_missing_fixture_reports_unevaluated() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-needs-fixture", "name": "Needs fixture", "object_key": "widget",
            "version": 1, "status": "draft",
            "run_data": {"fields": [{"key": "carrier", "type": "string", "required": true}]},
            "stages": [{"id": "verify", "name": "Verify", "steps": [
                {"id": "carrier-check", "name": "Carrier check", "kind": "check",
                 "check": {"scope": "anchor", "message": "no carrier",
                           "condition": {"field": "$run_data.carrier", "operator": "equals", "value": "ACME"}},
                 "branches": [
                    {"on": "pass", "end": {"outcome": "done"}},
                    {"on": "fail", "end": {"outcome": "held"}}]}
            ]}]
        }
    })
    .to_string();
    let report = process_lane_builder::builder_dry_run(
        &conn,
        &ws,
        &lane,
        &_record,
        &fixtures(json!({}), json!({}), json!({})),
        None,
        Some(&admin),
    )
    .unwrap();
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].outcome, "unevaluated");
    assert!(!report.completed);
}

// --------------------------------------------------------------------------
// The builder save path.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn save_builder_lane_creates_a_new_lane_as_a_draft() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        1,
    );
    let saved =
        process_lane_builder::save_builder_lane(&conn, &ws, None, &lane, Some(&admin)).unwrap();
    assert_eq!(saved.lane_key, "lane-widget-clean");
    assert_eq!(saved.version, 1);
    assert_eq!(saved.status, "draft");
    let stored =
        process_lane_service::get_lane(&conn, &ws, "lane-widget-clean", Some(&admin)).unwrap();
    assert_eq!(stored.lane_key, "lane-widget-clean");
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn save_builder_lane_creates_a_new_version_of_an_existing_lane() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let v1 = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        1,
    );
    import_lane(&conn, &ws, &admin, &v1);
    process_lane_service::publish_lane(&conn, &ws, "lane-widget-clean", Some(&admin)).unwrap();
    // The builder edits the published lane: version 2, one step renamed.
    let mut value: Value = serde_json::from_str(&v1).unwrap();
    value["lane"]["version"] = json!(2);
    value["lane"]["stages"][0]["steps"][0]["name"] = json!("Enter, renamed");
    let saved = process_lane_builder::save_builder_lane(
        &conn,
        &ws,
        Some("lane-widget-clean"),
        &value.to_string(),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(saved.lane_key, "lane-widget-clean");
    assert_eq!(saved.version, 2);
    assert_eq!(saved.status, "draft");
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn save_builder_lane_refuses_a_new_lane_key_that_already_exists() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        1,
    );
    import_lane(&conn, &ws, &admin, &lane);
    assert!(
        process_lane_builder::save_builder_lane(&conn, &ws, None, &lane, Some(&admin)).is_err(),
        "saving as a brand-new lane must refuse when the key exists - use the existing-lane path"
    );
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn save_builder_lane_updates_an_existing_draft_in_place() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let v1 = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        1,
    );
    import_lane(&conn, &ws, &admin, &v1);
    // First edit: rename a step, version stays 1.
    let mut value: Value = serde_json::from_str(&v1).unwrap();
    value["lane"]["stages"][0]["steps"][0]["name"] = json!("Enter, renamed once");
    let saved = process_lane_builder::save_builder_lane(
        &conn,
        &ws,
        Some("lane-widget-clean"),
        &value.to_string(),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(saved.version, 1);
    assert_eq!(saved.status, "draft");
    // Second edit: rename again - still version 1, still one version.
    value["lane"]["stages"][0]["steps"][0]["name"] = json!("Enter, renamed twice");
    let saved = process_lane_builder::save_builder_lane(
        &conn,
        &ws,
        Some("lane-widget-clean"),
        &value.to_string(),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(saved.version, 1);
    assert_eq!(saved.status, "draft");
    let stored =
        process_lane_service::get_lane(&conn, &ws, "lane-widget-clean", Some(&admin)).unwrap();
    assert_eq!(
        stored.versions.len(),
        1,
        "iterating on a draft must not mint versions"
    );
    let stored_value: Value = serde_json::from_str(&stored.versions[0].definition_json).unwrap();
    assert_eq!(
        stored_value["lane"]["stages"][0]["steps"][0]["name"],
        json!("Enter, renamed twice"),
        "the stored draft must carry the latest edit"
    );
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn save_builder_lane_refuses_a_draft_save_naming_the_wrong_version() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let v1 = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        1,
    );
    import_lane(&conn, &ws, &admin, &v1);
    // The draft is version 1; a save naming version 2 must refuse rather
    // than mint a second version or silently renumber the draft.
    let mut value: Value = serde_json::from_str(&v1).unwrap();
    value["lane"]["version"] = json!(2);
    assert!(
        process_lane_builder::save_builder_lane(
            &conn,
            &ws,
            Some("lane-widget-clean"),
            &value.to_string(),
            Some(&admin),
        )
        .is_err(),
        "a draft save must name the draft's own version"
    );
}

// Covers: REQ-LANE-024 (WP-0718 AC-1)
#[test]
fn save_builder_lane_refuses_a_draft_save_for_a_different_lane_id() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let v1 = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        1,
    );
    import_lane(&conn, &ws, &admin, &v1);
    let mut value: Value = serde_json::from_str(&v1).unwrap();
    value["lane"]["id"] = json!("lane-widget-other");
    assert!(
        process_lane_builder::save_builder_lane(
            &conn,
            &ws,
            Some("lane-widget-clean"),
            &value.to_string(),
            Some(&admin),
        )
        .is_err(),
        "the definition's id must match the lane being saved"
    );
}

// --------------------------------------------------------------------------
// Builder gaps fix: gap 7 (self-loop at P-06) + catalogue dedupe.
// Covers: REQ-LANE-024 (WP-0718 AC-2)
// --------------------------------------------------------------------------

/// Covers: REQ-LANE-024 (WP-0718 AC-2)
#[test]
fn builder_check_refuses_a_self_loop_branch_at_p06() {
    let (conn, ws, admin) = setup();
    let mut value: Value = serde_json::from_str(&example_lane("lane-wh-receiving")).unwrap();
    // The discrepancy check's fail branch points at itself.
    value["lane"]["stages"][1]["steps"][1]["branches"][1]["target"] =
        json!({"stage": "record-receipt", "step": "discrepancy-check"});
    let report =
        process_lane_builder::builder_check(&conn, &ws, &value.to_string(), Some(&admin)).unwrap();
    assert!(report.refused, "a self-loop lane must refuse: {report:?}");
    let p06 = report
        .checks
        .iter()
        .find(|c| c.id == "P-06" && c.status == "refuse")
        .expect("P-06 must refuse the self-loop");
    assert!(
        p06.reason
            .as_deref()
            .unwrap_or("")
            .contains("targets the step itself"),
        "P-06 names the self-loop: {p06:?}"
    );
}

/// Covers: REQ-LANE-024 (WP-0718 AC-4)
#[test]
fn builder_action_types_lists_each_name_once_with_adopted_winning() {
    let (conn, ws, admin) = setup();
    let _seed = seed_warehouse(&conn, &ws, &admin);
    for name in [
        "lane-wh-receiving",
        "lane-wh-pickpack",
        "lane-wh-count",
        "lane-wh-replenish",
    ] {
        import_lane(&conn, &ws, &admin, &example_lane(name));
    }
    // `warehouse.post_stock_move` is used by three lanes on three
    // different objects, and `purchase_order.set_status` by two: the
    // unscoped catalogue lists each name once.
    let all = process_lane_builder::builder_action_types(&conn, &ws, None, Some(&admin)).unwrap();
    for name in ["warehouse.post_stock_move", "purchase_order.set_status"] {
        let found = all.iter().filter(|e| e.name == name).count();
        assert_eq!(found, 1, "'{name}' lists once, not per lane: {all:?}");
    }
    // An adopted entry beats a provisional one with the same name.
    let adopted = process_lane_builder::adopt_builder_action_type(
        &conn,
        &ws,
        "purchase_order.set_status",
        "purchase_order",
        Some(&admin),
    )
    .unwrap();
    assert_eq!(adopted.target_object_type, "purchase_order");
    let all = process_lane_builder::builder_action_types(&conn, &ws, None, Some(&admin)).unwrap();
    let entry = all
        .iter()
        .find(|e| e.name == "purchase_order.set_status")
        .expect("the adopted type is listed");
    assert!(entry.adopted, "the adopted entry wins: {entry:?}");
    assert!(!entry.provisional, "the adopted entry is not provisional");
    assert_eq!(entry.target_object_type, "purchase_order");
}
