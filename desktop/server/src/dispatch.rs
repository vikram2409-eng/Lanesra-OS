//! Mirrors the Tauri commands in `src-tauri/src/commands/*.rs` 1:1, so the
//! same frontend code can call either transport. Each arm here should stay
//! a line-for-line match of its Tauri counterpart (same service call, same
//! argument shape) - if you add or change a command, change both.

use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde_json::Value;

use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::contract::ContractInput;
use lanesra_core::models::invoice::{InvoiceInput, PaymentInput};
use lanesra_core::models::opportunity::{OpportunityInput, OpportunityProductInput};
use lanesra_core::models::order::OrderInput;
use lanesra_core::models::product::ProductInput;
use lanesra_core::models::quote::QuoteInput;
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::custom_field::{CustomFieldDefinitionInput, CustomFieldDefinitionUpdate, CustomFieldValues};
use lanesra_core::models::custom_object::{CustomObjectDefinitionInput, CustomObjectDefinitionUpdate};
use lanesra_core::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use lanesra_core::models::custom_report::{CustomReportInput, CustomReportUpdate};
use lanesra_core::models::business_rule::{BusinessRuleInput, BusinessRuleUpdate};
use lanesra_core::models::numbering_override::NumberingOverrideInput;
use lanesra_core::models::relationship::{RelationshipDefinitionInput, RelationshipDefinitionUpdate};
use lanesra_core::models::report::ReportRange;
use lanesra_core::models::status_transition::StatusTransitionInput;
use lanesra_core::models::user::{ChangeOwnPassword, NewUser, PasswordChange, UserUpdate};
use lanesra_core::models::workflow::{WorkflowDefinitionInput, WorkflowDefinitionUpdate};
use lanesra_core::models::workspace::{DashboardKpiPrefs, WorkspaceLogo, WorkspaceUpdate};
use lanesra_core::repositories::{notification_repo, workspace_repo};
use lanesra_core::services::{
    auth_service, backup_service, business_rule_service, company_service, contact_service, contract_service,
    custom_field_service, custom_object_service, custom_record_service, custom_report_service, dashboard_service,
    invoice_service, numbering_service, opportunity_service, order_service, product_service,
    quote_service, relationship_service, report_service, status_transition_service, task_service, user_service,
    workflow_service, workspace_service,
};

pub(crate) fn arg<T: DeserializeOwned>(args: &Value, key: &str) -> AppResult<T> {
    let value = args.get(key).cloned().unwrap_or(Value::Null);
    serde_json::from_value(value).map_err(|e| AppError::Validation(format!("invalid argument '{key}': {e}")))
}

pub(crate) fn require_workspace_id(conn: &Connection) -> AppResult<String> {
    workspace_repo::get_current(conn)?
        .map(|w| w.id)
        .ok_or_else(|| AppError::Validation("No workspace has been set up yet".into()))
}

pub(crate) fn to_value<T: serde::Serialize>(value: T) -> AppResult<Value> {
    serde_json::to_value(value).map_err(|e| AppError::Validation(format!("could not serialize response: {e}")))
}

