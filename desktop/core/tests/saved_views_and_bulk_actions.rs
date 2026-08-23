//! Saved Views & Bulk Actions (product backlog): global search and
//! per-field list filtering shipped without a way to save a filter/sort/
//! column/grouping combination as a named view, or to apply an operation
//! across a multi-select. See `saved_view_service`/`bulk_action_service`'s
//! own module doc comments for exact scope.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::saved_view::SavedViewInput;
use lanesra_core::models::status_transition::StatusTransitionInput;
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    bulk_action_service, company_service, custom_field_service, custom_object_service, custom_record_service,
    saved_view_service, status_transition_service, task_service, user_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Views & Bulk Co".into(),
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

fn user_with_role(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str, role: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec![role.into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

fn make_company(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    let input = CompanyInput {
        name: name.into(), status: "Prospect".into(), owner_user_id: None, tax_number: None,
        billing_address: None, shipping_address: None, tags: None, notes: None,
        phone: None, email: None, website: None, annual_revenue_cents: None,
        employee_count: None, preferred_contact_method: None,
    };
    company_service::create(conn, ws, &input, Some(admin)).unwrap().id
}

fn make_task(conn: &rusqlite::Connection, ws: &str, admin: &str, title: &str) -> String {
    let input = TaskInput {
        title: title.into(), description: None, owner_user_id: None, priority: "Normal".into(),
        status: "Not Started".into(), due_date: None, reminder_at: None, related_type: None, related_id: None,
    };
    task_service::create(conn, ws, &input, Some(admin)).unwrap().id
}

fn make_custom_object_and_record(conn: &rusqlite::Connection, ws: &str, admin: &str) -> (String, String) {
    let obj_input = CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 };
    let def = custom_object_service::create_with_key(conn, ws, "vendor", &obj_input, Some(admin)).unwrap();
    let record_input = CustomRecordInput { object_key: def.key.clone(), primary_name: "Acme Supply".into(), status: "Active".into(), owner_user_id: None, notes: None };
    let record = custom_record_service::create(conn, ws, &record_input, Some(admin)).unwrap();
    (def.key, record.id)
}

fn base_view_input(object_key: &str, name: &str, visibility: &str) -> SavedViewInput {
    SavedViewInput {
        object_key: object_key.into(),
        name: name.into(),
        visibility: visibility.into(),
        filters: HashMap::new(),
        sort_field: Some("name".into()),
        sort_direction: "asc".into(),
        columns: Some(vec!["name".into(), "status".into()]),
        group_by_field: None,
    }
}

// ---------------------------------------------------------------- Saved Views

#[test]
fn a_saved_view_can_be_created_listed_updated_and_deleted() {
    let (conn, ws, admin) = setup_workspace();
    let created = saved_view_service::create(&conn, &ws, &base_view_input("Company", "My prospects", "private"), Some(&admin)).unwrap();
    assert_eq!(created.owner_user_id, admin);
    assert!(!created.is_object_default);

    let listed = saved_view_service::list_for_object(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let mut update = base_view_input("Company", "My prospects (renamed)", "private");
    update.sort_direction = "desc".into();
    let updated = saved_view_service::update(&conn, &created.id, &update, Some(&admin)).unwrap();
    assert_eq!(updated.name, "My prospects (renamed)");
    assert_eq!(updated.sort_direction, "desc");

    saved_view_service::delete(&conn, &created.id, Some(&admin)).unwrap();
    let after_delete = saved_view_service::list_for_object(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(after_delete.is_empty());
}

#[test]
fn private_views_are_not_visible_to_other_users_but_shared_views_are() {
    let (conn, ws, admin) = setup_workspace();
    let sam = user_with_role(&conn, &ws, &admin, "sam", "Sales");

    let private = saved_view_service::create(&conn, &ws, &base_view_input("Company", "Admin's private view", "private"), Some(&admin)).unwrap();
    let shared = saved_view_service::create(&conn, &ws, &base_view_input("Company", "Team view", "shared"), Some(&admin)).unwrap();

    let sam_views = saved_view_service::list_for_object(&conn, &ws, "Company", Some(&sam)).unwrap();
    let sam_ids: Vec<&str> = sam_views.iter().map(|v| v.id.as_str()).collect();
    assert!(!sam_ids.contains(&private.id.as_str()));
    assert!(sam_ids.contains(&shared.id.as_str()));

    let admin_views = saved_view_service::list_for_object(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert_eq!(admin_views.len(), 2);
}

#[test]
fn only_one_default_view_exists_per_object_key_and_setting_default_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sam = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let a = saved_view_service::create(&conn, &ws, &base_view_input("Company", "View A", "shared"), Some(&admin)).unwrap();
    let b = saved_view_service::create(&conn, &ws, &base_view_input("Company", "View B", "shared"), Some(&admin)).unwrap();

    let err = saved_view_service::set_default(&conn, &a.id, Some(&sam)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));

    let a_default = saved_view_service::set_default(&conn, &a.id, Some(&admin)).unwrap();
    assert!(a_default.is_object_default);
    assert_eq!(saved_view_service::get_default(&conn, &ws, "Company").unwrap().unwrap().id, a.id);

    let b_default = saved_view_service::set_default(&conn, &b.id, Some(&admin)).unwrap();
    assert!(b_default.is_object_default);
    let a_reloaded = saved_view_service::list_for_object(&conn, &ws, "Company", Some(&admin))
        .unwrap()
        .into_iter()
        .find(|v| v.id == a.id)
        .unwrap();
    assert!(!a_reloaded.is_object_default, "setting B as default must clear A's default flag");
}

#[test]
fn only_the_owner_or_an_admin_may_edit_or_delete_a_shared_view() {
    let (conn, ws, admin) = setup_workspace();
    let sam = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let view = saved_view_service::create(&conn, &ws, &base_view_input("Company", "Team view", "shared"), Some(&admin)).unwrap();

    let err = saved_view_service::update(&conn, &view.id, &base_view_input("Company", "Hijacked", "shared"), Some(&sam)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    let err = saved_view_service::delete(&conn, &view.id, Some(&sam)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));

    // The owner (admin) can still edit/delete their own shared view.
    saved_view_service::update(&conn, &view.id, &base_view_input("Company", "Renamed by owner", "shared"), Some(&admin)).unwrap();
}

// ---------------------------------------------------------------- Bulk Actions

#[test]
fn bulk_update_builtin_field_applies_to_every_selected_company() {
    let (conn, ws, admin) = setup_workspace();
    let a = make_company(&conn, &ws, &admin, "Acme");
    let b = make_company(&conn, &ws, &admin, "Zeta");

    let results = bulk_action_service::bulk_update_builtin_field(&conn, &ws, "Company", &[a.clone(), b.clone()], "tax_number", "TAX-999", Some(&admin)).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.ok));
    for id in [&a, &b] {
        let refreshed = company_service::get(&conn, id).unwrap();
        assert_eq!(refreshed.tax_number.as_deref(), Some("TAX-999"));
    }
}

