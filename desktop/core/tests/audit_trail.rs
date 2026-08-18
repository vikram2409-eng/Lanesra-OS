//! Consistency pass: every admin-configurable record should carry who
//! created/last modified it, matching the 9 built-in business entities.
//! Two shapes of gap existed before this test file (see migration
//! 0026_audit_columns.sql's own comment):
//!
//! - Group A (business_rules, workflow_definitions, relationship_definitions/
//!   instances, custom_field_definitions, custom_reports, custom_records):
//!   the columns and the repository writes already existed - only the
//!   Rust model + row-mapping read-back was missing. These tests assert
//!   the value now actually comes back out of `create`/`update`.
//! - Group B (status_transitions, numbering_configs): the columns didn't
//!   exist at all - these tests exercise the new migration + repo
//!   signature changes.
//!
//! A final group covers the audit trail itself: `custom_record_service`
//! now logs to `audit_repo` like every built-in entity's service already
//! did, and `audit_service::list_for_entity` (the new generic read
//! endpoint behind the `list_audit_events` command) can read it back.

use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput, BusinessRuleUpdate};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use lanesra_core::models::custom_report::CustomReportInput;
use lanesra_core::models::numbering_override::NumberingOverrideInput;
use lanesra_core::models::relationship::{RelationshipDefinitionInput, RelationshipDefinitionUpdate};
use lanesra_core::models::status_transition::StatusTransitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput, WorkflowDefinitionUpdate};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    audit_service, business_rule_service, custom_field_service, custom_object_service, custom_record_service,
    custom_report_service, numbering_service, relationship_service, status_transition_service, user_service,
    workflow_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Audit Trail Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "First Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

/// A second Administrator, distinct from the workspace's original one -
/// used to prove that an update's `updated_by` reflects whoever just
/// saved it, while `created_by` stays pinned to whoever made the record
/// in the first place.
fn second_admin(conn: &rusqlite::Connection, ws: &str, actor: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: "second_admin".into(), display_name: "Second Admin".into(), password: "anothersecretpw".into(), roles: vec!["Administrator".into()] },
        Some(actor),
    ).unwrap().id
}

// --- Group A: already captured, now exposed -----------------------------

fn rule_condition() -> BusinessRuleConditionInput {
    BusinessRuleConditionInput {
        field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(),
        compare_field_source: None, compare_field_key: None, group_id: None,
    }
}

fn rule_action() -> BusinessRuleActionInput {
    BusinessRuleActionInput { action_type: "show_warning".into(), target_field_key: None, target_field_source: "custom".into(), action_value: None, message: Some("Check this record".into()) }
}

#[test]
fn business_rule_exposes_created_by_and_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);

    let rule = business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            app_id: None,
            entity_type: "Company".into(), name: "Rule A".into(), description: None, match_type: "all".into(),
            priority: 0, effective_start_date: None, effective_end_date: None, conditions: vec![rule_condition()], actions: vec![rule_action()],
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(rule.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(rule.updated_by.as_deref(), Some(admin.as_str()));

    let updated = business_rule_service::update_rule(
        &conn, &rule.id,
        &BusinessRuleUpdate {
            app_id: None,
            name: "Rule A renamed".into(), description: None, match_type: "all".into(), priority: 1, is_active: true,
            effective_start_date: None, effective_end_date: None, conditions: vec![rule_condition()], actions: vec![rule_action()],
        },
        Some(&admin2),
    ).unwrap();
    // created_by never moves off the original author; updated_by tracks
    // whoever saved most recently.
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));
}

fn workflow_action() -> WorkflowActionInput {
    WorkflowActionInput {
        action_type: "create_task".into(),
        params_json: serde_json::json!({ "title": "Follow up", "description": "", "due_in_days": 1, "assignee_user_id": null }).to_string(),
    }
}

#[test]
fn workflow_definition_exposes_created_by_and_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);

    let wf = workflow_service::create_rule(
        &conn, &ws,
        &WorkflowDefinitionInput {
            app_id: None,
            entity_type: "Company".into(), name: "Workflow A".into(), description: None, trigger_type: "record_created".into(),
            trigger_status: None, trigger_field_key: None, trigger_field_source: "custom".into(), trigger_offset_days: 0,
            match_type: "all".into(), priority: 0, conditions: vec![], actions: vec![workflow_action()],
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(wf.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(wf.updated_by.as_deref(), Some(admin.as_str()));

    let updated = workflow_service::update_rule(
        &conn, &wf.id,
        &WorkflowDefinitionUpdate {
            app_id: None,
            name: "Workflow A renamed".into(), description: None, trigger_status: None, trigger_field_key: None,
            trigger_field_source: "custom".into(), trigger_offset_days: 0, match_type: "all".into(), priority: 0,
            is_active: true, conditions: vec![], actions: vec![workflow_action()],
        },
        Some(&admin2),
    ).unwrap();
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));
}

