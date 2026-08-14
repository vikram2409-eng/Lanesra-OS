use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, custom_field_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Custom Field Test Co".into(),
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

fn select_field_input(label: &str, required: bool) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(),
        label: label.into(),
        field_type: "select".into(),
        options: vec!["Retail".into(), "Manufacturing".into(), "Services".into()],
        required,
        show_in_list: false,
        sort_order: 0,
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

fn number_field_input(label: &str, min: Option<&str>, max: Option<&str>) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "number".into(), options: vec![],
        required: false, show_in_list: false, sort_order: 0,
        min_value: min.map(String::from), max_value: max.map(String::from), max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: None, is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
    }
}

fn text_field_input(label: &str, max_length: Option<i64>, regex_pattern: Option<&str>) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "text".into(), options: vec![],
        required: false, show_in_list: false, sort_order: 0,
        min_value: None, max_value: None, max_length, regex_pattern: regex_pattern.map(String::from),
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: None, is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
    }
}

#[test]
fn number_field_enforces_min_and_max() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &number_field_input("Employee Count", Some("1"), Some("500")), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut too_low = HashMap::new();
    too_low.insert(def.key.clone(), "0".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &too_low, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("must be at least 1"));

    let mut too_high = HashMap::new();
    too_high.insert(def.key.clone(), "501".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &too_high, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("must be at most 500"));

    let mut ok = HashMap::new();
    ok.insert(def.key.clone(), "250".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &ok, Some(&admin)).unwrap();
}

#[test]
fn min_cannot_exceed_max_when_defining_a_number_field() {
    let (conn, ws, admin) = setup_workspace();
    let err = custom_field_service::create_definition(&conn, &ws, &number_field_input("Bad Range", Some("100"), Some("1")), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("cannot be greater than"));
}

#[test]
fn text_field_enforces_max_length_and_pattern() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Reference Code", Some(6), Some(r"^[A-Z]{2}-\d{3}$")), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut too_long = HashMap::new();
    too_long.insert(def.key.clone(), "AB-1234".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &too_long, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("6 characters or fewer"));

    let mut bad_pattern = HashMap::new();
    bad_pattern.insert(def.key.clone(), "abcdef".into());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &bad_pattern, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("does not match"));

    let mut ok = HashMap::new();
    ok.insert(def.key.clone(), "AB-123".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &ok, Some(&admin)).unwrap();
}

#[test]
fn an_invalid_regex_pattern_is_rejected_at_definition_time() {
    let (conn, ws, admin) = setup_workspace();
    let err = custom_field_service::create_definition(&conn, &ws, &text_field_input("Bad Pattern", None, Some("(unclosed")), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("not a valid regular expression"));
}

#[test]
fn a_field_flagged_not_reportable_cannot_be_used_as_a_report_group_by() {
    let (conn, ws, admin) = setup_workspace();
    let mut input = select_field_input("Internal Only", false);
    input.is_reportable = false;
    let def = custom_field_service::create_definition(&conn, &ws, &input, Some(&admin)).unwrap();

    let err = lanesra_core::services::custom_report_service::create(
        &conn, &ws,
        &lanesra_core::models::custom_report::CustomReportInput {
            name: "Should fail".into(), entity_type: "Company".into(), group_by_source: "custom".into(),
            group_by_field: def.key.clone(), aggregate: "count".into(), sum_field_key: None,
        },
        Some(&admin),
    ).unwrap_err();
    assert!(format!("{err:?}").contains("not an active custom field to group by"));
}

#[test]
fn administrator_can_define_a_custom_field_with_an_auto_generated_key() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry Type!", false), Some(&admin)).unwrap();
    assert_eq!(def.key, "industry_type");
    assert_eq!(def.options, vec!["Retail", "Manufacturing", "Services"]);
    assert!(def.is_active);
}

#[test]
fn duplicate_labels_get_a_uniquified_key_not_a_rejection() {
    let (conn, ws, admin) = setup_workspace();
    let first = custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", false), Some(&admin)).unwrap();
    let second = custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", false), Some(&admin)).unwrap();
    assert_eq!(first.key, "industry");
    assert_eq!(second.key, "industry_2");
}

#[test]
fn non_administrator_cannot_define_a_custom_field() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    let err = custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", false), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn required_custom_field_blocks_save_when_missing_and_succeeds_when_present() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", true), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let empty_values: HashMap<String, String> = HashMap::new();
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &empty_values, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Industry is required"));

    let mut values = HashMap::new();
    values.insert("industry".to_string(), "Retail".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();

    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get("industry"), Some(&"Retail".to_string()));
}

#[test]
fn select_value_must_be_one_of_the_defined_options() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", false), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("industry".to_string(), "Not A Real Option".to_string());
    let err = custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("not a valid option"));
}

#[test]
fn clearing_a_value_removes_the_stored_row() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", false), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("industry".to_string(), "Retail".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();
    assert!(custom_field_service::get_entity_values(&conn, &company.id).unwrap().contains_key("industry"));

    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();
    assert!(!custom_field_service::get_entity_values(&conn, &company.id).unwrap().contains_key("industry"));
}

#[test]
fn deactivating_a_field_stops_enforcing_it_but_keeps_existing_values() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &select_field_input("Industry", true), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert("industry".to_string(), "Retail".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();

    custom_field_service::deactivate_definition(&conn, &def.id, Some(&admin)).unwrap();

    // No longer required (it's inactive), and its value is untouched.
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &HashMap::new(), Some(&admin)).unwrap();
    let stored = custom_field_service::get_entity_values(&conn, &company.id).unwrap();
    assert_eq!(stored.get("industry"), Some(&"Retail".to_string()));

    let active = custom_field_service::list_definitions(&conn, &ws, "Company", true).unwrap();
    assert!(active.is_empty());
    let all = custom_field_service::list_definitions(&conn, &ws, "Company", false).unwrap();
    assert_eq!(all.len(), 1);
}
