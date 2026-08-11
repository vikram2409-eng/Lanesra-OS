//! Admin Automation & Customization addendum, Phase 3 (spec §3.3): the
//! generic `create_record`/`update_related_record` workflow actions -
//! reaching beyond the triggering record itself, either to create a new
//! record (optionally linked) or to write a field on record(s) already
//! linked through a relationship - plus `test_workflows`, a dry-run mode
//! mirroring `business_rule_service::test_rules`.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    company_service, custom_field_service, custom_object_service, custom_record_service, relationship_service,
    user_service, workflow_service, workspace_service,
};
use std::collections::HashMap;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Workflow Action Expansion Co".into(),
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
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn company_input(name: &str, status: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(), status: status.into(), owner_user_id: None, tax_number: None,
        billing_address: None, shipping_address: None, tags: None, notes: None,
    }
}

fn vendor_object() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }
}

fn status_changed_workflow(entity_type: &str, trigger_status: &str, action: WorkflowActionInput) -> WorkflowDefinitionInput {
    WorkflowDefinitionInput {
        entity_type: entity_type.into(), name: format!("{entity_type} -> {trigger_status}"), description: None,
        trigger_type: "status_changed".into(), trigger_status: Some(trigger_status.into()), trigger_field_key: None, trigger_field_source: "custom".into(),
        trigger_offset_days: 0, match_type: "all".into(), priority: 0, conditions: vec![], actions: vec![action],
    }
}

// --- create_record --------------------------------------------------------

#[test]
fn create_record_action_creates_a_standalone_company_with_no_relationship() {
    let (conn, ws, admin) = setup_workspace();
    let action = WorkflowActionInput {
        action_type: "create_record".into(),
        params_json: serde_json::json!({"entity_type": "Company", "name_template": "Renewal Account", "relationship_definition_id": null}).to_string(),
    };
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Company", "Active Customer", action), Some(&admin)).unwrap();

    // Creating this company only fires record_created (not status_changed),
    // so it can't itself trigger a second round.
    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    company_service::update(&conn, &company.id, &company_input("Acme", "Active Customer"), Some(&admin)).unwrap();

    let all = company_service::list(&conn, &ws).unwrap();
    assert!(all.iter().any(|c| c.name == "Renewal Account" && c.status == "Prospect"), "expected a standalone 'Renewal Account' company to have been created");
    assert_eq!(all.len(), 2, "exactly one new company should have been created by the action, no more");
}

#[test]
fn create_record_action_creates_a_custom_object_record_and_links_it_via_the_relationship() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    let action = WorkflowActionInput {
        action_type: "create_record".into(),
        params_json: serde_json::json!({"entity_type": vendor.key, "name_template": "Acme's Vendor", "relationship_definition_id": def.id}).to_string(),
    };
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Company", "Active Customer", action), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    company_service::update(&conn, &company.id, &company_input("Acme", "Active Customer"), Some(&admin)).unwrap();

    let vendors = custom_record_service::list(&conn, &ws, &vendor.key).unwrap();
    assert_eq!(vendors.len(), 1);
    assert_eq!(vendors[0].primary_name, "Acme's Vendor");

    let related = relationship_service::related_records_for(&conn, &ws, "Company", &company.id).unwrap();
    assert!(related.iter().any(|r| r.entity_id == vendors[0].id), "the new Vendor record should be linked back to the Company that spawned it");
}

#[test]
fn create_record_action_rejects_a_core_entity_type_it_cannot_construct() {
    let (conn, ws, admin) = setup_workspace();
    let action = WorkflowActionInput {
        action_type: "create_record".into(),
        // Contact requires a company_id - not something a no-code action
        // can safely synthesize, so it's not in the creatable set.
        params_json: serde_json::json!({"entity_type": "Contact", "name_template": null, "relationship_definition_id": null}).to_string(),
    };
    let err = workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Company", "Active Customer", action), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("cannot be created by a workflow action"));
}

// --- update_related_record -------------------------------------------------

#[test]
fn update_related_record_action_writes_a_builtin_field_on_every_linked_company() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_many".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    let company_a = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let company_b = company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();

    let vendor_record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Vendor".into(), status: "Inactive".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_record.id, "Company", &company_a.id, Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_record.id, "Company", &company_b.id, Some(&admin)).unwrap();

    let action = WorkflowActionInput {
        action_type: "update_related_record".into(),
        params_json: serde_json::json!({
            "relationship_definition_id": def.id, "target_field_key": "tax_number", "target_field_source": "builtin",
            "value": "PREFERRED-VENDOR", "copy_from_field_key": null,
        }).to_string(),
    };
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow(&vendor.key, "Active", action), Some(&admin)).unwrap();

    custom_record_service::update(
        &conn, &vendor_record.id,
        &CustomRecordUpdate { primary_name: "Acme Vendor".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    // many_to_many: both linked companies get the field write.
    assert_eq!(company_service::get(&conn, &company_a.id).unwrap().tax_number.as_deref(), Some("PREFERRED-VENDOR"));
    assert_eq!(company_service::get(&conn, &company_b.id).unwrap().tax_number.as_deref(), Some("PREFERRED-VENDOR"));
}

#[test]
fn update_related_record_action_can_copy_a_value_from_the_triggering_record() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let field = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Vendor Notes".into(), field_type: "text".into(), options: vec![],
            required: false, show_in_list: false, sort_order: 0, min_value: None, max_value: None, max_length: None,
            regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: true,
        },
        Some(&admin),
    ).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let vendor_record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Vendor".into(), status: "Inactive".into(), owner_user_id: None, notes: Some("Preferred supplier".into()) },
        Some(&admin),
    ).unwrap();
    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_record.id, "Company", &company.id, Some(&admin)).unwrap();

    let action = WorkflowActionInput {
        action_type: "update_related_record".into(),
        params_json: serde_json::json!({
            "relationship_definition_id": def.id, "target_field_key": field.key, "target_field_source": "custom",
            "value": null, "copy_from_field_key": "notes",
        }).to_string(),
    };
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow(&vendor.key, "Active", action), Some(&admin)).unwrap();

    custom_record_service::update(
        &conn, &vendor_record.id,
        &CustomRecordUpdate { primary_name: "Acme Vendor".into(), status: "Active".into(), owner_user_id: None, notes: Some("Preferred supplier".into()) },
        Some(&admin),
    ).unwrap();

    let values: HashMap<String, String> = lanesra_core::repositories::custom_field_repo::get_values(&conn, &company.id).unwrap();
    assert_eq!(values.get(&field.key).map(String::as_str), Some("Preferred supplier"));
}

