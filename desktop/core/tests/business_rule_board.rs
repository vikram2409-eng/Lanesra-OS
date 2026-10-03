//! UX/UI Modernization, Business Rule Board 2.0 (issue #194): the new
//! IF/ELSE IF/ELSE visual chain (`create_rule_branch`) and the one
//! deliberate evaluation change it needed - an "else" role rule's
//! conditions list is allowed to be empty, and empty conditions always
//! match. Everything else (`domain::conditions::conditions_match`, the
//! AND/OR/group_id matcher Workflow Automation also uses) is proven
//! unchanged by every pre-existing `business_rules.rs` test still passing
//! unmodified.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput, BusinessRuleUpdate};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::custom_field_repo;
use lanesra_core::services::{business_rule_service, company_service, custom_field_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Business Rule Board Test Co".into(),
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
        ..Default::default()
    }
}

fn text_field_input(label: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "text".into(),
        options: vec![], required: false, show_in_list: false, sort_order: 0,
        min_value: None, max_value: None, max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: None, is_unique: false, help_text: None, placeholder: None,
        is_hidden_by_default: false,
    }
}

fn condition(field_key: &str, operator: &str, value: &str) -> BusinessRuleConditionInput {
    BusinessRuleConditionInput {
        field_source: "builtin".into(), field_key: field_key.into(), operator: operator.into(), value: value.into(),
        compare_field_source: None, compare_field_key: None, group_id: None, relationship_definition_id: None,
    }
}

fn set_value_action(target_key: &str, value: &str) -> BusinessRuleActionInput {
    BusinessRuleActionInput {
        action_type: "set_value".into(), target_field_key: Some(target_key.into()), target_field_source: "custom".into(),
        action_value: Some(value.into()), message: None,
    }
}

fn require_action(target_key: &str) -> BusinessRuleActionInput {
    BusinessRuleActionInput { action_type: "require".into(), target_field_key: Some(target_key.into()), target_field_source: "custom".into(), action_value: None, message: None }
}

fn rule_input(priority: i64, conditions: Vec<BusinessRuleConditionInput>, actions: Vec<BusinessRuleActionInput>) -> BusinessRuleInput {
    BusinessRuleInput {
        app_id: None,
        entity_type: "Company".into(), name: "If branch".into(), description: None, match_type: "all".into(),
        priority, effective_start_date: None, effective_end_date: None, conditions, actions,
    }
}

#[test]
fn create_rule_branch_promotes_the_parent_and_links_the_sibling() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tier"), Some(&admin)).unwrap();
    let parent = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(0, vec![condition("status", "equals", "Active Customer")], vec![set_value_action(&def.key, "Gold")]),
        Some(&admin),
    ).unwrap();
    assert_eq!(parent.branch_group_id, None);
    assert_eq!(parent.branch_role, "if");

    let else_if = business_rule_service::create_rule_branch(&conn, &parent.id, "else_if", Some(&admin)).unwrap();
    assert_eq!(else_if.branch_role, "else_if");
    assert_eq!(else_if.branch_group_id.as_deref(), Some(parent.id.as_str()));
    // one condition seeded, actionable immediately
    assert_eq!(else_if.conditions.len(), 1);
    assert_eq!(else_if.actions.len(), 1);

    // The parent is retroactively promoted to be its own chain's id, the
    // moment the first sibling is added.
    let reloaded_parent = business_rule_service::get_rule(&conn, &parent.id).unwrap().unwrap();
    assert_eq!(reloaded_parent.branch_group_id.as_deref(), Some(parent.id.as_str()));

    let else_branch = business_rule_service::create_rule_branch(&conn, &else_if.id, "else", Some(&admin)).unwrap();
    assert_eq!(else_branch.branch_role, "else");
    assert_eq!(else_branch.branch_group_id.as_deref(), Some(parent.id.as_str()));
    // "else" seeds with zero conditions - allowed only for this role.
    assert_eq!(else_branch.conditions.len(), 0);
}

#[test]
fn create_rule_branch_rejects_the_if_role() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tier"), Some(&admin)).unwrap();
    let parent = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(0, vec![condition("status", "equals", "Active Customer")], vec![set_value_action(&def.key, "Gold")]),
        Some(&admin),
    ).unwrap();
    let err = business_rule_service::create_rule_branch(&conn, &parent.id, "if", Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("else_if' or 'else'"));
}

