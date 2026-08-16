//! App Builder Phase 1: named, publishable groupings of already-existing
//! objects/screens/a dashboard, plus per-app access grants to a role or a
//! specific user - see `app_service`'s own doc comment for the full
//! rationale and what "editor" does and doesn't enforce yet.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::app_definition::{AppDefinitionInput, AppDefinitionUpdate, AppPermissionInput};
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::dashboard_layout::DashboardLayoutInput;
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{app_service, custom_object_service, custom_record_service, dashboard_layout_service, task_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "App Builder Co".into(),
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

fn app_input(name: &str) -> AppDefinitionInput {
    AppDefinitionInput { name: name.into(), icon: "⬡".into(), description: None }
}

#[test]
fn creating_an_app_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let err = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("Administrator"));
}

#[test]
fn create_list_and_get_round_trip() {
    let (conn, ws, admin) = setup_workspace();
    let created = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    assert_eq!(created.name, "Property Management");
    assert!(!created.is_published);
    assert!(created.object_keys.is_empty());

    let list = app_service::list(&conn, &ws).unwrap();
    assert_eq!(list.len(), 1);

    let fetched = app_service::get(&conn, &created.id).unwrap();
    assert_eq!(fetched.id, created.id);
}

#[test]
fn update_rejects_an_invalid_object_key() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    let update = AppDefinitionUpdate {
        name: app.name.clone(),
        icon: app.icon.clone(),
        description: None,
        object_keys: vec!["NotARealEntity".into()],
        dashboard_id: None,
    };
    let err = app_service::update(&conn, &app.id, &update, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("not a valid object"));
}

#[test]
fn update_accepts_built_in_and_custom_object_keys() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Property".into(), plural_label: "Properties".into(), icon: "🏠".into(), prefix: "PROP".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    let update = AppDefinitionUpdate {
        name: app.name.clone(),
        icon: app.icon.clone(),
        description: Some("Manage properties and tenants".into()),
        object_keys: vec!["Task".into(), obj.key.clone()],
        dashboard_id: None,
    };
    let updated = app_service::update(&conn, &app.id, &update, Some(&admin)).unwrap();
    assert_eq!(updated.object_keys, vec!["Task".to_string(), obj.key]);
    assert_eq!(updated.description.as_deref(), Some("Manage properties and tenants"));
}

#[test]
fn update_rejects_a_dashboard_from_another_workspace() {
    let (conn, ws, admin) = setup_workspace();
    let (conn2, ws2, admin2) = setup_workspace();
    let foreign_dashboard = dashboard_layout_service::create_layout(
        &conn2, &ws2,
        &DashboardLayoutInput { name: "Other workspace's dashboard".into(), initial_kpi_keys: vec![] },
        Some(&admin2),
    )
    .unwrap();
    drop(conn2);

    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    let update = AppDefinitionUpdate {
        name: app.name.clone(),
        icon: app.icon.clone(),
        description: None,
        object_keys: vec!["Task".into()],
        dashboard_id: Some(foreign_dashboard.id),
    };
    let err = app_service::update(&conn, &app.id, &update, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("Dashboard not found"));
}

#[test]
fn publish_requires_at_least_one_object_and_toggles_visibility() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();

    let err = app_service::publish(&conn, &app.id, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("at least one object"));

    let update = AppDefinitionUpdate { name: app.name.clone(), icon: app.icon.clone(), description: None, object_keys: vec!["Task".into()], dashboard_id: None };
    app_service::update(&conn, &app.id, &update, Some(&admin)).unwrap();

    let published = app_service::publish(&conn, &app.id, Some(&admin)).unwrap();
    assert!(published.is_published);

    let unpublished = app_service::unpublish(&conn, &app.id, Some(&admin)).unwrap();
    assert!(!unpublished.is_published);
}

#[test]
fn delete_removes_the_app() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    app_service::delete(&conn, &app.id, Some(&admin)).unwrap();
    assert!(app_service::list(&conn, &ws).unwrap().is_empty());
    assert!(app_service::get(&conn, &app.id).is_err());
}

