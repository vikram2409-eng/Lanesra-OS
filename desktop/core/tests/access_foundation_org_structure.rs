use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::org_unit::{OrgUnitInput, OrgUnitUpdate};
use lanesra_core::models::ownership::OwnerRef;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::work_team::{TeamMembershipInput, WorkTeamInput, WorkTeamUpdate};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, org_unit_service, organization_service, ownership_service, user_service, work_team_service, workspace_service};

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

fn child_unit_input(name: &str, parent_id: &str) -> OrgUnitInput {
    OrgUnitInput {
        name: name.into(),
        unit_type: "Department".into(),
        parent_org_unit_id: Some(parent_id.into()),
        manager_user_id: None,
        effective_from: None,
        effective_to: None,
    }
}

#[test]
fn organization_bootstraps_a_single_root_org_unit() {
    let (conn, ws, admin) = setup_workspace();

    let org = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    assert!(!org.root_org_unit_id.is_empty());
    assert!(!org.code.is_empty());

    let units = org_unit_service::list_tree(&conn, &ws).unwrap();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].id, org.root_org_unit_id);
    assert_eq!(units[0].depth, 0);
    assert!(units[0].parent_org_unit_id.is_none());

    // Calling get again must not create a second root.
    let org_again = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    assert_eq!(org_again.root_org_unit_id, org.root_org_unit_id);
    assert_eq!(org_unit_service::list_tree(&conn, &ws).unwrap().len(), 1);
}

#[test]
fn org_unit_create_requires_a_parent() {
    let (conn, ws, admin) = setup_workspace();
    organization_service::get(&conn, &ws, Some(&admin)).unwrap();

    let input = OrgUnitInput {
        name: "Rogue Root".into(),
        unit_type: "Division".into(),
        parent_org_unit_id: None,
        manager_user_id: None,
        effective_from: None,
        effective_to: None,
    };
    let result = org_unit_service::create(&conn, &ws, &input, Some(&admin));
    assert!(result.is_err(), "a second top-level Organization Unit must be rejected");
}

#[test]
fn org_unit_create_computes_path_and_depth_across_levels() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();

    let region = org_unit_service::create(&conn, &ws, &child_unit_input("East Region", &root.root_org_unit_id), Some(&admin)).unwrap();
    assert_eq!(region.depth, 1);
    assert!(region.path.starts_with(&format!("/{}/", root.root_org_unit_id)));

    let branch = org_unit_service::create(&conn, &ws, &child_unit_input("Boston Branch", &region.id), Some(&admin)).unwrap();
    assert_eq!(branch.depth, 2);
    assert!(branch.path.starts_with(&region.path));
}

#[test]
fn org_unit_non_administrator_cannot_create() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let sales = user_service::create(&conn, &ws, &non_admin_input("sam"), Some(&admin)).unwrap();

    let result = org_unit_service::create(&conn, &ws, &child_unit_input("Shadow Unit", &root.root_org_unit_id), Some(&sales.id));
    assert!(result.is_err(), "a non-administrator must not be able to create Organization Units");
}

#[test]
fn org_unit_move_rejects_cycles() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let parent = org_unit_service::create(&conn, &ws, &child_unit_input("Parent", &root.root_org_unit_id), Some(&admin)).unwrap();
    let child = org_unit_service::create(&conn, &ws, &child_unit_input("Child", &parent.id), Some(&admin)).unwrap();

    // Moving the parent under its own child is a cycle and must be rejected.
    let result = org_unit_service::move_unit(&conn, &ws, &parent.id, &child.id, Some(&admin));
    assert!(result.is_err(), "moving a unit under its own descendant must be rejected");

    // Moving a unit under itself is likewise rejected.
    let result = org_unit_service::move_unit(&conn, &ws, &parent.id, &parent.id, Some(&admin));
    assert!(result.is_err());

    // The root itself can never be moved.
    let result = org_unit_service::move_unit(&conn, &ws, &root.root_org_unit_id, &parent.id, Some(&admin));
    assert!(result.is_err(), "the root Organization Unit must not be movable");
}

