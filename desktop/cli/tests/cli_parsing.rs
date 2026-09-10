//! Argument-parsing shape: a couple of representative commands parse as
//! expected, not full clap coverage (clap itself is already tested).

use clap::Parser;
use lanesra_cli::cli::{Cli, Command, ObjectsCommand, RecordsCommand};

#[test]
fn base_url_and_api_key_are_required_globals() {
    let cli = Cli::try_parse_from(["lanesra", "--base-url", "http://localhost:8787", "--api-key", "client_x.secret", "objects", "list"]).unwrap();
    assert_eq!(cli.base_url, "http://localhost:8787");
    assert_eq!(cli.api_key, "client_x.secret");
    assert!(matches!(cli.command, Command::Objects { command: ObjectsCommand::List }));
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
        Command::Records { command: RecordsCommand::List { object_key, filter, page, page_size, sort } } => {
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
    let result = Cli::try_parse_from(["lanesra", "--api-key", "client_x.secret", "objects", "list"]);
    assert!(result.is_err());
}
