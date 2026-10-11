//! Next-Gen program, Ontology Layer epic, issue #343: Action Types - a
//! named, parameterized write operation scoped to one Object Type,
//! invoked through the target object's own real write path. Covers the
//! issue's own acceptance criteria: invoking applies exactly the
//! configured effect and runs the target's real business rules (a rule
//! that would block the resulting field values blocks the Action the
//! same way it blocks a manual edit), a caller lacking the Action's own
//! `required_capability` is denied even with base `update`, and an
//! inactive Action Type cannot be invoked but stays visible.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{access_role_service, action_type_service, business_rule_service, company_service, custom_field_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Action Types Co".into(),
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

fn company_input(name: &str) -> CompanyInput {
    CompanyInput { name: name.into(), status: "Prospect".into(), ..Default::default() }
}

fn action_type_input(target_object_type: &str, required_capability: &str, parameters: Vec<ActionParameterSpec>, effect: &str, effect_config_json: &str) -> ActionTypeDefinitionInput {
    ActionTypeDefinitionInput {
        name: "Test Action".into(),
        description: "".into(),
        target_object_type: target_object_type.into(),
        required_capability: required_capability.into(),
        parameters,
        effect: effect.into(),
        effect_config_json: effect_config_json.into(),
    }
}

#[test]
fn invoking_set_status_writes_through_the_real_path_and_audits_one_row() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let company = company_service::create(&conn, &ws, &company_input("Acme"), admin).unwrap();

    let def = action_type_service::create(&conn, &ws, &action_type_input("Company", "update", vec![], "set_status", r#"{"target_status":"Inactive"}"#), admin).unwrap();

    let before_count = lanesra_core::repositories::audit_repo::list_recent(&conn, &ws, 1000).unwrap().len();
    action_type_service::invoke(&conn, &ws, &def.id, &company.id, &HashMap::new(), admin).unwrap();

    let updated = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(updated.status, "Inactive");

    // invoke() writes its own "invoke_action" row naming the Action Type,
    // in addition to (not instead of) whatever audit row the underlying
    // company_service::update already writes for the plain field change -
    // that one is pre-existing behavior, not something invoke() suppresses.
    let after = lanesra_core::repositories::audit_repo::list_recent(&conn, &ws, 1000).unwrap();
    assert!(after.len() > before_count, "invoke must write at least one new audit row");
    assert!(after.iter().any(|e| e.event_type == "invoke_action" && e.summary.contains(&def.name)), "must audit the Action Type's own name: {after:?}");
}

#[test]
fn invoking_set_fields_runs_the_targets_real_business_rules_and_a_blocking_rule_rejects_it() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let company = company_service::create(&conn, &ws, &company_input("Acme"), admin).unwrap();

    let tier = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Tier".into(), field_type: "text".into(), options: vec![], required: false,
            show_in_list: false, sort_order: 0, min_value: None, max_value: None, max_length: None, regex_pattern: None,
            is_unique: false, default_value: None, help_text: None, placeholder: None, is_searchable: false, is_filterable: false, is_reportable: true,
            is_hidden_by_default: false,
        },
        admin,
    ).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            app_id: None, entity_type: "Company".into(), name: "Block tier".into(), description: None, match_type: "all".into(),
            priority: 0, effective_start_date: None, effective_end_date: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "custom".into(), field_key: tier.key.clone(), operator: "equals".into(), value: "Blocked".into(),
                compare_field_source: None, compare_field_key: None, group_id: None, relationship_definition_id: None,
            }],
            actions: vec![BusinessRuleActionInput { action_type: "block_save".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Blocked tier cannot be set".into()) }],
        },
        admin,
    ).unwrap();

    let def = action_type_service::create(
        &conn, &ws,
        &action_type_input(
            "Company", "update",
            vec![ActionParameterSpec { key: "tierValue".into(), label: "Tier".into(), field_type: "text".into(), required: true, default_value: None }],
            "set_fields",
            &format!(r#"{{"field_sources":{{"{}":"tierValue"}}}}"#, tier.key),
        ),
        admin,
    ).unwrap();

    let mut blocked_params = HashMap::new();
    blocked_params.insert("tierValue".to_string(), "Blocked".to_string());
    let err = action_type_service::invoke(&conn, &ws, &def.id, &company.id, &blocked_params, admin).unwrap_err();
    assert!(format!("{err:?}").contains("Blocked tier cannot be set"), "a rule that blocks the resulting field value must block the Action the same way it blocks a manual edit: {err:?}");

    let mut allowed_params = HashMap::new();
    allowed_params.insert("tierValue".to_string(), "Gold".to_string());
    action_type_service::invoke(&conn, &ws, &def.id, &company.id, &allowed_params, admin).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get(&tier.key).map(String::as_str), Some("Gold"));
}

