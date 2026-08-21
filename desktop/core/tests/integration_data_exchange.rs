//! Integration Hub (spec §12/§13/§14): proves `mapping_service` and
//! `data_exchange_service` end-to-end against a real SQLite workspace -
//! CSV parsing, column mapping (transforms/defaults/constants),
//! insert/update/upsert with duplicate handling, dry-run, and CSV export -
//! all through the same `api_object_service` dispatcher the inbound REST
//! API uses.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::{CsvImportInput, FieldMapEntry, MappingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, data_exchange_service, mapping_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Data Exchange Co".into(),
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

fn simple_field_map() -> Vec<FieldMapEntry> {
    vec![
        FieldMapEntry { source_column: "name".into(), target_field: "name".into(), transform: None, default_value: None, constant: None },
        FieldMapEntry { source_column: "status".into(), target_field: "status".into(), transform: None, default_value: Some("Prospect".into()), constant: None },
        FieldMapEntry { source_column: "email".into(), target_field: "email".into(), transform: None, default_value: None, constant: None },
    ]
}

#[test]
fn mapping_crud_and_validation_rules() {
    let (conn, workspace_id, admin_id) = setup_workspace();

    let created = mapping_service::create(
        &conn,
        &workspace_id,
        &MappingInput { name: "Company Import".into(), target_object_key: "Company".into(), operation: "upsert".into(), match_key: Some("email".into()), field_map: simple_field_map(), duplicate_policy: "update_matched".into() },
        Some(&admin_id),
    )
    .unwrap();
    assert_eq!(created.name, "Company Import");

    let listed = mapping_service::list_for_workspace(&conn, &workspace_id).unwrap();
    assert_eq!(listed.len(), 1);

    // An update/upsert mapping without a match key is rejected.
    let err = mapping_service::create(
        &conn,
        &workspace_id,
        &MappingInput { name: "Bad".into(), target_object_key: "Company".into(), operation: "upsert".into(), match_key: None, field_map: simple_field_map(), duplicate_policy: "skip".into() },
        Some(&admin_id),
    );
    assert!(err.is_err());

    // An unknown operation/duplicate_policy is rejected.
    let err = mapping_service::create(
        &conn,
        &workspace_id,
        &MappingInput { name: "Bad2".into(), target_object_key: "Company".into(), operation: "sync".into(), match_key: None, field_map: simple_field_map(), duplicate_policy: "skip".into() },
        Some(&admin_id),
    );
    assert!(err.is_err());

    // Non-admins cannot manage mappings.
    let err = mapping_service::create(
        &conn,
        &workspace_id,
        &MappingInput { name: "NoAuth".into(), target_object_key: "Company".into(), operation: "insert".into(), match_key: None, field_map: simple_field_map(), duplicate_policy: "skip".into() },
        None,
    );
    assert!(err.is_err());

    mapping_service::delete(&conn, &workspace_id, &created.id, Some(&admin_id)).unwrap();
    assert!(mapping_service::list_for_workspace(&conn, &workspace_id).unwrap().is_empty());
}

#[test]
fn csv_import_creates_new_companies() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let csv_text = "name,status,email\nAcme Corp,Active Customer,acme@example.com\nGlobex,Prospect,globex@example.com\n";
    let input = CsvImportInput { target_object_key: "Company".into(), csv_text: csv_text.into(), operation: "insert".into(), match_key: None, field_map: simple_field_map(), duplicate_policy: "skip".into(), dry_run: false };

    let result = data_exchange_service::import_csv(&conn, &workspace_id, &input, None).unwrap();
    assert_eq!(result.total_rows, 2);
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);
    assert!(result.row_results.iter().all(|r| r.status == "created"));

    let companies = company_service::list(&conn, &workspace_id).unwrap();
    assert_eq!(companies.len(), 2);
    assert!(companies.iter().any(|c| c.name == "Acme Corp" && c.status == "Active Customer"));
}

#[test]
fn csv_import_dry_run_writes_nothing() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let csv_text = "name,status,email\nAcme Corp,Active Customer,acme@example.com\n";
    let input = CsvImportInput { target_object_key: "Company".into(), csv_text: csv_text.into(), operation: "insert".into(), match_key: None, field_map: simple_field_map(), duplicate_policy: "skip".into(), dry_run: true };

    let result = data_exchange_service::import_csv(&conn, &workspace_id, &input, None).unwrap();
    assert_eq!(result.total_rows, 1);
    assert_eq!(result.row_results[0].status, "would_create");
    assert!(company_service::list(&conn, &workspace_id).unwrap().is_empty(), "dry run must not write anything");
}

