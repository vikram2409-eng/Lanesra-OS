use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput, Capability, RecordScope};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::org_unit::OrgUnitInput;
use lanesra_core::models::ownership::OwnerRef;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::work_team::WorkTeamInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{access_role_service, access_service, company_service, org_unit_service, organization_service, ownership_service, user_service, work_team_service, workspace_service};

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
    NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec!["Sales".to_string()] }
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

/// Creates a custom Access Role granting only `capability` on `object_key`
/// (default object key `"*"`) at `scope`, assigns it to `user_id`, and
/// returns the role id - the common shape almost every scope-level test
/// below needs.
fn grant_and_assign(conn: &rusqlite::Connection, ws: &str, admin: &str, user_id: &str, name: &str, scope: &str) -> String {
    let role = access_role_service::create(conn, ws, &AccessRoleInput { name: name.into(), description: "".into() }, Some(admin)).unwrap();
    access_role_service::upsert_grant(
        conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: false, can_read: false, can_update: false, can_delete: false, can_assign: true, record_scope: scope.into() },
        Some(admin),
    )
    .unwrap();
    access_role_service::assign_to_user(conn, user_id, &role.id, Some(admin)).unwrap();
    role.id
}

#[test]
fn system_roles_bootstrap_lazily_and_administrator_gets_full_access() {
    let (conn, ws, admin) = setup_workspace();

    // The admin was created via workspace_service::first_run_setup, which
    // predates any Access Role bootstrap - it must start with none.
    assert!(access_role_service::list_roles_for_user(&conn, &admin).unwrap().is_empty());

    let scope = access_service::capability_grant_for(&conn, &admin, "Company", Capability::Assign).unwrap();
    assert_eq!(scope, Some(RecordScope::Organization), "the legacy Administrator must bootstrap into Full Access, Organization scope");

    let roles = access_role_service::list(&conn, &ws).unwrap();
    assert_eq!(roles.len(), 2);
    assert!(roles.iter().all(|r| r.is_system));
    assert!(roles.iter().any(|r| r.name == "Full Access"));
    assert!(roles.iter().any(|r| r.name == "Standard User"));

    let admin_roles = access_role_service::list_roles_for_user(&conn, &admin).unwrap();
    assert_eq!(admin_roles.len(), 1);
    assert_eq!(admin_roles[0].name, "Full Access");
}

#[test]
fn new_user_gets_standard_user_role_by_default() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    let roles = access_role_service::list_roles_for_user(&conn, &rep.id).unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].name, "Standard User");

    let scope = access_service::capability_grant_for(&conn, &rep.id, "Company", Capability::Assign).unwrap();
    assert_eq!(scope, None, "Standard User does not grant Assign at all");
    let read_scope = access_service::capability_grant_for(&conn, &rep.id, "Company", Capability::Read).unwrap();
    assert_eq!(read_scope, Some(RecordScope::Organization), "ordinary CRUD must not regress today's fully-open behavior - only Assign is restricted by default");
}

#[test]
fn custom_role_crud_including_wildcard_grant_and_delete_blocked_while_assigned() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    let role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "Auditor".into(), description: "Read-only, everywhere".into() }, Some(&admin)).unwrap();
    assert!(!role.is_system);

    let grant = access_role_service::upsert_grant(
        &conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: false, can_read: true, can_update: false, can_delete: false, can_assign: false, record_scope: "ORGANIZATION".into() },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(grant.object_key, "*");
    assert_eq!(access_role_service::list_grants(&conn, &role.id).unwrap().len(), 1);

    let updated = access_role_service::update(&conn, &role.id, &lanesra_core::models::access_role::AccessRoleUpdate { name: "Auditor (Read-only)".into(), description: "Updated".into() }, Some(&admin)).unwrap();
    assert_eq!(updated.name, "Auditor (Read-only)");

    access_role_service::assign_to_user(&conn, &rep.id, &role.id, Some(&admin)).unwrap();
    assert!(access_role_service::delete(&conn, &role.id, Some(&admin)).is_err(), "deleting a role with an assignee must be blocked");

    access_role_service::remove_from_user(&conn, &rep.id, &role.id, Some(&admin)).unwrap();
    assert!(access_role_service::delete(&conn, &role.id, Some(&admin)).is_ok(), "deleting an unassigned custom role must succeed");
}

#[test]
fn a_system_role_cannot_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    // Trigger the lazy bootstrap so the system roles exist.
    access_service::capability_grant_for(&conn, &admin, "Company", Capability::Read).unwrap();
    let standard_user = access_role_service::list(&conn, &ws).unwrap().into_iter().find(|r| r.name == "Standard User").unwrap();
    assert!(access_role_service::delete(&conn, &standard_user.id, Some(&admin)).is_err());
}

