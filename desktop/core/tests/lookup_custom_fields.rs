//! Lookup Custom Field Type: building a Custom Relationship ("I built a
//! relationship between two objects - a lookup field should appear on the
//! objects") now auto-creates a pickable `field_type == "lookup"` field on
//! the appropriate side(s), instead of the relationship only ever
//! surfacing as a post-creation "Related list". A lookup field's value is
//! still the familiar flat `value_text` cache every other field type
//! uses, but writing it also writes through `relationship_service::link`/
//! `unlink` so related-list rendering and Object 360 never see a second,
//! driftable copy of the same fact.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::{RelationshipDefinitionInput, RelationshipDefinitionUpdate};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, custom_field_service, custom_object_service, custom_record_service, relationship_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lookup Field Co".into(),
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

fn policy_object() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Policy".into(), plural_label: "Policies".into(), icon: "📄".into(), prefix: "POL".into(), digits: 4 }
}

fn vendor_object() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }
}

fn company_input(name: &str) -> CompanyInput {
    CompanyInput { name: name.into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }
}

fn record_input(object_key: &str, name: &str) -> CustomRecordInput {
    CustomRecordInput { object_key: object_key.into(), primary_name: name.into(), status: "Active".into(), owner_user_id: None, notes: None }
}

fn many_to_one_def(source: &str, target: &str) -> RelationshipDefinitionInput {
    RelationshipDefinitionInput {
        source_entity_type: source.into(), target_entity_type: target.into(), target_is_polymorphic: false, relationship_type: "many_to_one".into(),
        forward_label: "Customer".into(), reverse_label: "Policies".into(), is_required: false, show_related_list: true,
        delete_behavior: "restrict".into(), sort_order: 0,
    }
}

#[test]
fn many_to_one_relationship_auto_creates_exactly_one_lookup_field_on_the_source_side() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();

    let def = relationship_service::create(&conn, &ws, &many_to_one_def(&policy.key, "Company"), admin).unwrap();

    let policy_fields = custom_field_service::list_definitions(&conn, &ws, &policy.key, false).unwrap();
    let lookup_fields: Vec<_> = policy_fields.iter().filter(|f| f.field_type == "lookup").collect();
    assert_eq!(lookup_fields.len(), 1, "many_to_one must create exactly one lookup field on the source side");
    let field = lookup_fields[0];
    assert_eq!(field.label, "Customer");
    assert_eq!(field.lookup_target_entity_type.as_deref(), Some("Company"));
    assert_eq!(field.lookup_relationship_definition_id.as_deref(), Some(def.id.as_str()));
    assert!(field.is_active);

    // The target side (Company) gets no lookup field - "many" can't
    // collapse into a single pickable value.
    let company_fields = custom_field_service::list_definitions(&conn, &ws, "Company", false).unwrap();
    assert!(!company_fields.iter().any(|f| f.field_type == "lookup"));
}

#[test]
fn one_to_one_relationship_auto_creates_a_lookup_field_on_both_sides() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), admin).unwrap();

    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), target_is_polymorphic: false, relationship_type: "one_to_one".into(),
            forward_label: "Primary Client".into(), reverse_label: "Primary Vendor".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        admin,
    ).unwrap();

    let vendor_fields = custom_field_service::list_definitions(&conn, &ws, &vendor.key, false).unwrap();
    let vendor_lookup = vendor_fields.iter().find(|f| f.field_type == "lookup").expect("vendor side must get a lookup field");
    assert_eq!(vendor_lookup.label, "Primary Client");
    assert_eq!(vendor_lookup.lookup_target_entity_type.as_deref(), Some("Company"));
    assert_eq!(vendor_lookup.lookup_relationship_definition_id.as_deref(), Some(def.id.as_str()));

    let company_fields = custom_field_service::list_definitions(&conn, &ws, "Company", false).unwrap();
    let company_lookup = company_fields.iter().find(|f| f.field_type == "lookup").expect("company side must get a lookup field too - one_to_one is symmetric");
    assert_eq!(company_lookup.label, "Primary Vendor");
    assert_eq!(company_lookup.lookup_target_entity_type.as_deref(), Some(vendor.key.as_str()));
    assert_eq!(company_lookup.lookup_relationship_definition_id.as_deref(), Some(def.id.as_str()));
}

