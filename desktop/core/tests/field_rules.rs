use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::field_rule::FieldRuleInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, custom_field_service, field_rule_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Field Rule Test Co".into(),
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
        name: name.into(),
        status: status.into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
    }
}

fn text_field_input(label: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(),
        label: label.into(),
        field_type: "text".into(),
        options: vec![],
        required: false,
        show_in_list: false,
        sort_order: 0,
    }
}

fn require_when_prospect_rule(target_key: &str) -> FieldRuleInput {
    FieldRuleInput {
        entity_type: "Company".into(),
        trigger_field_source: "builtin".into(),
        trigger_field_key: "status".into(),
        operator: "equals".into(),
        trigger_value: "Prospect".into(),
        target_field_key: target_key.into(),
        effect: "require".into(),
        sort_order: 0,
    }
}

#[test]
fn rule_requires_field_only_when_the_trigger_condition_matches() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    field_rule_service::create_rule(&conn, &ws, &require_when_prospect_rule(&def.key), Some(&admin)).unwrap();

    // A Prospect company must have Lead Source filled in.
    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Lead Source is required"));

    let mut values = HashMap::new();
    values.insert("lead_source".to_string(), "Trade show".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &values, Some(&admin)).unwrap();

    // An Active Customer is not subject to the rule - no value needed.
    let customer = company_service::create(&conn, &ws, &company_input("Globex", "Active Customer"), Some(&admin)).unwrap();
    custom_field_service::set_entity_values(&conn, "Company", &customer.id, &HashMap::new(), Some(&admin)).unwrap();
}

#[test]
fn non_administrator_cannot_define_a_business_rule() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    let sales_user = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    let err = field_rule_service::create_rule(&conn, &ws, &require_when_prospect_rule(&def.key), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn rule_target_must_be_an_active_custom_field() {
    let (conn, ws, admin) = setup_workspace();
    let mut rule = require_when_prospect_rule("does_not_exist");
    rule.target_field_key = "does_not_exist".into();
    let err = field_rule_service::create_rule(&conn, &ws, &rule, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("not an active custom field to target"));
}

#[test]
fn hidden_field_is_never_required_even_if_flagged_required_statically() {
    let (conn, ws, admin) = setup_workspace();
    let mut input = text_field_input("Referral Code");
    input.required = true; // statically required...
    let def = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap();

    // ...but hidden whenever the company is Archived.
    let hide_rule = FieldRuleInput {
        entity_type: "Company".into(),
        trigger_field_source: "builtin".into(),
        trigger_field_key: "status".into(),
        operator: "equals".into(),
        trigger_value: "Archived".into(),
        target_field_key: def.key.clone(),
        effect: "hide".into(),
        sort_order: 0,
    };
    field_rule_service::create_rule(&conn, &ws, &hide_rule, Some(&admin)).unwrap();

    let archived = company_service::create(&conn, &ws, &company_input("Old Co", "Archived"), Some(&admin)).unwrap();
    // Would fail if `required` were still enforced for a hidden field.
    custom_field_service::set_entity_values(&conn, "Company", &archived.id, &HashMap::new(), Some(&admin)).unwrap();

    // A non-Archived company is still bound by the static `required` flag.
    let active = company_service::create(&conn, &ws, &company_input("Active Co", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &active.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Referral Code is required"));
}

#[test]
fn later_sort_order_rule_wins_on_conflicting_effects_for_the_same_target() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();

    let mut hide_first = require_when_prospect_rule(&def.key);
    hide_first.effect = "hide".into();
    hide_first.sort_order = 0;
    field_rule_service::create_rule(&conn, &ws, &hide_first, Some(&admin)).unwrap();

    let mut require_second = require_when_prospect_rule(&def.key);
    require_second.sort_order = 1;
    field_rule_service::create_rule(&conn, &ws, &require_second, Some(&admin)).unwrap();

    // The higher-sort_order "require" rule should win over the "hide" one.
    let prospect = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let err = custom_field_service::set_entity_values(&conn, "Company", &prospect.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Lead Source is required"));
}
