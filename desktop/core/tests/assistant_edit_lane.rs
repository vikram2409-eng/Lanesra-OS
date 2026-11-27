//! AI Agent Platform v2, Phase 7: revise a lane by description - the Admin
//! Assistant edits a lane (WP-0723, REQ-LANE-022).
//!
//! Every test goes through the scripted provider (WP-0005), so each model
//! reply is scripted and the request the service made is captured and
//! checked. An edit stores a new draft version of the same lane and never
//! touches the published version (AC-2); the assistant cannot publish a
//! lane (AC-3); the prompt comes from the catalog (AC-1); the same code
//! serves desktop and Team Workspace (AC-4 - the server tests prove the
//! route calls this service).
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
    ai_service, assistant_draft_lane_service, assistant_edit_lane_service, custom_field_service,
    custom_object_service, process_lane_builder, process_lane_service, workspace_service,
};
use serde_json::json;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Edit Lane Test Co".into(),
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

/// The smallest lane that publishes: one form step, run-data fields
/// only, no record references, no writes. The publish checks pass on
/// it, so it can be the published v1 an edit revises into a v2 draft.
fn seed_lane_json() -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-edit-receive-purchase-order",
            "name": "Receive a purchase order",
            "description": "Receive a purchase order at the Toronto warehouse.",
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
                ]}
            ]
        }
    })
    .to_string()
}

/// The seed lane with one added form step, as the model would revise it
/// for "Add a step where the receiver records the delivery-note number."
/// Same id, same object, same version number - the service assigns the
/// stored version, whatever the reply says.
fn revised_lane_json() -> String {
    let mut reply: serde_json::Value = serde_json::from_str(&seed_lane_json()).unwrap();
    reply["lane"]["run_data"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key": "delivery_note_number", "type": "string", "required": true, "description": "Delivery-note number from the carrier's paperwork."}));
    reply["lane"]["stages"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "record-delivery-note", "name": "Record delivery-note number", "kind": "form", "role": "role_wh_receiver", "required": true,
            "description": "Record the delivery-note number from the carrier's paperwork.",
            "fields": [
                {"key": "delivery_note_number", "label": "Delivery-note number", "type": "string", "required": true, "binds_to": "run_data.delivery_note_number"}
            ]}));
    reply.to_string()
}

const LANE_KEY: &str = "lane-edit-receive-purchase-order";
const CHANGE: &str = "Add a step where the receiver records the delivery-note number.";

fn import_seed(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    process_lane_service::import_lane_definition(conn, ws, &seed_lane_json(), Some(admin)).unwrap();
}

/// Covers: REQ-LANE-022 (WP-0723 AC-2)
#[tokio::test]
async fn ac2_edit_of_a_published_lane_stores_a_new_draft_version() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    process_lane_service::publish_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text_reply(&revised_lane_json())]);

    let version = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        CHANGE,
    )
    .await
    .unwrap();

    // AC-2: the edit is a NEW draft version of the same lane - v2,
    // draft, no compiled graph - and the published v1 is untouched.
    assert_eq!(version.version, 2);
    assert_eq!(version.status, "draft");
    assert!(version.graph_id.is_none());
    let lane = process_lane_service::get_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    assert_eq!(lane.versions.len(), 2);
    assert_eq!(lane.versions[0].version, 1);
    assert_eq!(lane.versions[0].status, "published");
    assert!(lane.versions[0].graph_id.is_some());
    assert_eq!(lane.versions[1].version, 2);
    assert_eq!(lane.versions[1].status, "draft");
    let stored = parse_definition(&lane.versions[1].definition_json).unwrap();
    assert_eq!(stored.lane.stages[0].steps.len(), 2);
    assert_eq!(stored.lane.stages[0].steps[1].id, "record-delivery-note");
    // The published v1's definition is exactly what it was: one step.
    let published = parse_definition(&lane.versions[0].definition_json).unwrap();
    assert_eq!(published.lane.stages[0].steps.len(), 1);

    // The model was called exactly once, with the catalog prompt - the
    // current definition and the change the person asked for.
    let captured = scripted_provider::take_captured();
    assert_eq!(captured.len(), 1);
    assert!(captured[0].system_prompt.contains(LANE_KEY));
    assert!(captured[0].system_prompt.contains(CHANGE));
    assert!(captured[0].options.tools_off);
}

