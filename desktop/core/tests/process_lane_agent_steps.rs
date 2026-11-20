//! Phase 7, WP-0708: agent steps, assistance and autonomy per step
//! (ADR-0023; REQ-LANE-016, REQ-LANE-011).
//!
//! A form step can carry assistance at autonomy "propose": the named agent
//! proposes values for the step's fields, the person sees the proposal, and
//! nothing is written until they submit - the agent never writes at this
//! level. Each proposed field is recorded as kept or overridden, which feeds
//! the per-step override rate. Every run here goes through the scripted
//! provider (WP-0005), so the agent's reply is deterministic.
//!
//! Covers:
//!   AC-1  an agent proposal is shown as a proposal and written only when the
//!         person confirms it at the "propose" level
//!   AC-4  the override rate per step is reported
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Map, Value};

use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_provider_service, custom_field_service, custom_object_service,
    custom_record_service, process_lane_service, user_service, workspace_service,
};

const KEY: [u8; 32] = [121u8; 32];

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Agent Step Co".into(),
            legal_name: None,
            currency_code: "USD".into(),
            locale: "en-US".into(),
            timezone: "UTC".into(),
            default_tax_rate_bp: 0,
            admin_username: "admin".into(),
            admin_display_name: "Admin".into(),
            admin_password: "correct horse battery staple".into(),
            load_sample_data: false,
        },
    )
    .unwrap();
    (conn, workspace.id, admin.id)
}

/// A provider row that dispatches to the scripted provider (the settings API
/// cannot select `scripted`, so it is created as another provider and switched
/// underneath - WP-0005 AC-3).
fn scripted_provider_row(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    let provider = ai_provider_service::create(
        conn,
        ws,
        &KEY,
        &AiProviderInput {
            name: "primary".into(),
            provider: "anthropic".into(),
            base_url: None,
            model: "primary-model".into(),
            api_key: Some("k".into()),
        },
        Some(admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_providers SET provider = 'scripted' WHERE id = ?1",
        [&provider.id],
    )
    .unwrap();
    provider.id
}

/// An agent named `Assistant` with no tools, routed to the scripted provider.
fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str) -> AiAgentDefinition {
    let created = ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: "Assistant".into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: "Propose field values as a JSON object.".into(),
            action_names: vec![],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap();
    ai_agent_service::set_model_routing(
        conn,
        &created.id,
        ws,
        Some(AiAgentModelRouting {
            primary_provider_id: Some(scripted_provider_row(conn, ws, admin)),
            ..Default::default()
        }),
        Some(admin),
    )
    .unwrap()
}

fn seed_object(conn: &rusqlite::Connection, ws: &str, admin: &str) {
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
    custom_field_service::create_definition_with_key(
        conn,
        ws,
        "tracking",
        &CustomFieldDefinitionInput {
            entity_type: "widget".into(),
            label: "Tracking".into(),
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
}

fn make_widget(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    custom_record_service::create(
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
    )
    .unwrap()
    .id
}

/// Publishes a one-step lane whose form step is assisted at the given autonomy
/// level by the `Assistant` agent, writing `tracking_in` onto `record.tracking`.
fn publish_assisted_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, autonomy: &str) {
    let lane_id = format!("lane-{autonomy}");
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": lane_id, "name": "Assist", "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": []},
            "stages": [{
                "id": "do", "name": "Do", "steps": [{
                    "id": "fill", "name": "Fill", "kind": "form",
                    "role": "r", "required": true,
                    "assistance": {"autonomy": autonomy, "agent": "Assistant"},
                    "fields": [{
                        "key": "tracking_in", "label": "Tracking", "type": "string",
                        "binds_to": "record.tracking", "required": false
                    }]
                }]
            }]
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, &lane_id, Some(admin)).unwrap();
}

fn tracking(conn: &rusqlite::Connection, record: &str) -> Option<String> {
    custom_field_service::get_entity_values(conn, record)
        .unwrap()
        .get("tracking")
        .cloned()
}

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(v)))
        .collect()
}

/// Queues one scripted agent reply: the JSON proposal text the propose step
/// parses into field values.
fn script_proposal(json_text: &str) {
    scripted_provider::reset();
    scripted_provider::script_responses(vec![ScriptedResponse::Text {
        text: json_text.into(),
        usage: lanesra_core::services::ai_service::TokenUsage {
            input_tokens: 5,
            output_tokens: 5,
        },
    }]);
}

// --------------------------------------------------------------------------
// AC-1: an agent proposal is shown as a proposal and written only on confirm.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-016 (WP-0708 AC-1)
#[tokio::test]
async fn an_assisted_step_proposes_values_and_writes_only_when_the_person_confirms() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    make_agent(&conn, &ws, &admin);
    publish_assisted_lane(&conn, &ws, &admin, "propose");
    let widget = make_widget(&conn, &ws, &admin, "W1");

    // The agent will propose tracking_in = "ACME".
    script_proposal(r#"{"tracking_in": "ACME"}"#);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &KEY,
        "lane-propose",
        &widget,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    // The run view shows the proposal and the autonomy level...
    let view = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&admin)).unwrap();
    let current = view.current.expect("a current step");
    assert_eq!(current.autonomy.as_deref(), Some("propose"));
    assert_eq!(
        current.proposal.as_ref().and_then(|p| p.get("tracking_in")),
        Some(&json!("ACME")),
        "the agent's proposal is shown for the person to confirm"
    );
    // ...but nothing is written to the record yet (the proposal never writes).
    assert_eq!(
        tracking(&conn, &widget),
        None,
        "a proposal is not written until the person submits"
    );

    // The person confirms the proposed value: now it is written, and recorded
    // as accepted (no override).
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &KEY,
        &run.id,
        &values(&[("tracking_in", "ACME")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(tracking(&conn, &widget).as_deref(), Some("ACME"));

    let stats = process_lane_service::assist_stats(&conn, &ws, &run.id).unwrap();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].step_id, "fill");
    assert_eq!(stats[0].proposed, 1);
    assert_eq!(stats[0].overridden, 0);
    assert_eq!(stats[0].override_rate, 0.0);
}

