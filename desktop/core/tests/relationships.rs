//! Admin extensibility Phase B (spec §20.3/§21): admin-defined
//! relationships between any two entity types - built-in to built-in,
//! built-in to custom object, or custom object to custom object - with
//! cardinality enforcement, delete-behavior enforcement, and automatic
//! related-list resolution from either direction.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::services::{company_service, custom_object_service, custom_record_service, relationship_service, user_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = lanesra_core::models::workspace::WorkspaceSetup {
        business_name: "Relationships Co".into(),
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

fn vendor_object() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }
}

fn project_object() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Project".into(), plural_label: "Projects".into(), icon: "📁".into(), prefix: "PRJ".into(), digits: 4 }
}

#[test]
fn an_administrator_can_define_a_relationship_and_a_non_admin_cannot() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();

    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(),
            target_entity_type: "Company".into(),
            relationship_type: "many_to_one".into(),
            forward_label: "Client".into(),
            reverse_label: "Vendors".into(),
            is_required: false,
            show_related_list: true,
            delete_behavior: "restrict".into(),
            sort_order: 0,
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(def.key, format!("{}_company", vendor.key));

    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();
    let denied = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key, target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors 2".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&sales_user.id),
    );
    assert!(denied.is_err());
}

#[test]
fn many_to_one_limits_the_source_to_one_link_but_allows_many_sources_per_target() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    let company_a = company_service::create(&conn, &ws, &CompanyInput { name: "Acme".into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin)).unwrap();
    let company_b = company_service::create(&conn, &ws, &CompanyInput { name: "Beta".into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin)).unwrap();
    let vendor_1 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor One".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    let vendor_2 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor Two".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();

    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_1.id, "Company", &company_a.id, Some(&admin)).unwrap();
    // Vendor One is already linked - relinking it (even to a different
    // company) is blocked without an explicit unlink first.
    let blocked = relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_1.id, "Company", &company_b.id, Some(&admin));
    assert!(blocked.is_err());

    // A second, different source vendor can still link to the same target
    // company - that's the "many" side of many-to-one.
    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_2.id, "Company", &company_a.id, Some(&admin)).unwrap();

    let related_to_a = relationship_service::related_records_for(&conn, &ws, "Company", &company_a.id).unwrap();
    assert_eq!(related_to_a.len(), 2);
    assert!(related_to_a.iter().all(|r| r.label == "Vendors"));

    let related_to_vendor_1 = relationship_service::related_records_for(&conn, &ws, &vendor.key, &vendor_1.id).unwrap();
    assert_eq!(related_to_vendor_1.len(), 1);
    assert_eq!(related_to_vendor_1[0].label, "Client");
    assert_eq!(related_to_vendor_1[0].display_name, "Acme");
}

#[test]
fn one_to_one_limits_both_sides_to_a_single_link() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "one_to_one".into(),
            forward_label: "Primary Client".into(), reverse_label: "Primary Vendor".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    let company_a = company_service::create(&conn, &ws, &CompanyInput { name: "Acme".into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin)).unwrap();
    let vendor_1 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor One".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    let vendor_2 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor Two".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();

    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_1.id, "Company", &company_a.id, Some(&admin)).unwrap();
    // Company A already has a primary vendor - a second vendor cannot also
    // claim it, even though vendor_2 itself has no link yet.
    let blocked = relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_2.id, "Company", &company_a.id, Some(&admin));
    assert!(blocked.is_err());
}

