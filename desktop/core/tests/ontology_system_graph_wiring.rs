//! Ontology Layer epic (#340): the System Graph previously had no node
//! type for Action Types at all, and an AI agent's own `sync_graph_node`
//! only ever added `delegates_to` edges to other agents - its Ontology
//! scope (`object_type_scopes`/`action_type_ids`, issue #344) was never
//! reflected as a graph edge, so the Dependency Explorer could never
//! answer "which agents touch this object type" or "which agents invoke
//! this Action". Covers the fix directly: an Action Type syncs a
//! `depends_on` edge to its target object, and a scoped agent syncs a
//! `depends_on` edge per object type scope and an `invokes` edge per
//! granted Action Type.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::action_type::ActionTypeDefinitionInput;
use lanesra_core::models::ai_agent::{AiAgentInput, ObjectTypeScope};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{action_type_service, ai_agent_service, system_graph_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Ontology Wiring Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin User".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) =
        lanesra_core::services::workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn action_type_input() -> ActionTypeDefinitionInput {
    ActionTypeDefinitionInput {
        name: "Mark Inactive".into(),
        description: "".into(),
        target_object_type: "Company".into(),
        required_capability: "update".into(),
        parameters: vec![],
        effect: "set_status".into(),
        effect_config_json: serde_json::json!({"target_status": "Inactive"}).to_string(),
    }
}

fn agent_input(
    object_type_scopes: Vec<ObjectTypeScope>,
    action_type_ids: Vec<String>,
) -> AiAgentInput {
    AiAgentInput {
        name: "Scoped Agent".into(),
        description: None,
        icon: "🤖".into(),
        system_prompt: "P.".into(),
        action_names: vec![],
        delegate_agent_ids: vec![],
        skill_ids: vec![],
        object_type_scopes,
        action_type_ids,
    }
}

#[test]
fn creating_an_action_type_syncs_a_node_that_depends_on_its_target_object() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());

    let action = action_type_service::create(&conn, &ws, &action_type_input(), admin).unwrap();

    let node =
        system_graph_service::get_node(&conn, &ws, "action_type_definition", &action.id).unwrap();
    assert!(
        node.is_some(),
        "an Action Type must sync its own System Graph node"
    );

    let dependents =
        system_graph_service::get_dependents(&conn, &ws, "custom_object", "Company").unwrap();
    assert!(
        dependents.iter().any(|h| h.node.node_type == "action_type_definition" && h.node.component_id == action.id),
        "the Action Type must show up as a dependent of its target object in the Dependency Explorer"
    );
}

#[test]
fn an_agent_scoped_to_an_object_type_and_action_type_syncs_both_edges() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());

    let action = action_type_service::create(&conn, &ws, &action_type_input(), admin).unwrap();
    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(
            vec![ObjectTypeScope {
                object_type: "Company".into(),
                capabilities: vec!["read".into()],
            }],
            vec![action.id.clone()],
        ),
        admin,
    )
    .unwrap();

    let object_dependents =
        system_graph_service::get_dependents(&conn, &ws, "custom_object", "Company").unwrap();
    assert!(
        object_dependents
            .iter()
            .any(|h| h.node.node_type == "ai_agent" && h.node.component_id == agent.id),
        "an agent scoped to Company must show up as a dependent of the Company object type"
    );

    let action_dependents =
        system_graph_service::get_dependents(&conn, &ws, "action_type_definition", &action.id)
            .unwrap();
    assert!(
        action_dependents
            .iter()
            .any(|h| h.node.node_type == "ai_agent" && h.node.component_id == agent.id),
        "an agent granted an Action Type must show up as a dependent of that Action Type"
    );
}

#[test]
fn an_unscoped_agent_adds_no_object_type_or_action_type_edges() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());

    let agent = ai_agent_service::create(&conn, &ws, &agent_input(vec![], vec![]), admin).unwrap();

    let dependents =
        system_graph_service::get_dependents(&conn, &ws, "custom_object", "Company").unwrap();
    assert!(
        !dependents.iter().any(|h| h.node.component_id == agent.id),
        "an unscoped agent (applies to every object type) must not add a single misleading edge to just one of them"
    );
}

#[test]
fn removing_an_agents_scope_resyncs_its_edges_rather_than_leaking_the_old_one() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());

    let agent = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(
            vec![ObjectTypeScope {
                object_type: "Company".into(),
                capabilities: vec!["read".into()],
            }],
            vec![],
        ),
        admin,
    )
    .unwrap();
    let before =
        system_graph_service::get_dependents(&conn, &ws, "custom_object", "Company").unwrap();
    assert!(before.iter().any(|h| h.node.component_id == agent.id));

    ai_agent_service::update(&conn, &agent.id, &ws, &agent_input(vec![], vec![]), admin).unwrap();
    let after =
        system_graph_service::get_dependents(&conn, &ws, "custom_object", "Company").unwrap();
    assert!(
        !after.iter().any(|h| h.node.component_id == agent.id),
        "clearing the agent's scope must remove the now-stale edge, not leave it behind"
    );
}
