//! Admin UX polish (spec §10): the cross-cutting dependency-warning check -
//! `custom_field_service::describe_active_dependents` fans out to both
//! `business_rule_service::describe_active_rules_referencing_field` and
//! `workflow_service::describe_active_workflows_referencing_field`, so this
//! covers the combination rather than duplicating either engine's own
//! per-service coverage (see `business_rules.rs`/`workflow_automation.rs`
//! for those).

use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::db::open_in_memory_db;
use lanesra_core::services::{business_rule_service, custom_field_service, user_service, workflow_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Admin UX Polish Test Co".into(),
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

fn text_field_input(label: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: "Company".into(), label: label.into(), field_type: "text".into(),
        options: vec![], required: false, show_in_list: false, sort_order: 0,
        min_value: None, max_value: None, max_length: None, regex_pattern: None,
        is_searchable: false, is_filterable: false, is_reportable: true,
        default_value: None, is_unique: false, help_text: None, placeholder: None,
        is_hidden_by_default: false,
    }
}

#[test]
fn describe_active_dependents_reports_both_a_referencing_rule_and_a_referencing_workflow() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();

    // No rule or workflow references it yet.
    assert!(custom_field_service::describe_active_dependents(&conn, &def.id, Some(&admin)).unwrap().is_empty());

    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            app_id: None,
            entity_type: "Company".into(), name: "Requires lead source".into(), description: None, match_type: "all".into(),
            priority: 0, effective_start_date: None, effective_end_date: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(),
                compare_field_source: None, compare_field_key: None, group_id: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "require".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(),
                action_value: None, message: None,
            }],
        },
        Some(&admin),
    ).unwrap();

    workflow_service::create_rule(
        &conn, &ws,
        &WorkflowDefinitionInput {
            app_id: None,
            entity_type: "Company".into(), name: "Watch lead source".into(), description: None,
            trigger_type: "field_changed".into(), trigger_status: None, trigger_field_key: Some(def.key.clone()),
            trigger_field_source: "custom".into(), trigger_offset_days: 0, match_type: "all".into(), priority: 0,
            conditions: vec![],
            actions: vec![WorkflowActionInput {
                action_type: "add_notification".into(),
                params_json: serde_json::json!({"message": "Lead source changed", "audience": "all_admins"}).to_string(),
            }],
        },
        Some(&admin),
    ).unwrap();

    let dependents = custom_field_service::describe_active_dependents(&conn, &def.id, Some(&admin)).unwrap();
    assert_eq!(dependents.len(), 2);
    assert!(dependents.iter().any(|d| d.contains("Business rule")));
    assert!(dependents.iter().any(|d| d.contains("Workflow")));
}

#[test]
fn describe_active_dependents_is_advisory_not_blocking() {
    // Deactivating the field itself still succeeds even with an active
    // dependent - the check is a query the frontend confirms against,
    // never a server-side block (see custom_field_service's doc comment
    // on describe_active_dependents).
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            app_id: None,
            entity_type: "Company".into(), name: "Requires lead source".into(), description: None, match_type: "all".into(),
            priority: 0, effective_start_date: None, effective_end_date: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(),
                compare_field_source: None, compare_field_key: None, group_id: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "require".into(), target_field_key: Some(def.key.clone()), target_field_source: "custom".into(),
                action_value: None, message: None,
            }],
        },
        Some(&admin),
    ).unwrap();

    assert_eq!(custom_field_service::describe_active_dependents(&conn, &def.id, Some(&admin)).unwrap().len(), 1);
    let deactivated = custom_field_service::deactivate_definition(&conn, &def.id, Some(&admin)).unwrap();
    assert!(!deactivated.is_active);
}

#[test]
fn non_administrator_cannot_query_dependents() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_field_service::create_definition(&conn, &ws, &text_field_input("Lead Source"), Some(&admin)).unwrap();
    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();

    let err = custom_field_service::describe_active_dependents(&conn, &def.id, Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}
