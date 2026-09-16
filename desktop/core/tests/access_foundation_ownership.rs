use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::org_unit::OrgUnitInput;
use lanesra_core::models::ownership::OwnerRef;
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::work_team::WorkTeamInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, custom_object_service, custom_record_service, org_unit_service, organization_service, ownership_service, task_service, user_service, work_team_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Test Co".into(),
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

fn non_admin_input(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        display_name: username.into(),
        password: "anothersecretpw".into(),
        roles: vec!["Sales".to_string()],
    }
}

fn company_input(name: &str, owner_user_id: Option<&str>) -> CompanyInput {
    CompanyInput {
        name: name.into(),
        status: "Prospect".into(),
        owner_user_id: owner_user_id.map(|s| s.to_string()),
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
        phone: None,
        email: None,
        website: None,
        annual_revenue_cents: None,
        employee_count: None,
        preferred_contact_method: None,
    }
}

#[test]
fn company_gets_default_owner_from_creator_when_unspecified() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    let ownership = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    let owner = ownership.owner.unwrap();
    assert_eq!(owner.owner_type, "USER");
    assert_eq!(owner.owner_id, admin);
    assert_eq!(ownership.ownership_version, 1);
}

#[test]
fn company_input_owner_user_id_overrides_creator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", Some(&rep.id)), Some(&admin)).unwrap();

    let ownership = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    assert_eq!(ownership.owner.unwrap().owner_id, rep.id, "an explicit owner on the create input must win over the creator");
}

#[test]
fn task_and_custom_record_also_get_default_owner_on_create() {
    let (conn, ws, admin) = setup_workspace();

    let task = task_service::create(
        &conn,
        &ws,
        &TaskInput { title: "Follow up".into(), description: None, owner_user_id: None, priority: "Normal".into(), status: "Not Started".into(), due_date: None, reminder_at: None, related_type: None, related_id: None },
        Some(&admin),
    )
    .unwrap();
    let task_owner = ownership_service::get_owner(&conn, "Task", &task.id).unwrap().unwrap();
    assert_eq!(task_owner.owner.unwrap().owner_id, admin);

    let def = custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "◆".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    let record = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput { object_key: def.key.clone(), primary_name: "Acme Supply".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();
    let record_owner = ownership_service::get_owner(&conn, &def.key, &record.id).unwrap().unwrap();
    assert_eq!(record_owner.owner.unwrap().owner_id, admin);
}

#[test]
fn set_owner_bumps_ownership_version_and_records_audit_event() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    let before = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    assert_eq!(before.ownership_version, 1);

    ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user(&rep.id), None, Some(&admin)).unwrap();

    let after = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    assert_eq!(after.owner.unwrap().owner_id, rep.id);
    assert_eq!(after.ownership_version, 2, "ownership_version must increment on an explicit transfer");

    let events = lanesra_core::repositories::audit_repo::list_for_entity(&conn, "Company", &company.id).unwrap();
    assert!(events.iter().any(|e| e.event_type == "ownership_transferred"), "a transfer must be audit-logged");
}

#[test]
fn set_owner_requires_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    let result = ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user(&rep.id), None, Some(&rep.id));
    assert!(result.is_err(), "the placeholder Assign capability gate must reject a non-Administrator");
}

#[test]
fn set_owner_rejects_nonexistent_user() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    let result = ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user("does-not-exist"), None, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn set_owner_rejects_team_that_cannot_own_records() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let team = work_team_service::create(
        &conn,
        &ws,
        &WorkTeamInput { name: "Read-only pool".into(), code: "RO".into(), team_type: "Operational".into(), primary_org_unit_id: root.root_org_unit_id.clone(), owner_user_id: None, can_own_records: false, effective_from: None, effective_to: None },
        Some(&admin),
    )
    .unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    let result = ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::team(&team.id), None, Some(&admin));
    assert!(result.is_err(), "a team with can_own_records = false must not be assignable as an owner");
}

