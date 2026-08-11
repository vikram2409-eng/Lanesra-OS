//! Admin extensibility Phase C (spec §22/ADM-BR): the richer IF (AND/OR) /
//! THEN business rule engine that replaced the original single-condition
//! field_rules (require/hide only) - multiple conditions, more operators,
//! and actions beyond require/hide (lock, set default/value, block save
//! with a custom message, show a non-blocking message).

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{business_rule_service, company_service, custom_field_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Business Rule Test Co".into(),
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

fn text_field_input(label: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "text".into(),
        options: vec![], required: false, show_in_list: false, sort_order: 0,
    }
}

fn condition(field_key: &str, operator: &str, value: &str) -> BusinessRuleConditionInput {
    BusinessRuleConditionInput { field_source: "builtin".into(), field_key: field_key.into(), operator: operator.into(), value: value.into() }
}

fn custom_condition(field_key: &str, operator: &str, value: &str) -> BusinessRuleConditionInput {
    BusinessRuleConditionInput { field_source: "custom".into(), field_key: field_key.into(), operator: operator.into(), value: value.into() }
}

fn require_action(target_key: &str) -> BusinessRuleActionInput {
    BusinessRuleActionInput { action_type: "require".into(), target_field_key: Some(target_key.into()), action_value: None, message: None }
}

fn rule_input(entity_type: &str, priority: i64, match_type: &str, conditions: Vec<BusinessRuleConditionInput>, actions: Vec<BusinessRuleActionInput>) -> BusinessRuleInput {
    BusinessRuleInput {
        entity_type: entity_type.into(), name: "Test rule".into(), description: None, match_type: match_type.into(),
        priority, effective_start_date: None, effective_end_date: None, conditions, actions,
    }
}

#[test]
fn rule_requires_field_only_when_the_condition_matches() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("status", "equals", "Prospect")], vec![require_action(&def.key)]),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Lead Source is required"));

    let mut values = HashMap::new();
    values.insert("lead_source".to_string(), "Trade show".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &values, Some(&admin)).unwrap();

    let customer = company_service::create(&conn, &ws, &company_input("Globex", "Active Customer"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &customer.id, &HashMap::new(), Some(&admin)).unwrap();
}

#[test]
fn non_administrator_cannot_define_a_business_rule() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();

    let err = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("status", "equals", "Prospect")], vec![require_action(&def.key)]),
        Some(&sales_user.id),
    ).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn a_rule_needs_at_least_one_condition_and_one_action() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();

    let no_conditions = business_rule_service::create_rule(&conn, &ws, &rule_input("Company", 0, "all", vec![], vec![require_action(&def.key)]), Some(&admin));
    assert!(no_conditions.is_err());

    let no_actions = business_rule_service::create_rule(&conn, &ws, &rule_input("Company", 0, "all", vec![condition("status", "equals", "Prospect")], vec![]), Some(&admin));
    assert!(no_actions.is_err());
}

#[test]
fn any_match_type_fires_when_at_least_one_condition_matches() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Escalation Note"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "any",
            vec![condition("status", "equals", "Prospect"), condition("status", "equals", "Inactive")],
            vec![require_action(&def.key)],
        ),
        Some(&admin),
    ).unwrap();

    let inactive = company_service::create(&conn, &ws, &company_input("Old Co", "Inactive"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &inactive.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Escalation Note is required"));

    let active = company_service::create(&conn, &ws, &company_input("Active Co", "Active Customer"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &active.id, &HashMap::new(), Some(&admin)).unwrap();
}

#[test]
fn all_match_type_requires_every_condition_to_match() {
    let (conn, ws, admin) = setup_workspace();
    let tag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tag"), Some(&admin)).unwrap();
    let escalation_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Escalation Note"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect"), custom_condition(&tag_def.key, "is_not_empty", "")],
            vec![require_action(&escalation_def.key)],
        ),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    // Tag is empty, so only one of the two AND-ed conditions matches -
    // Escalation Note should not be required yet.
    custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap();

    // Filling in Tag satisfies both AND-ed conditions - now it's required.
    let mut values = HashMap::new();
    values.insert(tag_def.key.clone(), "VIP".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &values, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Escalation Note is required"));
}

#[test]
fn set_default_only_fills_an_empty_value_while_set_value_always_overwrites() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Region"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![BusinessRuleActionInput { action_type: "set_default".into(), target_field_key: Some(def.key.clone()), action_value: Some("Unassigned".into()), message: None }],
        ),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &prospect.id).unwrap();
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("Unassigned"));

    // An explicitly-provided value is not overwritten by set_default.
    let mut values = HashMap::new();
    values.insert(def.key.clone(), "West".into());
    custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &values, Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &prospect.id).unwrap();
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("West"));
}

#[test]
fn block_save_rejects_the_whole_save_with_the_rules_custom_message() {
    let (conn, ws, admin) = setup_workspace();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Archived")],
            vec![BusinessRuleActionInput { action_type: "block_save".into(), target_field_key: None, action_value: None, message: Some("Archived companies cannot be edited".into()) }],
        ),
        Some(&admin),
    ).unwrap();

    let archived = company_service::create(&conn, &ws, &company_input("Old Co", "Archived"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &archived.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Archived companies cannot be edited"));
}

#[test]
fn show_message_is_returned_but_does_not_block_the_save() {
    let (conn, ws, admin) = setup_workspace();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![BusinessRuleActionInput { action_type: "show_message".into(), target_field_key: None, action_value: None, message: Some("Remember to schedule a follow-up call".into()) }],
        ),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let messages = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap();
    assert_eq!(messages, vec!["Remember to schedule a follow-up call".to_string()]);
}

#[test]
fn higher_priority_rule_wins_on_conflicting_effects_for_the_same_target() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("status", "equals", "Prospect")], vec![BusinessRuleActionInput { action_type: "hide".into(), target_field_key: Some(def.key.clone()), action_value: None, message: None }]),
        Some(&admin),
    ).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 1, "all", vec![condition("status", "equals", "Prospect")], vec![require_action(&def.key)]),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Lead Source is required"));
}

#[test]
fn test_rules_evaluates_a_hypothetical_context_without_persisting_anything() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("status", "equals", "Prospect")], vec![require_action(&def.key)]),
        Some(&admin),
    ).unwrap();

    let mut ctx = HashMap::new();
    ctx.insert("status".to_string(), "Prospect".to_string());
    let evaluation = business_rule_service::test_rules(&conn, &ws, "Company", &ctx, Some(&admin)).unwrap();
    assert_eq!(evaluation.field_effects.get(&def.key).map(String::as_str), Some("require"));

    // A non-admin cannot use test mode either.
    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();
    assert!(business_rule_service::test_rules(&conn, &ws, "Company", &ctx, Some(&sales_user.id)).is_err());
}

#[test]
fn contains_and_numeric_operators_evaluate_correctly() {
    let (conn, ws, admin) = setup_workspace();
    let notes_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Notes"), Some(&admin)).unwrap();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![BusinessRuleConditionInput { field_source: "custom".into(), field_key: notes_def.key.clone(), operator: "contains".into(), value: "urgent".into() }],
            vec![require_action(&flag_def.key)],
        ),
        Some(&admin),
    ).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert(notes_def.key.clone(), "This is an urgent matter".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Flag is required"));
}