#[test]
fn csv_import_upsert_updates_an_existing_match_and_creates_the_rest() {
    let (conn, workspace_id, actor) = setup_workspace();
    let existing = company_service::create(
        &conn,
        &workspace_id,
        &lanesra_core::models::company::CompanyInput { name: "Acme Corp".into(), status: "Prospect".into(), email: Some("acme@example.com".into()), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() },
        Some(&actor),
    )
    .unwrap();

    let csv_text = "name,status,email\nAcme Corp Renamed,Active Customer,acme@example.com\nNew Co,Prospect,new@example.com\n";
    let input = CsvImportInput { target_object_key: "Company".into(), csv_text: csv_text.into(), operation: "upsert".into(), match_key: Some("email".into()), field_map: simple_field_map(), duplicate_policy: "update_matched".into(), dry_run: false };

    let result = data_exchange_service::import_csv(&conn, &workspace_id, &input, None).unwrap();
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);
    assert_eq!(result.row_results[0].status, "updated");
    assert_eq!(result.row_results[0].record_id.as_deref(), Some(existing.id.as_str()));
    assert_eq!(result.row_results[1].status, "created");

    let updated = company_service::get(&conn, &existing.id).unwrap();
    assert_eq!(updated.name, "Acme Corp Renamed");
    assert_eq!(updated.status, "Active Customer");
    assert_eq!(company_service::list(&conn, &workspace_id).unwrap().len(), 2);
}

#[test]
fn csv_import_skip_duplicate_policy_leaves_the_matched_record_untouched() {
    let (conn, workspace_id, actor) = setup_workspace();
    let existing = company_service::create(
        &conn,
        &workspace_id,
        &lanesra_core::models::company::CompanyInput { name: "Acme Corp".into(), status: "Prospect".into(), email: Some("acme@example.com".into()), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() },
        Some(&actor),
    )
    .unwrap();

    let csv_text = "name,status,email\nAcme Corp Renamed,Active Customer,acme@example.com\n";
    let input = CsvImportInput { target_object_key: "Company".into(), csv_text: csv_text.into(), operation: "upsert".into(), match_key: Some("email".into()), field_map: simple_field_map(), duplicate_policy: "skip".into(), dry_run: false };

    let result = data_exchange_service::import_csv(&conn, &workspace_id, &input, None).unwrap();
    assert_eq!(result.skipped_duplicates, 1);
    assert_eq!(result.row_results[0].status, "skipped");

    let unchanged = company_service::get(&conn, &existing.id).unwrap();
    assert_eq!(unchanged.name, "Acme Corp", "skip policy must not modify the matched record");
}

#[test]
fn csv_import_update_operation_fails_a_row_with_no_match() {
    let (conn, workspace_id, _actor) = setup_workspace();
    let csv_text = "name,status,email\nGhost Co,Prospect,ghost@example.com\n";
    let input = CsvImportInput { target_object_key: "Company".into(), csv_text: csv_text.into(), operation: "update".into(), match_key: Some("email".into()), field_map: simple_field_map(), duplicate_policy: "update_matched".into(), dry_run: false };

    let result = data_exchange_service::import_csv(&conn, &workspace_id, &input, None).unwrap();
    assert_eq!(result.failed, 1);
    assert_eq!(result.row_results[0].status, "failed");
    assert!(result.row_results[0].error.is_some());
}

#[test]
fn csv_import_applies_transform_and_default_value() {
    let (conn, workspace_id, _actor) = setup_workspace();
    let field_map = vec![
        FieldMapEntry { source_column: "name".into(), target_field: "name".into(), transform: Some("trim".into()), default_value: None, constant: None },
        FieldMapEntry { source_column: "status".into(), target_field: "status".into(), transform: None, default_value: Some("Prospect".into()), constant: None },
        FieldMapEntry { source_column: "unused".into(), target_field: "email".into(), transform: None, default_value: None, constant: Some("constant@example.com".into()) },
    ];
    // status column present but blank -> falls back to default_value;
    // name has stray whitespace the "trim" transform should remove.
    let csv_text = "name,status,unused\n  Acme Corp  ,,ignored\n";
    let input = CsvImportInput { target_object_key: "Company".into(), csv_text: csv_text.into(), operation: "insert".into(), match_key: None, field_map, duplicate_policy: "skip".into(), dry_run: false };

    let result = data_exchange_service::import_csv(&conn, &workspace_id, &input, None).unwrap();
    assert_eq!(result.successful, 1, "{result:?}");
    let companies = company_service::list(&conn, &workspace_id).unwrap();
    assert_eq!(companies[0].name, "Acme Corp");
    assert_eq!(companies[0].status, "Prospect");
    assert_eq!(companies[0].email.as_deref(), Some("constant@example.com"));
}

#[test]
fn export_csv_round_trips_selected_columns() {
    let (conn, workspace_id, actor) = setup_workspace();
    company_service::create(&conn, &workspace_id, &lanesra_core::models::company::CompanyInput { name: "Acme Corp".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&actor)).unwrap();
    company_service::create(&conn, &workspace_id, &lanesra_core::models::company::CompanyInput { name: "Globex".into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }, Some(&actor)).unwrap();

    let query = lanesra_core::models::integration::ApiListQuery { select: Some(vec!["name".into(), "status".into()]), sort: Some(vec!["name".into()]), page: None, page_size: None, filter: None };
    let csv_text = data_exchange_service::export_csv(&conn, &workspace_id, "Company", &query).unwrap();

    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(csv_text.as_bytes());
    let headers: Vec<String> = reader.headers().unwrap().iter().map(String::from).collect();
    assert_eq!(headers, vec!["name", "status"]);
    let rows: Vec<Vec<String>> = reader.records().map(|r| r.unwrap().iter().map(String::from).collect()).collect();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|r| r[0] == "Acme Corp" && r[1] == "Prospect"));
    assert!(rows.iter().any(|r| r[0] == "Globex" && r[1] == "Active Customer"));
}