#[test]
fn org_unit_move_previews_impact_then_recomputes_subtree_paths() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let east = org_unit_service::create(&conn, &ws, &child_unit_input("East", &root.root_org_unit_id), Some(&admin)).unwrap();
    let west = org_unit_service::create(&conn, &ws, &child_unit_input("West", &root.root_org_unit_id), Some(&admin)).unwrap();
    let boston = org_unit_service::create(&conn, &ws, &child_unit_input("Boston", &east.id), Some(&admin)).unwrap();

    // Give a company an owning_org_unit_id under the subtree being moved,
    // so the impact preview has something real to count.
    let company = company_service::create(
        &conn,
        &ws,
        &CompanyInput { name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, phone: None, email: None, website: None, annual_revenue_cents: None, employee_count: None, preferred_contact_method: None },
        Some(&admin),
    )
    .unwrap();
    ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::user(&admin), Some(&boston.id), Some(&admin)).unwrap();

    let preview = org_unit_service::preview_move(&conn, &ws, &east.id, &west.id, Some(&admin)).unwrap();
    assert_eq!(preview.descendant_unit_count, 1, "Boston is East's only descendant");
    assert!(preview.owned_record_counts.iter().any(|(key, count)| key == "Company" && *count == 1));

    let moved = org_unit_service::move_unit(&conn, &ws, &east.id, &west.id, Some(&admin)).unwrap();
    assert_eq!(moved.parent_org_unit_id.as_deref(), Some(west.id.as_str()));
    assert!(moved.path.starts_with(&west.path));
    assert_eq!(moved.depth, west.depth + 1);

    // Boston's path/depth must have been rewritten along with its parent.
    let units = org_unit_service::list_tree(&conn, &ws).unwrap();
    let boston_after = units.iter().find(|u| u.id == boston.id).unwrap();
    assert!(boston_after.path.starts_with(&moved.path));
    assert_eq!(boston_after.depth, moved.depth + 1);
}

#[test]
fn org_unit_delete_blocked_while_children_exist() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let parent = org_unit_service::create(&conn, &ws, &child_unit_input("Parent", &root.root_org_unit_id), Some(&admin)).unwrap();
    let child = org_unit_service::create(&conn, &ws, &child_unit_input("Child", &parent.id), Some(&admin)).unwrap();

    let result = org_unit_service::delete(&conn, &parent.id, Some(&admin));
    assert!(result.is_err(), "a unit with children must not be deletable");

    org_unit_service::delete(&conn, &child.id, Some(&admin)).unwrap();
    org_unit_service::delete(&conn, &parent.id, Some(&admin)).unwrap();
    assert!(org_unit_service::get(&conn, &parent.id).unwrap().is_none());
}

#[test]
fn org_unit_update_edits_name_and_status() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let unit = org_unit_service::create(&conn, &ws, &child_unit_input("Original", &root.root_org_unit_id), Some(&admin)).unwrap();

    let updated = org_unit_service::update(
        &conn,
        &unit.id,
        &OrgUnitUpdate { name: "Renamed".into(), unit_type: "Region".into(), manager_user_id: None, status: "Inactive".into(), effective_from: None, effective_to: None },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.unit_type, "Region");
    assert_eq!(updated.status, "Inactive");
}

fn team_input(name: &str, code: &str, org_unit_id: &str) -> WorkTeamInput {
    WorkTeamInput {
        name: name.into(),
        code: code.into(),
        team_type: "Operational".into(),
        primary_org_unit_id: org_unit_id.into(),
        owner_user_id: None,
        can_own_records: true,
        effective_from: None,
        effective_to: None,
    }
}

#[test]
fn work_team_crud_and_code_uniqueness() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();

    let team = work_team_service::create(&conn, &ws, &team_input("East Sales", "EAST-SALES", &root.root_org_unit_id), Some(&admin)).unwrap();
    assert_eq!(team.code, "EAST-SALES");
    assert!(team.can_own_records);

    let dup = work_team_service::create(&conn, &ws, &team_input("Duplicate", "EAST-SALES", &root.root_org_unit_id), Some(&admin));
    assert!(dup.is_err(), "team codes must be unique per workspace");

    let updated = work_team_service::update(
        &conn,
        &ws,
        &team.id,
        &WorkTeamUpdate { name: "East Sales Desk".into(), team_type: "Queue".into(), primary_org_unit_id: root.root_org_unit_id.clone(), owner_user_id: None, can_own_records: true, status: "Active".into(), effective_from: None, effective_to: None },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.name, "East Sales Desk");
    assert_eq!(updated.team_type, "Queue");

    assert_eq!(work_team_service::list(&conn, &ws).unwrap().len(), 1);
    work_team_service::delete(&conn, &team.id, Some(&admin)).unwrap();
    assert!(work_team_service::get(&conn, &team.id).unwrap().is_none());
}

