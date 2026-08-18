//! Dashboard customization Phase 1: multiple named dashboard layouts per
//! workspace, an ordered list of widgets, role-based assignment with a
//! required default fallback, and Draft -> Publish. Mirrors
//! `screen_layouts.rs` conceptually (same underlying feature, at the
//! workspace level instead of per entity_type) - see that file's own doc
//! comment for what each test category is checking and why.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::dashboard_layout::{DashboardLayoutInput, DashboardLayoutUpdate, DashboardWidget, DashboardWidgets};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{dashboard_layout_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Dashboard Layouts Co".into(),
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

fn layout_input(name: &str, initial_kpi_keys: &[&str]) -> DashboardLayoutInput {
    DashboardLayoutInput { name: name.into(), initial_kpi_keys: initial_kpi_keys.iter().map(|s| s.to_string()).collect(), app_id: None }
}

fn one_kpi_draft(key: &str) -> DashboardWidgets {
    DashboardWidgets { widgets: vec![DashboardWidget { id: "w1".into(), kind: "kpi".into(), config: serde_json::json!({ "kpi_key": key }) }] }
}

fn kpi_keys(widgets: &DashboardWidgets) -> Vec<String> {
    widgets
        .widgets
        .iter()
        .map(|w| w.config.get("kpi_key").and_then(|v| v.as_str()).unwrap_or_default().to_string())
        .collect()
}

#[test]
fn listing_auto_provisions_a_default_layout() {
    let (conn, ws, _admin) = setup_workspace();
    let layouts = dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].name, "Default");
    assert!(layouts[0].is_default);
    assert!(layouts[0].published.is_none(), "auto-provisioned layout must start unpublished");
    assert!(layouts[0].draft.widgets.is_empty(), "auto-provisioned layout starts with no widgets");
}

#[test]
fn a_new_layout_is_not_default_and_the_first_ever_layout_is() {
    let (conn, ws, admin) = setup_workspace();
    dashboard_layout_service::list_layouts(&conn, &ws).unwrap();

    let second = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &["open_pipeline"]), Some(&admin)).unwrap();
    assert!(!second.is_default);
    assert_eq!(kpi_keys(&second.draft), vec!["open_pipeline".to_string()]);
}

#[test]
fn the_first_layout_created_becomes_the_default() {
    let (conn, ws, admin) = setup_workspace();
    let only = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Only dashboard", &[]), Some(&admin)).unwrap();
    assert!(only.is_default);
}

#[test]
fn a_non_administrator_cannot_create_a_layout() {
    let (conn, ws, admin) = setup_workspace();
    let sam = sales_user(&conn, &ws, &admin);
    let result = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &[]), Some(&sam));
    assert!(result.is_err());
}

#[test]
fn publish_unpublish_and_revert_round_trip() {
    let (conn, ws, admin) = setup_workspace();
    let layout = dashboard_layout_service::list_layouts(&conn, &ws).unwrap().remove(0);

    let update = DashboardLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: one_kpi_draft("open_pipeline"), app_id: None };
    let updated = dashboard_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert!(updated.published.is_none());

    let published = dashboard_layout_service::publish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(published.published, Some(published.draft.clone()));

    let update2 = DashboardLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: one_kpi_draft("won_revenue"), app_id: None };
    let edited = dashboard_layout_service::update_layout(&conn, &layout.id, &update2, Some(&admin)).unwrap();
    assert_ne!(edited.draft, edited.published.clone().unwrap());

    let reverted = dashboard_layout_service::revert_layout_draft(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(reverted.draft, reverted.published.clone().unwrap());

    let unpublished = dashboard_layout_service::unpublish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert!(unpublished.published.is_none());
}

#[test]
fn make_default_moves_the_flag_and_only_one_layout_is_ever_default() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = dashboard_layout_service::list_layouts(&conn, &ws).unwrap().remove(0);
    let other = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &[]), Some(&admin)).unwrap();
    assert!(!other.is_default);

    let promoted = dashboard_layout_service::make_default(&conn, &other.id, Some(&admin)).unwrap();
    assert!(promoted.is_default);

    let old_default = dashboard_layout_service::get_layout(&conn, &default_layout.id).unwrap();
    assert!(!old_default.is_default);

    let all = dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    assert_eq!(all.iter().filter(|l| l.is_default).count(), 1);
}

