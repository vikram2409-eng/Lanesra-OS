//! AI Agent Platform v2, Phase 7: describe a process - the Admin Assistant
//! drafts a lane (WP-0715, extended to a full lane by WP-0723, REQ-LANE-022).
//!
//! Every test goes through the scripted provider (WP-0005), so each model
//! reply is scripted and the request the service made is captured and
//! checked. The draft is always a draft and always valid against the lane
//! schema (AC-1); the assistant cannot publish a lane (AC-2); the prompt
//! comes from the catalog (AC-3).
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::process_lane::parse_definition;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::TokenUsage;
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_service, assistant_draft_lane_service, custom_field_service, custom_object_service,
    process_lane_builder, process_lane_service, workspace_service,
};
use serde_json::json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Draft Lane Test Co".into(),
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

/// A named settings row that dispatches to the scripted provider. The
/// settings API cannot select `scripted` (WP-0005 AC-3), so the row is
/// created as another provider and switched underneath.
fn use_scripted_workspace_default(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    ai_service::save_settings(
        conn,
        ws,
        &master_key(),
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: None,
            model: "default-model".into(),
            api_key: Some("not-a-real-key".into()),
        },
        Some(admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_settings SET provider = 'scripted' WHERE workspace_id = ?1",
        [ws],
    )
    .unwrap();
}

fn text_reply(text: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: text.into(),
        usage: TokenUsage {
            input_tokens: 50,
            output_tokens: 200,
        },
    }
}

/// The lane a model would draft for "Receive a purchase order at the
/// Toronto warehouse" under the WP-0715 narrow contract: form steps for
/// the person's work, one check step where the description states
/// something to verify. The WP-0723 full-lane draft is
/// `drafted_full_lane_json` below.
fn drafted_lane_json() -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-draft-receive-purchase-order",
            "name": "Receive a purchase order",
            "description": "Receive a purchase order at the Toronto warehouse: verify the delivery, record what arrived, stage the goods and confirm the order is received.",
            "object_key": "purchase_order",
            "version": 1,
            "status": "draft",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true, "description": "Carrier who delivered the shipment."},
                {"key": "delivery_note_number", "type": "string", "required": true, "description": "Delivery-note number from the carrier's paperwork."},
                {"key": "received_date", "type": "date", "required": true, "description": "Date the shipment arrived."}
            ]},
            "start_condition": {"field": "$record.status", "operator": "equals", "value": "OPEN"},
            "stages": [
                {"id": "verify", "name": "Verify", "steps": [
                    {"id": "confirm-arrival", "name": "Confirm shipment arrived", "kind": "form", "role": "role_wh_receiver", "required": true,
                     "description": "Confirm the truck and delivery: carrier, delivery-note number, received date.",
                     "fields": [
                        {"key": "carrier", "label": "Carrier", "type": "string", "required": true, "binds_to": "run_data.carrier"},
                        {"key": "delivery_note_number", "label": "Delivery-note number", "type": "string", "required": true, "binds_to": "run_data.delivery_note_number"},
                        {"key": "received_date", "label": "Received date", "type": "date", "required": true, "binds_to": "run_data.received_date"}
                     ],
                     "validation": {"condition": {"field": "$run_data.received_date", "operator": "equals", "compare_field": "$record.expected_date"}, "message": "Received date must match the PO's expected date."}}
                ]},
                {"id": "record-receipt", "name": "Record receipt", "steps": [
                    {"id": "quantities-match", "name": "Quantities match", "kind": "check", "required": true,
                     "description": "Every line fully received.",
                     "check": {"scope": "anchor", "message": "A line is short.", "condition": {"field": "$record.status", "operator": "equals", "value": "OPEN"}},
                     "branches": [
                        {"on": "pass", "end": {"outcome": "done"}},
                        {"on": "fail", "end": {"outcome": "held"}}
                     ]}
                ]}
            ]
        }
    })
    .to_string()
}

const DESCRIPTION: &str = "Receive a purchase order at the Toronto warehouse: verify the delivery, record what arrived, stage the goods and confirm the order is received.";