/// Covers: REQ-LANE-022 (WP-0723 AC-2)
#[tokio::test]
async fn ac2_edit_of_a_draft_revises_that_draft_in_place() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text_reply(&revised_lane_json())]);

    let version = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        CHANGE,
    )
    .await
    .unwrap();

    // One draft at a time: the lane keeps a single v1 draft, revised in
    // place - no second draft version is minted.
    assert_eq!(version.version, 1);
    assert_eq!(version.status, "draft");
    let lane = process_lane_service::get_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    assert_eq!(lane.versions.len(), 1);
    let stored = parse_definition(&lane.versions[0].definition_json).unwrap();
    assert_eq!(stored.lane.stages[0].steps.len(), 2);
}

/// Covers: REQ-LANE-022 (WP-0723 AC-3)
#[tokio::test]
async fn ac3_model_reply_claiming_published_is_stored_as_draft() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    process_lane_service::publish_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    scripted_provider::reset();
    // The model disobeys the prompt: it claims the revision is already
    // published, version 9, of a newer format version.
    let mut reply: serde_json::Value = serde_json::from_str(&revised_lane_json()).unwrap();
    reply["format_version"] = json!("2.0.0");
    reply["lane"]["status"] = json!("published");
    reply["lane"]["version"] = json!(9);
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let version = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        CHANGE,
    )
    .await
    .unwrap();

    // AC-3: the assistant cannot publish - the stored version is a v2
    // draft whatever the model claimed, and the stored definition says
    // so too.
    assert_eq!(version.version, 2);
    assert_eq!(version.status, "draft");
    assert!(version.graph_id.is_none());
    let stored: serde_json::Value = serde_json::from_str(&version.definition_json).unwrap();
    assert_eq!(stored["format_version"], "1.0.0");
    assert_eq!(stored["lane"]["version"], 2);
    assert_eq!(stored["lane"]["status"], "draft");
}

/// Covers: REQ-LANE-022 (WP-0723 AC-2)
#[tokio::test]
async fn ac2_empty_description_and_unknown_lane_are_refused_before_any_model_call() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    scripted_provider::reset();

    for bad in ["", "   "] {
        let err = assistant_edit_lane_service::edit_lane(
            &conn,
            &ws,
            &master_key(),
            Some(&admin),
            LANE_KEY,
            bad,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("describe the change"));
    }
    let err = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        "lane-does-not-exist",
        CHANGE,
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("no lane with key 'lane-does-not-exist'"));
    let err = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        "   ",
        CHANGE,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("a lane key is needed"));

    // No model call was made for any refusal, and the seed lane is
    // unchanged - one draft version, one step.
    assert!(scripted_provider::take_captured().is_empty());
    let lane = process_lane_service::get_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    assert_eq!(lane.versions.len(), 1);
    let stored = parse_definition(&lane.versions[0].definition_json).unwrap();
    assert_eq!(stored.lane.stages[0].steps.len(), 1);
}

