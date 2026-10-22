//! Argument-parsing shape: a couple of representative commands parse as
//! expected, not full clap coverage (clap itself is already tested).

use clap::Parser;
use lanesra_cli::cli::{Cli, Command, ObjectsCommand, RecordsCommand};

#[test]
fn base_url_and_api_key_are_required_globals() {
    let cli = Cli::try_parse_from([
        "lanesra",
        "--base-url",
        "http://localhost:8787",
        "--api-key",
        "client_x.secret",
        "objects",
        "list",
    ])
    .unwrap();
    assert_eq!(cli.base_url, "http://localhost:8787");
    assert_eq!(cli.api_key, "client_x.secret");
    assert!(matches!(
        cli.command,
        Command::Objects {
            command: ObjectsCommand::List
        }
    ));
}

#[test]
fn records_list_parses_its_optional_flags() {
    let cli = Cli::try_parse_from([
        "lanesra",
        "--base-url",
        "http://localhost:8787",
        "--api-key",
        "client_x.secret",
        "records",
        "list",
        "Company",
        "--filter",
        r#"{"status":"Prospect"}"#,
        "--page",
        "2",
        "--page-size",
        "25",
    ])
    .unwrap();
    match cli.command {
        Command::Records {
            command:
                RecordsCommand::List {
                    object_key,
                    filter,
                    page,
                    page_size,
                    sort,
                },
        } => {
            assert_eq!(object_key, "Company");
            assert_eq!(filter.as_deref(), Some(r#"{"status":"Prospect"}"#));
            assert_eq!(page, Some(2));
            assert_eq!(page_size, Some(25));
            assert_eq!(sort, None);
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn missing_required_base_url_is_rejected() {
    let result =
        Cli::try_parse_from(["lanesra", "--api-key", "client_x.secret", "objects", "list"]);
    assert!(result.is_err());
}

#[test]
fn search_parses_its_text_and_filters_into_the_query_string() {
    // Covers: REQ-INT-006 (WP-0186 AC-5)
    let cli = Cli::try_parse_from([
        "lanesra",
        "--base-url",
        "http://localhost:8787",
        "--api-key",
        "client_x.secret",
        "search",
        "acme corp",
        "--object",
        "Company",
        "--page-size",
        "10",
    ])
    .unwrap();
    match cli.command {
        Command::Search {
            text,
            object,
            page,
            page_size,
        } => {
            assert_eq!(
                lanesra_cli::search_query(text, object, page, page_size),
                vec![
                    ("q", "acme corp".to_string()),
                    ("object", "Company".to_string()),
                    ("page_size", "10".to_string()),
                ]
            );
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn records_export_and_bulk_parse() {
    // Covers: REQ-INT-006 (WP-0187 AC-7)
    let export = Cli::try_parse_from([
        "lanesra",
        "--base-url",
        "http://localhost:8787",
        "--api-key",
        "client_x.secret",
        "records",
        "export",
        "Company",
        "--format",
        "csv",
    ])
    .unwrap();
    assert!(matches!(
        export.command,
        Command::Records { command: RecordsCommand::Export { ref object_key, ref format } }
            if object_key == "Company" && format == "csv"
    ));
    let bulk = Cli::try_parse_from([
        "lanesra",
        "--base-url",
        "http://localhost:8787",
        "--api-key",
        "client_x.secret",
        "records",
        "bulk",
        "Company",
        "--data",
        r#"[{"op":"archive","id":"abc"}]"#,
    ])
    .unwrap();
    match bulk.command {
        Command::Records {
            command: RecordsCommand::Bulk { data, .. },
        } => {
            let body = lanesra_cli::bulk_body(&data).unwrap();
            assert_eq!(body["operations"][0]["op"], "archive");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn bulk_data_can_come_from_a_file_and_must_be_an_array() {
    // Covers: REQ-INT-006 (WP-0187 AC-7)
    let path = std::env::temp_dir().join(format!("lanesra-bulk-{}.json", std::process::id()));
    std::fs::write(&path, r#"[{"op":"create","data":{"name":"Acme"}}]"#).unwrap();
    let body = lanesra_cli::bulk_body(&format!("@{}", path.display())).unwrap();
    assert_eq!(body["operations"][0]["data"]["name"], "Acme");
    std::fs::remove_file(&path).unwrap();
    assert_eq!(lanesra_cli::bulk_body(r#"{"op":"create"}"#), Err(2));
}