/// The core new behavior: an "else" rule's empty conditions list always
/// matches, giving true IF/ELSE precedence on top of "last matching rule
/// wins" - `create_rule_branch` assigns every new sibling a priority lower
/// than everything already in its chain (never touching the original "if"
/// rule's own priority), so the "if" rule always ends up running *last*
/// and winning on any field both branches happen to target, exactly
/// matching how the chain reads top-to-bottom.
#[test]
fn an_else_branch_always_matches_but_the_if_branch_still_wins_when_both_match() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tier"), Some(&admin)).unwrap();

    let if_rule = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(0, vec![condition("status", "equals", "Active Customer")], vec![set_value_action(&def.key, "Gold")]),
        Some(&admin),
    ).unwrap();
    let else_rule = business_rule_service::create_rule_branch(&conn, &if_rule.id, "else", Some(&admin)).unwrap();
    assert!(else_rule.priority < if_rule.priority, "the else branch must sort to run before the if rule it was branched from");
    business_rule_service::update_rule(
        &conn, &else_rule.id,
        &BusinessRuleUpdate {
            name: else_rule.name.clone(), description: None, match_type: "all".into(), priority: else_rule.priority, is_active: true,
            effective_start_date: None, effective_end_date: None, app_id: None,
            conditions: vec![],
            actions: vec![set_value_action(&def.key, "Standard")],
        },
        Some(&admin),
    ).unwrap();

    let active_customer = company_service::create(&conn, &ws, &company_input("Acme", "Active Customer"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &active_customer.id, &HashMap::new(), Some(&admin)).unwrap();
    let values = custom_field_repo::get_values(&conn, &active_customer.id).unwrap();
    // Both rules match this record (the else branch unconditionally, the
    // if branch because its own condition holds) - the if branch runs
    // last (higher priority number) and wins, same as the chain's visual
    // if-before-else reading order promises.
    assert_eq!(values.get("tier").map(|s: &String| s.as_str()), Some("Gold"));

    let prospect = company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap();
    let prospect_values = custom_field_repo::get_values(&conn, &prospect.id).unwrap();
    // Only the else branch matches a non-"Active Customer" record.
    assert_eq!(prospect_values.get("tier").map(|s: &String| s.as_str()), Some("Standard"));
}

/// `test_rules` (the board's dry-run panel) sees the same else-always-
/// matches behavior as a real save.
#[test]
fn test_rules_dry_run_sees_the_else_branch_match_with_no_values_at_all() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tier"), Some(&admin)).unwrap();
    let if_rule = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(0, vec![condition("status", "equals", "Active Customer")], vec![set_value_action(&def.key, "Gold")]),
        Some(&admin),
    ).unwrap();
    business_rule_service::create_rule_branch(&conn, &if_rule.id, "else", Some(&admin)).unwrap();

    // The empty context matches neither "status equals Active Customer"
    // (status isn't even set) nor anything else the if branch needs - only
    // the else branch's unconditional match fires, applying its seeded
    // default action (`default_branch_action`: "require" on the one active
    // custom field, since that's all this workspace has).
    let result = business_rule_service::test_rules(&conn, &ws, "Company", &HashMap::new(), Some(&admin)).unwrap();
    assert_eq!(result.field_effects.get("tier").map(|s| s.as_str()), Some("require"));
}

/// Saving an ordinary ("if") rule still requires at least one condition -
/// the relaxation is scoped to "else" rules only, exactly like
/// `validate_conditions`'s own doc comment says.
#[test]
fn an_ordinary_rule_still_rejects_empty_conditions() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tier"), Some(&admin)).unwrap();
    let err = business_rule_service::create_rule(&conn, &ws, &rule_input(0, vec![], vec![set_value_action(&def.key, "Gold")]), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("at least one condition"));

    let if_rule = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(0, vec![condition("status", "equals", "Active Customer")], vec![set_value_action(&def.key, "Gold")]),
        Some(&admin),
    ).unwrap();
    // Editing it down to zero conditions is rejected too, since its
    // branch_role is still "if".
    let err = business_rule_service::update_rule(
        &conn, &if_rule.id,
        &BusinessRuleUpdate {
            name: if_rule.name.clone(), description: None, match_type: "all".into(), priority: 0, is_active: true,
            effective_start_date: None, effective_end_date: None, app_id: None,
            conditions: vec![], actions: vec![set_value_action(&def.key, "Gold")],
        },
        Some(&admin),
    ).unwrap_err();
    assert!(format!("{err:?}").contains("at least one condition"));
}

/// An "else" rule can also be edited to keep zero conditions - the save
/// path that was previously impossible for any rule now succeeds for this
/// one role.
#[test]
fn an_else_rule_can_be_resaved_with_zero_conditions() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Tier"), Some(&admin)).unwrap();
    let if_rule = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(0, vec![condition("status", "equals", "Active Customer")], vec![set_value_action(&def.key, "Gold")]),
        Some(&admin),
    ).unwrap();
    let else_rule = business_rule_service::create_rule_branch(&conn, &if_rule.id, "else", Some(&admin)).unwrap();
    assert_eq!(else_rule.conditions.len(), 0);

    let saved = business_rule_service::update_rule(
        &conn, &else_rule.id,
        &BusinessRuleUpdate {
            name: "Renamed else".into(), description: None, match_type: "all".into(), priority: 1, is_active: true,
            effective_start_date: None, effective_end_date: None, app_id: None,
            conditions: vec![], actions: vec![require_action(&def.key)],
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(saved.conditions.len(), 0);
    assert_eq!(saved.name, "Renamed else");
}