/// Covers: REQ-LANE-022 (WP-0723 AC-2)
#[tokio::test]
async fn ac2_non_definition_reply_and_wrong_lane_reply_store_nothing() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    process_lane_service::publish_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![
        text_reply("Sure - I added the step you asked for, all done."),
        text_reply(&revised_lane_json().replace(LANE_KEY, "lane-some-other-lane")),
    ]);

    let err = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        CHANGE,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("not a JSON lane definition"));

    let err = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        CHANGE,
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("not a revision of lane 'lane-edit-receive-purchase-order'"));

    // Nothing was stored on either refusal: still v1 published, no v2.
    let lane = process_lane_service::get_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    assert_eq!(lane.versions.len(), 1);
    assert_eq!(lane.versions[0].status, "published");
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[tokio::test]
async fn ac1_edit_using_a_provisional_action_type_is_refused_and_stores_nothing() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    process_lane_service::publish_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    scripted_provider::reset();
    // The model revises the lane with a record step whose write names an
    // Action Type nobody adopted - the P-12 rule refuses it.
    let mut reply: serde_json::Value = serde_json::from_str(&revised_lane_json()).unwrap();
    reply["lane"]["stages"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "close-po", "name": "Close the PO", "kind": "record", "role": "role_wh_receiver", "required": true,
            "description": "Close the PO as RECEIVED.",
            "writes": [{"action_type": "purchase_order.close_receipt", "inputs": {"purchase_order_id": "$record.id"}}]}));
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let err = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        "Close the PO as RECEIVED when the receipt is complete.",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("purchase_order.close_receipt"));
    assert!(err.to_string().contains("not in the adopted catalog"));

    let lane = process_lane_service::get_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    assert_eq!(lane.versions.len(), 1);
    assert_eq!(lane.versions[0].status, "published");
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[tokio::test]
async fn ac1_edit_using_the_adopted_set_status_write_is_accepted() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    process_lane_service::publish_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    // Adopt the one set_status write the draft prompt can name, so the
    // catalog check passes for it. The adopted type can only target an
    // object the workspace has: a fresh test workspace has no
    // purchase_order until one is created, and its set_fields template
    // writes the `status` field, so that custom field must exist too.
    custom_object_service::create_with_key(
        &conn,
        &ws,
        "purchase_order",
        &CustomObjectDefinitionInput {
            singular_label: "Purchase order".into(),
            plural_label: "Purchase orders".into(),
            icon: "▣".into(),
            prefix: "PO".into(),
            digits: 6,
        },
        Some(&admin),
    )
    .unwrap();
    custom_field_service::create_definition_with_key(
        &conn,
        &ws,
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
        Some(&admin),
    )
    .unwrap();
    process_lane_builder::adopt_builder_action_type(
        &conn,
        &ws,
        "purchase_order.set_status",
        "purchase_order",
        Some(&admin),
    )
    .unwrap();
    scripted_provider::reset();
    let mut reply: serde_json::Value = serde_json::from_str(&revised_lane_json()).unwrap();
    reply["lane"]["stages"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "close-po", "name": "Close the PO", "kind": "record", "role": "role_wh_receiver", "required": true,
            "description": "Close the PO as RECEIVED.",
            "writes": [{"action_type": "purchase_order.set_status", "inputs": {"status": "RECEIVED"}}]}));
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let version = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        "Close the PO as RECEIVED when the receipt is complete.",
    )
    .await
    .unwrap();

    assert_eq!(version.version, 2);
    assert_eq!(version.status, "draft");
    let stored = parse_definition(&version.definition_json).unwrap();
    assert_eq!(stored.lane.stages[0].steps.len(), 3);
    assert_eq!(stored.lane.stages[0].steps[2].kind, "record");
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[tokio::test]
async fn ac1_edit_with_an_unnamed_agent_step_is_refused() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    import_seed(&conn, &ws, &admin);
    scripted_provider::reset();
    // An agent step that names no agent - the boundary refuses it with
    // the named reason, and nothing is stored.
    let mut reply: serde_json::Value = serde_json::from_str(&revised_lane_json()).unwrap();
    reply["lane"]["stages"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "suggest-bin", "name": "Suggest a bin", "kind": "agent", "role": "role_wh_receiver", "required": true,
            "description": "Suggest a put-away bin."}));
    scripted_provider::script_responses(vec![text_reply(&reply.to_string())]);

    let err = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        LANE_KEY,
        "Have an agent suggest the put-away bin.",
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("names no agent"));

    let lane = process_lane_service::get_lane(&conn, &ws, LANE_KEY, Some(&admin)).unwrap();
    assert_eq!(lane.versions.len(), 1);
    let stored = parse_definition(&lane.versions[0].definition_json).unwrap();
    assert_eq!(stored.lane.stages[0].steps.len(), 1);
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1)
#[test]
fn ac1_prompt_comes_from_the_catalog() {
    let (text, used) = lanesra_core::prompts::render_active(
        "assistant/edit-lane",
        &[
            ("lane_key", LANE_KEY),
            ("current_definition", &seed_lane_json()),
            ("description", CHANGE),
        ],
    )
    .unwrap();
    assert_eq!(used.version, 1);
    assert!(text.contains(LANE_KEY));
    assert!(text.contains(CHANGE));
    assert!(text.contains("Reply with one JSON object and nothing else"));
    assert!(text.contains("status is draft, always"));
    assert!(text.contains("Change only what the description asks to change"));
    assert!(text.contains("Never pick an agent the description does not name"));
}

