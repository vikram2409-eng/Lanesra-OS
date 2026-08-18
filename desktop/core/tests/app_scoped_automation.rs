//! Per-app scoped automation: business rules, workflows, and dashboard
//! layouts can each optionally carry an `app_id` tagging them to one App
//! Builder app (migration 0028) instead of always being workspace-wide -
//! the "Scoped views within one workspace" option from the user's own
//! bug report (users/roles/the business profile stay shared workspace-
//! wide; only these three admin surfaces get a "which app" tag). See
//! migration 0028's own doc comment for the full rationale.
//!
//! `app_id: None` (the default) must keep behaving exactly as every rule/
//! workflow/dashboard layout always has - that's covered incidentally by
//! every other test file in this crate still passing unmodified. This
//! file only covers the new behavior: tagging, validation, and that the
//! tag round-trips through create/update/duplicate/restore.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::app_definition::AppDefinitionInput;
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput, BusinessRuleUpdate};
use lanesra_core::models::dashboard_layout::{DashboardLayoutInput, DashboardLayoutUpdate, DashboardWidgets};
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput, WorkflowDefinitionUpdate};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{app_service, business_rule_service, dashboard_layout_service, workflow_service, workspace_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(),
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

fn app_input(name: &str) -> AppDefinitionInput {
    AppDefinitionInput { name: name.into(), icon: "⬡".into(), description: None }
}

fn condition(field_key: &str, operator: &str, value: &str) -> BusinessRuleConditionInput {
    BusinessRuleConditionInput {
        field_source: "builtin".into(), field_key: field_key.into(), operator: operator.into(), value: value.into(),
        compare_field_source: None, compare_field_key: None, group_id: None,
    }
}

fn block_action(message: &str) -> BusinessRuleActionInput {
    BusinessRuleActionInput { action_type: "block_save".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some(message.into()) }
}

fn rule_input(app_id: Option<&str>) -> BusinessRuleInput {
    BusinessRuleInput {
        app_id: app_id.map(str::to_string),
        entity_type: "Company".into(),
        name: "Test rule".into(),
        description: None,
        match_type: "all".into(),
        priority: 0,
        effective_start_date: None,
        effective_end_date: None,
        conditions: vec![condition("status", "equals", "Prospect")],
        actions: vec![block_action("Nope")],
    }
}

fn workflow_input(app_id: Option<&str>) -> WorkflowDefinitionInput {
    WorkflowDefinitionInput {
        app_id: app_id.map(str::to_string),
        entity_type: "Company".into(),
        name: "Test workflow".into(),
        description: None,
        trigger_type: "record_created".into(),
        trigger_status: None,
        trigger_field_key: None,
        trigger_field_source: "custom".into(),
        trigger_offset_days: 0,
        match_type: "all".into(),
        priority: 0,
        conditions: vec![],
        actions: vec![WorkflowActionInput { action_type: "add_notification".into(), params_json: serde_json::json!({ "audience": "all_admins", "message": "New company" }).to_string() }],
    }
}

fn layout_input(app_id: Option<&str>) -> DashboardLayoutInput {
    DashboardLayoutInput { name: "Test dashboard".into(), initial_kpi_keys: vec![], app_id: app_id.map(str::to_string) }
}

// --- Business rules -------------------------------------------------------

#[test]
fn a_rule_tagged_to_a_real_app_round_trips_through_get_and_list() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();

    let created = business_rule_service::create_rule(&conn, &ws, &rule_input(Some(&app.id)), Some(&admin)).unwrap();
    assert_eq!(created.app_id.as_deref(), Some(app.id.as_str()));

    let listed = business_rule_service::list_rules(&conn, &ws, "Company", false).unwrap();
    assert_eq!(listed.iter().find(|r| r.id == created.id).unwrap().app_id.as_deref(), Some(app.id.as_str()));
}

#[test]
fn a_rule_with_no_app_id_stays_workspace_wide() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let created = business_rule_service::create_rule(&conn, &ws, &rule_input(None), Some(&admin)).unwrap();
    assert!(created.app_id.is_none());
}

#[test]
fn creating_a_rule_tagged_to_a_nonexistent_app_is_rejected() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let err = business_rule_service::create_rule(&conn, &ws, &rule_input(Some("no-such-app")), Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("App not found"));
}

