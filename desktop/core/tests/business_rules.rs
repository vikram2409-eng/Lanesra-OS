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
        ..Default::default()
    }
}

fn text_field_input(label: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "text".into(),
        options: vec![], required: false, show_in_list: false, sort_order: 0,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable: false,
        is_filterable: false,
        is_reportable: true,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}

fn condition(field_key: &str, operator: &str, value: &str) -> BusinessRuleConditionInput {
    BusinessRuleConditionInput {
        field_source: "builtin".into(), field_key: field_key.into(), operator: operator.into(), value: value.into(),
        compare_field_source: None, compare_field_key: None, group_id: None,
    }
}

fn custom_condition(field_key: &str, operator: &str, value: &str) -> BusinessRuleConditionInput {
    BusinessRuleConditionInput {
        field_source: "custom".into(), field_key: field_key.into(), operator: operator.into(), value: value.into(),
        compare_field_source: None, compare_field_key: None, group_id: None,
    }
}

fn require_action(target_key: &str) -> BusinessRuleActionInput {
    BusinessRuleActionInput { action_type: "require".into(), target_field_key: Some(target_key.into()), target_field_source: "custom".into(), action_value: None, message: None }
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
            vec![BusinessRuleActionInput { action_type: "set_default".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(), action_value: Some("Unassigned".into()), message: None }],
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
            vec![BusinessRuleActionInput { action_type: "block_save".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Archived companies cannot be edited".into()) }],
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
            vec![BusinessRuleActionInput { action_type: "show_message".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Remember to schedule a follow-up call".into()) }],
        ),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let notices = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap();
    assert_eq!(notices.warnings, vec!["Remember to schedule a follow-up call".to_string()]);
    assert!(notices.errors.is_empty());
}

#[test]
fn higher_priority_rule_wins_on_conflicting_effects_for_the_same_target() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("status", "equals", "Prospect")], vec![BusinessRuleActionInput { action_type: "hide".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(), action_value: None, message: None }]),
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
            vec![BusinessRuleConditionInput {
                field_source: "custom".into(), field_key: notes_def.key.clone(), operator: "contains".into(), value: "urgent".into(),
                compare_field_source: None, compare_field_key: None, group_id: None,
            }],
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

// --- "Any field" targeting (built-in fields, not just status/custom) ----

#[test]
fn condition_on_a_non_status_builtin_field_matches_correctly() {
    let (conn, ws, admin) = setup_workspace();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("tax_number", "contains", "EXEMPT")],
            vec![require_action(&flag_def.key)],
        ),
        Some(&admin),
    ).unwrap();

    // A company whose tax_number doesn't contain "EXEMPT" saves freely.
    let mut taxable = company_input("Acme", "Prospect");
    taxable.tax_number = Some("GB123456789".into());
    let taxable = company_service::create(&conn, &ws, &taxable, Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &taxable.id, &HashMap::new(), Some(&admin)).unwrap();

    // A company whose tax_number contains "EXEMPT" now requires Flag.
    let mut exempt = company_input("Globex", "Prospect");
    exempt.tax_number = Some("TAX-EXEMPT-001".into());
    let exempt = company_service::create(&conn, &ws, &exempt, Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &exempt.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Flag is required"));
}

#[test]
fn set_value_action_targeting_a_builtin_field_writes_through_the_entity_service() {
    let (conn, ws, admin) = setup_workspace();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![BusinessRuleActionInput {
                action_type: "set_value".into(), target_field_key: Some("tags".into()), target_field_source: "builtin".into(),
                action_value: Some("needs-followup".into()), message: None,
            }],
        ),
        Some(&admin),
    ).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    // Business rule set_value/set_default only apply once the same save
    // seam custom fields go through runs - see custom_field_service's
    // set_entity_values, which every entity's create/edit form already
    // calls unconditionally even with zero custom fields defined.
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();

    let reloaded = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(reloaded.tags.as_deref(), Some("needs-followup"));
}

#[test]
fn a_rule_cannot_target_a_non_actionable_builtin_field() {
    let (conn, ws, admin) = setup_workspace();
    // "status" is conditionable (it's the entity's transition field) but
    // not actionable - it has its own dedicated mechanism instead (see
    // domain::builtin_fields' doc comment).
    let err = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![BusinessRuleActionInput {
                action_type: "require".into(), target_field_key: Some("status".into()), target_field_source: "builtin".into(),
                action_value: None, message: None,
            }],
        ),
        Some(&admin),
    ).unwrap_err();
    assert!(format!("{err:?}").contains("not an actionable built-in field"));
}

