//! Admin Control Center Modernization (issue #197): Recently Viewed /
//! Pinned admin items - `admin_nav_service` round-trips through the real
//! `admin_nav_history` table (migration 0071).

use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{admin_nav_service, workspace_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

#[test]
fn recording_a_visit_then_pinning_it_moves_it_from_recent_to_pinned() {
    let (conn, ws, admin) = setup_workspace("Admin Nav Co");

    admin_nav_service::record_visit(&conn, &ws, "objects", "Custom Objects", Some(&admin)).unwrap();
    admin_nav_service::record_visit(&conn, &ws, "rules", "Business Rules", Some(&admin)).unwrap();

    let recent = admin_nav_service::list_recent(&conn, &ws, 10, Some(&admin)).unwrap();
    assert_eq!(recent.len(), 2);
    assert!(recent.iter().all(|i| !i.is_pinned));

    admin_nav_service::set_pinned(&conn, &ws, "objects", true, Some(&admin)).unwrap();

    let recent_after = admin_nav_service::list_recent(&conn, &ws, 10, Some(&admin)).unwrap();
    assert_eq!(recent_after.len(), 1, "pinned items drop out of the Recent list");
    assert_eq!(recent_after[0].admin_tab, "rules");

    let pinned = admin_nav_service::list_pinned(&conn, &ws, Some(&admin)).unwrap();
    assert_eq!(pinned.len(), 1);
    assert_eq!(pinned[0].admin_tab, "objects");
}

#[test]
fn revisiting_the_same_tab_upserts_instead_of_duplicating() {
    let (conn, ws, admin) = setup_workspace("Admin Nav Upsert Co");
    admin_nav_service::record_visit(&conn, &ws, "objects", "Custom Objects", Some(&admin)).unwrap();
    admin_nav_service::record_visit(&conn, &ws, "objects", "Custom Objects", Some(&admin)).unwrap();
    let recent = admin_nav_service::list_recent(&conn, &ws, 10, Some(&admin)).unwrap();
    assert_eq!(recent.len(), 1);
}
