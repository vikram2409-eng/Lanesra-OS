//! Bulk Actions (product backlog "Saved Views & Bulk Actions"): apply one
//! operation across a multi-select of records, reusing each entity's own
//! existing create/update/archive path - never a second, parallel write
//! path, so every validation/business-rule/status-transition check the
//! single-record UI already enforces fires identically here, by
//! construction.
//!
//! Every operation is independent per record and never aborts partway: a
//! blocked status transition, a required-field violation, or an archive
//! blocked by a dependent record fails just that one id - the caller gets
//! a `BulkActionResult` per id and decides what to show, the same
//! "partial success is normal" shape `data_exchange_service`'s CSV import
//! already established for a multi-row operation in this codebase.
//!
//! Scope is narrower than Saved Views (any listable object_key), because
//! this only touches objects with a real, plain write path:
//! - **Update a built-in field**: delegates straight to
//!   `builtin_field_service::set_field`, so the object/field-key
//!   allowlist is whatever that registry already exposes (Company,
//!   Contact, Opportunity, Product, Contract, Task, Custom Objects) -
//!   not reimplemented here.
//! - **Update a custom field**: any entity type, via
//!   `custom_field_service` - already fully generic. `set_entity_values`
//!   is a full replace of every custom field value on the record, so
//!   this fetches the current values first and merges in just the one
//!   changed key, rather than wiping every other custom field blank.
//! - **Reassign owner**: only entities with an `owner_user_id` column -
//!   Company, Contract, Opportunity, Task, Custom Objects. Contact and
//!   Product have none.
//! - **Change status/stage**: Company, Contact, Contract, Opportunity
//!   (its `stage`), Task, Custom Objects - routes through each entity's
//!   own `update()`, so `status_transition_service` validates every move
//!   exactly as the single-record UI does.
//! - **Add/remove tags**: Company, Contact only - the two entities with a
//!   `tags` column. Merges into the existing comma-separated value rather
//!   than overwriting it.
//! - **Archive**: Company, Contact, Opportunity, Product, Contract, Task,
//!   Custom Objects - each entity's own existing `archive()`.
//!
//! Quote/Order/Invoice are out of scope for every operation here - the
//! same boundary `api_object_service` already draws: compound documents
//! with their own conversion workflow and status-transition commands
//! (issue/void/convert), not a plain field-level write.

use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;

use crate::domain::{AppError, AppResult};
use crate::models::company::CompanyInput;
use crate::models::contact::ContactInput;
use crate::models::contract::ContractInput;
use crate::models::custom_record::CustomRecordUpdate;
use crate::models::opportunity::OpportunityInput;
use crate::models::task::TaskInput;
use crate::repositories::{company_repo, contact_repo, contract_repo, custom_record_repo, opportunity_repo, task_repo};
use crate::services::{
    builtin_field_service, company_service, contact_service, contract_service, custom_field_service, custom_object_service,
    custom_record_service, opportunity_service, task_service,
};

#[derive(Debug, Clone, Serialize)]
pub struct BulkActionResult {
    pub id: String,
    pub ok: bool,
    pub error: Option<String>,
}

fn ok_result(id: &str) -> BulkActionResult {
    BulkActionResult { id: id.to_string(), ok: true, error: None }
}

fn err_result(id: &str, e: impl std::fmt::Display) -> BulkActionResult {
    BulkActionResult { id: id.to_string(), ok: false, error: Some(e.to_string()) }
}

const OWNER_ASSIGNABLE_BUILTINS: &[&str] = &["Company", "Contract", "Opportunity", "Task"];
const STATUS_CHANGEABLE_BUILTINS: &[&str] = &["Company", "Contact", "Contract", "Opportunity", "Task"];
const TAGGABLE_BUILTINS: &[&str] = &["Company", "Contact"];
const ARCHIVABLE_BUILTINS: &[&str] = &["Company", "Contact", "Opportunity", "Product", "Contract", "Task"];