#[test]
fn grant_permission_validates_principal_and_level() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();

    let bad_role = AppPermissionInput { principal_type: "role".into(), principal_id: "NotARole".into(), level: "viewer".into() };
    assert!(app_service::grant_permission(&conn, &app.id, &bad_role, Some(&admin)).is_err());

    let bad_level = AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "owner".into() };
    assert!(app_service::grant_permission(&conn, &app.id, &bad_level, Some(&admin)).is_err());

    let bad_user = AppPermissionInput { principal_type: "user".into(), principal_id: "nonexistent-id".into(), level: "viewer".into() };
    assert!(app_service::grant_permission(&conn, &app.id, &bad_user, Some(&admin)).is_err());

    let good = AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() };
    let granted = app_service::grant_permission(&conn, &app.id, &good, Some(&admin)).unwrap();
    assert_eq!(granted.level, "viewer");
}

#[test]
fn re_granting_the_same_principal_updates_the_level_in_place() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();

    let viewer = AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() };
    app_service::grant_permission(&conn, &app.id, &viewer, Some(&admin)).unwrap();
    let editor = AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "editor".into() };
    app_service::grant_permission(&conn, &app.id, &editor, Some(&admin)).unwrap();

    let perms = app_service::list_permissions(&conn, &app.id, Some(&admin)).unwrap();
    assert_eq!(perms.len(), 1);
    assert_eq!(perms[0].level, "editor");
}

#[test]
fn revoke_permission_removes_the_grant() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    let grant = app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    app_service::revoke_permission(&conn, &grant.id, Some(&admin)).unwrap();
    assert!(app_service::list_permissions(&conn, &app.id, Some(&admin)).unwrap().is_empty());
}

fn published_app(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> lanesra_core::models::app_definition::AppDefinition {
    let app = app_service::create(conn, ws, &app_input(name), Some(admin)).unwrap();
    let update = AppDefinitionUpdate { name: app.name.clone(), icon: app.icon.clone(), description: None, object_keys: vec!["Task".into()], dashboard_id: None };
    app_service::update(conn, &app.id, &update, Some(admin)).unwrap();
    app_service::publish(conn, &app.id, Some(admin)).unwrap()
}

#[test]
fn administrators_see_every_published_app_as_editor_without_a_grant() {
    let (conn, ws, admin) = setup_workspace();
    let app = published_app(&conn, &ws, &admin, "Property Management");
    let accessible = app_service::list_accessible(&conn, &ws, Some(&admin)).unwrap();
    assert_eq!(accessible.len(), 1);
    assert_eq!(accessible[0].app.id, app.id);
    assert_eq!(accessible[0].level, "editor");
}

#[test]
fn a_non_admin_with_no_grant_sees_no_apps() {
    let (conn, ws, admin) = setup_workspace();
    published_app(&conn, &ws, &admin, "Property Management");
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    assert!(app_service::list_accessible(&conn, &ws, Some(&sales)).unwrap().is_empty());
}

#[test]
fn a_role_grant_makes_the_app_visible_at_that_level() {
    let (conn, ws, admin) = setup_workspace();
    let app = published_app(&conn, &ws, &admin, "Property Management");
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let accessible = app_service::list_accessible(&conn, &ws, Some(&sales)).unwrap();
    assert_eq!(accessible.len(), 1);
    assert_eq!(accessible[0].level, "viewer");
}

#[test]
fn a_user_specific_grant_wins_over_a_role_grant() {
    let (conn, ws, admin) = setup_workspace();
    let app = published_app(&conn, &ws, &admin, "Property Management");
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "user".into(), principal_id: sales.clone(), level: "editor".into() },
        Some(&admin),
    )
    .unwrap();
    let accessible = app_service::list_accessible(&conn, &ws, Some(&sales)).unwrap();
    assert_eq!(accessible.len(), 1);
    assert_eq!(accessible[0].level, "editor");
}

#[test]
fn an_unpublished_app_never_appears_even_with_a_grant() {
    let (conn, ws, admin) = setup_workspace();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "editor".into() },
        Some(&admin),
    )
    .unwrap();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    assert!(app_service::list_accessible(&conn, &ws, Some(&sales)).unwrap().is_empty());
}

// ---- Phase 2: require_object_write_access enforced on real commands ------

fn task_input(title: &str) -> TaskInput {
    TaskInput {
        title: title.into(),
        description: None,
        owner_user_id: None,
        priority: "Normal".into(),
        status: "Not Started".into(),
        due_date: None,
        reminder_at: None,
        related_type: None,
        related_id: None,
    }
}

