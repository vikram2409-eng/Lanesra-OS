//! Admin Automation & Customization addendum, Phase 4 (spec §4): four more
//! settings on a custom field definition - default_value, is_unique,
//! help_text, placeholder. help_text/placeholder are presentation-only
//! (no server behavior to test beyond round-tripping); default_value and
//! is_unique are enforced in custom_field_service::set_entity_values,
//! covered here.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{business_rule_service, company_service, custom_field_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Custom Field Extensibility Co".into(),
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
    CompanyInput {
        name: name.into(), status: "Prospect".into(), owner_user_id: None, tax_number: None,
        billing_address: None, shipping_address: None, tags: None, notes: None,
        ..Default::default()
    }
}

fn text_field(label: &str, default_value: Option<&str>, is_unique: bool) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "text".into(), options: vec![],
        required: false, show_in_list: false, sort_order: 0,
        min_value: None, max_value: None, max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: default_value.map(String::from), is_unique, help_text: None, placeholder: None, is_hidden_by_default: false,
    }
}

#[test]
fn default_value_is_applied_when_a_save_omits_the_field() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field("Source", Some("Website"), false), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("Website"));
}

#[test]
fn default_value_does_not_override_an_explicitly_provided_value() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field("Source", Some("Website"), false), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert(def.key.clone(), "Referral".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("Referral"));
}

#[test]
fn a_business_rules_set_default_still_wins_over_the_fields_own_default_value() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field("Source", Some("Website"), false), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            app_id: None,
            entity_type: "Company".into(), name: "Prefer referral".into(), description: None, match_type: "all".into(),
            priority: 0, effective_start_date: None, effective_end_date: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(),
                compare_field_source: None, compare_field_key: None, group_id: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "set_default".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(),
                action_value: Some("Referral Program".into()), message: None,
            }],
        },
        Some(&admin),
    ).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    // The business rule's own set_default (evaluated against the raw, still-
    // empty context) is applied after the field-level default, so it wins.
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("Referral Program"));
}

#[test]
fn is_unique_rejects_a_value_already_used_by_a_different_record() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field("External ID", None, true), Some(&admin)).unwrap();
    let acme = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();
    let globex = company_service::create(&conn, &ws, &company_input("Globex"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert(def.key.clone(), "EXT-001".into());
    custom_field_service::set_entity_values(&conn, "Company", &acme.id, &values, Some(&admin)).unwrap();

    let err = custom_field_service::set_entity_values(&conn, "Company", &globex.id, &values, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("must be unique"));
}

#[test]
fn is_unique_allows_resaving_the_same_record_with_its_own_value_unchanged() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field("External ID", None, true), Some(&admin)).unwrap();
    let acme = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert(def.key.clone(), "EXT-001".into());
    custom_field_service::set_entity_values(&conn, "Company", &acme.id, &values, Some(&admin)).unwrap();
    // Re-saving the exact same record with the exact same value must not
    // trip over its own existing row.
    custom_field_service::set_entity_values(&conn, "Company", &acme.id, &values, Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &acme.id).unwrap();
    assert_eq!(stored.get(&def.key).map(String::as_str), Some("EXT-001"));
}

#[test]
fn is_unique_is_rejected_at_definition_time_for_a_boolean_field() {
    let (conn, ws, admin) = setup_workspace();
    let input = CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: "Flag".into(), field_type: "boolean".into(), options: vec![],
        required: false, show_in_list: false, sort_order: 0,
        min_value: None, max_value: None, max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: None, is_unique: true, help_text: None, placeholder: None, is_hidden_by_default: false,
    };
    let err = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("doesn't apply to a yes/no field"));
}

#[test]
fn default_value_is_validated_against_the_fields_own_type_at_definition_time() {
    let (conn, ws, admin) = setup_workspace();
    let input = CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: "Employee Count".into(), field_type: "number".into(), options: vec![],
        required: false, show_in_list: false, sort_order: 0,
        min_value: Some("1".into()), max_value: Some("500".into()), max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: Some("not-a-number".into()), is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
    };
    let err = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("must be a number"));
}

#[test]
fn help_text_and_placeholder_round_trip_through_create_and_update() {
    let (conn, ws, admin) = setup_workspace();
    let mut input = text_field("Notes", None, false);
    input.help_text = Some("Shown under the field".into());
    input.placeholder = Some("e.g. VIP account".into());
    let def = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(def.help_text.as_deref(), Some("Shown under the field"));
    assert_eq!(def.placeholder.as_deref(), Some("e.g. VIP account"));

    let update = lanesra_core::models::custom_field::CustomFieldDefinitionUpdate {
        label: def.label.clone(), options: def.options.clone(), required: def.required, show_in_list: def.show_in_list,
        sort_order: def.sort_order, is_active: def.is_active, min_value: def.min_value.clone(), max_value: def.max_value.clone(),
        max_length: def.max_length, regex_pattern: def.regex_pattern.clone(), is_searchable: def.is_searchable,
        is_filterable: def.is_filterable, is_reportable: def.is_reportable,
        default_value: Some("Default note".into()), is_unique: false,
        help_text: Some("Updated help".into()), placeholder: Some("Updated placeholder".into()),
        is_hidden_by_default: false,
    };
    let updated = custom_field_service::update_definition(&conn, &def.id, &update, Some(&admin)).unwrap();
    assert_eq!(updated.default_value.as_deref(), Some("Default note"));
    assert_eq!(updated.help_text.as_deref(), Some("Updated help"));
    assert_eq!(updated.placeholder.as_deref(), Some("Updated placeholder"));
}

// --- Admin Automation & Customization, second addendum: a field hidden by
// default is skipped even when required, unless a business rule's "show"
// action currently targets it --------------------------------------------

#[test]
fn a_field_hidden_by_default_is_skipped_unless_a_rule_shows_it() {
    let (conn, ws, admin) = setup_workspace();
    let input = CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: "VIP Tier".into(), field_type: "text".into(), options: vec![],
        required: true, show_in_list: false, sort_order: 0,
        min_value: None, max_value: None, max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: None, is_unique: false, help_text: None, placeholder: None,
        is_hidden_by_default: true,
    };
    let def = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    // Hidden by default overrides required - an empty save still succeeds.
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();

    // A business rule's "show" action un-hides the field - now required
    // actually applies, and the same empty save is rejected.
    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            app_id: None,
            entity_type: "Company".into(), name: "Reveal VIP tier".into(), description: None, match_type: "all".into(),
            priority: 0, effective_start_date: None, effective_end_date: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(),
                compare_field_source: None, compare_field_key: None, group_id: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "show".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(),
                action_value: None, message: None,
            }],
        },
        Some(&admin),
    ).unwrap();

    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("VIP Tier is required"));
}
