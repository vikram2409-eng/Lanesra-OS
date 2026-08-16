//! List-view filtering (roadmap "Global search & list-view filtering",
//! second half): exercises `custom_field_service::get_filterable_values`,
//! the bulk fetch a desktop list screen calls once per load to drive its
//! filter controls - confirms it's scoped to is_filterable + active
//! definitions of the right entity type/workspace only, and that it
//! reflects the same "no value stored means the key is absent" semantics
//! as `get_entity_values`.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, custom_field_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "List Filtering Co".into(),
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
        name: name.into(),
        status: "Prospect".into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
        ..Default::default()
    }
}

fn select_field(entity_type: &str, label: &str, is_filterable: bool) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: entity_type.into(),
        label: label.into(),
        field_type: "select".into(),
        options: vec!["Gold".into(), "Silver".into()],
        required: false,
        show_in_list: false,
        sort_order: 0,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable: false,
        is_filterable,
        is_reportable: false,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}

#[test]
fn returns_values_only_for_is_filterable_fields() {
    let (conn, ws, admin) = setup_workspace();
    let tier = custom_field_service::create_definition(&conn, &ws, &select_field("Company", "Tier", true), Some(&admin)).unwrap();
    let internal_code = custom_field_service::create_definition(&conn, &ws, &select_field("Company", "Internal Code", false), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert(tier.key.clone(), "Gold".into());
    values.insert(internal_code.key.clone(), "Silver".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();

    let filterable = custom_field_service::get_filterable_values(&conn, &ws, "Company").unwrap();
    let company_values = filterable.get(&company.id).expect("company has at least one filterable value");
    assert_eq!(company_values.get(&tier.key).map(String::as_str), Some("Gold"));
    assert!(!company_values.contains_key(&internal_code.key), "a non-filterable field's value must never be returned");
}

#[test]
fn a_record_with_no_filterable_values_is_absent_from_the_map() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(&conn, &ws, &select_field("Company", "Tier", true), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Untagged Co"), Some(&admin)).unwrap();

    let filterable = custom_field_service::get_filterable_values(&conn, &ws, "Company").unwrap();
    assert!(!filterable.contains_key(&company.id));
}

#[test]
fn is_scoped_to_the_requested_entity_type() {
    let (conn, ws, admin) = setup_workspace();
    let tier = custom_field_service::create_definition(&conn, &ws, &select_field("Company", "Tier", true), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();
    let mut values = HashMap::new();
    values.insert(tier.key.clone(), "Gold".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();

    let contact_filterable = custom_field_service::get_filterable_values(&conn, &ws, "Contact").unwrap();
    assert!(contact_filterable.is_empty(), "a Company's filterable values must never leak into a Contact query");
}