/// Covers: REQ-LANE-022 (WP-0715 AC-1, AC-3)
#[tokio::test]
async fn ac1_draft_is_valid_and_stays_a_draft() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text_reply(&drafted_lane_json())]);

    let lane = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap();

    // AC-1: the result is always a draft - one version, version 1, status
    // draft, no compiled graph (nothing published).
    assert_eq!(lane.lane_key, "lane-draft-receive-purchase-order");
    assert_eq!(lane.name, "Receive a purchase order");
    assert_eq!(lane.object_key, "purchase_order");
    assert_eq!(lane.versions.len(), 1);
    assert_eq!(lane.versions[0].version, 1);
    assert_eq!(lane.versions[0].status, "draft");
    assert!(lane.versions[0].graph_id.is_none());
    // AC-1: the stored definition is valid against the schema - the same
    // parse-and-validate the core runs on every import.
    let stored = parse_definition(&lane.versions[0].definition_json).unwrap();
    assert_eq!(stored.lane.stages.len(), 2);
    assert_eq!(stored.lane.stages[0].steps.len(), 1);
    assert_eq!(stored.lane.stages[1].steps.len(), 1);
    assert_eq!(stored.lane.stages[1].steps[0].kind, "check");

    // AC-3: the model was called exactly once, with the catalog prompt -
    // the description and the object the person asked about.
    let captured = scripted_provider::take_captured();
    assert_eq!(captured.len(), 1);
    assert!(captured[0].system_prompt.contains(DESCRIPTION));
    assert!(captured[0].system_prompt.contains("purchase_order"));
    assert!(captured[0]
        .system_prompt
        .contains("status is draft, always"));
    assert!(captured[0].options.tools_off);
}

/// Covers: REQ-LANE-022 (WP-0715 AC-2)
#[tokio::test]
async fn ac2_model_reply_claiming_published_is_stored_as_draft() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    scripted_provider::reset();
    // The model disobeys the prompt: it claims the draft is already
    // published, version 7, of a newer format version.
    let mut reply: serde_json::Value = serde_json::from_str(&drafted_lane_json()).unwrap();
    reply["format_version"] = json!("2.0.0");
    reply["lane"]["status"] = json!("published");
    reply["lane"]["version"] = json!(7);
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let lane = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap();

    // AC-2: the assistant cannot publish a lane - the stored version is a
    // first draft whatever the model claimed, and the stored definition
    // says so too.
    assert_eq!(lane.versions[0].version, 1);
    assert_eq!(lane.versions[0].status, "draft");
    assert!(lane.versions[0].graph_id.is_none());
    let stored: serde_json::Value =
        serde_json::from_str(&lane.versions[0].definition_json).unwrap();
    assert_eq!(stored["format_version"], "1.0.0");
    assert_eq!(stored["lane"]["version"], 1);
    assert_eq!(stored["lane"]["status"], "draft");
}

/// Covers: REQ-LANE-022 (WP-0715 AC-1)
#[tokio::test]
async fn ac1_non_json_reply_is_refused_and_nothing_is_stored() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text_reply(
        "Sure - here is the receiving process in words. First verify the delivery, then record what arrived.",
    )]);

    let err = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("not a JSON lane definition"));

    // Nothing was stored: a refused draft leaves no lane behind.
    let lanes = process_lane_service::list_lanes(&conn, &ws, Some(&admin)).unwrap();
    assert!(lanes.is_empty());
}

/// Covers: REQ-LANE-022 (WP-0715 AC-1)
#[tokio::test]
async fn ac1_invalid_lane_reply_is_refused_and_nothing_is_stored() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    scripted_provider::reset();
    // A lane id that does not start with lane- fails schema validation.
    let mut reply: serde_json::Value = serde_json::from_str(&drafted_lane_json()).unwrap();
    reply["lane"]["id"] = json!("receive-purchase-order");
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let err = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("lane id must start with 'lane-'"));

    let lanes = process_lane_service::list_lanes(&conn, &ws, Some(&admin)).unwrap();
    assert!(lanes.is_empty());
}

/// Covers: REQ-LANE-022 (WP-0715 AC-1, AC-4)
#[tokio::test]
async fn ac4_empty_description_is_refused_before_any_model_call() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    scripted_provider::reset();

    for bad in ["", "   "] {
        let err = assistant_draft_lane_service::draft_lane(
            &conn,
            &ws,
            &master_key(),
            Some(&admin),
            bad,
            "purchase_order",
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("describe the process"));
    }
    let err = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        DESCRIPTION,
        "   ",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("the object it runs on"));

    // No model call was made for any of the three refusals.
    assert!(scripted_provider::take_captured().is_empty());
}