fn is_custom(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<bool> {
    Ok(custom_object_service::get_by_key(conn, workspace_id, object_key)?.is_some())
}

fn require_supported(conn: &Connection, workspace_id: &str, object_key: &str, builtins: &[&str], op: &str) -> AppResult<bool> {
    if builtins.contains(&object_key) {
        return Ok(false); // not custom
    }
    if is_custom(conn, workspace_id, object_key)? {
        return Ok(true); // custom
    }
    Err(AppError::Validation(format!(
        "'{object_key}' doesn't support bulk {op} - it's either a compound document with its own workflow (Quote/Order/Invoice) or has no {op} concept on this object."
    )))
}

/// Bulk: update one built-in field to the same value across every id.
/// Delegates entirely to `builtin_field_service::set_field`, which
/// already knows exactly which fields are actionable per entity type.
pub fn bulk_update_builtin_field(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    ids: &[String],
    field_key: &str,
    value: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        results.push(match builtin_field_service::set_field(conn, workspace_id, object_key, id, field_key, value, actor_user_id) {
            Ok(()) => ok_result(id),
            Err(e) => err_result(id, e),
        });
    }
    Ok(results)
}

/// Bulk: update one custom field to the same value across every id.
/// `set_entity_values` replaces the entity's *entire* custom-field value
/// set, so each record's current values are fetched first and the one
/// changed key is merged in - every other custom field value on that
/// record is left exactly as it was.
pub fn bulk_update_custom_field(
    conn: &Connection,
    object_key: &str,
    ids: &[String],
    field_key: &str,
    value: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        let outcome = (|| -> AppResult<()> {
            let mut values: HashMap<String, String> = custom_field_service::get_entity_values(conn, id)?;
            values.insert(field_key.to_string(), value.to_string());
            custom_field_service::set_entity_values(conn, object_key, id, &values, actor_user_id)?;
            Ok(())
        })();
        results.push(match outcome {
            Ok(()) => ok_result(id),
            Err(e) => err_result(id, e),
        });
    }
    Ok(results)
}

pub fn bulk_reassign_owner(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    ids: &[String],
    owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    let is_custom_obj = require_supported(conn, workspace_id, object_key, OWNER_ASSIGNABLE_BUILTINS, "reassign owner")?;
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        let outcome: AppResult<()> = if is_custom_obj {
            let r = custom_record_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Record".into()))?;
            let input = CustomRecordUpdate { primary_name: r.primary_name, status: r.status, owner_user_id: owner_user_id.map(str::to_string), notes: r.notes };
            custom_record_service::update(conn, id, &input, actor_user_id).map(|_| ())
        } else {
            match object_key {
                "Company" => {
                    let r = company_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Company".into()))?;
                    let input = CompanyInput {
                        name: r.name, status: r.status, owner_user_id: owner_user_id.map(str::to_string), tax_number: r.tax_number,
                        billing_address: r.billing_address, shipping_address: r.shipping_address, tags: r.tags, notes: r.notes,
                        phone: r.phone, email: r.email, website: r.website, annual_revenue_cents: r.annual_revenue_cents,
                        employee_count: r.employee_count, preferred_contact_method: r.preferred_contact_method,
                    };
                    company_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Contract" => {
                    let r = contract_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Contract".into()))?;
                    let input = ContractInput {
                        company_id: r.company_id, contact_id: r.contact_id, source_quote_id: r.source_quote_id, title: r.title,
                        r#type: r.r#type, value_cents: r.value_cents, currency_code: r.currency_code, owner_user_id: owner_user_id.map(str::to_string),
                        start_date: r.start_date, end_date: r.end_date, renewal_date: r.renewal_date,
                        notice_period_days: r.notice_period_days, status: r.status, notes: r.notes,
                    };
                    contract_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Opportunity" => {
                    let r = opportunity_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Opportunity".into()))?;
                    let input = OpportunityInput {
                        company_id: r.company_id, primary_contact_id: r.primary_contact_id, name: r.name, stage: r.stage,
                        status: r.status, value_cents: r.value_cents, currency_code: r.currency_code, probability_bp: r.probability_bp,
                        expected_close_date: r.expected_close_date, owner_user_id: owner_user_id.map(str::to_string), lost_reason: r.lost_reason,
                        next_step: r.next_step,
                    };
                    opportunity_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Task" => {
                    let r = task_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Task".into()))?;
                    let input = TaskInput {
                        title: r.title, description: r.description, owner_user_id: owner_user_id.map(str::to_string), priority: r.priority,
                        status: r.status, due_date: r.due_date, reminder_at: r.reminder_at, related_type: r.related_type,
                        related_id: r.related_id,
                    };
                    task_service::update(conn, id, workspace_id, &input, actor_user_id).map(|_| ())
                }
                other => Err(AppError::Validation(format!("'{other}' doesn't support bulk reassign owner"))),
            }
        };
        results.push(match outcome {
            Ok(()) => ok_result(id),
            Err(e) => err_result(id, e),
        });
    }
    Ok(results)
}