#[test]
fn creating_a_rule_tagged_to_another_workspaces_app_is_rejected() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let (conn2, ws2, admin2) = setup_workspace("Other Co");
    let foreign_app = app_service::create(&conn2, &ws2, &app_input("Other Co's app"), Some(&admin2)).unwrap();
    drop(conn2);

    let err = business_rule_service::create_rule(&conn, &ws, &rule_input(Some(&foreign_app.id)), Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("App not found"));
}

#[test]
fn updating_a_rule_can_set_and_then_clear_its_app_id() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();
    let created = business_rule_service::create_rule(&conn, &ws, &rule_input(None), Some(&admin)).unwrap();
    assert!(created.app_id.is_none());

    let tag_update = BusinessRuleUpdate {
        name: created.name.clone(), description: created.description.clone(), match_type: created.match_type.clone(),
        priority: created.priority, is_active: created.is_active,
        effective_start_date: created.effective_start_date.clone(), effective_end_date: created.effective_end_date.clone(),
        app_id: Some(app.id.clone()),
        conditions: vec![BusinessRuleConditionInput { field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(), compare_field_source: None, compare_field_key: None, group_id: None }],
        actions: vec![BusinessRuleActionInput { action_type: "block_save".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Nope".into()) }],
    };
    let tagged = business_rule_service::update_rule(&conn, &created.id, &tag_update, Some(&admin)).unwrap();
    assert_eq!(tagged.app_id.as_deref(), Some(app.id.as_str()));

    let clear_update = BusinessRuleUpdate { app_id: None, ..tag_update };
    let cleared = business_rule_service::update_rule(&conn, &created.id, &clear_update, Some(&admin)).unwrap();
    assert!(cleared.app_id.is_none(), "clearing app_id back to None must fall the rule back to workspace-wide");
}

#[test]
fn duplicating_a_rule_carries_over_its_app_id() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();
    let created = business_rule_service::create_rule(&conn, &ws, &rule_input(Some(&app.id)), Some(&admin)).unwrap();

    let copy = business_rule_service::duplicate_rule(&conn, &created.id, Some(&admin)).unwrap();
    assert_eq!(copy.app_id.as_deref(), Some(app.id.as_str()));
}