/// Covers: REQ-LANE-022 (WP-0715 AC-3, WP-0723 AC-1)
#[test]
fn ac3_prompt_comes_from_the_catalog() {
    let (text, used) = lanesra_core::prompts::render_active(
        "assistant/draft-lane",
        &[
            ("description", DESCRIPTION),
            ("object_key", "purchase_order"),
        ],
    )
    .unwrap();
    assert_eq!(used.version, 2);
    assert!(text.contains(DESCRIPTION));
    assert!(text.contains("purchase_order"));
    assert!(text.contains("Reply with one JSON object and nothing else"));
    assert!(text.contains("status is draft, always"));
    assert!(text.contains("purchase_order.set_status"));
    assert!(text.contains("Never pick an agent the description does not name"));
}

/// The lane a model would draft for the WP-0723 full-lane description:
/// a form step, a check step, a record step with the adopted set_status
/// write, an approval step with the decision write and both branches,
/// and an agent step for the agent the description names. Every write
/// uses the adopted `purchase_order.set_status` Action Type, and every
/// agent is named - the AC-1 happy path.
fn drafted_full_lane_json() -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-draft-receive-purchase-order",
            "name": "Receive a purchase order",
            "description": "Receive a purchase order at the Toronto warehouse: verify the delivery, record what arrived, get the Warehouse Manager's approval for a short receipt, have the Receiving Agent propose the put-away bin, and close the order as RECEIVED.",
            "object_key": "purchase_order",
            "version": 1,
            "status": "draft",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true, "description": "Carrier who delivered the shipment."}
            ]},
            "stages": [
                {"id": "verify", "name": "Verify", "steps": [
                    {"id": "confirm-arrival", "name": "Confirm shipment arrived", "kind": "form", "role": "role_wh_receiver", "required": true,
                     "description": "Confirm the truck and delivery: carrier.",
                     "fields": [
                        {"key": "carrier", "label": "Carrier", "type": "string", "required": true, "binds_to": "run_data.carrier"}
                     ]}
                ]},
                {"id": "record-receipt", "name": "Record receipt", "steps": [
                    {"id": "quantities-match", "name": "Quantities match", "kind": "check", "required": true,
                     "description": "Every line fully received.",
                     "check": {"scope": "anchor", "message": "A line is short.", "condition": {"field": "$record.status", "operator": "equals", "value": "OPEN"}},
                     "branches": [
                        {"on": "pass", "target": {"stage": "confirm", "step": "close-po"}},
                        {"on": "fail", "target": {"stage": "hold-review", "step": "review-discrepancy"}}
                     ]}
                ]},
                {"id": "hold-review", "name": "Hold & review", "steps": [
                    {"id": "review-discrepancy", "name": "Review discrepancy", "kind": "approval", "role": "role_wh_warehouse_manager", "required": true,
                     "description": "The Warehouse Manager reviews a short receipt. Approve: the order is marked PARTIALLY_RECEIVED and the run continues to put-away. Reject: the run ends held.",
                     "writes": [{"action_type": "purchase_order.set_status", "inputs": {"status": "PARTIALLY_RECEIVED"}}],
                     "branches": [
                        {"on": "approved", "target": {"stage": "confirm", "step": "suggest-bin"}},
                        {"on": "rejected", "end": {"outcome": "held"}}
                     ]}
                ]},
                {"id": "confirm", "name": "Confirm", "steps": [
                    {"id": "suggest-bin", "name": "Suggest a put-away bin", "kind": "agent", "role": "role_wh_receiver", "required": true,
                     "description": "The Receiving Agent proposes the put-away bin.",
                     "assistance": {"autonomy": "propose", "agent": "wh-agt-receiving"}},
                    {"id": "close-po", "name": "Close the PO", "kind": "record", "role": "role_wh_receiver", "required": true,
                     "description": "Close the PO as RECEIVED.",
                     "writes": [{"action_type": "purchase_order.set_status", "inputs": {"status": "RECEIVED"}}]}
                ]}
            ]
        }
    })
    .to_string()
}

const FULL_DESCRIPTION: &str = "Receive a purchase order at the Toronto warehouse: verify the delivery, record what arrived, get the Warehouse Manager's approval for a short receipt, have the Receiving Agent propose the put-away bin, and close the order as RECEIVED.";