#[test]
fn capability_grant_for_folds_multiple_roles_to_the_broadest_scope() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    grant_and_assign(&conn, &ws, &admin, &rep.id, "Team Assigner", "TEAM");
    let after_team_role = access_service::capability_grant_for(&conn, &rep.id, "Company", Capability::Assign).unwrap();
    assert_eq!(after_team_role, Some(RecordScope::Team));

    grant_and_assign(&conn, &ws, &admin, &rep.id, "Org Assigner", "ORGANIZATION");
    let after_org_role = access_service::capability_grant_for(&conn, &rep.id, "Company", Capability::Assign).unwrap();
    assert_eq!(after_org_role, Some(RecordScope::Organization), "the broadest of the user's roles must win, not the first one found");
}

#[test]
fn owner_scope_allows_only_the_actors_own_record() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    grant_and_assign(&conn, &ws, &admin, &rep.id, "Owner Assigner", "OWNER");

    let reps_company = company_service::create(&conn, &ws, &company_input("Reps Co", Some(&rep.id)), Some(&admin)).unwrap();
    let admins_company = company_service::create(&conn, &ws, &company_input("Admins Co", None), Some(&admin)).unwrap();

    assert!(
        ownership_service::set_owner(&conn, &ws, "Company", &reps_company.id, &OwnerRef::user(&admin), None, Some(&rep.id)).is_ok(),
        "rep owns this record, so Owner scope must allow rep to reassign it"
    );
    assert!(
        ownership_service::set_owner(&conn, &ws, "Company", &admins_company.id, &OwnerRef::user(&rep.id), None, Some(&rep.id)).is_err(),
        "rep does not own this record, so Owner scope must deny it"
    );
}

#[test]
fn team_scope_allows_teammates_and_team_owned_records() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    let teammate = user_service::create(&conn, &ws, &non_admin_input("teammate"), Some(&admin)).unwrap();
    let outsider = user_service::create(&conn, &ws, &non_admin_input("outsider"), Some(&admin)).unwrap();

    let root_org_unit = organization_service::get(&conn, &ws, Some(&admin)).unwrap().root_org_unit_id;
    let team = work_team_service::create(
        &conn,
        &ws,
        &WorkTeamInput { name: "Sales Pod".into(), code: "POD1".into(), team_type: "Operational".into(), primary_org_unit_id: root_org_unit, owner_user_id: None, can_own_records: true, effective_from: None, effective_to: None },
        Some(&admin),
    )
    .unwrap();
    work_team_service::add_member(&conn, &ws, &team.id, &rep.id, None, Some(&admin)).unwrap();
    work_team_service::add_member(&conn, &ws, &team.id, &teammate.id, None, Some(&admin)).unwrap();

    grant_and_assign(&conn, &ws, &admin, &rep.id, "Team Assigner", "TEAM");

    let team_owned = company_service::create(&conn, &ws, &company_input("Team Co", None), Some(&admin)).unwrap();
    ownership_service::set_owner(&conn, &ws, "Company", &team_owned.id, &OwnerRef::team(&team.id), None, Some(&admin)).unwrap();
    let teammates_co = company_service::create(&conn, &ws, &company_input("Teammates Co", Some(&teammate.id)), Some(&admin)).unwrap();
    let outsiders_co = company_service::create(&conn, &ws, &company_input("Outsiders Co", Some(&outsider.id)), Some(&admin)).unwrap();

    assert!(ownership_service::set_owner(&conn, &ws, "Company", &team_owned.id, &OwnerRef::user(&rep.id), None, Some(&rep.id)).is_ok(), "rep is an active member of the owning team");
    assert!(ownership_service::set_owner(&conn, &ws, "Company", &teammates_co.id, &OwnerRef::user(&rep.id), None, Some(&rep.id)).is_ok(), "rep shares an active team membership with this record's owner");
    assert!(ownership_service::set_owner(&conn, &ws, "Company", &outsiders_co.id, &OwnerRef::user(&rep.id), None, Some(&rep.id)).is_err(), "rep shares no team with this record's owner");
}