/// The full journey, end to end (WP-0723 test plan): the Assistant drafts
/// a lane, the Assistant revises it by description, and the admin
/// publishes the revised draft - the Assistant never publishes, the
/// person does. The draft and the revision are scripted model replies;
/// the publish is the real `publish_lane`.
fn e2e_drafted_lane_json() -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-e2e-receive-purchase-order",
            "name": "Receive a purchase order",
            "description": "Receive a purchase order at the Toronto warehouse: verify the delivery and close the order as RECEIVED.",
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
                     ]},
                    {"id": "close-po", "name": "Close the PO", "kind": "record", "role": "role_wh_receiver", "required": true,
                     "description": "Close the PO as RECEIVED.",
                     "writes": [{"action_type": "purchase_order.set_status", "inputs": {"status": "RECEIVED"}}]}
                ]}
            ]
        }
    })
    .to_string()
}

fn e2e_revised_lane_json() -> String {
    let mut reply: serde_json::Value = serde_json::from_str(&e2e_drafted_lane_json()).unwrap();
    reply["lane"]["run_data"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key": "delivery_note_number", "type": "string", "required": true, "description": "Delivery-note number from the carrier's paperwork."}));
    reply["lane"]["stages"][0]["steps"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "record-delivery-note", "name": "Record delivery-note number", "kind": "form", "role": "role_wh_receiver", "required": true,
            "description": "Record the delivery-note number from the carrier's paperwork.",
            "fields": [
                {"key": "delivery_note_number", "label": "Delivery-note number", "type": "string", "required": true, "binds_to": "run_data.delivery_note_number"}
            ]}));
    reply.to_string()
}

/// Covers: REQ-LANE-022 (WP-0723 AC-1, AC-2, AC-3)
#[tokio::test]
async fn e2e_draft_then_edit_then_publish() {
    let (conn, ws, admin) = setup_workspace();
    use_scripted_workspace_default(&conn, &ws, &admin);
    // The draft's record step writes the adopted set_status type.
    custom_object_service::create_with_key(
        &conn,
        &ws,
        "purchase_order",
        &CustomObjectDefinitionInput {
            singular_label: "Purchase order".into(),
            plural_label: "Purchase orders".into(),
            icon: "▣".into(),
            prefix: "PO".into(),
            digits: 6,
        },
        Some(&admin),
    )
    .unwrap();
    custom_field_service::create_definition_with_key(
        &conn,
        &ws,
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
        Some(&admin),
    )
    .unwrap();
    process_lane_builder::adopt_builder_action_type(
        &conn,
        &ws,
        "purchase_order.set_status",
        "purchase_order",
        Some(&admin),
    )
    .unwrap();
    scripted_provider::reset();
    scripted_provider::script_responses(vec![
        text_reply(&e2e_drafted_lane_json()),
        text_reply(&e2e_revised_lane_json()),
    ]);

    // 1. Draft: a first draft, version 1, never published.
    let lane = assistant_draft_lane_service::draft_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        "Receive a purchase order at the Toronto warehouse: verify the delivery and close the order as RECEIVED.",
        "purchase_order",
    )
    .await
    .unwrap();
    assert_eq!(lane.lane_key, "lane-e2e-receive-purchase-order");
    assert_eq!(lane.versions[0].status, "draft");
    assert!(lane.versions[0].graph_id.is_none());

    // 2. Edit: the same lane, revised in place as the v1 draft.
    let version = assistant_edit_lane_service::edit_lane(
        &conn,
        &ws,
        &master_key(),
        Some(&admin),
        "lane-e2e-receive-purchase-order",
        "Add a step where the receiver records the delivery-note number.",
    )
    .await
    .unwrap();
    assert_eq!(version.version, 1);
    assert_eq!(version.status, "draft");
    assert!(version.graph_id.is_none());

    // 3. Publish: the admin publishes the revised draft - the
    // Assistant has no publish path, so this step is the person's.
    let published = process_lane_service::publish_lane(
        &conn,
        &ws,
        "lane-e2e-receive-purchase-order",
        Some(&admin),
    )
    .unwrap();
    assert_eq!(published.version, 1);
    assert_eq!(published.status, "published");
    assert!(published.graph_id.is_some());
    let stored = parse_definition(&published.definition_json).unwrap();
    assert_eq!(stored.lane.stages[0].steps.len(), 3);
    assert_eq!(stored.lane.stages[0].steps[2].id, "record-delivery-note");

    // Two model calls, one per Assistant step, both with tools off.
    let captured = scripted_provider::take_captured();
    assert_eq!(captured.len(), 2);
    assert!(captured.iter().all(|c| c.options.tools_off));
}
