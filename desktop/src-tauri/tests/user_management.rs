use lanesra_os_lib::db::open_in_memory_db;
use lanesra_os_lib::models::user::{Credentials, NewUser, PasswordChange, UserUpdate};
use lanesra_os_lib::models::workspace::WorkspaceSetup;
use lanesra_os_lib::services::{auth_service, user_service, workspace_service};

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

fn new_user_input(username: &str, roles: &[&str]) -> NewUser {
    NewUser {
        username: username.into(),
        display_name: username.into(),
        password: "anothersecretpw".into(),
        roles: roles.iter().map(|r| r.to_string()).collect(),
    }
}

#[test]
fn administrator_can_create_and_list_users() {
    let (conn, ws, admin) = setup_workspace();

    let sales_user = user_service::create(&conn, &ws, &new_user_input("sam", &["Sales"]), Some(&admin)).unwrap();
    assert_eq!(sales_user.roles, vec!["Sales".to_string()]);
    assert!(sales_user.is_active);

    let all = user_service::list(&conn, &ws).unwrap();
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|u| u.username == "admin"));
    assert!(all.iter().any(|u| u.username == "sam"));
}

#[test]
fn non_administrator_cannot_create_users() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(&conn, &ws, &new_user_input("sam", &["Sales"]), Some(&admin)).unwrap();

    let result = user_service::create(
        &conn,
        &ws,
        &new_user_input("eve", &["Sales"]),
        Some(&sales_user.id),
    );

    assert!(result.is_err(), "a non-administrator must not be able to create users");
}

#[test]
fn non_administrator_cannot_update_users() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(&conn, &ws, &new_user_input("sam", &["Sales"]), Some(&admin)).unwrap();

    let result = user_service::update(
        &conn,
        &admin,
        &ws,
        &UserUpdate {
            display_name: "Hacked Name".into(),
            roles: vec!["Administrator".into()],
            is_active: true,
        },
        Some(&sales_user.id),
    );

    assert!(result.is_err(), "a non-administrator must not be able to update users");
}

#[test]
fn cannot_demote_or_deactivate_the_last_active_administrator() {
    let (conn, ws, admin) = setup_workspace();

    let demote = user_service::update(
        &conn,
        &admin,
        &ws,
        &UserUpdate {
            display_name: "Admin User".into(),
            roles: vec!["Sales".into()],
            is_active: true,
        },
        Some(&admin),
    );
    assert!(demote.is_err(), "removing the only administrator's role must be rejected");

    let deactivate = user_service::update(
        &conn,
        &admin,
        &ws,
        &UserUpdate {
            display_name: "Admin User".into(),
            roles: vec!["Administrator".into()],
            is_active: false,
        },
        Some(&admin),
    );
    assert!(deactivate.is_err(), "deactivating the only administrator must be rejected");
}

#[test]
fn can_demote_administrator_when_another_one_exists() {
    let (conn, ws, admin) = setup_workspace();
    let second_admin = user_service::create(
        &conn,
        &ws,
        &new_user_input("morgan", &["Administrator"]),
        Some(&admin),
    )
    .unwrap();

    let updated = user_service::update(
        &conn,
        &admin,
        &ws,
        &UserUpdate {
            display_name: "Admin User".into(),
            roles: vec!["Sales".into()],
            is_active: true,
        },
        Some(&second_admin.id),
    )
    .unwrap();

    assert_eq!(updated.roles, vec!["Sales".to_string()]);
}

#[test]
fn password_reset_changes_the_login_credential() {
    let (conn, ws, admin) = setup_workspace();

    user_service::set_password(
        &conn,
        &admin,
        &ws,
        &PasswordChange { new_password: "brandnewpassword".into() },
        Some(&admin),
    )
    .unwrap();

    let old_password_result = auth_service::login(
        &conn,
        &ws,
        &Credentials { username: "admin".into(), password: "supersecretpassword".into() },
    );
    assert!(old_password_result.is_err(), "the old password must no longer work");

    let new_password_result = auth_service::login(
        &conn,
        &ws,
        &Credentials { username: "admin".into(), password: "brandnewpassword".into() },
    );
    assert!(new_password_result.is_ok(), "the new password must work");
}

#[test]
fn rejects_an_unknown_role() {
    let (conn, ws, admin) = setup_workspace();
    let result = user_service::create(&conn, &ws, &new_user_input("sam", &["SuperUser"]), Some(&admin));
    assert!(result.is_err(), "an unknown role must be rejected");
}
