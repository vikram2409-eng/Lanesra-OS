//! Screen/App Builder Phase 1: multiple named layouts per object, tabs of
//! drag-ordered field sections, role-based assignment with a required
//! default fallback, and Draft -> Preview -> Publish. Mirrors the online
//! demo's equivalent tests conceptually, but exercises the one thing the
//! demo can't: resolving a layout against a real signed-in user's roles.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::screen_layout::{LayoutSection, LayoutTab, LayoutTabs, ScreenLayoutInput, ScreenLayoutUpdate};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{screen_layout_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Screen Layouts Co".into(),
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

fn sales_user(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

fn layout_input(name: &str, initial_fields: &[&str]) -> ScreenLayoutInput {
    ScreenLayoutInput {
        entity_type: "Company".into(),
        name: name.into(),
        initial_fields: initial_fields.iter().map(|s| s.to_string()).collect(),
    }
}

fn one_field_draft(field: &str) -> LayoutTabs {
    LayoutTabs {
        tabs: vec![LayoutTab {
            id: "t1".into(),
            title: "Details".into(),
            sections: vec![LayoutSection { id: "s1".into(), title: "Details".into(), fields: vec![field.into()] }],
        }],
    }
}

#[test]
fn listing_auto_provisions_a_default_layout() {
    let (conn, ws, _admin) = setup_workspace();
    let layouts = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].name, "Default");
    assert!(layouts[0].is_default);
    assert!(layouts[0].published.is_none(), "auto-provisioned layout must start unpublished");
}

#[test]
fn a_new_layout_is_not_default_and_the_first_ever_layout_is() {
    let (conn, ws, admin) = setup_workspace();
    // Force auto-provisioning of the initial Default layout first.
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();

    let second = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &["industry"]), Some(&admin)).unwrap();
    assert!(!second.is_default);
    assert_eq!(second.draft.tabs[0].sections[0].fields, vec!["industry".to_string()]);
}

#[test]
fn the_first_layout_created_for_an_entity_becomes_the_default() {
    let (conn, ws, admin) = setup_workspace();
    // No list_layouts call yet - so nothing has auto-provisioned a Default.
    let only = screen_layout_service::create_layout(&conn, &ws, &layout_input("Only layout", &[]), Some(&admin)).unwrap();
    assert!(only.is_default);
}

#[test]
fn a_non_administrator_cannot_create_a_layout() {
    let (conn, ws, admin) = setup_workspace();
    let sam = sales_user(&conn, &ws, &admin);
    let result = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&sam));
    assert!(result.is_err());
}

#[test]
fn publish_unpublish_and_revert_round_trip() {
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);

    let update = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: one_field_draft("industry") };
    let updated = screen_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert!(updated.published.is_none());

    let published = screen_layout_service::publish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(published.published, Some(published.draft.clone()));

    // Edit the draft further without publishing - published stays as-is.
    let update2 = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: one_field_draft("city") };
    let edited = screen_layout_service::update_layout(&conn, &layout.id, &update2, Some(&admin)).unwrap();
    assert_ne!(edited.draft, edited.published.clone().unwrap());

    let reverted = screen_layout_service::revert_layout_draft(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(reverted.draft, reverted.published.clone().unwrap());

    let unpublished = screen_layout_service::unpublish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert!(unpublished.published.is_none());
}

#[test]
fn make_default_moves_the_flag_and_only_one_layout_is_ever_default() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let other = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    assert!(!other.is_default);

    let promoted = screen_layout_service::make_default(&conn, &other.id, Some(&admin)).unwrap();
    assert!(promoted.is_default);

    let old_default = screen_layout_service::get_layout(&conn, &default_layout.id).unwrap();
    assert!(!old_default.is_default);

    let all = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(all.iter().filter(|l| l.is_default).count(), 1);
}

#[test]
fn the_default_layout_cannot_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let result = screen_layout_service::delete_layout(&conn, &default_layout.id, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn a_non_default_layout_can_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let other = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    screen_layout_service::delete_layout(&conn, &other.id, Some(&admin)).unwrap();
    let remaining = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn resolve_effective_layout_returns_none_when_nothing_is_published() {
    let (conn, ws, _admin) = setup_workspace();
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", None).unwrap();
    assert!(effective.is_none());
}

#[test]
fn resolve_effective_layout_falls_back_to_the_published_default_for_an_unclaimed_role() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let update = ScreenLayoutUpdate { name: default_layout.name.clone(), roles: vec![], draft: one_field_draft("industry") };
    screen_layout_service::update_layout(&conn, &default_layout.id, &update, Some(&admin)).unwrap();
    screen_layout_service::publish_layout(&conn, &default_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert_eq!(effective.unwrap().tabs[0].sections[0].fields, vec!["industry".to_string()]);
}

#[test]
fn resolve_effective_layout_prefers_a_published_layout_matching_the_actors_role() {
    let (conn, ws, admin) = setup_workspace();
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let sales_layout = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    let update = ScreenLayoutUpdate { name: sales_layout.name.clone(), roles: vec!["Sales".into()], draft: one_field_draft("city") };
    screen_layout_service::update_layout(&conn, &sales_layout.id, &update, Some(&admin)).unwrap();
    screen_layout_service::publish_layout(&conn, &sales_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert_eq!(effective.unwrap().tabs[0].sections[0].fields, vec!["city".to_string()]);

    // The Administrator (no Sales role) is unaffected - falls back to
    // Default, which was never published, so still None.
    let admin_effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(admin_effective.is_none());
}

#[test]
fn an_unpublished_role_matching_layout_is_never_used_live() {
    let (conn, ws, admin) = setup_workspace();
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let sales_layout = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    let update = ScreenLayoutUpdate { name: sales_layout.name.clone(), roles: vec!["Sales".into()], draft: one_field_draft("city") };
    screen_layout_service::update_layout(&conn, &sales_layout.id, &update, Some(&admin)).unwrap();
    // Deliberately never published.

    let sam = sales_user(&conn, &ws, &admin);
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert!(effective.is_none(), "a draft-only layout must never affect the live form");
}

#[test]
fn an_invalid_entity_type_is_rejected() {
    let (conn, ws, _admin) = setup_workspace();
    let result = screen_layout_service::list_layouts(&conn, &ws, "NotARealEntity");
    assert!(result.is_err());
}
