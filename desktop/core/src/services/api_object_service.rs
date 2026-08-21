//! Integration Hub (spec §7): the generic, metadata-aware dispatcher every
//! external-facing surface sits on top of - the inbound REST API
//! (`server/src/routes.rs`), the Bulk API, the CSV Data Exchange wizard
//! and External Objects. One `object_key` -> the entity's own existing
//! `list`/`get`/`create`/`update`/`archive` service functions, the same
//! ones the desktop UI itself calls - so every validation/business-rule/
//! permission check the UI already relies on fires identically here, by
//! construction, rather than being reimplemented (spec §7.2: "API query
//! rules must use the same field metadata and permission engine as UI
//! list views"; §9: "Bulk writes must invoke mandatory field validation,
//! relationship validation and applicable server-side Business Rules").
//!
//! Scope, stated plainly rather than silently faked: full list/get/
//! create/update/archive dispatch covers Company, Contact, Product, Task
//! and every active Custom Object (the fully-dynamic path, needing zero
//! per-type code at all). Opportunity/Quote/Order/Invoice/Contract are
//! **read-only** here (list/get) - Quote/Order/Invoice are compound
//! line-item documents with their own conversion workflow (Quote -> Order
//! -> Invoice) and status-transition commands (issue/void), not a plain
//! field-level create/update; wiring generic writes for them needs a
//! purpose-built request shape, not this passthrough, and is real scope
//! left for a fast-follow rather than attempted here.

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::contact::ContactInput;
use crate::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use crate::models::integration::{ApiFieldMetadata, ApiListQuery, ApiObjectMetadata, ApiRecordPage};
use crate::models::product::ProductInput;
use crate::models::task::TaskInput;
use crate::models::company::CompanyInput;
use crate::services::{
    company_service, contact_service, custom_field_service, custom_object_service, custom_record_service, contract_service,
    invoice_service, opportunity_service, order_service, product_service, quote_service, task_service,
};

/// Built-ins this dispatcher can read - a superset of the ones it can
/// also write (see this module's own doc comment).
const READABLE_BUILTINS: &[&str] = &["Company", "Contact", "Product", "Task", "Opportunity", "Quote", "Order", "Invoice", "Contract"];
const WRITABLE_BUILTINS: &[&str] = &["Company", "Contact", "Product", "Task"];