#[test]
fn relationship_definition_and_instance_expose_created_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);
    let vendor = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    ).unwrap();

    let def = relationship_service::create(
        &conn, &ws,
        &RelationshipDefinitionInput {
            source_entity_type: vendor.key.clone(), target_entity_type: "Company".into(), relationship_type: "many_to_one".into(),
            forward_label: "Client".into(), reverse_label: "Vendors".into(), is_required: false, show_related_list: true,
            delete_behavior: "restrict".into(), sort_order: 0,
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(def.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(def.updated_by.as_deref(), Some(admin.as_str()));

    let updated = relationship_service::update(
        &conn, &def.id,
        &RelationshipDefinitionUpdate {
            forward_label: "Client renamed".into(), reverse_label: "Vendors".into(), is_required: false,
            show_related_list: true, delete_behavior: "restrict".into(), sort_order: 0, is_active: true,
        },
        Some(&admin2),
    ).unwrap();
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));

    // Instances are never updated, only created/deleted - only created_by
    // is meaningful for them (see RelationshipInstance's doc comment).
    let vendor_record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Supply".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin2),
    ).unwrap();
    let company = lanesra_core::services::company_service::create(
        &conn, &ws,
        &lanesra_core::models::company::CompanyInput { name: "Acme".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() },
        Some(&admin2),
    ).unwrap();
    let instance = relationship_service::link(&conn, &ws, &def.id, &vendor.key, &vendor_record.id, "Company", &company.id, Some(&admin2)).unwrap();
    assert_eq!(instance.created_by.as_deref(), Some(admin2.as_str()));
}

#[test]
fn custom_field_definition_exposes_created_by_and_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);

    let def = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Lead Source".into(), field_type: "text".into(), options: vec![],
            required: false, show_in_list: false, sort_order: 0, min_value: None, max_value: None, max_length: None,
            regex_pattern: None, is_searchable: false, is_filterable: false, is_reportable: true, default_value: None,
            is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
        },
        Some(&admin),
    ).unwrap();
    assert_eq!(def.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(def.updated_by.as_deref(), Some(admin.as_str()));

    let updated = custom_field_service::update_definition(
        &conn, &def.id,
        &lanesra_core::models::custom_field::CustomFieldDefinitionUpdate {
            label: "Lead Source renamed".into(), options: vec![], required: false, show_in_list: false, sort_order: 0,
            is_active: true, min_value: None, max_value: None, max_length: None, regex_pattern: None, is_searchable: false,
            is_filterable: false, is_reportable: true, default_value: None, is_unique: false, help_text: None,
            placeholder: None, is_hidden_by_default: false,
        },
        Some(&admin2),
    ).unwrap();
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));
}

#[test]
fn custom_report_exposes_created_by_and_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);

    let report = custom_report_service::create(
        &conn, &ws,
        &CustomReportInput { name: "Companies by status".into(), entity_type: "Company".into(), group_by_source: "builtin".into(), group_by_field: "status".into(), aggregate: "count".into(), sum_field_key: None },
        Some(&admin),
    ).unwrap();
    assert_eq!(report.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(report.updated_by.as_deref(), Some(admin.as_str()));

    let updated = custom_report_service::update(
        &conn, &report.id,
        &CustomReportInput { name: "Companies by status v2".into(), entity_type: "Company".into(), group_by_source: "builtin".into(), group_by_field: "status".into(), aggregate: "count".into(), sum_field_key: None },
        Some(&admin2),
    ).unwrap();
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));
}

#[test]
fn custom_record_exposes_created_by_and_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);
    let vendor = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    ).unwrap();

    let record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Supply".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    assert_eq!(record.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(record.updated_by.as_deref(), Some(admin.as_str()));

    let updated = custom_record_service::update(
        &conn, &record.id,
        &CustomRecordUpdate { primary_name: "Acme Supply Co".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin2),
    ).unwrap();
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));

    let archived = custom_record_service::archive(&conn, &record.id, Some(&admin2)).unwrap();
    assert_eq!(archived.updated_by.as_deref(), Some(admin2.as_str()));
}

// --- Group B: genuine schema gap, closed by migration 0026 --------------

