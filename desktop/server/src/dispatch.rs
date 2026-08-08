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
use lanesra_core::models::user::{ChangeOwnPassword, NewUser, PasswordChange, UserUpdate};
use lanesra_core::repositories::workspace_repo;
use lanesra_core::services::{
    auth_service, backup_service, company_service, contact_service, contract_service,
    dashboard_service, invoice_service, opportunity_service, order_service, product_service,
    quote_service, task_service, user_service,
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

        "create_backup" => to_value(backup_service::create_backup(conn, actor)?),
        // "restore_backup" is deliberately absent here - it replaces the
        // live connection itself, which this function only ever borrows
        // immutably. routes.rs special-cases it before locking the
        // connection, the same way login/logout mutate the session cookie
        // outside this function.

        other => Err(AppError::Validation(format!("Unknown command '{other}'"))),
    }
}
