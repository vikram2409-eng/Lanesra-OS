//! Phase 7, WP-0714: dry run, publish checks, lane-case framework and
//! Solution export/import (ADR-0023, REQ-LANE-021).
//!
//! A dry run walks a lane against a seed record and writes nothing; the
//! publish report names every failed check (P-01..P-16) instead of the
//! first one; lane cases run a lane for real inside a transaction that
//! rolls back; a Solution export carries every version plus its cases.
//!
//! Covers: REQ-LANE-021 (WP-0714 AC-1), REQ-LANE-021 (WP-0714 AC-2),
//! REQ-LANE-021 (WP-0714 AC-3)

use serde_json::{json, Map, Value};

use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    action_type_service, custom_field_service, custom_object_service, custom_record_service,
    industry_package_service, process_lane_publish, process_lane_service, relationship_service,
    workflow_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Publish Co".into(),
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

/// Adopts the `set_fields` Action Type the runner tests already prove the
/// runtime executes: writes its `val` parameter into `tracking`.
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

fn import_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, definition: &str) {
    process_lane_service::import_lane_definition(conn, ws, definition, Some(admin)).unwrap();
}

fn example_lane(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../program/spec/schemas/examples/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|_| panic!("demo lane example {name} exists"))
}

/// The warehouse objects, fields and `lines` relationships the four demo
/// lanes reference, so P-11 (no undeclared references) passes on the
/// verbatim lane JSONs and only P-12/P-13 can still refuse them.
struct WarehouseSeed {
    po: String,
    po_lines: Vec<String>,
    cc: String,
    cc_lines: Vec<String>,
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
    // The demo lanes read the warehouse status through `$record.status`;
    // the core's built-in status vocabulary (Active/Inactive/Archived)
    // cannot hold "OPEN"/"COUNTING", so a custom field of the same name
    // shadows it in the condition context (built-in first, custom values
    // overwrite) - the value the dry run and the start condition see.
    create_field(conn, ws, admin, "purchase_order", "status");
    create_field(conn, ws, admin, "purchase_order", "expected_date");
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
        let _def = relationship_service::create(
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
    // Variances -2, 0, +1, -2, 0: the two zero-variance lines are the
    // dry run's zero-quantity case (WP-0714 §3 assertion 4).
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

    // Link the lines to their anchors through the `lines` relationships.
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

    WarehouseSeed {
        po,
        po_lines,
        cc,
        cc_lines,
    }
}

fn report_refuses(report: &process_lane_publish::PublishReport, id: &str) -> bool {
    report
        .checks
        .iter()
        .any(|c| c.id == id && c.status == "refuse")
}

fn report_entries<'a>(
    report: &'a process_lane_publish::PublishReport,
    id: &str,
    status: &str,
) -> Vec<&'a process_lane_publish::PublishCheckEntry> {
    report
        .checks
        .iter()
        .filter(|c| c.id == id && c.status == status)
        .collect()
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

// --------------------------------------------------------------------------
// AC-1: a dry run walks the lane and writes nothing.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-021 (WP-0714 AC-1)
#[test]
fn dry_run_walks_receiving_on_po_seed_and_writes_nothing() {
    let (conn, ws, admin) = setup();
    let seed = seed_warehouse(&conn, &ws, &admin);
    import_lane(&conn, &ws, &admin, &example_lane("lane-wh-receiving"));
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
    let report = process_lane_publish::dry_run_lane(
        &conn,
        &ws,
        "lane-wh-receiving",
        &seed.po,
        &fixtures(
            json!({"carrier": "FastFreight", "delivery_note_number": "DN-8812",
                   "received_date": "2026-11-20", "staging_bin": "RCV-01"}),
            Value::Object(per_child),
            json!({}),
        ),
        Some("completed"),
        Some(&admin),
    )
    .unwrap();

    assert!(report.would_start, "the PO is OPEN, so the run starts");
    assert!(
        report.completed,
        "the walk reaches an end: {:?}",
        report.steps
    );
    assert_eq!(report.end_outcome.as_deref(), Some("completed"));
    assert!(report.matches_expected);
    assert_eq!(
        report.stages_visited,
        vec!["verify", "record-receipt", "stage-put-away", "confirm"],
        "the hold stage is never visited on the happy path"
    );
    let stock_moves: Vec<_> = report
        .writes
        .iter()
        .filter(|w| w.action_type == "warehouse.post_stock_move")
        .collect();
    assert_eq!(
        stock_moves.len(),
        4,
        "receipt + putaway per line: {:?}",
        report.writes
    );
    assert!(
        stock_moves.iter().all(
            |w| w.inputs.get("reference_id").and_then(|v| v.as_str()) == Some(seed.po.as_str())
        ),
        "every simulated move names the PO: {:?}",
        stock_moves
    );
    assert!(
        report
            .writes
            .iter()
            .any(|w| w.action_type == "purchase_order.close_receipt"),
        "the close write is simulated too"
    );
    assert!(report.zero_quantity_flags.is_empty());
    assert!(
        report.unevaluated.is_empty(),
        "every check evaluated: {:?}",
        report.unevaluated
    );

    // Nothing was written: no new records anywhere, the PO's own values
    // are untouched, and no line gained a received quantity.
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
    assert_eq!(before, after, "a dry run creates no records");
    assert_eq!(
        get_values(&conn, &seed.po),
        po_before,
        "the PO itself is untouched"
    );
    for line in &seed.po_lines {
        assert_eq!(
            get_values(&conn, line)
                .get("qty_received")
                .map(|v| v.as_str()),
            Some(Some("0")),
            "a line's received quantity is still 0"
        );
    }
}