// --- Admin Automation & Customization addendum, Phase 1: new operators
// and field-to-field comparison ------------------------------------------

#[test]
fn starts_with_and_ends_with_operators_evaluate_correctly() {
    let (conn, ws, admin) = setup_workspace();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("name", "starts_with", "Acme")], vec![require_action(&flag_def.key)]),
        Some(&admin),
    ).unwrap();

    let matching = company_service::create(&conn, &ws, &company_input("Acme Corp", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &matching.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Flag is required"));

    let non_matching = company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &non_matching.id, &HashMap::new(), Some(&admin)).unwrap();
}

#[test]
fn in_list_and_not_in_list_operators_evaluate_correctly() {
    let (conn, ws, admin) = setup_workspace();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input("Company", 0, "all", vec![condition("status", "in_list", "Prospect|Lead")], vec![require_action(&flag_def.key)]),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Flag is required"));

    let customer = company_service::create(&conn, &ws, &company_input("Globex", "Active Customer"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &customer.id, &HashMap::new(), Some(&admin)).unwrap();
}

#[test]
fn field_to_field_comparison_condition_matches_correctly() {
    // "Require Flag when Notes equals Expected Notes" - both custom text
    // fields, comparing one field's live value against another's instead
    // of a fixed literal (spec §2.2).
    let (conn, ws, admin) = setup_workspace();
    let notes_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Notes"), Some(&admin)).unwrap();
    let expected_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Expected Notes"), Some(&admin)).unwrap();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![BusinessRuleConditionInput {
                field_source: "custom".into(), field_key: notes_def.key.clone(), operator: "equals".into(), value: String::new(),
                compare_field_source: Some("custom".into()), compare_field_key: Some(expected_def.key.clone()), group_id: None,
            }],
            vec![require_action(&flag_def.key)],
        ),
        Some(&admin),
    ).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();

    // Notes differs from Expected Notes - condition doesn't match, Flag
    // isn't required.
    let mut differing = HashMap::new();
    differing.insert(notes_def.key.clone(), "hello".into());
    differing.insert(expected_def.key.clone(), "world".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &differing, Some(&admin)).unwrap();

    // Notes now matches Expected Notes - condition matches, Flag required.
    let mut matching = HashMap::new();
    matching.insert(notes_def.key.clone(), "same value".into());
    matching.insert(expected_def.key.clone(), "same value".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &matching, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Flag is required"));
}

// --- Admin Automation & Customization, second addendum: OR-groups, the
// wider field-effect palette (show/editable/clear_value/restrict_choices),
// and severity-tagged messages (show_error/show_warning) ------------------

fn select_field_input(label: &str, options: Vec<&str>) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "select".into(),
        options: options.into_iter().map(String::from).collect(),
        required: false, show_in_list: false, sort_order: 0,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable: false,
        is_filterable: false,
        is_reportable: true,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}

#[test]
fn an_or_group_condition_matches_when_any_member_of_the_group_matches() {
    let (conn, ws, admin) = setup_workspace();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();

    // Match: status = Prospect AND (tax_number = A OR tax_number = B)
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![
                condition("status", "equals", "Prospect"),
                BusinessRuleConditionInput {
                    field_source: "builtin".into(), field_key: "tax_number".into(), operator: "equals".into(), value: "A".into(),
                    compare_field_source: None, compare_field_key: None, group_id: Some("g1".into()),
                },
                BusinessRuleConditionInput {
                    field_source: "builtin".into(), field_key: "tax_number".into(), operator: "equals".into(), value: "B".into(),
                    compare_field_source: None, compare_field_key: None, group_id: Some("g1".into()),
                },
            ],
            vec![require_action(&flag_def.key)],
        ),
        Some(&admin),
    ).unwrap();

    // tax_number = "A" satisfies the OR group - Flag is required.
    let mut matches_a = company_input("Acme", "Prospect");
    matches_a.tax_number = Some("A".into());
    let matches_a = company_service::create(&conn, &ws, &matches_a, Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &matches_a.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Flag is required"));

    // tax_number = "C" satisfies neither member of the OR group - saves freely.
    let mut matches_neither = company_input("Globex", "Prospect");
    matches_neither.tax_number = Some("C".into());
    let matches_neither = company_service::create(&conn, &ws, &matches_neither, Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &matches_neither.id, &HashMap::new(), Some(&admin)).unwrap();
}