#[test]
fn restoring_a_rule_version_restores_its_app_id_too() {
    let (conn, ws, admin) = setup_workspace("Rules Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();
    let created = business_rule_service::create_rule(&conn, &ws, &rule_input(Some(&app.id)), Some(&admin)).unwrap();

    let clear_update = BusinessRuleUpdate {
        name: created.name.clone(), description: created.description.clone(), match_type: created.match_type.clone(),
        priority: created.priority, is_active: created.is_active,
        effective_start_date: created.effective_start_date.clone(), effective_end_date: created.effective_end_date.clone(),
        app_id: None,
        conditions: vec![BusinessRuleConditionInput { field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(), compare_field_source: None, compare_field_key: None, group_id: None }],
        actions: vec![BusinessRuleActionInput { action_type: "block_save".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Nope".into()) }],
    };
    business_rule_service::update_rule(&conn, &created.id, &clear_update, Some(&admin)).unwrap();

    let versions = business_rule_service::list_versions(&conn, &created.id, Some(&admin)).unwrap();
    let pre_edit_snapshot = versions.into_iter().find(|v| v.snapshot.app_id.as_deref() == Some(app.id.as_str())).expect("the pre-clear snapshot kept its app_id");
    let restored = business_rule_service::restore_version(&conn, &created.id, &pre_edit_snapshot.id, Some(&admin)).unwrap();
    assert_eq!(restored.app_id.as_deref(), Some(app.id.as_str()));
}

// --- Workflows --------------------------------------------------------------

#[test]
fn a_workflow_tagged_to_a_real_app_round_trips() {
    let (conn, ws, admin) = setup_workspace("Workflows Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();

    let created = workflow_service::create_rule(&conn, &ws, &workflow_input(Some(&app.id)), Some(&admin)).unwrap();
    assert_eq!(created.app_id.as_deref(), Some(app.id.as_str()));

    let listed = workflow_service::list_rules(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert_eq!(listed.iter().find(|w| w.id == created.id).unwrap().app_id.as_deref(), Some(app.id.as_str()));
}

#[test]
fn creating_a_workflow_tagged_to_a_nonexistent_app_is_rejected() {
    let (conn, ws, admin) = setup_workspace("Workflows Co");
    let err = workflow_service::create_rule(&conn, &ws, &workflow_input(Some("no-such-app")), Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("App not found"));
}

#[test]
fn updating_a_workflow_can_change_its_app_id() {
    let (conn, ws, admin) = setup_workspace("Workflows Co");
    let app_a = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();
    let app_b = app_service::create(&conn, &ws, &app_input("Back Office"), Some(&admin)).unwrap();
    let created = workflow_service::create_rule(&conn, &ws, &workflow_input(Some(&app_a.id)), Some(&admin)).unwrap();

    let update = WorkflowDefinitionUpdate {
        name: created.name.clone(), description: created.description.clone(), trigger_status: created.trigger_status.clone(),
        trigger_field_key: created.trigger_field_key.clone(), trigger_field_source: created.trigger_field_source.clone(),
        trigger_offset_days: created.trigger_offset_days, match_type: created.match_type.clone(), priority: created.priority,
        is_active: created.is_active, app_id: Some(app_b.id.clone()), conditions: vec![],
        actions: vec![WorkflowActionInput { action_type: "add_notification".into(), params_json: serde_json::json!({ "audience": "all_admins", "message": "New company" }).to_string() }],
    };
    let moved = workflow_service::update_rule(&conn, &created.id, &update, Some(&admin)).unwrap();
    assert_eq!(moved.app_id.as_deref(), Some(app_b.id.as_str()));
}

#[test]
fn duplicating_a_workflow_carries_over_its_app_id() {
    let (conn, ws, admin) = setup_workspace("Workflows Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();
    let created = workflow_service::create_rule(&conn, &ws, &workflow_input(Some(&app.id)), Some(&admin)).unwrap();

    let copy = workflow_service::duplicate_rule(&conn, &created.id, Some(&admin)).unwrap();
    assert_eq!(copy.app_id.as_deref(), Some(app.id.as_str()));
}

// --- Dashboard layouts --------------------------------------------------------

#[test]
fn a_dashboard_layout_tagged_to_a_real_app_round_trips() {
    let (conn, ws, admin) = setup_workspace("Dashboards Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();

    let created = dashboard_layout_service::create_layout(&conn, &ws, &layout_input(Some(&app.id)), Some(&admin)).unwrap();
    assert_eq!(created.app_id.as_deref(), Some(app.id.as_str()));

    let listed = dashboard_layout_service::list_layouts(&conn, &ws).unwrap();
    assert_eq!(listed.iter().find(|l| l.id == created.id).unwrap().app_id.as_deref(), Some(app.id.as_str()));
}

#[test]
fn creating_a_dashboard_layout_tagged_to_a_nonexistent_app_is_rejected() {
    let (conn, ws, admin) = setup_workspace("Dashboards Co");
    let err = dashboard_layout_service::create_layout(&conn, &ws, &layout_input(Some("no-such-app")), Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("App not found"));
}

#[test]
fn updating_a_dashboard_layout_can_change_its_app_id() {
    let (conn, ws, admin) = setup_workspace("Dashboards Co");
    let app = app_service::create(&conn, &ws, &app_input("Field Ops"), Some(&admin)).unwrap();
    let created = dashboard_layout_service::create_layout(&conn, &ws, &layout_input(None), Some(&admin)).unwrap();
    assert!(created.app_id.is_none());

    let update = DashboardLayoutUpdate { name: created.name.clone(), roles: vec![], draft: DashboardWidgets::default(), app_id: Some(app.id.clone()) };
    let tagged = dashboard_layout_service::update_layout(&conn, &created.id, &update, Some(&admin)).unwrap();
    assert_eq!(tagged.app_id.as_deref(), Some(app.id.as_str()));
}

#[test]
fn updating_a_dashboard_layout_tagged_to_a_nonexistent_app_is_rejected() {
    let (conn, ws, admin) = setup_workspace("Dashboards Co");
    let created = dashboard_layout_service::create_layout(&conn, &ws, &layout_input(None), Some(&admin)).unwrap();
    let update = DashboardLayoutUpdate { name: created.name.clone(), roles: vec![], draft: DashboardWidgets::default(), app_id: Some("no-such-app".into()) };
    let err = dashboard_layout_service::update_layout(&conn, &created.id, &update, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("App not found"));
}