#[test]
fn many_to_many_and_polymorphic_target_relationships_create_no_lookup_field() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), admin).unwrap();
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();

    relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: policy.key.clone(), target_is_polymorphic: false, relationship_type: "many_to_many".into(),
            forward_label: "Policies".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "archive".into(), sort_order: 0,
        },
        admin,
    ).unwrap();
    relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: String::new(), target_is_polymorphic: true, relationship_type: "many_to_one".into(),
            forward_label: "Attached To".into(), reverse_label: "Attached Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        admin,
    ).unwrap();

    let vendor_fields = custom_field_service::list_definitions(&conn, &ws, &vendor.key, false).unwrap();
    assert!(!vendor_fields.iter().any(|f| f.field_type == "lookup"), "many_to_many and a polymorphic target must not produce a lookup field");
    let policy_fields = custom_field_service::list_definitions(&conn, &ws, &policy.key, false).unwrap();
    assert!(!policy_fields.iter().any(|f| f.field_type == "lookup"));
}

#[test]
fn a_lookup_field_cannot_be_created_or_edited_through_the_ordinary_admin_form() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let direct = custom_field_service::create_definition(
        &conn, &ws,
        &lanesra_core::models::custom_field::CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Hand-rolled Lookup".into(), field_type: "lookup".into(),
            options: vec![], required: false, show_in_list: false, sort_order: 0,
            min_value: None, max_value: None, max_length: None, regex_pattern: None,
            is_searchable: false, is_filterable: false, is_reportable: false,
            default_value: None, is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
        },
        admin,
    );
    assert!(direct.is_err(), "an admin must not be able to hand-create an orphan lookup field with no relationship behind it");
}

#[test]
fn setting_a_lookup_fields_value_links_the_real_record_and_clearing_it_unlinks() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();
    relationship_service::create(&conn, &ws, &many_to_one_def(&policy.key, "Company"), admin).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme"), admin).unwrap();
    let policy_1 = custom_record_service::create(&conn, &ws, &record_input(&policy.key, "Policy One"), admin).unwrap();

    let mut values = HashMap::new();
    values.insert("customer".to_string(), company.id.clone());
    custom_field_service::set_entity_values(&conn, &policy.key, &policy_1.id, &values, admin).unwrap();

    // The value cache reads back the linked record's id.
    let stored = custom_field_service::get_entity_values(&conn, &policy_1.id).unwrap();
    assert_eq!(stored.get("customer"), Some(&company.id));

    // The real data of record is a relationship_instances row - visible
    // from both directions, exactly like an admin-linked Related list.
    let related_to_policy = relationship_service::related_records_for(&conn, &ws, &policy.key, &policy_1.id).unwrap();
    assert_eq!(related_to_policy.len(), 1);
    assert_eq!(related_to_policy[0].entity_id, company.id);
    let related_to_company = relationship_service::related_records_for(&conn, &ws, "Company", &company.id).unwrap();
    assert_eq!(related_to_company.len(), 1);
    assert_eq!(related_to_company[0].entity_id, policy_1.id);

    // Clearing the field unlinks the real record too, not just the cache.
    let mut cleared = HashMap::new();
    cleared.insert("customer".to_string(), String::new());
    custom_field_service::set_entity_values(&conn, &policy.key, &policy_1.id, &cleared, admin).unwrap();
    let stored_after_clear = custom_field_service::get_entity_values(&conn, &policy_1.id).unwrap();
    assert!(!stored_after_clear.contains_key("customer"));
    let related_after_clear = relationship_service::related_records_for(&conn, &ws, &policy.key, &policy_1.id).unwrap();
    assert!(related_after_clear.is_empty());
}