#[test]
fn bulk_update_custom_field_only_touches_the_targeted_key() {
    let (conn, ws, admin) = setup_workspace();
    let company = make_company(&conn, &ws, &admin, "Acme");

    fn field_input(label: &str) -> CustomFieldDefinitionInput {
        CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: label.into(), field_type: "text".into(), options: vec![],
            required: false, show_in_list: true, sort_order: 1, min_value: None, max_value: None, max_length: None,
            regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: false, default_value: None,
            is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
        }
    }
    let region_def = custom_field_service::create_definition(&conn, &ws, &field_input("Region"), Some(&admin)).unwrap();
    let segment_def = custom_field_service::create_definition(&conn, &ws, &field_input("Segment"), Some(&admin)).unwrap();

    let mut initial = HashMap::new();
    initial.insert(region_def.key.clone(), "West".to_string());
    initial.insert(segment_def.key.clone(), "Enterprise".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &company, &initial, Some(&admin)).unwrap();

    bulk_action_service::bulk_update_custom_field(&conn, "Company", &[company.clone()], &region_def.key, "East", Some(&admin)).unwrap();

    let values = custom_field_service::get_entity_values(&conn, &company).unwrap();
    assert_eq!(values.get(&region_def.key).map(String::as_str), Some("East"));
    assert_eq!(
        values.get(&segment_def.key).map(String::as_str),
        Some("Enterprise"),
        "an unrelated custom field's value must survive a bulk update targeting a different key"
    );
}