#[test]
fn status_transition_exposes_created_by_and_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);

    let rule = status_transition_service::create(
        &conn, &ws,
        &StatusTransitionInput { entity_type: "Company".into(), from_status: Some("Prospect".into()), to_status: "Active Customer".into() },
        Some(&admin),
    ).unwrap();
    assert_eq!(rule.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(rule.updated_by.as_deref(), Some(admin.as_str()));

    status_transition_service::set_active(&conn, &rule.id, false, Some(&admin2)).unwrap();
    let after = status_transition_service::list(&conn, &ws, "Company", Some(&admin)).unwrap();
    let rule = after.iter().find(|r| r.id == rule.id).unwrap();
    // created_by is untouched by a later set_active; updated_by moves to
    // whoever flipped the flag.
    assert_eq!(rule.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(rule.updated_by.as_deref(), Some(admin2.as_str()));
}

#[test]
fn numbering_override_upsert_preserves_created_by_but_updates_updated_by() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);

    let set = numbering_service::set_override(
        &conn, &ws,
        &NumberingOverrideInput { entity_type: "Company".into(), prefix: "ACC".into(), digits: 4 },
        Some(&admin),
    ).unwrap();
    assert!(set.is_custom);
    assert_eq!(set.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(set.updated_by.as_deref(), Some(admin.as_str()));

    // Upsert (ON CONFLICT branch): a second admin changes the same
    // override - created_by must stay pinned to whoever set it up first.
    let updated = numbering_service::set_override(
        &conn, &ws,
        &NumberingOverrideInput { entity_type: "Company".into(), prefix: "CUST".into(), digits: 5 },
        Some(&admin2),
    ).unwrap();
    assert_eq!(updated.created_by.as_deref(), Some(admin.as_str()));
    assert_eq!(updated.updated_by.as_deref(), Some(admin2.as_str()));
    assert_eq!(updated.prefix, "CUST");
}

#[test]
fn effective_numbering_has_no_actor_when_falling_back_to_the_built_in_default() {
    let (conn, ws, admin) = setup_workspace();
    // No override has ever been set for any entity type - every effective
    // row must fall back to the hardcoded default with no real record
    // (and therefore no actor) behind it.
    let all = numbering_service::list_effective(&conn, &ws, Some(&admin)).unwrap();
    let company = all.iter().find(|e| e.entity_type == "Company").unwrap();
    assert!(!company.is_custom);
    assert_eq!(company.created_by, None);
    assert_eq!(company.updated_by, None);

    numbering_service::set_override(
        &conn, &ws,
        &NumberingOverrideInput { entity_type: "Company".into(), prefix: "ACC".into(), digits: 4 },
        Some(&admin),
    ).unwrap();
    let all = numbering_service::list_effective(&conn, &ws, Some(&admin)).unwrap();
    let company = all.iter().find(|e| e.entity_type == "Company").unwrap();
    assert!(company.is_custom);
    assert_eq!(company.created_by.as_deref(), Some(admin.as_str()));

    // Reset back to the default: no override row survives, so the actor
    // fields go back to None even though one existed a moment ago.
    let reset = numbering_service::reset_override(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(!reset.is_custom);
    assert_eq!(reset.created_by, None);
    assert_eq!(reset.updated_by, None);
}

// --- Audit trail: custom_record_service -> audit_repo -> audit_service --

#[test]
fn custom_record_writes_are_logged_and_readable_through_the_audit_trail() {
    let (conn, ws, admin) = setup_workspace();
    let admin2 = second_admin(&conn, &ws, &admin);
    let vendor = custom_object_service::create(
        &conn, &ws,
        &CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 },
        Some(&admin),
    ).unwrap();

    // Before any writes, an unrelated entity has no history.
    let empty = audit_service::list_for_entity(&conn, &vendor.key, "nonexistent-id").unwrap();
    assert!(empty.is_empty());

    let record = custom_record_service::create(
        &conn, &ws,
        &CustomRecordInput { object_key: vendor.key.clone(), primary_name: "Acme Supply".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    ).unwrap();
    custom_record_service::update(
        &conn, &record.id,
        &CustomRecordUpdate { primary_name: "Acme Supply Co".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin2),
    ).unwrap();
    custom_record_service::archive(&conn, &record.id, Some(&admin2)).unwrap();

    let history = audit_service::list_for_entity(&conn, &vendor.key, &record.id).unwrap();
    assert_eq!(history.len(), 3);
    // Newest first (see audit_repo::list_for_entity's ORDER BY).
    assert_eq!(history[0].event_type, "archive");
    assert_eq!(history[1].event_type, "update");
    assert_eq!(history[2].event_type, "create");
    assert_eq!(history[2].user_id.as_deref(), Some(admin.as_str()));
    assert_eq!(history[0].user_id.as_deref(), Some(admin2.as_str()));
    assert!(history[2].summary.contains(&record.display_number));
}
