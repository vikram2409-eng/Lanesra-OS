//! Admin Control Center Modernization (issue #197): "Recent Changes
//! (actor/domain/time)". Proves `audit_service::list_recent` surfaces
//! real entries written by the admin-config services this phase newly
//! instrumented (business rules, workflows, custom objects/fields,
//! relationships, screen/page/dashboard layouts, themes, execution
//! graphs, AI agents/skills/pipelines, integration connections) - not
//! just the record-CRUD services that already called `audit_repo::record`
//! before this issue.

use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{audit_service, business_rule_service, custom_object_service, workspace_service};

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
fn list_recent_surfaces_admin_config_changes_across_domains() {
    let (conn, ws, admin) = setup_workspace("Admin Audit Trail Co");

    let object = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏷️".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();

    let rule = business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            entity_type: "Company".into(),
            name: "Require phone for prospects".into(),
            description: None,
            match_type: "all".into(),
            priority: 0,
            effective_start_date: None,
            effective_end_date: None,
            app_id: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "builtin".into(),
                field_key: "status".into(),
                operator: "equals".into(),
                value: "Prospect".into(),
                compare_field_source: None,
                compare_field_key: None,
                group_id: None,
                relationship_definition_id: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "show_message".into(),
                target_field_key: None,
                target_field_source: "builtin".into(),
                action_value: None,
                message: Some("This company is a prospect".into()),
            }],
        },
        Some(&admin),
    )
    .unwrap();

    let recent = audit_service::list_recent(&conn, &ws, 50, Some(&admin)).unwrap();

    let object_entry = recent.iter().find(|e| e.entity_type.as_deref() == Some("custom_object") && e.entity_id.as_deref() == Some(&object.id));
    assert!(object_entry.is_some(), "{recent:?}");
    assert_eq!(object_entry.unwrap().event_type, "create");

    let rule_entry = recent.iter().find(|e| e.entity_type.as_deref() == Some("business_rule") && e.entity_id.as_deref() == Some(&rule.id));
    assert!(rule_entry.is_some(), "{recent:?}");
    assert_eq!(rule_entry.unwrap().event_type, "create");

    // Newest first.
    assert_eq!(recent[0].entity_id.as_deref(), Some(rule.id.as_str()));
}

#[test]
fn non_admin_cannot_list_recent_changes() {
    let (conn, ws, admin) = setup_workspace("Admin Audit Trail Guard Co");
    let non_admin_input = lanesra_core::models::user::NewUser {
        username: "sales_rep".into(),
        display_name: "Sales Rep".into(),
        password: "supersecretpassword".into(),
        roles: vec!["Sales".into()],
    };
    let non_admin = lanesra_core::services::user_service::create(&conn, &ws, &non_admin_input, Some(&admin)).unwrap();
    let err = audit_service::list_recent(&conn, &ws, 50, Some(&non_admin.id)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("admin"), "{err}");
}
