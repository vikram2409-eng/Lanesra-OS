use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::{WorkspaceLogo, WorkspaceSetup, WorkspaceUpdate};
use lanesra_core::services::{user_service, workspace_service};

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

fn update_input() -> WorkspaceUpdate {
    WorkspaceUpdate {
        business_name: "Renamed Co".into(),
        legal_name: Some("Renamed Co LLC".into()),
        business_address: Some("1 Main Street, Springfield".into()),
        phone: Some("+1 555-0100".into()),
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 500,
    }
}

#[test]
fn administrator_can_update_workspace_profile() {
    let (conn, _ws, admin) = setup_workspace();

    let updated = workspace_service::update(&conn, &update_input(), Some(&admin)).unwrap();
    assert_eq!(updated.business_name, "Renamed Co");
    assert_eq!(updated.legal_name.as_deref(), Some("Renamed Co LLC"));
    assert_eq!(updated.business_address.as_deref(), Some("1 Main Street, Springfield"));
    assert_eq!(updated.phone.as_deref(), Some("+1 555-0100"));
    assert_eq!(updated.default_tax_rate_bp, 500);
}

#[test]
fn empty_business_name_is_rejected() {
    let (conn, _ws, admin) = setup_workspace();

    let mut input = update_input();
    input.business_name = "  ".into();
    let err = workspace_service::update(&conn, &input, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Business name is required"));
}

#[test]
fn non_administrator_cannot_update_workspace() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "sam".into(),
            display_name: "Sam Sales".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".into()],
        },
        Some(&admin),
    )
    .unwrap();

    let err = workspace_service::update(&conn, &update_input(), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn administrator_can_set_and_clear_the_logo() {
    let (conn, _ws, admin) = setup_workspace();

    let logo_bytes = vec![0u8; 1024]; // stand-in PNG bytes - content doesn't matter, only size/mime are validated
    let input = WorkspaceLogo {
        logo_base64: BASE64.encode(&logo_bytes),
        logo_mime: "image/png".into(),
    };

    let with_logo = workspace_service::set_logo(&conn, &input, Some(&admin)).unwrap();
    assert!(with_logo.logo_base64.is_some());
    assert_eq!(with_logo.logo_mime.as_deref(), Some("image/png"));

    let cleared = workspace_service::clear_logo(&conn, Some(&admin)).unwrap();
    assert!(cleared.logo_base64.is_none());
    assert!(cleared.logo_mime.is_none());
}

#[test]
fn logo_with_disallowed_mime_type_is_rejected() {
    let (conn, _ws, admin) = setup_workspace();

    let input = WorkspaceLogo {
        logo_base64: BASE64.encode(b"not really a gif"),
        logo_mime: "image/gif".into(),
    };
    let err = workspace_service::set_logo(&conn, &input, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("must be a PNG or JPEG"));
}

#[test]
fn oversized_logo_is_rejected() {
    let (conn, _ws, admin) = setup_workspace();

    let too_big = vec![0u8; 300 * 1024]; // over the 256 KB cap
    let input = WorkspaceLogo {
        logo_base64: BASE64.encode(&too_big),
        logo_mime: "image/png".into(),
    };
    let err = workspace_service::set_logo(&conn, &input, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("too large"));
}