#[test]
fn org_unit_and_below_scope_allows_the_actors_subtree_only() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    let root_org_unit = organization_service::get(&conn, &ws, Some(&admin)).unwrap().root_org_unit_id;
    let east = org_unit_service::create(&conn, &ws, &OrgUnitInput { name: "East".into(), unit_type: "Region".into(), parent_org_unit_id: Some(root_org_unit.clone()), manager_user_id: None, effective_from: None, effective_to: None }, Some(&admin)).unwrap();
    let east_ny = org_unit_service::create(&conn, &ws, &OrgUnitInput { name: "East NY".into(), unit_type: "Branch".into(), parent_org_unit_id: Some(east.id.clone()), manager_user_id: None, effective_from: None, effective_to: None }, Some(&admin)).unwrap();
    let west = org_unit_service::create(&conn, &ws, &OrgUnitInput { name: "West".into(), unit_type: "Region".into(), parent_org_unit_id: Some(root_org_unit.clone()), manager_user_id: None, effective_from: None, effective_to: None }, Some(&admin)).unwrap();

    lanesra_core::repositories::user_repo::set_primary_org_unit(&conn, &rep.id, &east.id).unwrap();
    grant_and_assign(&conn, &ws, &admin, &rep.id, "Region Assigner", "ORG_UNIT_AND_BELOW");

    let in_subtree = company_service::create(&conn, &ws, &company_input("NY Co", None), Some(&admin)).unwrap();
    ownership_service::set_owner(&conn, &ws, "Company", &in_subtree.id, &OwnerRef::user(&admin), Some(&east_ny.id), Some(&admin)).unwrap();
    let in_sibling = company_service::create(&conn, &ws, &company_input("West Co", None), Some(&admin)).unwrap();
    ownership_service::set_owner(&conn, &ws, "Company", &in_sibling.id, &OwnerRef::user(&admin), Some(&west.id), Some(&admin)).unwrap();

    assert!(
        ownership_service::set_owner(&conn, &ws, "Company", &in_subtree.id, &OwnerRef::user(&rep.id), None, Some(&rep.id)).is_ok(),
        "East NY is at/below rep's own primary Organization Unit (East)"
    );
    assert!(
        ownership_service::set_owner(&conn, &ws, "Company", &in_sibling.id, &OwnerRef::user(&rep.id), None, Some(&rep.id)).is_err(),
        "West is a sibling subtree, not at/below East"
    );
}

#[test]
fn require_assign_capability_denies_without_a_grant_and_allows_full_access() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();

    let denied = ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user(&rep.id), None, Some(&rep.id));
    assert!(denied.is_err(), "Standard User grants no Assign capability at all");

    let allowed = ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user(&rep.id), None, Some(&admin));
    assert!(allowed.is_ok(), "Full Access (Organization scope) must be able to reassign any record");
}

#[test]
fn bulk_transfer_commit_partial_success_reflects_the_actors_scope() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    grant_and_assign(&conn, &ws, &admin, &rep.id, "Owner Assigner", "OWNER");

    let reps_company = company_service::create(&conn, &ws, &company_input("Reps Co", Some(&rep.id)), Some(&admin)).unwrap();
    let admins_company = company_service::create(&conn, &ws, &company_input("Admins Co", None), Some(&admin)).unwrap();

    let ids = vec![reps_company.id.clone(), admins_company.id.clone()];
    let results = ownership_service::bulk_transfer_commit(&conn, &ws, "Company", &ids, &OwnerRef::user(&admin), None, Some(&rep.id)).unwrap();

    assert_eq!(results.len(), 2);
    let reps_result = results.iter().find(|r| r.id == reps_company.id).unwrap();
    let admins_result = results.iter().find(|r| r.id == admins_company.id).unwrap();
    assert!(reps_result.ok, "rep owns this one - within Owner scope");
    assert!(!admins_result.ok, "rep does not own this one - outside Owner scope");
}

#[test]
fn explain_access_reports_a_correct_trace_for_allow_and_deny() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();
    grant_and_assign(&conn, &ws, &admin, &rep.id, "Owner Assigner", "OWNER");

    let reps_company = company_service::create(&conn, &ws, &company_input("Reps Co", Some(&rep.id)), Some(&admin)).unwrap();
    let admins_company = company_service::create(&conn, &ws, &company_input("Admins Co", None), Some(&admin)).unwrap();

    let allow = access_service::explain_access(&conn, &rep.id, "Company", Capability::Assign, Some(&reps_company.id)).unwrap();
    assert!(allow.decision.allowed);
    assert_eq!(allow.decision.scope, Some(RecordScope::Owner));
    assert_eq!(allow.decision.matched_role.as_deref(), Some("Owner Assigner"));
    assert_eq!(allow.record_summary.as_deref(), Some("Reps Co"));
    assert!(allow.roles_checked.iter().any(|c| c.role_name == "Owner Assigner" && c.capability_granted));
    assert!(!allow.decision.reason.is_empty());

    let deny = access_service::explain_access(&conn, &rep.id, "Company", Capability::Assign, Some(&admins_company.id)).unwrap();
    assert!(!deny.decision.allowed);
    assert!(!deny.decision.reason.is_empty());
}