// Covers: REQ-LANE-021 (WP-0714 AC-1)
#[test]
fn dry_run_count_flags_the_two_zero_variance_lines() {
    let (conn, ws, admin) = setup();
    let seed = seed_warehouse(&conn, &ws, &admin);
    import_lane(&conn, &ws, &admin, &example_lane("lane-wh-count"));

    let mut per_child = Map::new();
    for (i, line) in seed.cc_lines.iter().enumerate() {
        let counted = ["8", "20", "31", "38", "50"][i];
        per_child.insert(line.clone(), json!({"counted_qty": counted}));
    }
    let report = process_lane_publish::dry_run_lane(
        &conn,
        &ws,
        "lane-wh-count",
        &seed.cc,
        &fixtures(
            json!({}),
            Value::Object(per_child),
            json!({"review-variances": "approved"}),
        ),
        Some("completed"),
        Some(&admin),
    )
    .unwrap();

    assert!(report.would_start);
    assert!(
        report.completed,
        "the walk reaches an end: {:?}",
        report.steps
    );
    assert_eq!(
        report.zero_quantity_flags.len(),
        1,
        "one flagged step: {:?}",
        report.zero_quantity_flags
    );
    let flag = &report.zero_quantity_flags[0];
    assert_eq!(flag.step_id, "post-count-adjustments");
    assert_eq!(
        flag.skipped_children, 2,
        "CC-2026-001 has 2 zero-variance lines"
    );
    assert!(
        flag.message
            .contains("would skip 2 children with nothing to post"),
        "the flag reports in the spec's words: {}",
        flag.message
    );
    let adjustments: Vec<_> = report
        .writes
        .iter()
        .filter(|w| w.action_type == "warehouse.post_stock_move")
        .collect();
    assert_eq!(adjustments.len(), 5, "one simulated row per line");
    assert_eq!(adjustments.iter().filter(|w| w.zero_quantity).count(), 2);
}

// Covers: REQ-LANE-021 (WP-0714 AC-1)
#[test]
fn dry_run_check_with_a_missing_fixture_reports_unevaluated() {
    let (conn, ws, admin) = setup();
    create_object(&conn, &ws, &admin, "widget");
    create_field(&conn, &ws, &admin, "widget", "tracking");
    let record = create_record(&conn, &ws, &admin, "widget", "Widget one");
    import_lane(
        &conn,
        &ws,
        &admin,
        &json!({
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
        .to_string(),
    );
    let report = process_lane_publish::dry_run_lane(
        &conn,
        &ws,
        "lane-needs-fixture",
        &record,
        &fixtures(json!({}), json!({}), json!({})),
        None,
        Some(&admin),
    )
    .unwrap();
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].outcome, "unevaluated");
    assert!(
        report.checks[0]
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("needs fixture"),
        "the reason names the missing fixture: {:?}",
        report.checks[0].reason
    );
    assert!(
        report
            .unevaluated
            .iter()
            .any(|u| u.step_id == "carrier-check"),
        "the unevaluated list names the check"
    );
    assert!(
        !report.completed,
        "a walk that cannot evaluate its check does not claim an end"
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-1)
#[test]
fn dry_run_refuses_to_start_when_the_start_condition_fails() {
    let (conn, ws, admin) = setup();
    let seed = seed_warehouse(&conn, &ws, &admin);
    // Close the PO first: the start condition ($record.status == OPEN)
    // no longer holds.
    set_values(
        &conn,
        "purchase_order",
        &seed.po,
        &admin,
        &[("status", "RECEIVED")],
    );
    import_lane(&conn, &ws, &admin, &example_lane("lane-wh-receiving"));
    let report = process_lane_publish::dry_run_lane(
        &conn,
        &ws,
        "lane-wh-receiving",
        &seed.po,
        &fixtures(json!({}), json!({}), json!({})),
        None,
        Some(&admin),
    )
    .unwrap();
    assert!(!report.would_start);
    assert!(
        report
            .refusal_reason
            .as_deref()
            .unwrap_or("")
            .contains("would refuse to start"),
        "the refusal reports in the spec's words: {:?}",
        report.refusal_reason
    );
    assert!(
        report.steps.is_empty(),
        "no step runs when the lane refuses to start"
    );
}

// --------------------------------------------------------------------------
// AC-2: publish is refused with a named reason for each failed check.
// --------------------------------------------------------------------------

/// A minimal publishable lane on `widget`: one form step, one anchor
/// check (last in its stage), one record step writing through an adopted
/// Action Type. `writes` names the Action Type to adopt for the record
/// step; `check_field`/`check_value` shape the check condition.
fn widget_lane_json(id: &str, writes: Value, extra: Value) -> String {
    let mut lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": "Widget Lane", "object_key": "widget",
            "version": 1, "status": "draft",
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
    });
    // `extra` is a JSON merge patch for the lane object (replaces / start
    // condition / a mid-stage check), applied key by key.
    if let Some(patch) = extra.as_object() {
        let lane_obj = lane.get_mut("lane").unwrap().as_object_mut().unwrap();
        for (k, v) in patch {
            lane_obj.insert(k.clone(), v.clone());
        }
    }
    lane.to_string()
}