#[test]
fn a_caller_lacking_the_required_capability_is_denied_even_with_base_update() {
    let (conn, ws, admin) = setup_workspace();
    let admin_ref = Some(admin.as_str());
    let company = company_service::create(&conn, &ws, &company_input("Acme"), admin_ref).unwrap();

    // A role granting Update but explicitly not Assign on every object.
    let role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "Updater".into(), description: "".into() }, admin_ref).unwrap();
    access_role_service::upsert_grant(
        &conn, &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: false, can_assign: false, record_scope: "ORGANIZATION".into() },
        admin_ref,
    ).unwrap();
    let updater = user_service::create(&conn, &ws, &NewUser { username: "updater".into(), display_name: "Updater".into(), password: "anothersecretpw".into(), roles: vec!["Sales".to_string()] }, admin_ref).unwrap();
    access_role_service::assign_to_user(&conn, &updater.id, &role.id, admin_ref).unwrap();

    // An Assign-gated Action Type (e.g. "Reassign Company Owner").
    let def = action_type_service::create(
        &conn, &ws,
        &action_type_input(
            "Company", "assign",
            vec![ActionParameterSpec { key: "newOwner".into(), label: "New owner".into(), field_type: "text".into(), required: true, default_value: None }],
            "reassign_owner",
            r#"{"owner_param":"newOwner"}"#,
        ),
        admin_ref,
    ).unwrap();

    let mut params = HashMap::new();
    params.insert("newOwner".to_string(), updater.id.clone());
    let err = action_type_service::invoke(&conn, &ws, &def.id, &company.id, &params, Some(&updater.id)).unwrap_err();
    assert!(format!("{err:?}").to_lowercase().contains("assign"), "an Updater without Assign must be denied an Assign-gated Action: {err:?}");
}

#[test]
fn an_inactive_action_type_cannot_be_invoked_but_remains_visible() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let company = company_service::create(&conn, &ws, &company_input("Acme"), admin).unwrap();
    let def = action_type_service::create(&conn, &ws, &action_type_input("Company", "update", vec![], "set_status", r#"{"target_status":"Inactive"}"#), admin).unwrap();

    action_type_service::deactivate(&conn, &def.id, &ws, admin).unwrap();

    let err = action_type_service::invoke(&conn, &ws, &def.id, &company.id, &HashMap::new(), admin).unwrap_err();
    assert!(format!("{err:?}").contains("deactivated"));

    // Still visible in a full listing - soft-deactivate, never delete.
    let all = action_type_service::list(&conn, &ws, Some("Company"), false).unwrap();
    assert!(all.iter().any(|a| a.id == def.id));
    let active_only = action_type_service::list(&conn, &ws, Some("Company"), true).unwrap();
    assert!(!active_only.iter().any(|a| a.id == def.id));
}

#[test]
fn an_action_type_cannot_target_a_read_only_builtin_object() {
    let (conn, ws, admin) = setup_workspace();
    let result = action_type_service::create(&conn, &ws, &action_type_input("Opportunity", "update", vec![], "set_status", r#"{"target_status":"Won"}"#), Some(&admin));
    assert!(result.is_err(), "Opportunity is read-only through the generic record API, so no Action Type can target it yet");
}