#[test]
fn bulk_reassign_owner_only_supports_entities_with_an_owner_column() {
    let (conn, ws, admin) = setup_workspace();
    let sam = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let task = make_task(&conn, &ws, &admin, "Follow up");

    let results = bulk_action_service::bulk_reassign_owner(&conn, &ws, "Task", &[task.clone()], Some(&sam), Some(&admin)).unwrap();
    assert!(results[0].ok);
    assert_eq!(task_service::get(&conn, &task).unwrap().owner_user_id.as_deref(), Some(sam.as_str()));

    // Product has no owner_user_id column at all - a clean, fail-fast
    // validation error, not a panic or a silent no-op.
    let err = bulk_action_service::bulk_reassign_owner(&conn, &ws, "Product", &["nonexistent".into()], Some(&sam), Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn bulk_change_status_is_independent_per_record_and_respects_status_transition_rules() {
    let (conn, ws, admin) = setup_workspace();
    let allowed = make_company(&conn, &ws, &admin, "Will succeed"); // starts Prospect
    let blocked = make_company(&conn, &ws, &admin, "Will fail");
    // Move `blocked` off Prospect while no rule exists yet to restrict it -
    // this fixture-setup update is itself unrestricted.
    company_service::update(
        &conn, &blocked,
        &CompanyInput {
            name: "Will fail".into(), status: "Inactive".into(), owner_user_id: None, tax_number: None,
            billing_address: None, shipping_address: None, tags: None, notes: None, phone: None, email: None,
            website: None, annual_revenue_cents: None, employee_count: None, preferred_contact_method: None,
        },
        Some(&admin),
    )
    .unwrap();

    // Now restrict Company transitions to Prospect -> Active Customer
    // only, so `allowed` (still Prospect) can reach "Active Customer" but
    // `blocked` (now Inactive) cannot.
    status_transition_service::create(
        &conn, &ws,
        &StatusTransitionInput { entity_type: "Company".into(), from_status: Some("Prospect".into()), to_status: "Active Customer".into() },
        Some(&admin),
    )
    .unwrap();

    let results = bulk_action_service::bulk_change_status(&conn, &ws, "Company", &[allowed.clone(), blocked.clone()], "Active Customer", Some(&admin)).unwrap();
    let allowed_result = results.iter().find(|r| r.id == allowed).unwrap();
    let blocked_result = results.iter().find(|r| r.id == blocked).unwrap();
    assert!(allowed_result.ok, "Prospect -> Active Customer is the one rule explicitly permitted");
    assert!(!blocked_result.ok, "Inactive -> Active Customer has no rule permitting it");
    assert_eq!(company_service::get(&conn, &allowed).unwrap().status, "Active Customer");
    assert_eq!(company_service::get(&conn, &blocked).unwrap().status, "Inactive", "a rejected bulk status change must leave the record exactly as it was");
}

#[test]
fn bulk_tag_merges_rather_than_overwrites() {
    let (conn, ws, admin) = setup_workspace();
    let company = make_company(&conn, &ws, &admin, "Acme");
    company_service::update(
        &conn, &company,
        &CompanyInput {
            name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None,
            billing_address: None, shipping_address: None, tags: Some("vip".into()), notes: None, phone: None,
            email: None, website: None, annual_revenue_cents: None, employee_count: None, preferred_contact_method: None,
        },
        Some(&admin),
    )
    .unwrap();

    bulk_action_service::bulk_update_tags(&conn, &ws, "Company", &[company.clone()], &["west-region".to_string()], true, Some(&admin)).unwrap();
    let tags_after_add = company_service::get(&conn, &company).unwrap().tags.unwrap();
    assert!(tags_after_add.contains("vip"));
    assert!(tags_after_add.contains("west-region"));

    bulk_action_service::bulk_update_tags(&conn, &ws, "Company", &[company.clone()], &["vip".to_string()], false, Some(&admin)).unwrap();
    let tags_after_remove = company_service::get(&conn, &company).unwrap().tags.unwrap();
    assert!(!tags_after_remove.contains("vip"));
    assert!(tags_after_remove.contains("west-region"));

    // Contact has tags too, but Product doesn't - the bulk tag action
    // itself (unlike other bulk operations) never extends to Custom
    // Objects either, since CustomRecordUpdate has no tags field.
    let err = bulk_action_service::bulk_update_tags(&conn, &ws, "Product", &["nonexistent".into()], &["x".into()], true, Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn bulk_archive_works_across_a_built_in_entity_and_a_custom_object() {
    let (conn, ws, admin) = setup_workspace();
    let company = make_company(&conn, &ws, &admin, "Acme");
    let (object_key, record_id) = make_custom_object_and_record(&conn, &ws, &admin);

    let company_results = bulk_action_service::bulk_archive(&conn, &ws, "Company", &[company.clone()], Some(&admin)).unwrap();
    assert!(company_results[0].ok);
    assert_eq!(company_service::get(&conn, &company).unwrap().status, "Archived");

    let custom_results = bulk_action_service::bulk_archive(&conn, &ws, &object_key, &[record_id.clone()], Some(&admin)).unwrap();
    assert!(custom_results[0].ok);
}

#[test]
fn an_unsupported_object_key_fails_the_whole_call_rather_than_per_record() {
    let (conn, ws, admin) = setup_workspace();
    let err = bulk_action_service::bulk_archive(&conn, &ws, "Quote", &["whatever".into()], Some(&admin)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    assert!(err.to_string().contains("Quote"));
}