#[test]
fn many_to_many_allows_multiple_links_but_rejects_an_exact_duplicate() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let project = custom_object_service::create(&conn, &ws, &project_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: project.key.clone(), target_entity_type: vendor.key.clone(), relationship_type: "many_to_many".into(),
            forward_label: "Vendors".into(), reverse_label: "Projects".into(), is_required: false, show_related_list: true,
            delete_behavior: "archive".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();

    let project_1 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: project.key.clone(), primary_name: "Website Revamp".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    let vendor_1 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor One".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    let vendor_2 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor Two".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();

    relationship_service::link(&conn, &ws, &def.id, &project.key, &project_1.id, &vendor.key, &vendor_1.id, Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &def.id, &project.key, &project_1.id, &vendor.key, &vendor_2.id, Some(&admin)).unwrap();

    let dup = relationship_service::link(&conn, &ws, &def.id, &project.key, &project_1.id, &vendor.key, &vendor_1.id, Some(&admin));
    assert!(dup.is_err());

    let related = relationship_service::related_records_for(&conn, &ws, &project.key, &project_1.id).unwrap();
    assert_eq!(related.len(), 2);
}

#[test]
fn restrict_blocks_archive_while_archive_behavior_clears_the_link_silently() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();

    let restrict_def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();
    let company = company_service::create(&conn, &ws, &CompanyInput { name: "Acme".into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin)).unwrap();
    let vendor_1 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor One".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &restrict_def.id, &vendor.key, &vendor_1.id, "Company", &company.id, Some(&admin)).unwrap();

    let blocked = custom_record_service::archive(&conn, &vendor_1.id, Some(&admin));
    assert!(blocked.is_err());

    // Unlinking first clears the way.
    let related = relationship_service::related_records_for(&conn, &ws, &vendor.key, &vendor_1.id).unwrap();
    relationship_service::unlink(&conn, &related[0].instance_id, Some(&admin)).unwrap();
    custom_record_service::archive(&conn, &vendor_1.id, Some(&admin)).unwrap();

    // Switch to a fresh `archive` behavior relationship on a second vendor -
    // archiving it should succeed and silently drop the link.
    relationship_service::update(
        &conn, &restrict_def.id,
        &lanesra_core::models::relationship::RelationshipDefinitionUpdate {
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "archive".into(), sort_order: 0, is_active: true,
        },
        Some(&admin),
    ).unwrap();
    let vendor_2 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor Two".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &restrict_def.id, &vendor.key, &vendor_2.id, "Company", &company.id, Some(&admin)).unwrap();
    custom_record_service::archive(&conn, &vendor_2.id, Some(&admin)).unwrap();
    let related_after = relationship_service::related_records_for(&conn, &ws, "Company", &company.id).unwrap();
    assert!(related_after.is_empty());
}

#[test]
fn deleting_a_relationship_definition_is_blocked_while_links_exist() {
    let (conn, ws, admin) = setup_workspace();
    let vendor = custom_object_service::create(&conn, &ws, &vendor_object(), Some(&admin)).unwrap();
    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();
    let company = company_service::create(&conn, &ws, &CompanyInput { name: "Acme".into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&admin)).unwrap();
    let vendor_1 = custom_record_service::create(&conn, &ws, &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Vendor One".into(), status: "Active".into(), owner_user_id: None, notes: None }, Some(&admin)).unwrap();
    relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_1.id, "Company", &company.id, Some(&admin)).unwrap();

    assert!(relationship_service::delete(&conn, &def.id, Some(&admin)).is_err());

    let related = relationship_service::related_records_for(&conn, &ws, &vendor.key, &vendor_1.id).unwrap();
    relationship_service::unlink(&conn, &related[0].instance_id, Some(&admin)).unwrap();
    relationship_service::delete(&conn, &def.id, Some(&admin)).unwrap();
}

#[test]
fn a_relationship_cannot_connect_an_object_type_to_itself_or_an_unknown_type() {
    let (conn, ws, admin) = setup_workspace();
    let self_link = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: "Company".into(), target_entity_type: "Company".into(), relationship_type: "many_to_many".into(),
            forward_label: "Related Company".into(), reverse_label: "Related Company".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    );
    assert!(self_link.is_err());

    let unknown = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: "NotARealType".into(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "X".into(), reverse_label: "Y".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    );
    assert!(unknown.is_err());
}