#[test]
fn a_lookup_fields_value_must_resolve_to_a_real_record_of_the_target_type() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();
    relationship_service::create(&conn, &ws, &many_to_one_def(&policy.key, "Company"), admin).unwrap();
    let policy_1 = custom_record_service::create(&conn, &ws, &record_input(&policy.key, "Policy One"), admin).unwrap();

    let mut bad_values = HashMap::new();
    bad_values.insert("customer".to_string(), "not-a-real-company-id".to_string());
    let err = custom_field_service::set_entity_values(&conn, &policy.key, &policy_1.id, &bad_values, admin).unwrap_err();
    assert!(err.to_string().contains("Customer"));
}

#[test]
fn switching_a_lookup_fields_value_unlinks_the_old_record_and_links_the_new_one() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();
    relationship_service::create(&conn, &ws, &many_to_one_def(&policy.key, "Company"), admin).unwrap();
    let company_a = company_service::create(&conn, &ws, &company_input("Acme"), admin).unwrap();
    let company_b = company_service::create(&conn, &ws, &company_input("Beta"), admin).unwrap();
    let policy_1 = custom_record_service::create(&conn, &ws, &record_input(&policy.key, "Policy One"), admin).unwrap();

    let mut to_a = HashMap::new();
    to_a.insert("customer".to_string(), company_a.id.clone());
    custom_field_service::set_entity_values(&conn, &policy.key, &policy_1.id, &to_a, admin).unwrap();

    let mut to_b = HashMap::new();
    to_b.insert("customer".to_string(), company_b.id.clone());
    custom_field_service::set_entity_values(&conn, &policy.key, &policy_1.id, &to_b, admin).unwrap();

    let related = relationship_service::related_records_for(&conn, &ws, &policy.key, &policy_1.id).unwrap();
    assert_eq!(related.len(), 1, "switching the lookup value must unlink the old record, not add a second link");
    assert_eq!(related[0].entity_id, company_b.id);
}

#[test]
fn deactivating_the_relationship_deactivates_its_auto_created_lookup_fields_without_deleting_them() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();
    let def = relationship_service::create(&conn, &ws, &many_to_one_def(&policy.key, "Company"), admin).unwrap();

    relationship_service::update(
        &conn, &def.id,
        &RelationshipDefinitionUpdate {
            forward_label: "Customer".into(), reverse_label: "Policies".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0, is_active: false,
        },
        admin,
    ).unwrap();

    let policy_fields = custom_field_service::list_definitions(&conn, &ws, &policy.key, false).unwrap();
    let field = policy_fields.iter().find(|f| f.field_type == "lookup").expect("the field must still exist, just deactivated");
    assert!(!field.is_active, "deactivating the relationship must deactivate its lookup field, not leave it active");

    let active_only = custom_field_service::list_definitions(&conn, &ws, &policy.key, true).unwrap();
    assert!(!active_only.iter().any(|f| f.field_type == "lookup"), "an active-only listing (what a create/edit form renders) must no longer offer it");
}

#[test]
fn deleting_a_relationship_with_no_links_deactivates_its_lookup_field_and_succeeds() {
    let (conn, ws, admin) = setup_workspace();
    let admin = Some(admin.as_str());
    let policy = custom_object_service::create(&conn, &ws, &policy_object(), admin).unwrap();
    let def = relationship_service::create(&conn, &ws, &many_to_one_def(&policy.key, "Company"), admin).unwrap();

    relationship_service::delete(&conn, &def.id, admin).unwrap();

    let policy_fields = custom_field_service::list_definitions(&conn, &ws, &policy.key, false).unwrap();
    let field = policy_fields.iter().find(|f| f.field_type == "lookup").expect("the field record itself must survive a hard delete of the relationship");
    assert!(!field.is_active);
}