#[test]
fn an_entity_type_in_no_app_is_unaffected() {
    // Task is in zero apps here (no app was ever created) - Phase 2's
    // enforcement must never engage for an entity type App Builder hasn't
    // touched, regardless of the actor's role.
    let (conn, ws, admin) = setup_workspace();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    task_service::create(&conn, &ws, &task_input("Unscoped task"), Some(&sales)).unwrap();
}

#[test]
fn administrator_can_write_even_with_only_a_viewer_grant() {
    let (conn, ws, admin) = setup_workspace();
    let app = published_app(&conn, &ws, &admin, "Property Management");
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Administrator".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    // Administrators bypass app-level grants entirely (list_accessible's
    // own rule) - a "viewer" grant on their own role changes nothing.
    task_service::create(&conn, &ws, &task_input("Admin task"), Some(&admin)).unwrap();
}

#[test]
fn a_viewer_grant_blocks_create_update_and_archive() {
    let (conn, ws, admin) = setup_workspace();
    let app = published_app(&conn, &ws, &admin, "Property Management"); // object_keys: ["Task"]
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");

    let err = task_service::create(&conn, &ws, &task_input("Blocked task"), Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("view-only access"));

    // An Administrator creates one so update/archive have something to act on.
    let task = task_service::create(&conn, &ws, &task_input("Admin-created task"), Some(&admin)).unwrap();

    let err = task_service::update(&conn, &task.id, &ws, &task_input("Renamed"), Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("view-only access"));

    let err = task_service::archive(&conn, &task.id, &ws, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("view-only access"));
}

#[test]
fn an_editor_grant_allows_create_update_and_archive() {
    let (conn, ws, admin) = setup_workspace();
    let app = published_app(&conn, &ws, &admin, "Property Management"); // object_keys: ["Task"]
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "editor".into() },
        Some(&admin),
    )
    .unwrap();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");

    let task = task_service::create(&conn, &ws, &task_input("Editable task"), Some(&sales)).unwrap();
    task_service::update(&conn, &task.id, &ws, &task_input("Renamed"), Some(&sales)).unwrap();
    task_service::archive(&conn, &task.id, &ws, Some(&sales)).unwrap();
}

#[test]
fn no_grant_on_any_app_containing_the_type_blocks_write_even_though_unpublished_apps_dont_count() {
    let (conn, ws, admin) = setup_workspace();
    published_app(&conn, &ws, &admin, "Property Management"); // Task, published, zero grants
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    let err = task_service::create(&conn, &ws, &task_input("No access"), Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("don't have access"));
}

#[test]
fn the_strongest_grant_across_multiple_apps_wins() {
    // Sales is a viewer on one app containing Task and an editor on a
    // second app that also contains Task - the stronger grant applies,
    // mirroring how a single app already folds multiple role grants.
    let (conn, ws, admin) = setup_workspace();
    let viewer_app = published_app(&conn, &ws, &admin, "Property Management");
    app_service::grant_permission(
        &conn, &viewer_app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    let editor_app = published_app(&conn, &ws, &admin, "Field Service");
    app_service::grant_permission(
        &conn, &editor_app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "editor".into() },
        Some(&admin),
    )
    .unwrap();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");
    task_service::create(&conn, &ws, &task_input("Covered by the editor app"), Some(&sales)).unwrap();
}

#[test]
fn write_enforcement_also_gates_custom_object_records() {
    let (conn, ws, admin) = setup_workspace();
    let obj = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Property".into(), plural_label: "Properties".into(), icon: "🏠".into(), prefix: "PROP".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    let app = app_service::create(&conn, &ws, &app_input("Property Management"), Some(&admin)).unwrap();
    let update = AppDefinitionUpdate { name: app.name.clone(), icon: app.icon.clone(), description: None, object_keys: vec![obj.key.clone()], dashboard_id: None };
    app_service::update(&conn, &app.id, &update, Some(&admin)).unwrap();
    app_service::publish(&conn, &app.id, Some(&admin)).unwrap();
    app_service::grant_permission(
        &conn, &app.id,
        &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "viewer".into() },
        Some(&admin),
    )
    .unwrap();
    let sales = user_with_role(&conn, &ws, &admin, "sam", "Sales");

    let input = CustomRecordInput { object_key: obj.key.clone(), primary_name: "123 Main St".into(), status: "Active".into(), owner_user_id: None, notes: None };
    let err = custom_record_service::create(&conn, &ws, &input, Some(&sales)).unwrap_err();
    assert!(err.to_string().contains("view-only access"));
}