#[test]
fn bulk_transfer_dry_run_flags_ineligible_without_writing() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    let c1 = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let c2 = company_service::create(&conn, &ws, &company_input("Globex", None), Some(&admin)).unwrap();

    let ids = vec![c1.id.clone(), c2.id.clone(), "missing-id".to_string()];
    let dry_run = ownership_service::bulk_transfer_dry_run(&conn, &ws, "Company", &ids, &OwnerRef::user(&rep.id)).unwrap();

    assert_eq!(dry_run.eligible_ids.len(), 2);
    assert_eq!(dry_run.ineligible.len(), 1);
    assert_eq!(dry_run.ineligible[0].id, "missing-id");

    // A dry run must not write anything.
    let still_admin = ownership_service::get_owner(&conn, "Company", &c1.id).unwrap().unwrap();
    assert_eq!(still_admin.owner.unwrap().owner_id, admin);
}

#[test]
fn bulk_transfer_commit_reassigns_eligible_and_reports_ineligible() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    let c1 = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let c2 = company_service::create(&conn, &ws, &company_input("Globex", None), Some(&admin)).unwrap();

    let ids = vec![c1.id.clone(), c2.id.clone(), "missing-id".to_string()];
    let results = ownership_service::bulk_transfer_commit(&conn, &ws, "Company", &ids, &OwnerRef::user(&rep.id), None, Some(&admin)).unwrap();

    assert_eq!(results.len(), 3);
    assert!(results.iter().filter(|r| r.ok).count() == 2);
    assert!(results.iter().any(|r| r.id == "missing-id" && !r.ok));

    let c1_owner = ownership_service::get_owner(&conn, "Company", &c1.id).unwrap().unwrap();
    assert_eq!(c1_owner.owner.unwrap().owner_id, rep.id);
    let c2_owner = ownership_service::get_owner(&conn, "Company", &c2.id).unwrap().unwrap();
    assert_eq!(c2_owner.owner.unwrap().owner_id, rep.id);
}

#[test]
fn custom_object_ownership_mode_defaults_to_user_team_owned() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "◆".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(def.ownership_mode, "USER_TEAM_OWNED");

    let mode = ownership_service::ownership_mode_for(&conn, &ws, &def.key).unwrap();
    assert_eq!(mode, lanesra_core::models::ownership::OwnershipMode::UserTeamOwned);
}

#[test]
fn custom_object_ownership_mode_can_be_changed_and_blocks_owner_assignment() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput { singular_label: "Rate Card".into(), plural_label: "Rate Cards".into(), icon: "◆".into(), prefix: "RC".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    custom_object_service::set_ownership_mode(&conn, &def.id, "ORG_OWNED", Some(&admin)).unwrap();

    let record = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput { object_key: def.key.clone(), primary_name: "Standard Rates".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();

    // Org Owned objects have no individual owner - the default-owner-on-create
    // call must silently no-op rather than fail the create.
    let ownership = ownership_service::get_owner(&conn, &def.key, &record.id).unwrap();
    assert!(ownership.is_none() || ownership.unwrap().owner.is_none());

    let result = ownership_service::set_owner(&conn, &ws, &def.key, &record.id, &OwnerRef::user(&admin), None, Some(&admin));
    assert!(result.is_err(), "an Org Owned object must reject an explicit owner assignment");
}

#[test]
fn owning_org_unit_can_be_set_alongside_owner() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let east = org_unit_service::create(
        &conn,
        &ws,
        &OrgUnitInput { name: "East".into(), unit_type: "Region".into(), parent_org_unit_id: Some(root.root_org_unit_id.clone()), manager_user_id: None, effective_from: None, effective_to: None },
        Some(&admin),
    )
    .unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user(&admin), Some(&east.id), Some(&admin)).unwrap();

    let ownership = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    assert_eq!(ownership.owning_org_unit_id.as_deref(), Some(east.id.as_str()));

    let count = ownership_service::count_owned_in_org_unit(&conn, "Company", &east.id).unwrap();
    assert_eq!(count, 1);
}