#[test]
fn the_default_layout_cannot_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = dashboard_layout_service::list_layouts(&conn, &ws).unwrap().remove(0);
    let result = dashboard_layout_service::delete_layout(&conn, &default_layout.id, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn a_non_default_layout_can_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    let other = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &[]), Some(&admin)).unwrap();
    dashboard_layout_service::delete_layout(&conn, &other.id, Some(&admin)).unwrap();
    let remaining = dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn resolve_effective_dashboard_returns_none_when_nothing_is_published() {
    let (conn, ws, _admin) = setup_workspace();
    let effective = dashboard_layout_service::resolve_effective_dashboard(&conn, &ws, None).unwrap();
    assert!(effective.is_none());
}

#[test]
fn resolve_effective_dashboard_falls_back_to_the_published_default_for_an_unclaimed_role() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = dashboard_layout_service::list_layouts(&conn, &ws).unwrap().remove(0);
    let update = DashboardLayoutUpdate { name: default_layout.name.clone(), roles: vec![], draft: one_kpi_draft("open_pipeline"), app_id: None };
    dashboard_layout_service::update_layout(&conn, &default_layout.id, &update, Some(&admin)).unwrap();
    dashboard_layout_service::publish_layout(&conn, &default_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = dashboard_layout_service::resolve_effective_dashboard(&conn, &ws, Some(&sam)).unwrap();
    assert_eq!(kpi_keys(&effective.unwrap()), vec!["open_pipeline".to_string()]);
}

#[test]
fn resolve_effective_dashboard_prefers_a_published_layout_matching_the_actors_role() {
    let (conn, ws, admin) = setup_workspace();
    dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    let sales_layout = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &[]), Some(&admin)).unwrap();
    let update = DashboardLayoutUpdate { name: sales_layout.name.clone(), roles: vec!["Sales".into()], draft: one_kpi_draft("won_revenue"), app_id: None };
    dashboard_layout_service::update_layout(&conn, &sales_layout.id, &update, Some(&admin)).unwrap();
    dashboard_layout_service::publish_layout(&conn, &sales_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = dashboard_layout_service::resolve_effective_dashboard(&conn, &ws, Some(&sam)).unwrap();
    assert_eq!(kpi_keys(&effective.unwrap()), vec!["won_revenue".to_string()]);

    // The Administrator (no Sales role) is unaffected - falls back to
    // Default, which was never published, so still None.
    let admin_effective = dashboard_layout_service::resolve_effective_dashboard(&conn, &ws, Some(&admin)).unwrap();
    assert!(admin_effective.is_none());
}

#[test]
fn an_unpublished_role_matching_layout_is_never_used_live() {
    let (conn, ws, admin) = setup_workspace();
    dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    let sales_layout = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &[]), Some(&admin)).unwrap();
    let update = DashboardLayoutUpdate { name: sales_layout.name.clone(), roles: vec!["Sales".into()], draft: one_kpi_draft("won_revenue"), app_id: None };
    dashboard_layout_service::update_layout(&conn, &sales_layout.id, &update, Some(&admin)).unwrap();
    // Deliberately never published.

    let sam = sales_user(&conn, &ws, &admin);
    let effective = dashboard_layout_service::resolve_effective_dashboard(&conn, &ws, Some(&sam)).unwrap();
    assert!(effective.is_none(), "a draft-only layout must never affect the live dashboard");
}

#[test]
fn a_newly_created_layout_seeds_kpi_widgets_in_order() {
    let (conn, ws, admin) = setup_workspace();
    let layout = dashboard_layout_service::create_layout(&conn, &ws, &layout_input("Sales dashboard", &["open_pipeline", "won_revenue"]), Some(&admin)).unwrap();
    assert_eq!(kpi_keys(&layout.draft), vec!["open_pipeline".to_string(), "won_revenue".to_string()]);
    assert!(layout.draft.widgets.iter().all(|w| w.kind == "kpi"));
}

#[test]
fn widgets_round_trip_through_save_and_a_fresh_reload() {
    let (conn, ws, admin) = setup_workspace();
    let layout = dashboard_layout_service::list_layouts(&conn, &ws).unwrap().remove(0);

    let draft = DashboardWidgets {
        widgets: vec![
            DashboardWidget { id: "w1".into(), kind: "kpi".into(), config: serde_json::json!({ "kpi_key": "open_pipeline" }) },
            DashboardWidget { id: "w2".into(), kind: "kpi".into(), config: serde_json::json!({ "kpi_key": "overdue_invoices" }) },
        ],
    };
    let update = DashboardLayoutUpdate { name: layout.name.clone(), roles: vec![], draft, app_id: None };
    let saved = dashboard_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert_eq!(kpi_keys(&saved.draft), vec!["open_pipeline".to_string(), "overdue_invoices".to_string()]);

    let reloaded = dashboard_layout_service::get_layout(&conn, &layout.id).unwrap();
    assert_eq!(reloaded.draft, saved.draft);
}
