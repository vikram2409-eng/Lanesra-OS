//! ADM-BR/ADM-WF "any field" targeting, the database-facing half of
//! `domain::builtin_fields`'s static registry:
//!
//! - `field_values` serializes every registered built-in field of a record
//!   into the same `HashMap<String, String>` shape custom field values
//!   already use, so `domain::conditions::condition_matches` can evaluate a
//!   condition against either kind of field with zero special-casing.
//! - `set_field` is the write side for the fields the registry marks
//!   `actionable` - it fetches the record, applies the one field on top of
//!   its *current* other values, and calls the entity's own `*_service::
//!   update` (never a raw repo write) so ordinary validation, audit
//!   logging and `workflow_service::fire_event` still run exactly as they
//!   would for a manual edit. This is what both a business rule's
//!   `set_value`/`set_default` action and a workflow's `update_field`
//!   action call when their target is a built-in field rather than a
//!   custom one.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::domain::money::{format_bp_as_percent, format_major, parse_major, parse_percent_to_bp};
use crate::domain::{builtin_fields, AppError, AppResult};
use crate::models::company::CompanyInput;
use crate::models::contact::ContactInput;
use crate::models::contract::ContractInput;
use crate::models::custom_record::CustomRecordUpdate;
use crate::models::opportunity::OpportunityInput;
use crate::models::product::ProductInput;
use crate::models::task::TaskInput;
use crate::repositories::{
    company_repo, contact_repo, contract_repo, custom_record_repo, invoice_repo, opportunity_repo, order_repo,
    product_repo, quote_repo, task_repo,
};
use crate::services::{company_service, contact_service, contract_service, custom_record_service, opportunity_service, product_service, task_service};