/// Dispatches every command except workspace_status/first_run_setup/login/
/// logout/current_user, which the HTTP layer handles directly because they
/// mutate the session cookie.
pub fn dispatch(command: &str, args: &Value, conn: &Connection, actor: Option<&str>) -> AppResult<Value> {
    match command {
        "list_companies" => to_value(company_service::list(conn, &require_workspace_id(conn)?)?),
        "get_company" => to_value(company_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_company" => {
            let input: CompanyInput = arg(args, "input")?;
            to_value(company_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_company" => {
            let id: String = arg(args, "id")?;
            let input: CompanyInput = arg(args, "input")?;
            to_value(company_service::update(conn, &id, &input, actor)?)
        }
        "archive_company" => {
            company_service::archive(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }
        "check_company_duplicates" => {
            let name: String = arg(args, "name")?;
            let exclude_id: Option<String> = arg(args, "excludeId")?;
            to_value(company_service::check_duplicates(
                conn,
                &require_workspace_id(conn)?,
                &name,
                exclude_id.as_deref(),
            )?)
        }

        "list_contacts" => to_value(contact_service::list(conn, &require_workspace_id(conn)?)?),
        "list_contacts_by_company" => {
            to_value(contact_service::list_by_company(conn, &arg::<String>(args, "companyId")?)?)
        }
        "get_contact" => to_value(contact_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_contact" => {
            let input: ContactInput = arg(args, "input")?;
            to_value(contact_service::create(conn, &input, actor)?)
        }
        "update_contact" => {
            let id: String = arg(args, "id")?;
            let input: ContactInput = arg(args, "input")?;
            to_value(contact_service::update(conn, &id, &input, actor)?)
        }
        "archive_contact" => {
            contact_service::archive(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }
        "check_contact_duplicates" => {
            let company_id: String = arg(args, "companyId")?;
            let email: String = arg(args, "email")?;
            let exclude_id: Option<String> = arg(args, "excludeId")?;
            to_value(contact_service::check_duplicates(conn, &company_id, &email, exclude_id.as_deref())?)
        }

        "list_products" => to_value(product_service::list(conn, &require_workspace_id(conn)?)?),
        "get_product" => to_value(product_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_product" => {
            let input: ProductInput = arg(args, "input")?;
            to_value(product_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_product" => {
            let id: String = arg(args, "id")?;
            let input: ProductInput = arg(args, "input")?;
            to_value(product_service::update(conn, &id, &input, actor)?)
        }
        "archive_product" => {
            product_service::archive(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }

        "list_opportunities" => to_value(opportunity_service::list(conn, &require_workspace_id(conn)?)?),
        "list_opportunities_by_company" => to_value(opportunity_service::list_by_company(
            conn,
            &arg::<String>(args, "companyId")?,
        )?),
        "get_opportunity" => to_value(opportunity_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_opportunity" => {
            let input: OpportunityInput = arg(args, "input")?;
            to_value(opportunity_service::create(conn, &input, actor)?)
        }
        "update_opportunity" => {
            let id: String = arg(args, "id")?;
            let input: OpportunityInput = arg(args, "input")?;
            to_value(opportunity_service::update(conn, &id, &input, actor)?)
        }
        "archive_opportunity" => {
            opportunity_service::archive(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }
        "set_opportunity_products" => {
            let opportunity_id: String = arg(args, "opportunityId")?;
            let products: Vec<OpportunityProductInput> = arg(args, "products")?;
            to_value(opportunity_service::set_products(conn, &opportunity_id, &products)?)
        }
        "list_opportunity_products" => to_value(opportunity_service::list_products(
            conn,
            &arg::<String>(args, "opportunityId")?,
        )?),

        "list_quotes" => to_value(quote_service::list(conn, &require_workspace_id(conn)?)?),
        "get_quote" => to_value(quote_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_quote" => {
            let input: QuoteInput = arg(args, "input")?;
            to_value(quote_service::create(conn, &input, actor)?)
        }
        "set_quote_status" => {
            let id: String = arg(args, "id")?;
            let status: String = arg(args, "status")?;
            to_value(quote_service::set_status(conn, &id, &status, actor)?)
        }
        "convert_quote_to_order" => {
            to_value(quote_service::convert_to_order(conn, &arg::<String>(args, "quoteId")?, actor)?)
        }

        "list_orders" => to_value(order_service::list(conn, &require_workspace_id(conn)?)?),
        "get_order" => to_value(order_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_order" => {
            let input: OrderInput = arg(args, "input")?;
            to_value(order_service::create(conn, &input, actor)?)
        }
        "set_order_status" => {
            let id: String = arg(args, "id")?;
            let status: String = arg(args, "status")?;
            to_value(order_service::set_status(conn, &id, &status, actor)?)
        }
        "convert_order_to_invoice" => {
            to_value(order_service::convert_to_invoice(conn, &arg::<String>(args, "orderId")?, actor)?)
        }

        "list_invoices" => to_value(invoice_service::list(conn, &require_workspace_id(conn)?)?),
        "get_invoice" => to_value(invoice_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_invoice" => {
            let input: InvoiceInput = arg(args, "input")?;
            to_value(invoice_service::create(conn, &input, actor)?)
        }
        "issue_invoice" => to_value(invoice_service::issue(conn, &arg::<String>(args, "id")?, actor)?),
        "void_invoice" => to_value(invoice_service::void(conn, &arg::<String>(args, "id")?, actor)?),
        "record_invoice_payment" => {
            let id: String = arg(args, "id")?;
            let payment: PaymentInput = arg(args, "payment")?;
            to_value(invoice_service::record_payment(conn, &id, &payment, actor)?)
        }
        "refresh_overdue_invoices" => {
            to_value(invoice_service::refresh_overdue(conn, &require_workspace_id(conn)?)?)
        }

        "list_contracts" => to_value(contract_service::list(conn, &require_workspace_id(conn)?)?),
        "list_contracts_by_company" => {
            to_value(contract_service::list_by_company(conn, &arg::<String>(args, "companyId")?)?)
        }
        "get_contract" => to_value(contract_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_contract" => {
            let input: ContractInput = arg(args, "input")?;
            to_value(contract_service::create(conn, &input, actor)?)
        }
        "update_contract" => {
            let id: String = arg(args, "id")?;
            let input: ContractInput = arg(args, "input")?;
            to_value(contract_service::update(conn, &id, &input, actor)?)
        }
        "archive_contract" => {
            contract_service::archive(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }

        "list_tasks" => to_value(task_service::list(conn, &require_workspace_id(conn)?)?),
        "list_tasks_by_related" => {
            let related_type: String = arg(args, "relatedType")?;
            let related_id: String = arg(args, "relatedId")?;
            to_value(task_service::list_by_related(conn, &related_type, &related_id)?)
        }
        "get_task" => to_value(task_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_task" => {
            let input: TaskInput = arg(args, "input")?;
            to_value(task_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_task" => {
            let id: String = arg(args, "id")?;
            let input: TaskInput = arg(args, "input")?;
            to_value(task_service::update(conn, &id, &require_workspace_id(conn)?, &input, actor)?)
        }
        "archive_task" => {
            let id: String = arg(args, "id")?;
            task_service::archive(conn, &id, &require_workspace_id(conn)?, actor)?;
            Ok(Value::Null)
        }

        "list_users" => to_value(user_service::list(conn, &require_workspace_id(conn)?)?),
        "create_user" => {
            let input: NewUser = arg(args, "input")?;
            to_value(user_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_user" => {
            let id: String = arg(args, "id")?;
            let input: UserUpdate = arg(args, "input")?;
            to_value(user_service::update(conn, &id, &require_workspace_id(conn)?, &input, actor)?)
        }
        "set_user_password" => {
            let id: String = arg(args, "id")?;
            let input: PasswordChange = arg(args, "input")?;
            user_service::set_password(conn, &id, &require_workspace_id(conn)?, &input, actor)?;
            Ok(Value::Null)
        }

        "change_my_password" => {
            let input: ChangeOwnPassword = arg(args, "input")?;
            auth_service::change_own_password(conn, &require_workspace_id(conn)?, actor, &input)?;
            Ok(Value::Null)
        }

        "dashboard_summary" => to_value(dashboard_service::summary(conn, &require_workspace_id(conn)?)?),

        "update_workspace" => {
            let input: WorkspaceUpdate = arg(args, "input")?;
            to_value(workspace_service::update(conn, &input, actor)?)
        }
        "set_workspace_logo" => {
            let input: WorkspaceLogo = arg(args, "input")?;
            to_value(workspace_service::set_logo(conn, &input, actor)?)
        }
        "clear_workspace_logo" => to_value(workspace_service::clear_logo(conn, actor)?),
        "set_dashboard_kpis" => {
            let prefs: DashboardKpiPrefs = arg(args, "prefs")?;
            to_value(workspace_service::set_dashboard_kpis(conn, &prefs, actor)?)
        }

        "report_revenue_by_month" => {
            let range: ReportRange = arg(args, "range")?;
            to_value(report_service::revenue_by_month(conn, &require_workspace_id(conn)?, &range.from, &range.to)?)
        }
        "report_win_rate_by_owner" => {
            let range: ReportRange = arg(args, "range")?;
            to_value(report_service::win_rate_by_owner(conn, &require_workspace_id(conn)?, &range.from, &range.to)?)
        }
        "report_lost_reasons" => {
            let range: ReportRange = arg(args, "range")?;
            to_value(report_service::lost_reason_breakdown(
                conn,
                &require_workspace_id(conn)?,
                &range.from,
                &range.to,
            )?)
        }
        "report_ar_aging" => {
            let as_of_date: Option<String> = arg(args, "asOfDate")?;
            to_value(report_service::ar_aging(conn, &require_workspace_id(conn)?, &as_of_date)?)
        }
        "report_sales_by_owner" => {
            let range: ReportRange = arg(args, "range")?;
            to_value(report_service::sales_by_owner(conn, &require_workspace_id(conn)?, &range.from, &range.to)?)
        }

        "list_custom_field_definitions" => {
            let entity_type: String = arg(args, "entityType")?;
            let active_only: bool = arg(args, "activeOnly")?;
            to_value(custom_field_service::list_definitions(
                conn,
                &require_workspace_id(conn)?,
                &entity_type,
                active_only,
            )?)
        }
        "create_custom_field_definition" => {
            let input: CustomFieldDefinitionInput = arg(args, "input")?;
            to_value(custom_field_service::create_definition(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_custom_field_definition" => {
            let id: String = arg(args, "id")?;
            let input: CustomFieldDefinitionUpdate = arg(args, "input")?;
            to_value(custom_field_service::update_definition(conn, &id, &input, actor)?)
        }
        "deactivate_custom_field_definition" => {
            to_value(custom_field_service::deactivate_definition(conn, &arg::<String>(args, "id")?, actor)?)
        }
        "set_custom_field_values" => {
            let entity_type: String = arg(args, "entityType")?;
            let entity_id: String = arg(args, "entityId")?;
            let values: CustomFieldValues = arg(args, "values")?;
            to_value(custom_field_service::set_entity_values(conn, &entity_type, &entity_id, &values, actor)?)
        }
        "get_custom_field_values" => {
            to_value(custom_field_service::get_entity_values(conn, &arg::<String>(args, "entityId")?)?)
        }

        "list_business_rules" => {
            let entity_type: String = arg(args, "entityType")?;
            let active_only: bool = arg(args, "activeOnly")?;
            to_value(business_rule_service::list_rules(conn, &require_workspace_id(conn)?, &entity_type, active_only)?)
        }
        "create_business_rule" => {
            let input: BusinessRuleInput = arg(args, "input")?;
            to_value(business_rule_service::create_rule(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_business_rule" => {
            let id: String = arg(args, "id")?;
            let input: BusinessRuleUpdate = arg(args, "input")?;
            to_value(business_rule_service::update_rule(conn, &id, &input, actor)?)
        }
        "test_business_rules" => {
            let entity_type: String = arg(args, "entityType")?;
            let context: CustomFieldValues = arg(args, "context")?;
            to_value(business_rule_service::test_rules(conn, &require_workspace_id(conn)?, &entity_type, &context, actor)?)
        }

        "list_status_transitions" => {
            let entity_type: String = arg(args, "entityType")?;
            to_value(status_transition_service::list(conn, &require_workspace_id(conn)?, &entity_type, actor)?)
        }
        "create_status_transition" => {
            let input: StatusTransitionInput = arg(args, "input")?;
            to_value(status_transition_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "set_status_transition_active" => {
            let id: String = arg(args, "id")?;
            let is_active: bool = arg(args, "isActive")?;
            to_value(status_transition_service::set_active(conn, &id, is_active, actor)?)
        }
        "delete_status_transition" => {
            let id: String = arg(args, "id")?;
            to_value(status_transition_service::delete(conn, &id, actor)?)
        }

        "list_workflow_rules" => {
            let entity_type: String = arg(args, "entityType")?;
            to_value(workflow_service::list_rules(conn, &require_workspace_id(conn)?, &entity_type, actor)?)
        }
        "create_workflow_rule" => {
            let input: WorkflowDefinitionInput = arg(args, "input")?;
            to_value(workflow_service::create_rule(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_workflow_rule" => {
            let id: String = arg(args, "id")?;
            let input: WorkflowDefinitionUpdate = arg(args, "input")?;
            to_value(workflow_service::update_rule(conn, &id, &input, actor)?)
        }
        "list_workflow_runs" => {
            let workflow_id: String = arg(args, "workflowId")?;
            to_value(workflow_service::list_runs(conn, &require_workspace_id(conn)?, &workflow_id, actor)?)
        }
        "run_scheduled_workflows" => to_value(workflow_service::run_scheduled(conn, &require_workspace_id(conn)?, actor)?),
        "test_workflows" => {
            let entity_type: String = arg(args, "entityType")?;
            let context: CustomFieldValues = arg(args, "context")?;
            to_value(workflow_service::test_workflows(conn, &require_workspace_id(conn)?, &entity_type, &context, actor)?)
        }

        "list_notifications" => {
            let unread_only: bool = arg(args, "unreadOnly")?;
            let workspace_id = require_workspace_id(conn)?;
            let user_id = actor.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
            to_value(notification_repo::list_for_user(conn, &workspace_id, user_id, unread_only)?)
        }
        "mark_notification_read" => {
            notification_repo::mark_read(conn, &arg::<String>(args, "id")?)?;
            Ok(Value::Null)
        }
        "mark_all_notifications_read" => {
            let workspace_id = require_workspace_id(conn)?;
            let user_id = actor.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
            notification_repo::mark_all_read(conn, &workspace_id, user_id)?;
            Ok(Value::Null)
        }

        "list_numbering_formats" => to_value(numbering_service::list_effective(conn, &require_workspace_id(conn)?, actor)?),
        "set_numbering_format" => {
            let input: NumberingOverrideInput = arg(args, "input")?;
            to_value(numbering_service::set_override(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "reset_numbering_format" => {
            let entity_type: String = arg(args, "entityType")?;
            to_value(numbering_service::reset_override(conn, &require_workspace_id(conn)?, &entity_type, actor)?)
        }

        "list_custom_reports" => to_value(custom_report_service::list(conn, &require_workspace_id(conn)?)?),
        "create_custom_report" => {
            let input: CustomReportInput = arg(args, "input")?;
            to_value(custom_report_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_custom_report" => {
            let id: String = arg(args, "id")?;
            let input: CustomReportUpdate = arg(args, "input")?;
            to_value(custom_report_service::update(conn, &id, &input, actor)?)
        }
        "delete_custom_report" => {
            custom_report_service::delete(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }
        "run_custom_report" => {
            let id: String = arg(args, "id")?;
            let report = lanesra_core::repositories::custom_report_repo::get(conn, &id)?
                .ok_or_else(|| AppError::NotFound("Custom report".into()))?;
            to_value(custom_report_service::run(conn, &report)?)
        }

        "list_custom_objects" => {
            let active_only: bool = arg(args, "activeOnly")?;
            to_value(custom_object_service::list(conn, &require_workspace_id(conn)?, active_only)?)
        }
        "create_custom_object" => {
            let input: CustomObjectDefinitionInput = arg(args, "input")?;
            to_value(custom_object_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_custom_object" => {
            let id: String = arg(args, "id")?;
            let input: CustomObjectDefinitionUpdate = arg(args, "input")?;
            to_value(custom_object_service::update(conn, &id, &input, actor)?)
        }
        "deactivate_custom_object" => {
            let id: String = arg(args, "id")?;
            to_value(custom_object_service::deactivate(conn, &id, actor)?)
        }
        "delete_custom_object" => {
            custom_object_service::delete(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }

        "list_custom_records" => {
            let object_key: String = arg(args, "objectKey")?;
            to_value(custom_record_service::list(conn, &require_workspace_id(conn)?, &object_key)?)
        }
        "get_custom_record" => to_value(custom_record_service::get(conn, &arg::<String>(args, "id")?)?),
        "create_custom_record" => {
            let input: CustomRecordInput = arg(args, "input")?;
            to_value(custom_record_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_custom_record" => {
            let id: String = arg(args, "id")?;
            let input: CustomRecordUpdate = arg(args, "input")?;
            to_value(custom_record_service::update(conn, &id, &input, actor)?)
        }
        "archive_custom_record" => {
            let id: String = arg(args, "id")?;
            to_value(custom_record_service::archive(conn, &id, actor)?)
        }

        "list_relationship_definitions" => {
            let active_only: bool = arg(args, "activeOnly")?;
            to_value(relationship_service::list(conn, &require_workspace_id(conn)?, active_only)?)
        }
        "create_relationship_definition" => {
            let input: RelationshipDefinitionInput = arg(args, "input")?;
            to_value(relationship_service::create(conn, &require_workspace_id(conn)?, &input, actor)?)
        }
        "update_relationship_definition" => {
            let id: String = arg(args, "id")?;
            let input: RelationshipDefinitionUpdate = arg(args, "input")?;
            to_value(relationship_service::update(conn, &id, &input, actor)?)
        }
        "delete_relationship_definition" => {
            relationship_service::delete(conn, &arg::<String>(args, "id")?, actor)?;
            Ok(Value::Null)
        }
        "link_records" => {
            let workspace_id = require_workspace_id(conn)?;
            to_value(relationship_service::link(
                conn, &workspace_id,
                &arg::<String>(args, "definitionId")?,
                &arg::<String>(args, "sourceEntityType")?,
                &arg::<String>(args, "sourceId")?,
                &arg::<String>(args, "targetEntityType")?,
                &arg::<String>(args, "targetId")?,
                actor,
            )?)
        }
        "unlink_records" => {
            relationship_service::unlink(conn, &arg::<String>(args, "instanceId")?, actor)?;
            Ok(Value::Null)
        }
        "list_related_records" => {
            let workspace_id = require_workspace_id(conn)?;
            to_value(relationship_service::related_records_for(
                conn, &workspace_id, &arg::<String>(args, "entityType")?, &arg::<String>(args, "entityId")?,
            )?)
        }

        "create_backup" => to_value(backup_service::create_backup(conn, actor)?),
        // "restore_backup" is deliberately absent here - it replaces the
        // live connection itself, which this function only ever borrows
        // immutably. routes.rs special-cases it before locking the
        // connection, the same way login/logout mutate the session cookie
        // outside this function.

        other => Err(AppError::Validation(format!("Unknown command '{other}'"))),
    }
}