pub fn bulk_change_status(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    ids: &[String],
    new_status: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    let is_custom_obj = require_supported(conn, workspace_id, object_key, STATUS_CHANGEABLE_BUILTINS, "change status")?;
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        let outcome: AppResult<()> = if is_custom_obj {
            let r = custom_record_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Record".into()))?;
            let input = CustomRecordUpdate { primary_name: r.primary_name, status: new_status.to_string(), owner_user_id: r.owner_user_id, notes: r.notes };
            custom_record_service::update(conn, id, &input, actor_user_id).map(|_| ())
        } else {
            match object_key {
                "Company" => {
                    let r = company_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Company".into()))?;
                    let input = CompanyInput {
                        name: r.name, status: new_status.to_string(), owner_user_id: r.owner_user_id, tax_number: r.tax_number,
                        billing_address: r.billing_address, shipping_address: r.shipping_address, tags: r.tags, notes: r.notes,
                        phone: r.phone, email: r.email, website: r.website, annual_revenue_cents: r.annual_revenue_cents,
                        employee_count: r.employee_count, preferred_contact_method: r.preferred_contact_method,
                    };
                    company_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Contact" => {
                    let r = contact_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Contact".into()))?;
                    let input = ContactInput {
                        company_id: r.company_id, first_name: r.first_name, last_name: r.last_name, job_title: r.job_title,
                        email: r.email, phone: r.phone, mobile: r.mobile, is_primary: r.is_primary, status: new_status.to_string(),
                        tags: r.tags, notes: r.notes, department: r.department,
                        preferred_contact_method: r.preferred_contact_method, linkedin_url: r.linkedin_url,
                    };
                    contact_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Contract" => {
                    let r = contract_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Contract".into()))?;
                    let input = ContractInput {
                        company_id: r.company_id, contact_id: r.contact_id, source_quote_id: r.source_quote_id, title: r.title,
                        r#type: r.r#type, value_cents: r.value_cents, currency_code: r.currency_code, owner_user_id: r.owner_user_id,
                        start_date: r.start_date, end_date: r.end_date, renewal_date: r.renewal_date,
                        notice_period_days: r.notice_period_days, status: new_status.to_string(), notes: r.notes,
                    };
                    contract_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Opportunity" => {
                    let r = opportunity_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Opportunity".into()))?;
                    let input = OpportunityInput {
                        company_id: r.company_id, primary_contact_id: r.primary_contact_id, name: r.name, stage: new_status.to_string(),
                        status: r.status, value_cents: r.value_cents, currency_code: r.currency_code, probability_bp: r.probability_bp,
                        expected_close_date: r.expected_close_date, owner_user_id: r.owner_user_id, lost_reason: r.lost_reason,
                        next_step: r.next_step,
                    };
                    opportunity_service::update(conn, id, &input, actor_user_id).map(|_| ())
                }
                "Task" => {
                    let r = task_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Task".into()))?;
                    let input = TaskInput {
                        title: r.title, description: r.description, owner_user_id: r.owner_user_id, priority: r.priority,
                        status: new_status.to_string(), due_date: r.due_date, reminder_at: r.reminder_at, related_type: r.related_type,
                        related_id: r.related_id,
                    };
                    task_service::update(conn, id, workspace_id, &input, actor_user_id).map(|_| ())
                }
                other => Err(AppError::Validation(format!("'{other}' doesn't support bulk change status"))),
            }
        };
        results.push(match outcome {
            Ok(()) => ok_result(id),
            Err(e) => err_result(id, e),
        });
    }
    Ok(results)
}

