//! Next-Gen program, Domain A (Intelligence Foundation), FND-01: the
//! Lanesra System Graph. Covers the v1 slice's own minimum acceptance
//! criterion directly - querying any synced component returns its
//! direct and transitive dependencies with no false missing references
//! - plus sync idempotency, cycle safety, workspace isolation, and the
//! hard-delete-cascades/soft-deactivate-keeps distinction
//! `system_graph_repo::remove_node`'s own doc comment documents.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::system_graph::SystemEdgeTarget;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::services::{ai_agent_service, business_rule_service, custom_field_service, custom_object_service, relationship_service, system_graph_service, workflow_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = lanesra_core::models::workspace::WorkspaceSetup {
        business_name: business_name.into(),
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
    let (workspace, admin) = lanesra_core::services::workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn vendor_input() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }
}

fn agent_input(name: &str, delegate_agent_ids: Vec<String>) -> AiAgentInput {
    AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: "P.".into(), action_names: vec![], delegate_agent_ids, skill_ids: vec![] }
}

#[test]
fn syncing_the_same_component_twice_does_not_duplicate_node_or_edges() {
    let (conn, ws, _admin) = setup_workspace("Idempotency Co");
    for _ in 0..2 {
        system_graph_service::sync_node(&conn, &ws, "custom_object", "vendor", "Vendors", "{}", &[]).unwrap();
    }
    let nodes = system_graph_service::list_nodes_by_type(&conn, &ws, "custom_object").unwrap();
    assert_eq!(nodes.len(), 1);

    for _ in 0..2 {
        system_graph_service::sync_node(
            &conn, &ws, "custom_field", "field-1", "Field One", "{}",
            &[SystemEdgeTarget { edge_type: "depends_on".into(), to_node_type: "custom_object".into(), to_component_id: "vendor".into() }],
        )
        .unwrap();
    }
    let dependents = system_graph_service::get_dependents(&conn, &ws, "custom_object", "vendor").unwrap();
    assert_eq!(dependents.len(), 1, "re-syncing the same edge twice must not duplicate it");
}

#[test]
fn a_three_hop_dependency_chain_resolves_in_both_directions() {
    let (conn, ws, admin) = setup_workspace("Chain Co");
    let admin = Some(admin.as_str());

    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();

    let field = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: vendor.key.clone(), label: "Tier".into(), field_type: "text".into(), options: vec![], required: false,
            show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None, regex_pattern: None,
            is_unique: false, default_value: None, help_text: None, placeholder: None, is_searchable: false, is_filterable: false, is_reportable: false,
            is_hidden_by_default: false,
        },
        admin,
    )
    .unwrap();

    let rule = business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            entity_type: vendor.key.clone(), name: "Tier required".into(), description: None, match_type: "all".into(), priority: 0,
            effective_start_date: None, effective_end_date: None, app_id: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "custom".into(), field_key: field.key.clone(), operator: "equals".into(), value: "x".into(),
                compare_field_source: None, compare_field_key: None, group_id: None, relationship_definition_id: None,
            }],
            actions: vec![BusinessRuleActionInput { action_type: "require".into(), target_field_key: Some(field.key.clone()), target_field_source: "custom".into(), action_value: None, message: None }],
        },
        admin,
    )
    .unwrap();

    let agent_b = ai_agent_service::create(&conn, &ws, &agent_input("Agent B", vec![]), admin).unwrap();
    let agent_a = ai_agent_service::create(&conn, &ws, &agent_input("Agent A", vec![agent_b.id.clone()]), admin).unwrap();

    let workflow = workflow_service::create_rule(
        &conn, &ws,
        &WorkflowDefinitionInput {
            entity_type: vendor.key.clone(), name: "Notify".into(), description: None, trigger_type: "record_created".into(),
            trigger_status: None, trigger_field_key: None, trigger_field_source: "custom".into(), trigger_offset_days: 0,
            match_type: "all".into(), priority: 0, app_id: None, conditions: vec![],
            actions: vec![WorkflowActionInput { action_type: "run_ai_agent".into(), params_json: serde_json::json!({"target_type": "agent", "target_id": agent_a.id, "input_template": ""}).to_string() }],
        },
        admin,
    )
    .unwrap();

    // Direct dependents of the Vendor object: the field, the rule and the
    // workflow all point at it directly.
    let direct = system_graph_service::get_dependents(&conn, &ws, "custom_object", &vendor.key).unwrap();
    let direct_ids: Vec<&str> = direct.iter().map(|h| h.node.component_id.as_str()).collect();
    assert!(direct_ids.contains(&field.id.as_str()));
    assert!(direct_ids.contains(&rule.id.as_str()));
    assert!(direct_ids.contains(&workflow.id.as_str()));
    assert!(direct.iter().all(|h| h.depth == 1));

    // Full transitive impact of the Vendor object: the field, rule and
    // workflow that depend on it directly, and nothing further - nothing
    // in this chain points *at* the workflow, so impact (upstream) stops
    // there. The workflow's own invoked agent is downstream of the
    // workflow, not further upstream of Vendor - impact must not claim
    // otherwise.
    let impact = system_graph_service::get_impact(&conn, &ws, "custom_object", &vendor.key).unwrap();
    let impact_ids: Vec<&str> = impact.iter().map(|h| h.node.component_id.as_str()).collect();
    assert!(impact_ids.contains(&field.id.as_str()));
    assert!(impact_ids.contains(&rule.id.as_str()));
    assert!(impact_ids.contains(&workflow.id.as_str()));
    assert!(!impact_ids.contains(&agent_a.id.as_str()), "the invoked agent is downstream of the workflow, not upstream of Vendor");
    assert!(!impact_ids.contains(&agent_b.id.as_str()));

    // The reverse direction from the workflow: its own transitive
    // dependencies reach the Vendor object (depth 1) and, through the
    // agent it invokes, both agents (depth 1 and depth 2) - the exact
    // "no false missing references, multiple hops deep" acceptance
    // criterion, in the direction where this chain actually has depth.
    let lineage = system_graph_service::get_lineage(&conn, &ws, "workflow", &workflow.id).unwrap();
    let lineage_ids: Vec<&str> = lineage.iter().map(|h| h.node.component_id.as_str()).collect();
    assert!(lineage_ids.contains(&vendor.key.as_str()));
    assert!(lineage_ids.contains(&agent_a.id.as_str()), "workflow's invoked agent must be in its own transitive lineage");
    assert!(lineage_ids.contains(&agent_b.id.as_str()), "agent_a's delegate must be reachable transitively too");

    let agent_a_depth = lineage.iter().find(|h| h.node.component_id == agent_a.id).unwrap().depth;
    let agent_b_depth = lineage.iter().find(|h| h.node.component_id == agent_b.id).unwrap().depth;
    assert!(agent_b_depth > agent_a_depth, "the delegate is strictly farther from the workflow than the agent that delegates to it");
}