/// Extended scope (per the user's own "Additive: Administrator still always
/// works" + "Read/create/update/delete gating for ordinary CRUD" answers):
/// today's actual baseline for ordinary CRUD has zero owner-based
/// restriction, so Standard User's default `'*'` grant was seeded at
/// Organization scope (not Owner) specifically so this must keep working -
/// a non-owner, non-admin rep can still update and archive someone else's
/// Company exactly as they could before this phase existed.
#[test]
fn standard_user_default_does_not_regress_todays_open_crud() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    let admins_company = company_service::create(&conn, &ws, &company_input("Admins Co", None), Some(&admin)).unwrap();

    let mut update_input = company_input("Admins Co Renamed", None);
    update_input.name = "Admins Co Renamed".into();
    let updated = company_service::update(&conn, &admins_company.id, &update_input, Some(&rep.id));
    assert!(updated.is_ok(), "Standard User's Organization-scope default must let a non-owner update a record they don't own, same as before this phase");

    assert!(
        company_service::archive(&conn, &admins_company.id, Some(&rep.id)).is_ok(),
        "Standard User's Organization-scope default must let a non-owner archive a record they don't own, same as before this phase"
    );
}

/// Genuine restriction only shows up once a user's *broadest* matching
/// grant is actually narrow - since multi-role folding always takes the
/// broadest scope (see `capability_grant_for_folds_multiple_roles_to_the_broadest_scope`
/// above), a custom Owner-scope role layered on top of the auto-assigned
/// Standard User (Organization scope) would have no effect at all. This
/// test removes Standard User first so the custom role is the only grant.
#[test]
fn a_real_custom_role_can_narrow_below_standard_users_default() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &non_admin_input("rep"), Some(&admin)).unwrap();

    let standard_user = access_role_service::list_roles_for_user(&conn, &rep.id).unwrap().into_iter().find(|r| r.name == "Standard User").unwrap();
    access_role_service::remove_from_user(&conn, &rep.id, &standard_user.id, Some(&admin)).unwrap();

    let role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "Owner-Only Editor".into(), description: "".into() }, Some(&admin)).unwrap();
    access_role_service::upsert_grant(
        &conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: false, record_scope: "OWNER".into() },
        Some(&admin),
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &rep.id, &role.id, Some(&admin)).unwrap();

    let reps_company = company_service::create(&conn, &ws, &company_input("Reps Co", Some(&rep.id)), Some(&admin)).unwrap();
    let admins_company = company_service::create(&conn, &ws, &company_input("Admins Co", None), Some(&admin)).unwrap();

    let mut own_update = company_input("Reps Co Renamed", Some(&rep.id));
    own_update.name = "Reps Co Renamed".into();
    assert!(company_service::update(&conn, &reps_company.id, &own_update, Some(&rep.id)).is_ok(), "rep owns this record - Owner scope covers it");

    let mut others_update = company_input("Admins Co Renamed", None);
    others_update.name = "Admins Co Renamed".into();
    assert!(
        company_service::update(&conn, &admins_company.id, &others_update, Some(&rep.id)).is_err(),
        "rep does not own this record and no longer holds Standard User's broader default - Owner scope must not cover it"
    );
}

/// The additive legacy-admin-check migration (task #221): an Administrator
/// still always passes unchanged, and a non-Administrator who is not
/// otherwise granted anything still cannot manage Organization Units - but
/// one holding an explicit Access Role grant of Update on "OrgUnit" now can,
/// purely additively (see `access_service::require_admin_or_explicit_update`).
#[test]
fn legacy_admin_check_is_additively_satisfied_by_an_explicit_access_role_grant() {
    let (conn, ws, admin) = setup_workspace();
    let ungranted_rep = user_service::create(&conn, &ws, &non_admin_input("ungranted"), Some(&admin)).unwrap();
    let granted_rep = user_service::create(&conn, &ws, &non_admin_input("granted"), Some(&admin)).unwrap();

    let role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "Org Structure Editor".into(), description: "".into() }, Some(&admin)).unwrap();
    access_role_service::upsert_grant(
        &conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "OrgUnit".into(), can_create: false, can_read: false, can_update: true, can_delete: false, can_assign: false, record_scope: "ORGANIZATION".into() },
        Some(&admin),
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &granted_rep.id, &role.id, Some(&admin)).unwrap();

    let root_org_unit = organization_service::get(&conn, &ws, Some(&admin)).unwrap().root_org_unit_id;
    let unit_input = OrgUnitInput { name: "Shadow Unit".into(), unit_type: "Region".into(), parent_org_unit_id: Some(root_org_unit.clone()), manager_user_id: None, effective_from: None, effective_to: None };

    assert!(
        org_unit_service::create(&conn, &ws, &unit_input, Some(&ungranted_rep.id)).is_err(),
        "a non-administrator with no Access Role grant on OrgUnit must still be denied, exactly as today"
    );
    assert!(
        org_unit_service::create(&conn, &ws, &unit_input, Some(&granted_rep.id)).is_ok(),
        "an explicit Update grant on OrgUnit must additively unlock this legacy admin-gated action"
    );
    assert!(
        org_unit_service::create(&conn, &ws, &unit_input, Some(&admin)).is_ok(),
        "the Administrator bypass must remain completely unchanged"
    );
}