fn seed_widget_with_tracking_action(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    create_object(conn, ws, admin, "widget");
    create_field(conn, ws, admin, "widget", "tracking");
    adopt_set_tracking(conn, ws, admin);
    create_record(conn, ws, admin, "widget", "Widget one")
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn warehouse_lanes_refuse_publish_with_named_p12_reasons() {
    let (conn, ws, admin) = setup();
    let _seed = seed_warehouse(&conn, &ws, &admin);
    for (name, expected_types) in [
        (
            "lane-wh-receiving",
            vec![
                "purchase_order.set_status",
                "warehouse.post_stock_move",
                "purchase_order.close_receipt",
            ],
        ),
        (
            "lane-wh-pickpack",
            vec![
                "picking.set_status",
                "run_data.set",
                "warehouse.post_stock_move",
                "shipment.create",
                "sales_order.set_status",
                "sales_order.ship",
            ],
        ),
        (
            "lane-wh-count",
            vec!["cycle_count.set_status", "warehouse.post_stock_move"],
        ),
        (
            "lane-wh-replenish",
            vec!["purchase_order.create", "purchase_order.set_status"],
        ),
    ] {
        import_lane(&conn, &ws, &admin, &example_lane(name));
        let report = process_lane_publish::check_publish(&conn, &ws, name, Some(&admin)).unwrap();
        assert!(report.refused, "{name} must refuse publish");
        let p12 = report_entries(&report, "P-12", "refuse");
        assert!(
            !p12.is_empty(),
            "{name} must list P-12 refusals: {:?}",
            report.checks
        );
        for action_type in expected_types {
            assert!(
                p12.iter()
                    .any(|e| e.reason.as_deref().unwrap_or("").contains(action_type)),
                "{name} must name provisional Action Type '{action_type}': {:?}",
                p12.iter().map(|e| e.reason.clone()).collect::<Vec<_>>()
            );
        }
        // P-11 passes on the seeded workspace: the refusal is the
        // provisional catalog, not a dangling reference.
        assert!(
            report_entries(&report, "P-11", "refuse").is_empty(),
            "{name} must not fail P-11: {:?}",
            report.checks
        );
    }
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn synthetic_lane_with_only_adopted_writes_publishes() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-widget-clean",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({}),
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-widget-clean", Some(&admin)).unwrap();
    assert!(
        !report.refused,
        "a clean lane passes every check: {:?}",
        report.checks
    );
    assert!(report.checks.iter().all(|c| c.status != "refuse"));
    let published =
        process_lane_service::publish_lane(&conn, &ws, "lane-widget-clean", Some(&admin)).unwrap();
    assert_eq!(published.status, "published");
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p09_outcome_coverage_missing_fail_branch_refuses_and_full_coverage_passes() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    // A check with only a pass branch: validate_definition accepts it
    // (it still reaches the end), P-09 outcome coverage refuses it.
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-p09-missing", "name": "P09 missing", "object_key": "widget",
            "version": 1, "status": "draft",
            "run_data": {"fields": [{"key": "count", "type": "integer"}]},
            "stages": [
                {"id": "verify", "name": "Verify", "steps": [
                    {"id": "only-pass", "name": "Only pass", "kind": "check",
                     "check": {"scope": "anchor", "message": "x",
                               "condition": {"field": "$run_data.count", "operator": "greater_than", "value": 0}},
                     "branches": [{"on": "pass", "end": {"outcome": "done"}}]}
                ]},
                {"id": "post", "name": "Post", "steps": [
                    {"id": "post-tracking", "name": "Post tracking", "kind": "record", "role": "r",
                     "required": true,
                     "writes": [{"action_type": "Set Tracking", "inputs": {"val": "x"}}]}
                ]}
            ]
        }
    })
    .to_string();
    import_lane(&conn, &ws, &admin, &lane);
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p09-missing", Some(&admin)).unwrap();
    assert!(report.refused);
    let p09 = report_entries(&report, "P-09", "refuse");
    assert_eq!(p09.len(), 1, "exactly the missing fail branch: {:?}", p09);
    assert_eq!(p09[0].step.as_deref(), Some("only-pass"));
    assert!(
        p09[0]
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("has no fail branch"),
        "the reason names the missing outcome: {:?}",
        p09[0].reason
    );

    // The same lane with both branches passes P-09.
    let lane = lane.replace(
        r#""branches":[{"end":{"outcome":"done"},"on":"pass"}]"#,
        r#""branches":[{"end":{"outcome":"done"},"on":"pass"},{"end":{"outcome":"held"},"on":"fail"}]"#,
    );
    let lane = lane.replace("lane-p09-missing", "lane-p09-full");
    import_lane(&conn, &ws, &admin, &lane);
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p09-full", Some(&admin)).unwrap();
    assert!(
        report_entries(&report, "P-09", "refuse").is_empty(),
        "full outcome coverage passes P-09: {:?}",
        report.checks
    );

    // A branch with neither a target step nor an end outcome refuses
    // P-09, naming the branch and the step (WP-0714 §2, P-09). It cannot
    // be imported (the definition validator refuses it first, with its
    // own message), so this runs the report directly over the leniently
    // parsed definition - the path the Solution import uses in report
    // mode.
    let lane = widget_lane_json(
        "lane-p09-noexit",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "x"}}]),
        json!({}),
    )
    .replace(
        r#""branches":[{"on":"pass","target":{"stage":"post","step":"post-tracking"}},{"end":{"outcome":"held"},"on":"fail"}]"#,
        r#""branches":[{"on":"pass"},{"end":{"outcome":"held"},"on":"fail"}]"#,
    );
    let file: lanesra_core::models::process_lane::LaneDefinitionFile =
        serde_json::from_str(&lane).unwrap();
    let report =
        process_lane_publish::check_publish_definition(&conn, &ws, &file.lane, true, Some(1))
            .unwrap();
    assert!(report.refused);
    let noexit = report_entries(&report, "P-09", "refuse");
    assert_eq!(
        noexit.len(),
        1,
        "exactly the branch with no exit: {:?}",
        noexit
    );
    assert!(
        noexit[0]
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("has no exit"),
        "the reason names the missing exit: {:?}",
        noexit[0].reason
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p10_check_not_last_in_stage_warns_but_does_not_block_publish() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    // The check sits mid-stage: the step after it runs on the pass path.
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-p10", "name": "P10", "object_key": "widget",
            "version": 1, "status": "draft",
            "run_data": {"fields": [{"key": "count", "type": "integer"}]},
            "stages": [{"id": "verify", "name": "Verify", "steps": [
                {"id": "mid-check", "name": "Mid check", "kind": "check",
                 "check": {"scope": "anchor", "message": "x",
                           "condition": {"field": "$run_data.count", "operator": "greater_than", "value": 0}},
                 "branches": [
                    {"on": "pass", "target": {"stage": "verify", "step": "after-check"}},
                    {"on": "fail", "end": {"outcome": "held"}}]},
                {"id": "after-check", "name": "After check", "kind": "record", "role": "r",
                 "required": true,
                 "writes": [{"action_type": "Set Tracking", "inputs": {"val": "x"}}]}
            ]}]
        }
    })
    .to_string();
    import_lane(&conn, &ws, &admin, &lane);
    let report = process_lane_publish::check_publish(&conn, &ws, "lane-p10", Some(&admin)).unwrap();
    assert!(
        !report.refused,
        "a WARN never blocks publish: {:?}",
        report.checks
    );
    let warns = report_entries(&report, "P-10", "warn");
    assert_eq!(warns.len(), 1);
    assert_eq!(warns[0].step.as_deref(), Some("mid-check"));
    assert!(
        warns[0]
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("is not the last step of stage"),
        "the warning explains the consequence: {:?}",
        warns[0].reason
    );
    process_lane_service::publish_lane(&conn, &ws, "lane-p10", Some(&admin)).unwrap();
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p11_undeclared_references_refuse_with_the_field_named() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-p11",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.nope"}}]),
        json!({}),
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report = process_lane_publish::check_publish(&conn, &ws, "lane-p11", Some(&admin)).unwrap();
    assert!(report.refused);
    let p11 = report_entries(&report, "P-11", "refuse");
    assert!(
        p11.iter()
            .any(|e| e.reason.as_deref().unwrap_or("").contains("nope")),
        "P-11 names the undeclared field: {:?}",
        p11.iter().map(|e| e.reason.clone()).collect::<Vec<_>>()
    );

    // An unknown record field refuses the same way.
    let lane = widget_lane_json(
        "lane-p11-record",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$record.not_a_field"}}]),
        json!({}),
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p11-record", Some(&admin)).unwrap();
    assert!(report.refused);
    assert!(
        report_entries(&report, "P-11", "refuse").iter().any(|e| e
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("not_a_field")),
        "P-11 names the unknown record field: {:?}",
        report.checks
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p13_replaces_unknown_workflow_refuses_and_a_real_one_passes() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane_with_replaces = |id: &str, workflow_id: &str, action_index: i64| {
        widget_lane_json(
            id,
            json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
            json!({}),
        )
        .replace(
            r#""required":true,"role":"r","writes""#,
            &format!(
                r#""required":true,"replaces":[{{"workflow_id":"{workflow_id}","action_index":{action_index}}}],"role":"r","writes""#
            ),
        )
    };
    // Unknown workflow id.
    import_lane(
        &conn,
        &ws,
        &admin,
        &lane_with_replaces("lane-p13-unknown", "no-such-workflow", 0),
    );
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p13-unknown", Some(&admin)).unwrap();
    assert!(report.refused);
    assert!(
        report_entries(&report, "P-13", "refuse").iter().any(|e| e
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("unknown workflow 'no-such-workflow'")),
        "P-13 names the unknown workflow: {:?}",
        report.checks
    );

    // A real workflow with one action: index 0 passes, index 1 refuses.
    let workflow = workflow_service::create_rule(
        &conn,
        &ws,
        &WorkflowDefinitionInput {
            app_id: None,
            entity_type: "Company".into(),
            name: "Company -> Active Customer".into(),
            description: None,
            trigger_type: "status_changed".into(),
            trigger_status: Some("Active Customer".into()),
            trigger_field_key: None,
            trigger_field_source: "custom".into(),
            trigger_offset_days: 0,
            match_type: "all".into(),
            priority: 0,
            conditions: vec![],
            actions: vec![WorkflowActionInput {
                action_type: "add_notification".into(),
                params_json: json!({"message": "hi", "audience": "all_admins"}).to_string(),
            }],
        },
        Some(&admin),
    )
    .unwrap();
    import_lane(
        &conn,
        &ws,
        &admin,
        &lane_with_replaces("lane-p13-ok", &workflow.id, 0),
    );
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p13-ok", Some(&admin)).unwrap();
    assert!(
        report_entries(&report, "P-13", "refuse").is_empty(),
        "a real workflow + a real action index passes P-13: {:?}",
        report.checks
    );
    import_lane(
        &conn,
        &ws,
        &admin,
        &lane_with_replaces("lane-p13-bad-index", &workflow.id, 1),
    );
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p13-bad-index", Some(&admin))
            .unwrap();
    assert!(
        report_entries(&report, "P-13", "refuse").iter().any(|e| e
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("has no such action")),
        "P-13 names the missing action index: {:?}",
        report.checks
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p14_string_field_used_as_a_number_warns_and_an_integer_passes() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    // `carrier` is declared string but the check compares it numerically.
    let lane = widget_lane_json(
        "lane-p14",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({}),
    )
    .replace(
        r#""field":"$run_data.count","operator":"greater_than","value":0"#,
        r#""field":"$run_data.carrier","operator":"greater_than","value":0"#,
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report = process_lane_publish::check_publish(&conn, &ws, "lane-p14", Some(&admin)).unwrap();
    assert!(!report.refused, "P-14 is a WARN: {:?}", report.checks);
    let warns = report_entries(&report, "P-14", "warn");
    assert_eq!(warns.len(), 1);
    assert!(
        warns[0]
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("run-data field 'carrier' is 'string' but is used as 'number'"),
        "the warning names field, type and use: {:?}",
        warns[0].reason
    );

    // The integer `count` in the same comparison passes P-14.
    let lane = widget_lane_json(
        "lane-p14-ok",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({}),
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-p14-ok", Some(&admin)).unwrap();
    assert!(report_entries(&report, "P-14", "warn").is_empty());
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p15_start_condition_may_only_reference_declared_fields() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-p15",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({"start_condition": {"field": "$run_data.nope", "operator": "equals", "value": "x"}}),
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report = process_lane_publish::check_publish(&conn, &ws, "lane-p15", Some(&admin)).unwrap();
    assert!(report.refused);
    assert!(
        report_entries(&report, "P-15", "refuse").iter().any(|e| e
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("nope")),
        "P-15 names the undeclared start-condition field: {:?}",
        report.checks
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn publish_refusal_lists_every_failed_check_not_just_the_first() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    // One lane failing P-11 (bad $run_data), P-12 (provisional write) and
    // P-13 (unknown workflow) at once.
    let lane = widget_lane_json(
        "lane-everything",
        json!([{"action_type": "widget.not_adopted", "inputs": {"val": "$run_data.nope"}}]),
        json!({}),
    )
    .replace(
        r#""required":true,"role":"r","writes""#,
        r#""required":true,"replaces":[{"workflow_id":"no-such-workflow","action_index":0}],"role":"r","writes""#,
    );
    import_lane(&conn, &ws, &admin, &lane);
    let report =
        process_lane_publish::check_publish(&conn, &ws, "lane-everything", Some(&admin)).unwrap();
    assert!(report.refused);
    for id in ["P-11", "P-12", "P-13"] {
        assert!(
            report_refuses(&report, id),
            "the report refuses {id}: {:?}",
            report.checks
        );
    }
    let err = process_lane_service::publish_lane(&conn, &ws, "lane-everything", Some(&admin))
        .expect_err("publish must refuse");
    let message = err.to_string();
    for id in ["P-11", "P-12", "P-13"] {
        assert!(
            message.contains(id),
            "the publish error lists {id}: {message}"
        );
    }
    assert!(message.contains("nope") && message.contains("widget.not_adopted"));
    // No partial publish: the version is still a draft.
    let lane = process_lane_service::get_lane(&conn, &ws, "lane-everything", Some(&admin)).unwrap();
    assert_eq!(lane.versions[0].status, "draft");
}

// Covers: REQ-LANE-021 (WP-0714 AC-2)
#[test]
fn p16_publishing_with_no_draft_version_refuses() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-p16",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({}),
    );
    import_lane(&conn, &ws, &admin, &lane);
    process_lane_service::publish_lane(&conn, &ws, "lane-p16", Some(&admin)).unwrap();
    let report = process_lane_publish::check_publish(&conn, &ws, "lane-p16", Some(&admin)).unwrap();
    assert!(report.refused);
    assert!(
        report_entries(&report, "P-16", "refuse").iter().any(|e| e
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("has no draft version")),
        "P-16 names the missing draft: {:?}",
        report.checks
    );
    let err = process_lane_service::publish_lane(&conn, &ws, "lane-p16", Some(&admin))
        .expect_err("a second publish must refuse");
    assert!(
        err.to_string().contains("P-16"),
        "the error carries the check id: {err}"
    );
}

// --------------------------------------------------------------------------
// AC-3: a lane exports and imports in a Solution with its versions.
// --------------------------------------------------------------------------

fn case_json(case: &str, lane: &str, version: i64, anchor: &str, assert: Value) -> String {
    json!({
        "case": case,
        "lane": lane,
        "version": version,
        "anchor": {"object": "widget", "id": anchor},
        "fixtures": {"run_data": {"carrier": "ACME"}},
        "assert": assert,
    })
    .to_string()
}

// Covers: REQ-LANE-021 (WP-0714 AC-3)
#[test]
fn export_import_round_trip_preserves_versions_cases_and_reports_checks() {
    let (conn, ws, admin) = setup();
    let record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane_v1 = widget_lane_json(
        "lane-export",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({}),
    );
    import_lane(&conn, &ws, &admin, &lane_v1);
    process_lane_service::publish_lane(&conn, &ws, "lane-export", Some(&admin)).unwrap();
    process_lane_publish::store_lane_case(
        &conn,
        &ws,
        "lane-export",
        1,
        &case_json(
            "widget-happy-v1",
            "lane-export",
            1,
            &record,
            json!([{"run.ends_with": "completed"}]),
        ),
        Some(&admin),
    )
    .unwrap();
    // A second, draft version with its own case.
    let lane_v2 = lane_v1.replace(r#""version":1"#, r#""version":2"#);
    process_lane_service::create_version(&conn, &ws, "lane-export", &lane_v2, Some(&admin))
        .unwrap();
    process_lane_publish::store_lane_case(
        &conn,
        &ws,
        "lane-export",
        2,
        &case_json(
            "widget-happy-v2",
            "lane-export",
            2,
            &record,
            json!([{"run.ends_with": "completed"}]),
        ),
        Some(&admin),
    )
    .unwrap();

    // Export: every version travels, with its cases and dependencies.
    let exported =
        industry_package_service::export_local_workspace(&conn, &ws, Some(&admin)).unwrap();
    let manifest: Value = serde_json::from_str(&exported).unwrap();
    let lanes = manifest["process_lanes"]
        .as_array()
        .expect("process_lanes in the manifest");
    let entry = lanes
        .iter()
        .find(|e| e["lane"]["id"] == "lane-export")
        .expect("the lane exports");
    let versions = entry["lane"]["versions"].as_array().unwrap();
    assert_eq!(
        versions.len(),
        2,
        "all versions export, not just the published one"
    );
    assert_eq!(versions[0]["version"], 1);
    assert_eq!(versions[0]["status"], "published");
    assert_eq!(versions[1]["version"], 2);
    assert_eq!(versions[1]["status"], "draft");
    assert_eq!(
        versions[0]["cases"].as_array().unwrap().len(),
        1,
        "v1 carries its case"
    );
    assert_eq!(
        versions[1]["cases"].as_array().unwrap().len(),
        1,
        "v2 carries its case"
    );
    let deps = &entry["dependencies"];
    assert!(deps["objects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o == "widget"));
    assert!(deps["action_types"]
        .as_array()
        .unwrap()
        .iter()
        .any(|a| a == "Set Tracking"));

    // Import into a fresh workspace: a new draft version with its cases.
    // (The Action Type is not part of the manifest, so the imported lane
    // reports P-12 provisional in report mode - the import itself is not
    // refused.)
    let (conn2, ws2, admin2) = setup();
    let package = industry_package_service::import_package(
        &conn2,
        &ws2,
        &lanesra_core::models::industry_package::ImportPackageInput {
            manifest_json: exported.clone(),
        },
        Some(&admin2),
    )
    .unwrap();
    industry_package_service::install(&conn2, &ws2, &package.id, Some(&admin2)).unwrap();
    let imported =
        process_lane_service::get_lane(&conn2, &ws2, "lane-export", Some(&admin2)).unwrap();
    assert_eq!(imported.versions.len(), 2, "both versions arrive");
    assert!(
        imported.versions.iter().all(|v| v.status == "draft"),
        "an imported version is a draft - a published version from another workspace has no compiled graph here: {:?}",
        imported.versions.iter().map(|v| (v.version, v.status.clone())).collect::<Vec<_>>()
    );
    for version in [1, 2] {
        let cases = process_lane_publish::list_lane_cases(
            &conn2,
            &ws2,
            "lane-export",
            Some(version),
            Some(&admin2),
        )
        .unwrap();
        assert_eq!(cases.len(), 1, "v{version} keeps its case");
    }
    let report =
        process_lane_publish::check_publish(&conn2, &ws2, "lane-export", Some(&admin2)).unwrap();
    assert!(report.refused, "the import reports its P-12 provisional");
    assert!(
        report_entries(&report, "P-12", "refuse").iter().any(|e| e
            .reason
            .as_deref()
            .unwrap_or("")
            .contains("Set Tracking")),
        "report mode lists the missing Action Type: {:?}",
        report.checks
    );

    // Re-importing into the source workspace never overwrites a published
    // version: both versions already exist there, so both are skipped.
    let manifest_parsed: lanesra_core::models::industry_package::IndustryPackageManifest =
        serde_json::from_str(&exported).unwrap();
    let entries =
        industry_package_service::install_process_lanes(&conn, &ws, &manifest_parsed, Some(&admin))
            .unwrap();
    assert!(
        entries.iter().all(|e| e.status == "skipped"),
        "existing versions are skipped, not overwritten: {:?}",
        entries
            .iter()
            .map(|e| (e.version, e.status.clone(), e.reason.clone()))
            .collect::<Vec<_>>()
    );
    let lane = process_lane_service::get_lane(&conn, &ws, "lane-export", Some(&admin)).unwrap();
    assert_eq!(
        lane.versions
            .iter()
            .find(|v| v.version == 1)
            .unwrap()
            .status,
        "published",
        "the published v1 is untouched by the re-import"
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-3)
#[test]
fn legacy_definition_only_manifest_still_imports() {
    let (conn, ws, admin) = setup();
    let _record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = widget_lane_json(
        "lane-legacy",
        json!([{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]),
        json!({}),
    );
    // A WP-0703-era manifest entry: `{definition: {...}}` only.
    let manifest_json = json!({
        "format_version": 1,
        "package_id": "local.legacy",
        "name": "Legacy",
        "industry": "general",
        "version": "1.0.0",
        "min_lanesra_version": "0.0.0",
        "process_lanes": [{"definition": serde_json::from_str::<Value>(&lane).unwrap()}],
    })
    .to_string();
    let manifest: lanesra_core::models::industry_package::IndustryPackageManifest =
        serde_json::from_str(&manifest_json).unwrap();
    let entries =
        industry_package_service::install_process_lanes(&conn, &ws, &manifest, Some(&admin))
            .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].status, "imported");
    let imported = process_lane_service::get_lane(&conn, &ws, "lane-legacy", Some(&admin)).unwrap();
    assert_eq!(imported.versions.len(), 1);
    assert_eq!(imported.versions[0].status, "draft");
}

// --------------------------------------------------------------------------
// Lane cases: run for real, in a transaction that rolls back.
// --------------------------------------------------------------------------

/// A lane the case runner can drive end to end: a form step, an anchor
/// check on the record's own status, a record step through the adopted
/// `Set Tracking` Action Type, and a finishing form step that ends the
/// run as `received`.
fn case_lane_json(id: &str) -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": "Case Lane", "object_key": "widget",
            "version": 1, "status": "draft",
            "run_data": {"fields": [{"key": "carrier", "type": "string", "required": true}]},
            "stages": [
                {"id": "capture", "name": "Capture", "steps": [
                    {"id": "enter", "name": "Enter", "kind": "form", "role": "r",
                     "required": true,
                     "fields": [{"key": "carrier", "label": "Carrier", "type": "string",
                                 "binds_to": "run_data.carrier", "required": true}]},
                    {"id": "status-check", "name": "Status check", "kind": "check",
                     "check": {"scope": "anchor", "message": "must be active",
                               "condition": {"field": "$record.status", "operator": "equals", "value": "Active"}},
                     "branches": [
                        {"on": "pass", "target": {"stage": "post", "step": "post-tracking"}},
                        {"on": "fail", "end": {"outcome": "held"}}]}
                ]},
                {"id": "post", "name": "Post", "steps": [
                    {"id": "post-tracking", "name": "Post tracking", "kind": "record", "role": "r",
                     "required": true,
                     "writes": [{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]}
                ]},
                {"id": "done", "name": "Done", "steps": [
                    {"id": "finish", "name": "Finish", "kind": "form", "role": "r",
                     "required": true, "fields": [], "ends_run": {"outcome": "received"}}
                ]}
            ]
        }
    })
    .to_string()
}

fn happy_case(id: &str, record: &str) -> String {
    json!({
        "case": "widget-happy",
        "lane": id,
        "version": 1,
        "anchor": {"object": "widget", "id": record},
        "fixtures": {"run_data": {"carrier": "ACME"}},
        "assert": [
            {"run.ends_with": "received"},
            {"step.completed": "enter"},
            {"step.completed": "post-tracking"},
            {"check.passed": "status-check"},
            {"write.posted": {"action_type": "Set Tracking", "count": 1}},
            {"record.equals": {"object": "widget", "id": record, "field": "tracking", "value": "ACME"}},
            {"run_data.equals": {"field": "carrier", "value": "ACME"}}
        ],
    })
    .to_string()
}

// Covers: REQ-LANE-021 (WP-0714 AC-1), REQ-LANE-021 (WP-0714 AC-3)
#[tokio::test]
async fn lane_case_runs_for_real_and_rolls_back() {
    let (conn, ws, admin) = setup();
    let record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    import_lane(&conn, &ws, &admin, &case_lane_json("lane-widget-case"));
    process_lane_service::publish_lane(&conn, &ws, "lane-widget-case", Some(&admin)).unwrap();
    let case = happy_case("lane-widget-case", &record);
    process_lane_publish::store_lane_case(&conn, &ws, "lane-widget-case", 1, &case, Some(&admin))
        .unwrap();

    let result =
        process_lane_publish::run_lane_case(&conn, &ws, &master_key(), &case, Some(&admin))
            .await
            .unwrap();
    assert!(
        result.passed,
        "the happy-path case passes: {:?}",
        result.failures
    );
    assert!(result.failures.is_empty());
    assert_eq!(result.end_outcome.as_deref(), Some("received"));

    // The rollback left no trace: no run on the record, and the write the
    // run posted is gone - the record's tracking is still unset.
    let runs =
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &record).unwrap();
    assert!(runs.runs.is_empty(), "no run survives a case");
    assert_eq!(
        get_values(&conn, &record).get("tracking"),
        None,
        "the posted write was rolled back with the run"
    );
}

// Covers: REQ-LANE-021 (WP-0714 AC-1), REQ-LANE-021 (WP-0714 AC-3)
#[tokio::test]
async fn lane_case_reports_expected_vs_actual_on_a_failing_assertion() {
    let (conn, ws, admin) = setup();
    let record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    import_lane(&conn, &ws, &admin, &case_lane_json("lane-widget-case"));
    process_lane_service::publish_lane(&conn, &ws, "lane-widget-case", Some(&admin)).unwrap();
    let mut case: Value = serde_json::from_str(&happy_case("lane-widget-case", &record)).unwrap();
    case["assert"] = json!([
        {"record.equals": {"object": "widget", "id": record, "field": "tracking", "value": "WRONG"}},
    ]);
    let result = process_lane_publish::run_lane_case(
        &conn,
        &ws,
        &master_key(),
        &case.to_string(),
        Some(&admin),
    )
    .await
    .unwrap();
    assert!(!result.passed);
    assert_eq!(result.failures.len(), 1);
    assert!(
        result.failures[0].contains("WRONG") && result.failures[0].contains("ACME"),
        "a failing assertion reports expected vs actual: {}",
        result.failures[0]
    );
    // Even a failing case rolls back.
    let runs =
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &record).unwrap();
    assert!(runs.runs.is_empty());
}

// Covers: REQ-LANE-021 (WP-0714 AC-3)
#[tokio::test]
async fn lane_case_fails_with_a_named_reason_when_an_approval_has_no_fixture() {
    let (conn, ws, admin) = setup();
    let record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-approval-case", "name": "Approval Case", "object_key": "widget",
            "version": 1, "status": "draft",
            "run_data": {"fields": [{"key": "carrier", "type": "string", "required": true}]},
            "stages": [
                {"id": "decide", "name": "Decide", "steps": [
                    {"id": "decide", "name": "Decide", "kind": "approval", "role": "r",
                     "required": true,
                     "writes": [{"action_type": "Set Tracking", "inputs": {"val": "ACME"}}],
                     "branches": [
                        {"on": "approved", "target": {"stage": "post", "step": "post-tracking"}},
                        {"on": "rejected", "end": {"outcome": "held"}}]}
                ]},
                {"id": "post", "name": "Post", "steps": [
                    {"id": "post-tracking", "name": "Post tracking", "kind": "record", "role": "r",
                     "required": true,
                     "writes": [{"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}]}
                ]}
            ]
        }
    })
    .to_string();
    import_lane(&conn, &ws, &admin, &lane);
    process_lane_service::publish_lane(&conn, &ws, "lane-approval-case", Some(&admin)).unwrap();
    // No `approvals` fixture at all: the case must fail, naming the step.
    let case = json!({
        "case": "approval-without-fixture",
        "lane": "lane-approval-case",
        "version": 1,
        "anchor": {"object": "widget", "id": record},
        "fixtures": {"run_data": {"carrier": "ACME"}},
        "assert": [{"run.ends_with": "received"}],
    })
    .to_string();
    let result =
        process_lane_publish::run_lane_case(&conn, &ws, &master_key(), &case, Some(&admin))
            .await
            .unwrap();
    assert!(!result.passed);
    assert!(
        result
            .failures
            .iter()
            .any(|f| f.contains("decide") && f.contains("without a fixture decision")),
        "the failure names the approval step: {:?}",
        result.failures
    );
    let runs =
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &record).unwrap();
    assert!(runs.runs.is_empty(), "the failed case still rolls back");
}

// Covers: REQ-LANE-021 (WP-0714 AC-3)
#[tokio::test]
async fn unified_test_run_discovers_and_runs_stored_cases() {
    let (conn, ws, admin) = setup();
    let record = seed_widget_with_tracking_action(&conn, &ws, &admin);
    import_lane(&conn, &ws, &admin, &case_lane_json("lane-widget-case"));
    process_lane_service::publish_lane(&conn, &ws, "lane-widget-case", Some(&admin)).unwrap();
    let case = happy_case("lane-widget-case", &record);
    process_lane_publish::store_lane_case(&conn, &ws, "lane-widget-case", 1, &case, Some(&admin))
        .unwrap();

    let found = process_lane_publish::discover_lane_cases(&conn, &ws).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "widget-happy");
    let results = process_lane_publish::run_all_lane_cases(&conn, &ws, &master_key(), Some(&admin))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        results[0].passed,
        "the discovered case passes: {:?}",
        results[0].failures
    );
}