fn adopt_set_status(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    // The adopted type can only target an object the workspace has: a
    // fresh test workspace has no purchase_order until one is created,
    // and the adoption refuses an unknown object. Its set_fields
    // template writes the `status` field, so that custom field must
    // exist on the object too - the same seeding the builder tests do.
    custom_object_service::create_with_key(
        conn,
        ws,
        "purchase_order",
        &CustomObjectDefinitionInput {
            singular_label: "Purchase order".into(),
            plural_label: "Purchase orders".into(),
            icon: "▣".into(),
            prefix: "PO".into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
    custom_field_service::create_definition_with_key(
        conn,
        ws,
        "status",
        &CustomFieldDefinitionInput {
            entity_type: "purchase_order".into(),
            label: "Status".into(),
            field_type: "text".into(),
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
        },
        Some(admin),
    )
    .unwrap();
    process_lane_builder::adopt_builder_action_type(
        conn,
        ws,
        "purchase_order.set_status",
        "purchase_order",
        Some(admin),
    )
    .unwrap();
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[tokio::test]
async fn ac1_full_lane_draft_is_valid_when_writes_are_adopted_and_agents_named() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    adopt_set_status(&conn, &ws, &admin);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text_reply(&drafted_full_lane_json())]);

    let lane = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        FULL_DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap();

    // AC-1: the full lane drafts validly as a first draft - one version,
    // version 1, status draft, no compiled graph.
    assert_eq!(lane.lane_key, "lane-draft-receive-purchase-order");
    assert_eq!(lane.versions.len(), 1);
    assert_eq!(lane.versions[0].version, 1);
    assert_eq!(lane.versions[0].status, "draft");
    assert!(lane.versions[0].graph_id.is_none());
    let stored = parse_definition(&lane.versions[0].definition_json).unwrap();
    let kinds: Vec<&str> = stored
        .lane
        .stages
        .iter()
        .flat_map(|s| s.steps.iter().map(|st| st.kind.as_str()))
        .collect();
    assert_eq!(kinds, ["form", "check", "approval", "agent", "record"]);
    let approval = &stored.lane.stages[2].steps[0];
    assert_eq!(approval.branches.as_ref().unwrap().len(), 2);
    let agent = &stored.lane.stages[3].steps[0];
    assert_eq!(
        agent.assistance.as_ref().unwrap().agent.as_deref(),
        Some("wh-agt-receiving")
    );

    // The model was called exactly once, with the v2 catalog prompt.
    let captured = scripted_provider::take_captured();
    assert_eq!(captured.len(), 1);
    assert!(captured[0].system_prompt.contains(FULL_DESCRIPTION));
    assert!(captured[0].options.tools_off);
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[tokio::test]
async fn ac1_full_lane_with_a_provisional_write_is_refused_and_stores_nothing() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    adopt_set_status(&conn, &ws, &admin);
    scripted_provider::reset();
    // The model drafts the full lane but its record step names an Action
    // Type nobody adopted - the P-12 rule refuses the whole draft.
    let mut reply: serde_json::Value = serde_json::from_str(&drafted_full_lane_json()).unwrap();
    reply["lane"]["stages"][3]["steps"][1]["writes"][0]["action_type"] =
        json!("purchase_order.close_receipt");
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let err = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        FULL_DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("purchase_order.close_receipt"));
    assert!(err.to_string().contains("not in the adopted catalog"));

    let lanes = process_lane_service::list_lanes(&conn, &ws, Some(&admin)).unwrap();
    assert!(lanes.is_empty());
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[tokio::test]
async fn ac1_full_lane_with_an_unnamed_agent_step_is_refused_and_stores_nothing() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    adopt_set_status(&conn, &ws, &admin);
    scripted_provider::reset();
    // The model drafts the full lane but its agent step names no agent.
    let mut reply: serde_json::Value = serde_json::from_str(&drafted_full_lane_json()).unwrap();
    reply["lane"]["stages"][3]["steps"][0]
        .as_object_mut()
        .unwrap()
        .remove("assistance");
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let err = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        FULL_DESCRIPTION,
        "purchase_order",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("names no agent"));

    let lanes = process_lane_service::list_lanes(&conn, &ws, Some(&admin)).unwrap();
    assert!(lanes.is_empty());
}