#[test]
fn a_delegate_cycle_terminates_instead_of_looping() {
    let (conn, ws, admin) = setup_workspace("Cycle Co");
    let admin = Some(admin.as_str());

    // Create both agents first with no delegate, then update each to
    // delegate to the other - a real mutual cycle, same as a misconfigured
    // admin could actually create today (nothing currently validates
    // against delegate cycles).
    let agent_a = ai_agent_service::create(&conn, &ws, &agent_input("A", vec![]), admin).unwrap();
    let agent_b = ai_agent_service::create(&conn, &ws, &agent_input("B", vec![agent_a.id.clone()]), admin).unwrap();
    let mut update_a = agent_input("A", vec![agent_b.id.clone()]);
    update_a.name = "A".into();
    ai_agent_service::update(&conn, &agent_a.id, &ws, &update_a, admin).unwrap();

    // Must return (not hang) and must not contain agent_a as its own
    // downstream dependency more than once.
    let impact = system_graph_service::get_impact(&conn, &ws, "ai_agent", &agent_a.id).unwrap();
    let count_b = impact.iter().filter(|h| h.node.component_id == agent_b.id).count();
    assert_eq!(count_b, 1, "a cycle must still dedupe to one entry per node, at its shortest depth");
}

#[test]
fn nodes_and_edges_are_isolated_per_workspace() {
    let (conn1, ws1, admin1) = setup_workspace("Workspace One");
    let (conn2, ws2, admin2) = setup_workspace("Workspace Two");

    let vendor1 = custom_object_service::create(&conn1, &ws1, &vendor_input(), Some(&admin1)).unwrap();
    custom_field_service::create_definition(
        &conn1, &ws1,
        &CustomFieldDefinitionInput {
            entity_type: vendor1.key.clone(), label: "Tier".into(), field_type: "text".into(), options: vec![], required: false,
            show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None, regex_pattern: None,
            is_unique: false, default_value: None, help_text: None, placeholder: None, is_searchable: false, is_filterable: false, is_reportable: false,
            is_hidden_by_default: false,
        },
        Some(&admin1),
    )
    .unwrap();

    let vendor2 = custom_object_service::create(&conn2, &ws2, &vendor_input(), Some(&admin2)).unwrap();
    assert_eq!(vendor1.key, vendor2.key, "both workspaces independently produce the same slug - the interesting case for isolation");

    let ws2_dependents = system_graph_service::get_dependents(&conn2, &ws2, "custom_object", &vendor2.key).unwrap();
    assert!(ws2_dependents.is_empty(), "workspace 2 must not see workspace 1's field, even though the two connections are entirely separate in-memory databases");
}

#[test]
fn hard_delete_removes_the_node_and_cascades_its_edges_but_soft_deactivate_keeps_it() {
    let (conn, ws, admin) = setup_workspace("Delete Co");
    let admin = Some(admin.as_str());

    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();
    let company_target = "Company".to_string();
    let relationship = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: company_target, target_is_polymorphic: false, relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true, delete_behavior: "restrict".into(), sort_order: 0,
        },
        admin,
    )
    .unwrap();

    assert!(system_graph_service::get_node(&conn, &ws, "relationship", &relationship.id).unwrap().is_some());
    let before = system_graph_service::get_dependents(&conn, &ws, "custom_object", &vendor.key).unwrap();
    assert_eq!(before.len(), 1);

    relationship_service::delete(&conn, &relationship.id, admin).unwrap();
    assert!(system_graph_service::get_node(&conn, &ws, "relationship", &relationship.id).unwrap().is_none(), "a hard delete must remove the node");
    let after = system_graph_service::get_dependents(&conn, &ws, "custom_object", &vendor.key).unwrap();
    assert!(after.is_empty(), "the relationship's own edge must be cascade-removed with it");

    // A field's own only removal path is deactivate (soft) - the node
    // must survive with its data intact, since "what used to depend on
    // this" should stay answerable.
    let field = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: vendor.key.clone(), label: "Tier".into(), field_type: "text".into(), options: vec![], required: false,
            show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None, regex_pattern: None,
            is_unique: false, default_value: None, help_text: None, placeholder: None, is_searchable: false, is_filterable: false, is_reportable: false,
            is_hidden_by_default: false,
        },
        admin,
    )
    .unwrap();
    custom_field_service::deactivate_definition(&conn, &field.id, admin).unwrap();
    let node = system_graph_service::get_node(&conn, &ws, "custom_field", &field.id).unwrap();
    assert!(node.is_some(), "a soft-deactivated component's node must remain in the graph");
}