// --------------------------------------------------------------------------
// AC-4: the override rate per step is reported.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-016 (WP-0708 AC-4)
#[tokio::test]
async fn overriding_an_agent_proposal_is_recorded_in_the_override_rate() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    make_agent(&conn, &ws, &admin);
    publish_assisted_lane(&conn, &ws, &admin, "propose");
    let widget = make_widget(&conn, &ws, &admin, "W2");

    script_proposal(r#"{"tracking_in": "ACME"}"#);
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &KEY,
        "lane-propose",
        &widget,
        Some(&admin),
    )
    .await
    .unwrap();

    // The person overrides the proposed value with their own.
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &KEY,
        &run.id,
        &values(&[("tracking_in", "OVERRIDDEN")]),
        Some(&admin),
    )
    .await
    .unwrap();

    // The person's value is written, and the override is recorded.
    assert_eq!(tracking(&conn, &widget).as_deref(), Some("OVERRIDDEN"));
    let stats = process_lane_service::assist_stats(&conn, &ws, &run.id).unwrap();
    assert_eq!(stats[0].proposed, 1);
    assert_eq!(stats[0].overridden, 1);
    assert_eq!(stats[0].override_rate, 1.0);
}

// --------------------------------------------------------------------------
// AC-2: at "does it" (auto) the agent completes the step through the governed
// write path; at "does it with approval" the write waits in the approvals
// inbox until a person decides.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-016 (WP-0708 AC-2)
#[tokio::test]
async fn at_autonomy_auto_the_agent_does_the_step_itself() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    make_agent(&conn, &ws, &admin);
    publish_assisted_lane(&conn, &ws, &admin, "auto");
    let widget = make_widget(&conn, &ws, &admin, "W3");

    script_proposal(r#"{"tracking_in": "AUTO"}"#);
    let run =
        process_lane_service::start_lane_run(&conn, &ws, &KEY, "lane-auto", &widget, Some(&admin))
            .await
            .unwrap();

    // No person submitted - the agent did the step itself and the run finished.
    assert_eq!(
        run.status, "completed",
        "at autonomy auto the agent completes the step with no person"
    );
    assert_eq!(tracking(&conn, &widget).as_deref(), Some("AUTO"));
    let stats = process_lane_service::assist_stats(&conn, &ws, &run.id).unwrap();
    assert_eq!(stats[0].proposed, 1);
    assert_eq!(stats[0].overridden, 0);
}

// Covers: REQ-LANE-016 (WP-0708 AC-2)
#[tokio::test]
async fn at_autonomy_approve_the_write_waits_in_the_inbox_until_approved() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    make_agent(&conn, &ws, &admin);
    publish_assisted_lane(&conn, &ws, &admin, "approve");
    let widget = make_widget(&conn, &ws, &admin, "W4");

    script_proposal(r#"{"tracking_in": "APPROVED"}"#);
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &KEY,
        "lane-approve",
        &widget,
        Some(&admin),
    )
    .await
    .unwrap();

    // The agent proposed, but the write waits: the run is still open, nothing
    // is written, and the proposal is held in the approvals inbox.
    assert_eq!(run.status, "waiting_person");
    assert_eq!(tracking(&conn, &widget), None);
    let rd: Value = serde_json::from_str(run.run_data_json.as_deref().unwrap_or("{}")).unwrap();
    assert!(
        rd.get("__assist_approval")
            .and_then(|v| v.get("fill"))
            .and_then(|v| v.as_str())
            .is_some(),
        "the agent's write is held in the approvals inbox"
    );

    // Approving applies the agent's proposed value through the governed path
    // and advances the run to completion.
    let after = process_lane_service::resolve_assist_approval(
        &conn,
        &ws,
        &KEY,
        &run.id,
        true,
        None,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(after.status, "completed");
    assert_eq!(tracking(&conn, &widget).as_deref(), Some("APPROVED"));
}

// --------------------------------------------------------------------------
// AC-3: in a run a person started, the agent acts as that person - its reads
// and writes are bounded by the starter's identity, not the agent's or an
// administrator's (ADR-0024 §7).
// --------------------------------------------------------------------------

// Covers: REQ-LANE-011 (WP-0708 AC-3)
#[tokio::test]
async fn an_agent_step_acts_as_the_person_who_started_the_run() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    make_agent(&conn, &ws, &admin);
    publish_assisted_lane(&conn, &ws, &admin, "auto");
    let widget = make_widget(&conn, &ws, &admin, "W5");

    // A specific non-admin person starts the run.
    let alice = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "alice".into(),
            display_name: "Alice".into(),
            password: "alice-password-123".into(),
            roles: vec!["Manager".into()],
        },
        Some(&admin),
    )
    .unwrap();

    script_proposal(r#"{"tracking_in": "AUTO"}"#);
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &KEY,
        "lane-auto",
        &widget,
        Some(&alice.id),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed");
    assert_eq!(tracking(&conn, &widget).as_deref(), Some("AUTO"));

    // The agent applied the step acting AS Alice - its assist record is
    // attributed to her, not to the agent or an administrator.
    let rd: Value = serde_json::from_str(run.run_data_json.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(
        rd["__assist"]["fill"]["by"],
        json!(alice.id),
        "the agent step is attributed to the person who started the run"
    );
}