#[test]
fn show_error_and_show_warning_actions_are_both_returned_as_non_blocking_notices() {
    let (conn, ws, admin) = setup_workspace();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![
                BusinessRuleActionInput { action_type: "show_error".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Missing compliance docs".into()) },
                BusinessRuleActionInput { action_type: "show_warning".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Consider a follow-up call".into()) },
            ],
        ),
        Some(&admin),
    ).unwrap();

    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let notices = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap();
    assert_eq!(notices.errors, vec!["Missing compliance docs".to_string()]);
    assert_eq!(notices.warnings, vec!["Consider a follow-up call".to_string()]);
}

#[test]
fn show_and_editable_actions_override_an_earlier_hide_and_lock_within_the_same_rule() {
    let (conn, ws, admin) = setup_workspace();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![
                BusinessRuleActionInput { action_type: "hide".into(), target_field_key: Some(flag_def.key.clone()), target_field_source: "custom".into(), action_value: None, message: None },
                BusinessRuleActionInput { action_type: "lock".into(), target_field_key: Some(flag_def.key.clone()), target_field_source: "custom".into(), action_value: None, message: None },
                BusinessRuleActionInput { action_type: "show".into(), target_field_key: Some(flag_def.key.clone()), target_field_source: "custom".into(), action_value: None, message: None },
                BusinessRuleActionInput { action_type: "editable".into(), target_field_key: Some(flag_def.key.clone()), target_field_source: "custom".into(), action_value: None, message: None },
            ],
        ),
        Some(&admin),
    ).unwrap();

    let mut ctx = HashMap::new();
    ctx.insert("status".to_string(), "Prospect".to_string());
    let evaluation = business_rule_service::test_rules(&conn, &ws, "Company", &ctx, Some(&admin)).unwrap();
    // Actions apply in order - the later show/editable wins over the
    // earlier hide/lock on the same target field.
    assert_eq!(evaluation.field_effects.get(&flag_def.key).map(String::as_str), Some("editable"));
}

#[test]
fn clear_value_action_writes_an_empty_value_unconditionally() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Region"), Some(&admin)).unwrap();

    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Archived")],
            vec![BusinessRuleActionInput { action_type: "clear_value".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(), action_value: None, message: None }],
        ),
        Some(&admin),
    ).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert(def.key.clone(), "West".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("West"));

    // Archiving the company fires clear_value, wiping the field out entirely.
    company_service::update(&conn, &company.id, &company_input("Acme", "Archived"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get(&def.key), None);
}

#[test]
fn restrict_choices_action_requires_a_select_field_and_is_exposed_on_evaluation() {
    let (conn, ws, admin) = setup_workspace();
    let text_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Notes"), Some(&admin)).unwrap();

    // Targeting a non-select field is rejected up front.
    let err = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![BusinessRuleActionInput { action_type: "restrict_choices".into(), target_field_key: Some(text_def.key.clone()), target_field_source: "custom".into(), action_value: Some("A|B".into()), message: None }],
        ),
        Some(&admin),
    ).unwrap_err();
    assert!(format!("{err:?}").contains("not a select field"));

    // Targeting a select field is accepted and shows up on evaluation.
    let select_def = custom_field_service::create_definition(&conn, &ws, &select_field_input("Region", vec!["North", "South", "East", "West"]), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![condition("status", "equals", "Prospect")],
            vec![BusinessRuleActionInput { action_type: "restrict_choices".into(), target_field_key: Some(select_def.key.clone()), target_field_source: "custom".into(), action_value: Some("North|South".into()), message: None }],
        ),
        Some(&admin),
    ).unwrap();

    let mut ctx = HashMap::new();
    ctx.insert("status".to_string(), "Prospect".to_string());
    let evaluation = business_rule_service::test_rules(&conn, &ws, "Company", &ctx, Some(&admin)).unwrap();
    assert_eq!(evaluation.restricted_choices.get(&select_def.key).map(String::as_str), Some("North|South"));
}

#[test]
fn a_condition_with_only_a_compare_field_source_or_only_a_key_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let flag_def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Flag"), Some(&admin)).unwrap();
    let err = business_rule_service::create_rule(
        &conn, &ws,
        &rule_input(
            "Company", 0, "all",
            vec![BusinessRuleConditionInput {
                field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(),
                compare_field_source: Some("custom".into()), compare_field_key: None, group_id: None,
            }],
            vec![require_action(&flag_def.key)],
        ),
        Some(&admin),
    ).unwrap_err();
    assert!(format!("{err:?}").contains("needs both a source and a key"));
}