/// `add`: true to append tags not already present, false to remove them -
/// both merge against each record's own current comma-separated `tags`
/// value rather than overwriting it, so a bulk tag action never clobbers
/// tags a bulk-selected record already had that weren't part of this
/// action.
pub fn bulk_update_tags(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    ids: &[String],
    tags_to_change: &[String],
    add: bool,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    // Only Company/Contact have a `tags` column at all - unlike the other
    // bulk operations, this one never extends to Custom Objects, so it
    // checks the builtins list directly rather than via
    // `require_supported` (which would otherwise wave a custom object
    // key through).
    if !TAGGABLE_BUILTINS.contains(&object_key) {
        return Err(AppError::Validation(format!(
            "'{object_key}' doesn't support bulk tag - only Company and Contact have a tags field"
        )));
    }
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        let outcome: AppResult<()> = (|| -> AppResult<()> {
            let current = match object_key {
                "Company" => company_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Company".into()))?.tags,
                "Contact" => contact_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Contact".into()))?.tags,
                other => return Err(AppError::Validation(format!("'{other}' doesn't support tags"))),
            };
            let mut set: Vec<String> = current
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            if add {
                for t in tags_to_change {
                    let t = t.trim().to_string();
                    if !t.is_empty() && !set.iter().any(|existing| existing.eq_ignore_ascii_case(&t)) {
                        set.push(t);
                    }
                }
            } else {
                set.retain(|existing| !tags_to_change.iter().any(|t| t.trim().eq_ignore_ascii_case(existing)));
            }
            let new_value = if set.is_empty() { None } else { Some(set.join(", ")) };
            builtin_field_service::set_field(conn, workspace_id, object_key, id, "tags", new_value.as_deref().unwrap_or(""), actor_user_id)
        })();
        results.push(match outcome {
            Ok(()) => ok_result(id),
            Err(e) => err_result(id, e),
        });
    }
    Ok(results)
}

pub fn bulk_archive(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    ids: &[String],
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    let is_custom_obj = require_supported(conn, workspace_id, object_key, ARCHIVABLE_BUILTINS, "archive")?;
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        let outcome: AppResult<()> = if is_custom_obj {
            custom_record_service::archive(conn, id, actor_user_id).map(|_| ())
        } else {
            match object_key {
                "Company" => company_service::archive(conn, id, actor_user_id),
                "Contact" => contact_service::archive(conn, id, actor_user_id),
                "Opportunity" => opportunity_service::archive(conn, id, actor_user_id),
                "Product" => crate::services::product_service::archive(conn, id, actor_user_id),
                "Contract" => contract_service::archive(conn, id, actor_user_id),
                "Task" => task_service::archive(conn, id, workspace_id, actor_user_id),
                other => Err(AppError::Validation(format!("'{other}' doesn't support bulk archive"))),
            }
        };
        results.push(match outcome {
            Ok(()) => ok_result(id),
            Err(e) => err_result(id, e),
        });
    }
    Ok(results)
}
