//! Connector Template Library: proves every curated template in
//! `connector_template_service::list()` actually delivers what it
//! promises - it parses through `connector_service::preview_import`
//! with zero warnings (nothing ambiguous or unsupported in the spec
//! itself), and imports cleanly enough that every body-bearing action
//! ends up with a real, informative `request_schema_json` - the same
//! bar `connector_tool_service::agent_tools` requires before turning an
//! action into an AI Agent tool. An automated regression check, not just
//! an assertion in a doc comment.

use lanesra_core::models::integration::ConnectorImportInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{connector_service, connector_template_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Template Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

#[test]
fn every_template_lists_with_complete_metadata() {
    let templates = connector_template_service::list();
    assert!(!templates.is_empty());
    for t in &templates {
        assert!(!t.key.is_empty());
        assert!(!t.name.is_empty());
        assert!(matches!(t.category.as_str(), "ai_model" | "saas" | "data_warehouse"), "unexpected category '{}' on '{}'", t.category, t.key);
        assert!(!t.description.is_empty(), "'{}' has no description", t.key);
        assert!(!t.auth_mode.is_empty(), "'{}' has no auth_mode", t.key);
        assert!(!t.setup_notes.is_empty(), "'{}' has no setup_notes", t.key);
    }
}

#[test]
fn get_spec_rejects_an_unknown_key() {
    assert!(connector_template_service::get_spec("not-a-real-template").is_err());
}

#[test]
fn every_template_parses_with_zero_warnings_and_at_least_one_operation() {
    for t in connector_template_service::list() {
        let spec = connector_template_service::get_spec(&t.key).unwrap();
        let preview = connector_service::preview_import(&spec.spec_text, &spec.spec_format).unwrap_or_else(|e| panic!("template '{}' failed to parse: {e}", t.key));
        assert!(!preview.operations.is_empty(), "template '{}' discovered no operations", t.key);
        assert!(preview.warnings.is_empty(), "template '{}' produced warnings (should be a fully clean, hand-trimmed spec): {:?}", t.key, preview.warnings);
    }
}

#[test]
fn every_template_imports_with_every_body_bearing_action_carrying_an_informative_schema() {
    for t in connector_template_service::list() {
        let (conn, workspace_id, admin_id) = setup_workspace();
        let spec = connector_template_service::get_spec(&t.key).unwrap();
        let preview = connector_service::preview_import(&spec.spec_text, &spec.spec_format).unwrap();
        let input = ConnectorImportInput {
            name: t.name.clone(),
            description: Some(t.description.clone()),
            spec_text: spec.spec_text,
            spec_format: spec.spec_format,
            selected_operation_ids: preview.operations.iter().map(|o| o.operation_id.clone()).collect(),
        };
        let connector = connector_service::import(&conn, &workspace_id, &input, Some(&admin_id)).unwrap_or_else(|e| panic!("template '{}' failed to import: {e}", t.key));
        assert_eq!(connector.actions.len(), preview.operations.len(), "template '{}' dropped an operation on import", t.key);

        for action in &connector.actions {
            let has_body = action.params.iter().any(|p| p.location == "body");
            if !has_body {
                continue;
            }
            let schema_json = action.request_schema_json.as_deref().unwrap_or_else(|| panic!("template '{}' action '{}' has a body but no request_schema_json - not Tool Bridge-eligible", t.key, action.action_key));
            let schema: serde_json::Value = serde_json::from_str(schema_json).unwrap();
            let properties = schema.get("properties").and_then(|p| p.as_object()).unwrap_or_else(|| panic!("template '{}' action '{}' has an empty/uninformative body schema", t.key, action.action_key));
            assert!(!properties.is_empty(), "template '{}' action '{}' has a properties map but it's empty", t.key, action.action_key);
        }
    }
}
