use std::path::{Path, PathBuf};

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::user::{ChangeOwnPassword, NewUser};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{auth_service, backup_service, company_service, user_service, workspace_service};

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lanesra-core-test-{label}-{}.sqlite3",
        lanesra_core::domain::ids::new_uuid()
    ))
}

/// Real file-backed databases, not `:memory:` - required here since
/// restore works by replacing the database *file* out from under the live
/// connection.
fn setup_workspace(path: &Path) -> (rusqlite::Connection, String, String) {
    let conn = open_workspace_db(path).unwrap();
    let setup = WorkspaceSetup {
        business_name: "Backup Test Co".into(),
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

fn company_input(name: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(),
        status: "Prospect".into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
    }
}

#[test]
fn backup_then_restore_reverts_to_the_snapshot() {
    let path = temp_db_path("roundtrip");
    let (mut conn, ws, admin) = setup_workspace(&path);

    company_service::create(&conn, &ws, &company_input("Acme Ltd"), Some(&admin)).unwrap();

    let package = backup_service::create_backup(&conn, Some(&admin)).unwrap();
    assert_eq!(package.manifest.workspace_name, "Backup Test Co");
    assert!(!package.package_base64.is_empty());

    // Mutate after the backup was taken.
    company_service::create(&conn, &ws, &company_input("Widgets Inc"), Some(&admin)).unwrap();
    assert_eq!(company_service::list(&conn, &ws).unwrap().len(), 2);

    backup_service::restore_from_package(&mut conn, &path, &package.package_base64, Some(&admin)).unwrap();

    // The connection now points at the restored database - only the
    // company that existed at backup time should be there.
    let companies = company_service::list(&conn, &ws).unwrap();
    assert_eq!(companies.len(), 1);
    assert_eq!(companies[0].name, "Acme Ltd");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn restore_requires_an_administrator() {
    let path = temp_db_path("non-admin");
    let (mut conn, ws, admin) = setup_workspace(&path);
    let sales = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "sam".into(),
            display_name: "Sam".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".into()],
        },
        Some(&admin),
    )
    .unwrap();

    let package = backup_service::create_backup(&conn, Some(&admin)).unwrap();
    let result = backup_service::restore_from_package(&mut conn, &path, &package.package_base64, Some(&sales.id));

    assert!(result.is_err(), "a non-administrator must not be able to restore a backup");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn restore_rejects_garbage_input() {
    let path = temp_db_path("garbage");
    let (mut conn, _ws, admin) = setup_workspace(&path);

    let result = backup_service::restore_from_package(&mut conn, &path, "not a real backup file", Some(&admin));

    assert!(result.is_err(), "restore must reject input that isn't a valid .lanesra package");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn restore_rejects_a_newer_schema_version_than_this_build_supports() {
    let path = temp_db_path("future-schema");
    let (mut conn, ws, admin) = setup_workspace(&path);

    let mut package = backup_service::create_backup(&conn, Some(&admin)).unwrap();
    package.manifest.schema_version = i64::MAX;
    // Re-encode a package whose manifest claims a schema version far newer
    // than anything this build has ever run a migration for.
    let tampered = tamper_manifest_schema_version(&package.package_base64, i64::MAX);

    let result = backup_service::restore_from_package(&mut conn, &path, &tampered, Some(&admin));
    assert!(result.is_err(), "restore must refuse a backup newer than this build supports");

    // Original data must still be intact - the rejected restore must not
    // have touched anything.
    assert!(company_service::list(&conn, &ws).unwrap().is_empty());
    let _ = std::fs::remove_file(&path);
}

/// Test-only helper: unzips a package, overwrites its manifest's
/// schema_version, and re-zips it - used to simulate a backup from a
/// future version of the app without needing one to actually exist.
fn tamper_manifest_schema_version(package_base64: &str, new_version: i64) -> String {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use std::io::{Read, Write};

    let bytes = BASE64.decode(package_base64).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();

    let mut manifest_json = String::new();
    archive.by_name("manifest.json").unwrap().read_to_string(&mut manifest_json).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_json).unwrap();
    manifest["schema_version"] = serde_json::json!(new_version);

    let mut db_bytes = Vec::new();
    archive.by_name("workspace.sqlite3").unwrap().read_to_end(&mut db_bytes).unwrap();

    let mut out = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer.start_file("manifest.json", options).unwrap();
        writer.write_all(manifest.to_string().as_bytes()).unwrap();
        writer.start_file("workspace.sqlite3", options).unwrap();
        writer.write_all(&db_bytes).unwrap();
        writer.finish().unwrap();
    }
    BASE64.encode(out)
}

#[test]
fn change_own_password_requires_the_current_one() {
    let path = temp_db_path("password");
    let (conn, ws, admin) = setup_workspace(&path);

    let wrong = auth_service::change_own_password(
        &conn,
        &ws,
        Some(&admin),
        &ChangeOwnPassword {
            current_password: "totally wrong".into(),
            new_password: "brandnewsecretpw".into(),
        },
    );
    assert!(wrong.is_err(), "the current password must be verified");

    auth_service::change_own_password(
        &conn,
        &ws,
        Some(&admin),
        &ChangeOwnPassword {
            current_password: "supersecretpassword".into(),
            new_password: "brandnewsecretpw".into(),
        },
    )
    .unwrap();

    // The new password now logs in; the old one no longer does.
    let login_new = auth_service::login(
        &conn,
        &ws,
        &lanesra_core::models::user::Credentials {
            username: "admin".into(),
            password: "brandnewsecretpw".into(),
        },
    );
    assert!(login_new.is_ok());

    let login_old = auth_service::login(
        &conn,
        &ws,
        &lanesra_core::models::user::Credentials {
            username: "admin".into(),
            password: "supersecretpassword".into(),
        },
    );
    assert!(login_old.is_err());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn change_own_password_rejects_a_short_new_password() {
    let path = temp_db_path("short-password");
    let (conn, ws, admin) = setup_workspace(&path);

    let result = auth_service::change_own_password(
        &conn,
        &ws,
        Some(&admin),
        &ChangeOwnPassword {
            current_password: "supersecretpassword".into(),
            new_password: "short".into(),
        },
    );
    assert!(result.is_err());
    let _ = std::fs::remove_file(&path);
}