fn opt(s: String) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Every registered built-in field's current value, keyed the same way
/// `domain::builtin_fields::builtin_fields_for` names them - the "read"
/// half of this module. Entities the registry doesn't cover fall through
/// to `custom_record_repo` (any admin-defined custom object).
pub fn field_values(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<HashMap<String, String>> {
    let mut ctx = HashMap::new();
    match entity_type {
        "Company" => {
            if let Some(r) = company_repo::get(conn, entity_id)? {
                ctx.insert("name".into(), r.name);
                ctx.insert("status".into(), r.status);
                ctx.insert("tax_number".into(), r.tax_number.unwrap_or_default());
                ctx.insert("billing_address".into(), r.billing_address.unwrap_or_default());
                ctx.insert("shipping_address".into(), r.shipping_address.unwrap_or_default());
                ctx.insert("tags".into(), r.tags.unwrap_or_default());
                ctx.insert("notes".into(), r.notes.unwrap_or_default());
            }
        }
        "Contact" => {
            if let Some(r) = contact_repo::get(conn, entity_id)? {
                ctx.insert("first_name".into(), r.first_name);
                ctx.insert("last_name".into(), r.last_name);
                ctx.insert("job_title".into(), r.job_title.unwrap_or_default());
                ctx.insert("email".into(), r.email.unwrap_or_default());
                ctx.insert("phone".into(), r.phone.unwrap_or_default());
                ctx.insert("mobile".into(), r.mobile.unwrap_or_default());
                ctx.insert("is_primary".into(), r.is_primary.to_string());
                ctx.insert("status".into(), r.status);
                ctx.insert("tags".into(), r.tags.unwrap_or_default());
                ctx.insert("notes".into(), r.notes.unwrap_or_default());
            }
        }
        "Opportunity" => {
            if let Some(r) = opportunity_repo::get(conn, entity_id)? {
                ctx.insert("name".into(), r.name);
                ctx.insert("stage".into(), r.stage);
                ctx.insert("status".into(), r.status);
                ctx.insert("value".into(), format_major(r.value_cents));
                ctx.insert("probability".into(), format_bp_as_percent(r.probability_bp));
                ctx.insert("expected_close_date".into(), r.expected_close_date.unwrap_or_default());
                ctx.insert("lost_reason".into(), r.lost_reason.unwrap_or_default());
                ctx.insert("next_step".into(), r.next_step.unwrap_or_default());
            }
        }
        "Product" => {
            if let Some(r) = product_repo::get(conn, entity_id)? {
                ctx.insert("name".into(), r.name);
                ctx.insert("sku".into(), r.sku.unwrap_or_default());
                ctx.insert("type".into(), r.r#type);
                ctx.insert("category".into(), r.category.unwrap_or_default());
                ctx.insert("description".into(), r.description.unwrap_or_default());
                ctx.insert("unit_price".into(), format_major(r.unit_price_cents));
                ctx.insert("cost".into(), format_major(r.cost_cents));
                ctx.insert("tax_rate".into(), format_bp_as_percent(r.tax_rate_bp));
                ctx.insert("is_active".into(), r.is_active.to_string());
            }
        }
        "Quote" => {
            if let Some(r) = quote_repo::get(conn, entity_id)? {
                ctx.insert("status".into(), r.status);
                ctx.insert("issue_date".into(), r.issue_date.unwrap_or_default());
                ctx.insert("expiry_date".into(), r.expiry_date.unwrap_or_default());
                ctx.insert("terms".into(), r.terms.unwrap_or_default());
                ctx.insert("notes".into(), r.notes.unwrap_or_default());
            }
        }
        "Order" => {
            if let Some(r) = order_repo::get(conn, entity_id)? {
                ctx.insert("status".into(), r.status);
                ctx.insert("order_date".into(), r.order_date.unwrap_or_default());
                ctx.insert("notes".into(), r.notes.unwrap_or_default());
            }
        }
        "Invoice" => {
            if let Some(r) = invoice_repo::get(conn, entity_id)? {
                ctx.insert("status".into(), r.status);
                ctx.insert("issue_date".into(), r.issue_date.unwrap_or_default());
                ctx.insert("due_date".into(), r.due_date.unwrap_or_default());
                ctx.insert("payment_terms".into(), r.payment_terms.unwrap_or_default());
                ctx.insert("notes".into(), r.notes.unwrap_or_default());
            }
        }
        "Contract" => {
            if let Some(r) = contract_repo::get(conn, entity_id)? {
                ctx.insert("title".into(), r.title);
                ctx.insert("type".into(), r.r#type.unwrap_or_default());
                ctx.insert("value".into(), format_major(r.value_cents));
                ctx.insert("start_date".into(), r.start_date.unwrap_or_default());
                ctx.insert("end_date".into(), r.end_date.unwrap_or_default());
                ctx.insert("renewal_date".into(), r.renewal_date.unwrap_or_default());
                ctx.insert("notice_period_days".into(), r.notice_period_days.map(|d| d.to_string()).unwrap_or_default());
                ctx.insert("status".into(), r.status);
                ctx.insert("notes".into(), r.notes.unwrap_or_default());
            }
        }
        "Task" => {
            if let Some(r) = task_repo::get(conn, entity_id)? {
                ctx.insert("title".into(), r.title);
                ctx.insert("description".into(), r.description.unwrap_or_default());
                ctx.insert("priority".into(), r.priority);
                ctx.insert("status".into(), r.status);
                ctx.insert("due_date".into(), r.due_date.unwrap_or_default());
                ctx.insert("reminder_at".into(), r.reminder_at.unwrap_or_default());
            }
        }
        _ => {
            if let Some(r) = custom_record_repo::get(conn, entity_id)? {
                if r.object_key == entity_type {
                    ctx.insert("primary_name".into(), r.primary_name);
                    ctx.insert("status".into(), r.status);
                    ctx.insert("notes".into(), r.notes.unwrap_or_default());
                }
            }
        }
    }
    Ok(ctx)
}

/// Sets one actionable built-in field to `value`, by fetching the record's
/// other current values and re-saving through the entity's own service -
/// see this module's doc comment for why. Rejects the field up front if
/// `domain::builtin_fields` doesn't mark it actionable (status-equivalent
/// fields, and every Quote/Order/Invoice field - see that module's
/// comments for why each is excluded).
pub fn set_field(conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str, field_key: &str, value: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    if !builtin_fields::is_actionable_builtin_field(entity_type, field_key) {
        return Err(AppError::Validation(format!("'{field_key}' is not an actionable built-in field on {entity_type}")));
    }
    match entity_type {
        "Company" => {
            let r = company_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Company".into()))?;
            let mut input = CompanyInput {
                name: r.name, status: r.status, owner_user_id: r.owner_user_id, tax_number: r.tax_number,
                billing_address: r.billing_address, shipping_address: r.shipping_address, tags: r.tags, notes: r.notes,
            };
            match field_key {
                "name" => input.name = value.to_string(),
                "tax_number" => input.tax_number = opt(value.to_string()),
                "billing_address" => input.billing_address = opt(value.to_string()),
                "shipping_address" => input.shipping_address = opt(value.to_string()),
                "tags" => input.tags = opt(value.to_string()),
                "notes" => input.notes = opt(value.to_string()),
                _ => unreachable!("actionable Company fields are exhaustively handled above"),
            }
            company_service::update(conn, entity_id, &input, actor_user_id)?;
        }
        "Contact" => {
            let r = contact_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Contact".into()))?;
            let mut input = ContactInput {
                company_id: r.company_id, first_name: r.first_name, last_name: r.last_name, job_title: r.job_title,
                email: r.email, phone: r.phone, mobile: r.mobile, is_primary: r.is_primary, status: r.status,
                tags: r.tags, notes: r.notes,
            };
            match field_key {
                "first_name" => input.first_name = value.to_string(),
                "last_name" => input.last_name = value.to_string(),
                "job_title" => input.job_title = opt(value.to_string()),
                "email" => input.email = opt(value.to_string()),
                "phone" => input.phone = opt(value.to_string()),
                "mobile" => input.mobile = opt(value.to_string()),
                "is_primary" => input.is_primary = value == "true",
                "tags" => input.tags = opt(value.to_string()),
                "notes" => input.notes = opt(value.to_string()),
                _ => unreachable!("actionable Contact fields are exhaustively handled above"),
            }
            contact_service::update(conn, entity_id, &input, actor_user_id)?;
        }
        "Opportunity" => {
            let r = opportunity_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Opportunity".into()))?;
            let mut input = OpportunityInput {
                company_id: r.company_id, primary_contact_id: r.primary_contact_id, name: r.name, stage: r.stage,
                status: r.status, value_cents: r.value_cents, currency_code: r.currency_code, probability_bp: r.probability_bp,
                expected_close_date: r.expected_close_date, owner_user_id: r.owner_user_id, lost_reason: r.lost_reason,
                next_step: r.next_step,
            };
            match field_key {
                "name" => input.name = value.to_string(),
                "value" => input.value_cents = parse_major(value).ok_or_else(|| AppError::Validation(format!("'{value}' is not a valid amount")))?,
                "probability" => input.probability_bp = parse_percent_to_bp(value).ok_or_else(|| AppError::Validation(format!("'{value}' is not a valid percentage")))?,
                "expected_close_date" => input.expected_close_date = opt(value.to_string()),
                "lost_reason" => input.lost_reason = opt(value.to_string()),
                "next_step" => input.next_step = opt(value.to_string()),
                _ => unreachable!("actionable Opportunity fields are exhaustively handled above"),
            }
            opportunity_service::update(conn, entity_id, &input, actor_user_id)?;
        }
        "Product" => {
            let r = product_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Product".into()))?;
            let mut input = ProductInput {
                sku: r.sku, r#type: r.r#type, name: r.name, category: r.category, description: r.description,
                unit_price_cents: r.unit_price_cents, cost_cents: r.cost_cents, tax_rate_bp: r.tax_rate_bp,
                default_quantity_milli: r.default_quantity_milli, is_active: r.is_active,
            };
            match field_key {
                "name" => input.name = value.to_string(),
                "sku" => input.sku = opt(value.to_string()),
                "type" => input.r#type = value.to_string(),
                "category" => input.category = opt(value.to_string()),
                "description" => input.description = opt(value.to_string()),
                "unit_price" => input.unit_price_cents = parse_major(value).ok_or_else(|| AppError::Validation(format!("'{value}' is not a valid amount")))?,
                "cost" => input.cost_cents = parse_major(value).ok_or_else(|| AppError::Validation(format!("'{value}' is not a valid amount")))?,
                "tax_rate" => input.tax_rate_bp = parse_percent_to_bp(value).ok_or_else(|| AppError::Validation(format!("'{value}' is not a valid percentage")))?,
                _ => unreachable!("actionable Product fields are exhaustively handled above"),
            }
            product_service::update(conn, entity_id, &input, actor_user_id)?;
        }
        "Contract" => {
            let r = contract_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Contract".into()))?;
            let mut input = ContractInput {
                company_id: r.company_id, contact_id: r.contact_id, source_quote_id: r.source_quote_id, title: r.title,
                r#type: r.r#type, value_cents: r.value_cents, currency_code: r.currency_code, owner_user_id: r.owner_user_id,
                start_date: r.start_date, end_date: r.end_date, renewal_date: r.renewal_date,
                notice_period_days: r.notice_period_days, status: r.status, notes: r.notes,
            };
            match field_key {
                "title" => input.title = value.to_string(),
                "type" => input.r#type = opt(value.to_string()),
                "value" => input.value_cents = parse_major(value).ok_or_else(|| AppError::Validation(format!("'{value}' is not a valid amount")))?,
                "start_date" => input.start_date = opt(value.to_string()),
                "end_date" => input.end_date = opt(value.to_string()),
                "renewal_date" => input.renewal_date = opt(value.to_string()),
                "notice_period_days" => input.notice_period_days = value.trim().parse().ok(),
                "notes" => input.notes = opt(value.to_string()),
                _ => unreachable!("actionable Contract fields are exhaustively handled above"),
            }
            contract_service::update(conn, entity_id, &input, actor_user_id)?;
        }
        "Task" => {
            let r = task_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Task".into()))?;
            let mut input = TaskInput {
                title: r.title, description: r.description, owner_user_id: r.owner_user_id, priority: r.priority,
                status: r.status, due_date: r.due_date, reminder_at: r.reminder_at, related_type: r.related_type,
                related_id: r.related_id,
            };
            match field_key {
                "title" => input.title = value.to_string(),
                "description" => input.description = opt(value.to_string()),
                "priority" => input.priority = value.to_string(),
                "due_date" => input.due_date = opt(value.to_string()),
                "reminder_at" => input.reminder_at = opt(value.to_string()),
                _ => unreachable!("actionable Task fields are exhaustively handled above"),
            }
            task_service::update(conn, entity_id, workspace_id, &input, actor_user_id)?;
        }
        _ => {
            let r = custom_record_repo::get(conn, entity_id)?.ok_or_else(|| AppError::NotFound("Record".into()))?;
            let mut input = CustomRecordUpdate { primary_name: r.primary_name, status: r.status, owner_user_id: r.owner_user_id, notes: r.notes };
            match field_key {
                "primary_name" => input.primary_name = value.to_string(),
                "notes" => input.notes = opt(value.to_string()),
                _ => unreachable!("actionable custom object fields are exhaustively handled above"),
            }
            custom_record_service::update(conn, entity_id, &input, actor_user_id)?;
        }
    }
    Ok(())
}