fn is_custom(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<bool> {
    Ok(!READABLE_BUILTINS.contains(&object_key) && custom_object_service::get_by_key(conn, workspace_id, object_key)?.is_some())
}

pub fn list_object_keys(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ApiObjectMetadata>> {
    let mut out: Vec<ApiObjectMetadata> = READABLE_BUILTINS.iter().map(|k| ApiObjectMetadata { object_key: k.to_string(), label: k.to_string(), is_custom: false, fields: vec![] }).collect();
    for def in custom_object_service::list(conn, workspace_id, true)? {
        out.push(ApiObjectMetadata { object_key: def.key, label: def.singular_label, is_custom: true, fields: vec![] });
    }
    Ok(out)
}

pub fn get_metadata(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<ApiObjectMetadata> {
    let (label, is_custom_obj) = if is_custom(conn, workspace_id, object_key)? {
        (custom_object_service::get_by_key(conn, workspace_id, object_key)?.map(|d| d.singular_label).unwrap_or_else(|| object_key.to_string()), true)
    } else if READABLE_BUILTINS.contains(&object_key) {
        (object_key.to_string(), false)
    } else {
        return Err(AppError::NotFound(format!("Object '{object_key}'")));
    };
    let custom_fields = custom_field_service::list_definitions(conn, workspace_id, object_key, true)?;
    let fields = custom_fields
        .into_iter()
        .map(|f| ApiFieldMetadata { key: f.key, label: f.label, field_type: f.field_type, required: f.required, is_custom: true })
        .collect();
    Ok(ApiObjectMetadata { object_key: object_key.to_string(), label, is_custom: is_custom_obj, fields })
}

fn paginate(mut records: Vec<Value>, query: &ApiListQuery) -> ApiRecordPage {
    let total = records.len() as i64;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 500);
    if let Some(sort_fields) = &query.sort {
        if let Some(first) = sort_fields.first() {
            let (field, desc) = match first.strip_prefix('-') {
                Some(f) => (f, true),
                None => (first.as_str(), false),
            };
            records.sort_by(|a, b| {
                let av = a.get(field).map(|v| v.to_string()).unwrap_or_default();
                let bv = b.get(field).map(|v| v.to_string()).unwrap_or_default();
                if desc { bv.cmp(&av) } else { av.cmp(&bv) }
            });
        }
    }
    let start = ((page - 1) * page_size) as usize;
    let page_records = records.into_iter().skip(start).take(page_size as usize).collect();
    ApiRecordPage { records: page_records, total, page, page_size }
}

fn matches_filter(record: &Value, filter: &Value) -> bool {
    let Some(obj) = filter.as_object() else { return true };
    obj.iter().all(|(key, expected)| record.get(key).map(|actual| actual == expected).unwrap_or(false))
}

pub fn list_records(conn: &Connection, workspace_id: &str, object_key: &str, query: &ApiListQuery) -> AppResult<ApiRecordPage> {
    let mut records: Vec<Value> = if is_custom(conn, workspace_id, object_key)? {
        custom_record_service::list(conn, workspace_id, object_key)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect()
    } else {
        match object_key {
            "Company" => company_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Contact" => contact_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Product" => product_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Task" => task_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Opportunity" => opportunity_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Quote" => quote_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Order" => order_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Invoice" => invoice_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            "Contract" => contract_service::list(conn, workspace_id)?.into_iter().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)).collect(),
            other => return Err(AppError::NotFound(format!("Object '{other}'"))),
        }
    };
    if let Some(filter) = &query.filter {
        records.retain(|r| matches_filter(r, filter));
    }
    Ok(paginate(records, query))
}

pub fn get_record(conn: &Connection, workspace_id: &str, object_key: &str, id: &str) -> AppResult<Value> {
    if is_custom(conn, workspace_id, object_key)? {
        let record = custom_record_service::get(conn, id)?;
        if record.workspace_id != workspace_id || record.object_key != object_key {
            return Err(AppError::NotFound("Record".into()));
        }
        return Ok(serde_json::to_value(record).unwrap_or(Value::Null));
    }
    let value = match object_key {
        "Company" => serde_json::to_value(company_service::get(conn, id)?),
        "Contact" => serde_json::to_value(contact_service::get(conn, id)?),
        "Product" => serde_json::to_value(product_service::get(conn, id)?),
        "Task" => serde_json::to_value(task_service::get(conn, id)?),
        "Opportunity" => serde_json::to_value(opportunity_service::get(conn, id)?),
        "Quote" => serde_json::to_value(quote_service::get(conn, id)?),
        "Order" => serde_json::to_value(order_service::get(conn, id)?),
        "Invoice" => serde_json::to_value(invoice_service::get(conn, id)?),
        "Contract" => serde_json::to_value(contract_service::get(conn, id)?),
        other => return Err(AppError::NotFound(format!("Object '{other}'"))),
    };
    value.map_err(|e| AppError::Validation(format!("could not serialize record: {e}")))
}

fn not_writable(object_key: &str) -> AppError {
    AppError::Validation(format!(
        "'{object_key}' is read-only through the generic API for now - it's a compound document with its own conversion workflow, not a plain field-level create/update. See api_object_service's own doc comment."
    ))
}

pub fn create_record(conn: &Connection, workspace_id: &str, object_key: &str, body: &Value, actor_user_id: Option<&str>) -> AppResult<Value> {
    if is_custom(conn, workspace_id, object_key)? {
        let mut merged = body.clone();
        if let Some(obj) = merged.as_object_mut() {
            obj.insert("object_key".to_string(), json!(object_key));
        }
        let input: CustomRecordInput = serde_json::from_value(merged).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
        let record = custom_record_service::create(conn, workspace_id, &input, actor_user_id)?;
        return Ok(serde_json::to_value(record).unwrap_or(Value::Null));
    }
    if !WRITABLE_BUILTINS.contains(&object_key) {
        return Err(not_writable(object_key));
    }
    let value = match object_key {
        "Company" => {
            let input: CompanyInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(company_service::create(conn, workspace_id, &input, actor_user_id)?)
        }
        "Contact" => {
            let input: ContactInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(contact_service::create(conn, &input, actor_user_id)?)
        }
        "Product" => {
            let input: ProductInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(product_service::create(conn, workspace_id, &input, actor_user_id)?)
        }
        "Task" => {
            let input: TaskInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(task_service::create(conn, workspace_id, &input, actor_user_id)?)
        }
        other => return Err(AppError::NotFound(format!("Object '{other}'"))),
    };
    value.map_err(|e| AppError::Validation(format!("could not serialize record: {e}")))
}

pub fn update_record(conn: &Connection, workspace_id: &str, object_key: &str, id: &str, body: &Value, actor_user_id: Option<&str>) -> AppResult<Value> {
    if is_custom(conn, workspace_id, object_key)? {
        let input: CustomRecordUpdate = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
        let record = custom_record_service::update(conn, id, &input, actor_user_id)?;
        if record.workspace_id != workspace_id || record.object_key != object_key {
            return Err(AppError::NotFound("Record".into()));
        }
        return Ok(serde_json::to_value(record).unwrap_or(Value::Null));
    }
    if !WRITABLE_BUILTINS.contains(&object_key) {
        return Err(not_writable(object_key));
    }
    let value = match object_key {
        "Company" => {
            let input: CompanyInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(company_service::update(conn, id, &input, actor_user_id)?)
        }
        "Contact" => {
            let input: ContactInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(contact_service::update(conn, id, &input, actor_user_id)?)
        }
        "Product" => {
            let input: ProductInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(product_service::update(conn, id, &input, actor_user_id)?)
        }
        "Task" => {
            let input: TaskInput = serde_json::from_value(body.clone()).map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(task_service::update(conn, id, workspace_id, &input, actor_user_id)?)
        }
        other => return Err(AppError::NotFound(format!("Object '{other}'"))),
    };
    value.map_err(|e| AppError::Validation(format!("could not serialize record: {e}")))
}

pub fn archive_record(conn: &Connection, workspace_id: &str, object_key: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    if is_custom(conn, workspace_id, object_key)? {
        custom_record_service::archive(conn, id, actor_user_id)?;
        return Ok(());
    }
    if !WRITABLE_BUILTINS.contains(&object_key) {
        return Err(not_writable(object_key));
    }
    match object_key {
        "Company" => company_service::archive(conn, id, actor_user_id),
        "Contact" => contact_service::archive(conn, id, actor_user_id),
        "Product" => product_service::archive(conn, id, actor_user_id),
        "Task" => task_service::archive(conn, id, workspace_id, actor_user_id),
        other => Err(AppError::NotFound(format!("Object '{other}'"))),
    }
}