#[test]
fn update_related_record_action_is_a_no_op_when_nothing_is_linked_yet() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();
    let action = WorkflowActionInput {
        action_type: "update_related_record".into(),
        params_json: serde_json::json!({
            "relationship_definition_id": def.id, "target_field_key": "tax_number", "target_field_source": "builtin",
            "value": "PREFERRED-VENDOR", "copy_from_field_key": null,
        }).to_string(),
    };
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow(&vendor.key, "Active", action), Some(&admin)).unwrap();

    // Never linked to anything - the action must not error, just no-op.
    let vendor_record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Orphan Vendor".into(), status: "Inactive".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    let updated = custom_record_service::update(
        &conn, &vendor_record.id,
        &CustomRecordUpdate { primary_name: "Orphan Vendor".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    assert_eq!(updated.status, "Active");
}

#[test]
fn update_related_record_action_rejects_a_relationship_that_does_not_connect_to_this_entity_type() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let project = custom_object_service::create(&conn, &ws, &CustomObjectDefinitionInput { singular_label: "Project".into(), plural_label: "Projects".into(), icon: "📁".into(), prefix: "PRJ".into(), digits: 4 }, Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: project.key.clone(), relationship_type: "many_to_one".into(),
            forward_label: "Project".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    // This relationship connects Vendor <-> Project, not Company at all.
    let action = WorkflowActionInput {
        action_type: "update_related_record".into(),
        params_json: serde_json::json!({
            "relationship_definition_id": def.id, "target_field_key": "status", "target_field_source": "builtin",
            "value": "Active Customer", "copy_from_field_key": null,
        }).to_string(),
    };
    let err = workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Company", "Active Customer", action), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("does not connect to this record type"));
}

// --- test_workflows (dry-run / simulation mode) ----------------------------

#[test]
fn test_workflows_reports_a_matching_workflow_without_running_its_actions() {
    let (conn, ws, admin) = setup_workspace();
    let action = WorkflowActionInput {
        action_type: "create_record".into(),
        params_json: serde_json::json!({"entity_type": "Company", "name_template": "Should Not Exist", "relationship_definition_id": null}).to_string(),
    };
    let wf = workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Company", "Active Customer", action), Some(&admin)).unwrap();

    let mut ctx = HashMap::new();
    ctx.insert("status".into(), "Active Customer".into());
    let result = workflow_service::test_workflows(&conn, &ws, "Company", &ctx, Some(&admin)).unwrap();

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].workflow_id, wf.id);
    assert_eq!(result.matches[0].action_descriptions, vec!["would create a new Company"]);

    // Nothing was actually created - it's a dry run.
    assert_eq!(company_service::list(&conn, &ws).unwrap().len(), 0);
}

#[test]
fn test_workflows_omits_workflows_whose_conditions_do_not_match_the_hypothetical_context() {
    let (conn, ws, admin) = setup_workspace();
    let action = WorkflowActionInput { action_type: "create_task".into(), params_json: serde_json::json!({"title": "X", "description": null, "due_in_days": 0, "assignee_user_id": null}).to_string() };
    let mut wf = status_changed_workflow("Company", "Active Customer", action);
    // An explicit extra condition, not just the trigger status itself - a
    // workflow with zero extra conditions always matches regardless of ctx
    // (see workflow_matches's doc comment), so this needs one to exercise
    // the "ctx doesn't satisfy it" path test_workflows is meant to prove.
    wf.conditions = vec![lanesra_core::models::workflow::WorkflowConditionInput {
        field_source: "builtin".into(), field_key: "tax_number".into(), operator: "equals".into(), value: "VIP".into(),
        compare_field_source: None, compare_field_key: None,
    }];
    workflow_service::create_rule(&conn, &ws, &wf, Some(&admin)).unwrap();

    let mut ctx = HashMap::new();
    ctx.insert("status".into(), "Active Customer".into());
    ctx.insert("tax_number".into(), "not VIP".into());
    let result = workflow_service::test_workflows(&conn, &ws, "Company", &ctx, Some(&admin)).unwrap();
    assert!(result.matches.is_empty());
}

#[test]
fn non_administrator_cannot_test_workflows() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();
    let err = workflow_service::test_workflows(&conn, &ws, "Company", &HashMap::new(), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}
