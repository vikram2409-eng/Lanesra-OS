//! Admin extensibility (spec §20.2): an admin-defined custom object gets
//! full CRUD, admin-configurable numbering, and - by reusing the exact
//! same generalized subsystems every built-in entity already goes
//! through - custom fields, business rules and the custom report builder,
//! all without any of those three subsystems needing to know custom
//! objects exist.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::custom_report::CustomReportInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::services::{business_rule_service, custom_field_service, custom_object_service, custom_record_service, custom_report_service, user_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = lanesra_core::models::workspace::WorkspaceSetup {
        business_name: "Custom Objects Co".into(),
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
    CustomObjectDefinitionInput {
        singular_label: "Vendor".into(),
        plural_label: "Vendors".into(),
        icon: "🏭".into(),
        prefix: "VEN".into(),
        digits: 4,
    }
}

#[test]
fn an_administrator_can_define_a_custom_object_and_a_non_admin_cannot() {
    let (conn, ws, admin) = setup_workspace();

    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();
    assert_eq!(vendor.key, "vendor");
    assert_eq!(vendor.singular_label, "Vendor");
    assert!(vendor.is_active);

    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();

    let denied = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&sales_user.id));
    assert!(denied.is_err());
}

#[test]
fn two_objects_with_the_same_name_get_auto_uniquified_keys() {
    let (conn, ws, admin) = setup_workspace();
    let first = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();
    let second = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();
    assert_eq!(first.key, "vendor");
    assert_eq!(second.key, "vendor_2");
}

#[test]
fn records_of_a_custom_object_are_numbered_from_its_own_definition() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();

    let first = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Supplies".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    let second = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Beta Parts".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    assert_eq!(first.display_number, "VEN-0001");
    assert_eq!(second.display_number, "VEN-0002");
    assert_eq!(custom_record_service::list(&conn, &ws, &vendor.key).unwrap().len(), 2);
}

#[test]
fn a_record_cannot_be_created_for_an_unknown_or_deactivated_object() {
    let (conn, ws, admin) = setup_workspace();

    let unknown = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: "not_a_real_object".into(), primary_name: "X".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    );
    assert!(unknown.is_err());

    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();
    custom_object_service::deactivate(&conn, &vendor.id, Some(&admin)).unwrap();
    let after_deactivate = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "X".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    );
    assert!(after_deactivate.is_err());
}

#[test]
fn deleting_an_object_definition_is_blocked_while_records_exist() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();
    custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Supplies".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    let blocked = custom_object_service::delete(&conn, &vendor.id, Some(&admin));
    assert!(blocked.is_err());

    // Deactivating instead is always allowed, and is non-destructive.
    let deactivated = custom_object_service::deactivate(&conn, &vendor.id, Some(&admin)).unwrap();
    assert!(!deactivated.is_active);
    assert_eq!(custom_record_service::list(&conn, &ws, &vendor.key).unwrap().len(), 1);
}

/// The real proof this is a first-class object type: custom fields,
/// business rules and the custom report builder all work on it, composed
/// together, exactly like the existing admin_flexibility.rs test proves
/// for a built-in entity (Opportunity) - none of those three subsystems
/// needed a single line of custom-object-specific logic beyond the one
/// shared `is_valid_dynamic_entity_type` check.
#[test]
fn custom_fields_business_rules_and_reports_all_work_on_a_custom_object() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), Some(&admin)).unwrap();

    let rating_field = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: vendor.key.clone(),
            label: "Preferred".into(),
            field_type: "select".into(),
            options: vec!["Yes".into(), "No".into()],
            required: false,
            show_in_list: false,
            sort_order: 0,
            min_value: None,
            max_value: None,
            max_length: None,
            regex_pattern: None,
            is_searchable: false,
            is_filterable: false,
            is_reportable: true,
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(rating_field.entity_type, vendor.key);

    let contact_email_field = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: vendor.key.clone(),
            label: "Contact Email".into(),
            field_type: "text".into(),
            options: vec![],
            required: false,
            show_in_list: false,
            sort_order: 1,
            min_value: None,
            max_value: None,
            max_length: None,
            regex_pattern: None,
            is_searchable: false,
            is_filterable: false,
            is_reportable: true,
        },
        Some(&admin),
    ).unwrap();

    // Business rule: require Contact Email once status is Inactive.
    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            entity_type: vendor.key.clone(),
            name: "Require Contact Email when Inactive".into(),
            description: None,
            match_type: "all".into(),
            priority: 0,
            effective_start_date: None,
            effective_end_date: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "builtin".into(),
                field_key: "status".into(),
                operator: "equals".into(),
                value: "Inactive".into(),
                compare_field_source: None,
                compare_field_key: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "require".into(),
                target_field_key: Some(contact_email_field.key.clone()),
                target_field_source: "custom".into(),
                action_value: None,
                message: None,
            }],
        },
        Some(&admin),
    ).unwrap();

    let record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Supplies".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    // While Active, the rule doesn't apply - empty custom values are fine.
    custom_field_service::set_entity_values(&conn, &vendor.key, &record.id, &HashMap::new(), Some(&admin)).unwrap();

    // Move to Inactive, then the required rule should reject an empty value.
    custom_record_service::update(
        &conn, &record.id,
        &CustomRecordUpdate { primary_name: record.primary_name.clone(), status: "Inactive".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    let rejected = custom_field_service::set_entity_values(&conn, &vendor.key, &record.id, &HashMap::new(), Some(&admin));
    assert!(rejected.is_err());

    let mut values = HashMap::new();
    values.insert(contact_email_field.key.clone(), "ap@acme.example".into());
    values.insert(rating_field.key.clone(), "Yes".into());
    custom_field_service::set_entity_values(&conn, &vendor.key, &record.id, &values, Some(&admin)).unwrap();

    let stored = custom_field_service::get_entity_values(&conn, &record.id).unwrap();
    assert_eq!(stored.get(&rating_field.key).map(String::as_str), Some("Yes"));

    // A second vendor, still Active, to prove the report groups correctly.
    custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Beta Parts".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();

    let report = custom_report_service::create(
        &conn, &ws,
        &CustomReportInput {
            name: "Vendors by status".into(),
            entity_type: vendor.key.clone(),
            group_by_source: "builtin".into(),
            group_by_field: "status".into(),
            aggregate: "count".into(),
            sum_field_key: None,
        },
        Some(&admin),
    ).unwrap();
    let rows = custom_report_service::run(&conn, &report).unwrap();
    let by_group: HashMap<String, f64> = rows.into_iter().map(|r| (r.group, r.value)).collect();
    assert_eq!(by_group.get("Active"), Some(&1.0));
    assert_eq!(by_group.get("Inactive"), Some(&1.0));
}

#[test]
fn a_custom_object_cannot_be_named_the_same_as_a_built_in_entity() {
    let (conn, ws, admin) = setup_workspace();
    let clash = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Company".into(), plural_label: "Companies".into(), icon: "◆".into(), prefix: "CMP".into(), digits: 4 },
        Some(&admin),
    );
    assert!(clash.is_err());
}