#[test]
fn work_team_membership_add_and_end() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let team = work_team_service::create(&conn, &ws, &team_input("Queue", "Q1", &root.root_org_unit_id), Some(&admin)).unwrap();
    let sam = user_service::create(&conn, &ws, &non_admin_input("sam"), Some(&admin)).unwrap();

    let membership = work_team_service::add_member(&conn, &ws, &team.id, &sam.id, None, Some(&admin)).unwrap();
    assert_eq!(work_team_service::list_members(&conn, &team.id).unwrap().len(), 1);

    work_team_service::end_membership(&conn, &membership.id, Some(&admin)).unwrap();
    assert_eq!(work_team_service::list_members(&conn, &team.id).unwrap().len(), 0, "an ended membership must drop out of the active list");
}

#[test]
fn work_team_delete_blocked_while_active_members_exist() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let team = work_team_service::create(&conn, &ws, &team_input("Queue", "Q1", &root.root_org_unit_id), Some(&admin)).unwrap();
    let sam = user_service::create(&conn, &ws, &non_admin_input("sam"), Some(&admin)).unwrap();
    let membership = work_team_service::add_member(&conn, &ws, &team.id, &sam.id, None, Some(&admin)).unwrap();

    let result = work_team_service::delete(&conn, &team.id, Some(&admin));
    assert!(result.is_err(), "a team with active members must not be deletable");

    work_team_service::end_membership(&conn, &membership.id, Some(&admin)).unwrap();
    work_team_service::delete(&conn, &team.id, Some(&admin)).unwrap();
}

/// Regression for the spec's own stated invariant (see 0052_work_teams.sql's
/// doc comment): ending a Work Team membership must never touch any record
/// the team owns. Membership is who currently belongs to the team;
/// ownership is a separate fact recorded on the record itself.
#[test]
fn ending_team_membership_never_touches_owned_records() {
    let (conn, ws, admin) = setup_workspace();
    let root = organization_service::get(&conn, &ws, Some(&admin)).unwrap();
    let team = work_team_service::create(&conn, &ws, &team_input("Queue", "Q1", &root.root_org_unit_id), Some(&admin)).unwrap();
    let sam = user_service::create(&conn, &ws, &non_admin_input("sam"), Some(&admin)).unwrap();
    let membership = work_team_service::add_member(&conn, &ws, &team.id, &sam.id, None, Some(&admin)).unwrap();

    let company = company_service::create(
        &conn,
        &ws,
        &CompanyInput { name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, phone: None, email: None, website: None, annual_revenue_cents: None, employee_count: None, preferred_contact_method: None },
        Some(&admin),
    )
    .unwrap();
    ownership_service::set_owner(&conn, &ws, "Company", &company.id, &OwnerRef::team(&team.id), None, Some(&admin)).unwrap();

    let owner_before = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    assert_eq!(owner_before.owner.as_ref().unwrap().owner_id, team.id);

    work_team_service::end_membership(&conn, &membership.id, Some(&admin)).unwrap();

    let owner_after = ownership_service::get_owner(&conn, "Company", &company.id).unwrap().unwrap();
    assert_eq!(owner_after.owner.as_ref().unwrap().owner_id, team.id, "ending a membership must not change record ownership");
    assert_eq!(owner_after.ownership_version, owner_before.ownership_version, "ending a membership must not bump ownership_version");
}

#[test]
fn add_member_input_type_is_constructible() {
    // TeamMembershipInput isn't consumed by a service function directly
    // today (add_member takes its fields individually), but it's part of
    // the public model surface (e.g. for a future bulk-import path) and
    // must stay a valid, constructible shape.
    let _ = TeamMembershipInput { user_id: "u1".into(), role_in_team: Some("Lead".into()), effective_from: None };
}
