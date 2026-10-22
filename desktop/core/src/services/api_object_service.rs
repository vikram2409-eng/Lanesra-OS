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
use crate::models::company::CompanyInput;
use crate::models::contact::ContactInput;
use crate::models::custom_record::{CustomRecordInput, CustomRecordUpdate};
use crate::models::integration::{
    ApiBulkItemResult, ApiBulkOperation, ApiBulkResult, ApiFieldMetadata, ApiListQuery,
    ApiObjectMetadata, ApiRecordPage, ApiRelationshipMetadata, ApiSearchPage, ApiSearchResult,
};
use crate::models::product::ProductInput;
use crate::models::relationship::RelatedRecord;
use crate::models::task::TaskInput;
use crate::repositories::custom_field_repo;
use crate::services::{
    builtin_field_service, business_rule_service, company_service, contact_service,
    contract_service, custom_field_service, custom_object_service, custom_record_service,
    invoice_service, opportunity_service, order_service, product_service, quote_service,
    relationship_service, search_service, task_service,
};

/// Built-ins this dispatcher can read - a superset of the ones it can
/// also write (see this module's own doc comment).
const READABLE_BUILTINS: &[&str] = &[
    "Company",
    "Contact",
    "Product",
    "Task",
    "Opportunity",
    "Quote",
    "Order",
    "Invoice",
    "Contract",
];
pub(crate) const WRITABLE_BUILTINS: &[&str] = &["Company", "Contact", "Product", "Task"];

fn is_custom(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<bool> {
    Ok(!READABLE_BUILTINS.contains(&object_key)
        && custom_object_service::get_by_key(conn, workspace_id, object_key)?.is_some())
}

pub fn list_object_keys(
    conn: &Connection,
    workspace_id: &str,
) -> AppResult<Vec<ApiObjectMetadata>> {
    let mut out: Vec<ApiObjectMetadata> = READABLE_BUILTINS
        .iter()
        .map(|k| ApiObjectMetadata {
            object_key: k.to_string(),
            label: k.to_string(),
            is_custom: false,
            fields: vec![],
            relationships: vec![],
        })
        .collect();
    for def in custom_object_service::list(conn, workspace_id, true)? {
        out.push(ApiObjectMetadata {
            object_key: def.key,
            label: def.singular_label,
            is_custom: true,
            fields: vec![],
            relationships: vec![],
        });
    }
    Ok(out)
}

pub fn get_metadata(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
) -> AppResult<ApiObjectMetadata> {
    let (label, is_custom_obj) = if is_custom(conn, workspace_id, object_key)? {
        (
            custom_object_service::get_by_key(conn, workspace_id, object_key)?
                .map(|d| d.singular_label)
                .unwrap_or_else(|| object_key.to_string()),
            true,
        )
    } else if READABLE_BUILTINS.contains(&object_key) {
        (object_key.to_string(), false)
    } else {
        return Err(AppError::NotFound(format!("Object '{object_key}'")));
    };
    let custom_fields =
        custom_field_service::list_definitions(conn, workspace_id, object_key, true)?;
    let fields = custom_fields
        .into_iter()
        .map(|f| ApiFieldMetadata {
            key: f.key,
            label: f.label,
            field_type: f.field_type,
            required: f.required,
            is_custom: true,
        })
        .collect();
    let relationships = relationship_service::list(conn, workspace_id, true)?
        .into_iter()
        .filter_map(|def| {
            if def.source_entity_type == object_key {
                // `def.target_entity_type` is the unused '' placeholder
                // when target_is_polymorphic - report that explicitly
                // rather than a misleading empty-string "type". Since the
                // target varies per link, the reverse direction (which
                // object types can point back at a polymorphic target)
                // isn't statically resolvable per object_key, so it's not
                // reported here - fetch `.../related` on an actual record.
                Some(ApiRelationshipMetadata {
                    relationship_key: def.key,
                    related_object_key: def.target_entity_type,
                    relationship_type: def.relationship_type,
                    label: def.forward_label,
                    direction: "forward".into(),
                    is_polymorphic: def.target_is_polymorphic,
                })
            } else if def.target_entity_type == object_key {
                Some(ApiRelationshipMetadata {
                    relationship_key: def.key,
                    related_object_key: def.source_entity_type,
                    relationship_type: def.relationship_type,
                    label: def.reverse_label,
                    direction: "reverse".into(),
                    is_polymorphic: false,
                })
            } else {
                None
            }
        })
        .collect();
    Ok(ApiObjectMetadata {
        object_key: object_key.to_string(),
        label,
        is_custom: is_custom_obj,
        fields,
        relationships,
    })
}

/// The most matches `search_records` collects; `total` never exceeds it.
pub const SEARCH_RESULT_CAP: usize = 10_000;

/// `GET /api/v1/search` (WP-0186): the product's global search over every
/// object this API can read (the readable built-ins and active custom
/// objects), optionally narrowed to one, paged like the records endpoint.
/// Archived records are never returned.
pub fn search_records(
    conn: &Connection,
    workspace_id: &str,
    q: &str,
    object_key: Option<&str>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> AppResult<ApiSearchPage> {
    let q = q.trim();
    if q.is_empty() {
        return Err(AppError::Validation("q is required".into()));
    }
    let object_key = object_key.map(str::trim).filter(|k| !k.is_empty());
    let readable: Vec<String> = list_object_keys(conn, workspace_id)?
        .into_iter()
        .map(|o| o.object_key)
        .collect();
    if let Some(key) = object_key {
        if !readable.iter().any(|k| k == key) {
            return Err(AppError::NotFound(format!("Object '{key}'")));
        }
    }
    let mut seen = std::collections::HashSet::new();
    let hits: Vec<ApiSearchResult> =
        search_service::search_records(conn, workspace_id, q, object_key, SEARCH_RESULT_CAP)?
            .into_iter()
            .filter(|r| readable.contains(&r.entity_type))
            .filter(|r| seen.insert((r.entity_type.clone(), r.entity_id.clone())))
            .map(|r| ApiSearchResult {
                object_key: r.entity_type,
                id: r.entity_id,
                title: r.title,
                snippet: r.subtitle,
            })
            .collect();
    let total = hits.len() as i64;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).clamp(1, 500);
    let results = hits
        .into_iter()
        .skip(usize::try_from((page - 1).saturating_mul(page_size)).unwrap_or(usize::MAX))
        .take(page_size as usize)
        .collect();
    Ok(ApiSearchPage {
        results,
        total,
        page,
        page_size,
    })
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
                if desc {
                    bv.cmp(&av)
                } else {
                    av.cmp(&bv)
                }
            });
        }
    }
    let start = ((page - 1) * page_size) as usize;
    let page_records = records
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();
    ApiRecordPage {
        records: page_records,
        total,
        page,
        page_size,
    }
}

fn matches_filter(record: &Value, filter: &Value) -> bool {
    let Some(obj) = filter.as_object() else {
        return true;
    };
    obj.iter().all(|(key, expected)| {
        record
            .get(key)
            .map(|actual| actual == expected)
            .unwrap_or(false)
    })
}

pub fn list_records(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    query: &ApiListQuery,
) -> AppResult<ApiRecordPage> {
    let mut records = all_records(conn, workspace_id, object_key)?;
    if let Some(filter) = &query.filter {
        records.retain(|r| matches_filter(r, filter));
    }
    Ok(paginate(records, query))
}

/// Every non-archived record of one readable object, unpaged - what
/// `list_records` pages over and what a bulk export (WP-0187) writes out.
pub fn all_records(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
) -> AppResult<Vec<Value>> {
    let records: Vec<Value> = if is_custom(conn, workspace_id, object_key)? {
        custom_record_service::list(conn, workspace_id, object_key)?
            .into_iter()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
            .collect()
    } else {
        match object_key {
            "Company" => company_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Contact" => contact_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Product" => product_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Task" => task_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Opportunity" => opportunity_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Quote" => quote_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Order" => order_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Invoice" => invoice_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            "Contract" => contract_service::list(conn, workspace_id)?
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
                .collect(),
            other => return Err(AppError::NotFound(format!("Object '{other}'"))),
        }
    };
    Ok(records)
}

/// The most operations one bulk write may carry (WP-0187).
pub const MAX_BULK_OPERATIONS: usize = 1000;

/// `POST /api/v1/objects/{key}/bulk` (WP-0187): applies each operation
/// through the same validated path as a single create, update or archive
/// (validation, Business Rules, the writable-object rule), in order. Each
/// item runs in its own savepoint, so a failing item leaves nothing behind
/// and never stops the others; there is no all-or-nothing mode.
pub fn bulk_write(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    operations: &[ApiBulkOperation],
    actor_user_id: Option<&str>,
) -> AppResult<ApiBulkResult> {
    if operations.len() > MAX_BULK_OPERATIONS {
        return Err(AppError::Validation(format!(
            "A bulk write carries at most {MAX_BULK_OPERATIONS} operations; this one has {}",
            operations.len()
        )));
    }
    if !READABLE_BUILTINS.contains(&object_key) && !is_custom(conn, workspace_id, object_key)? {
        return Err(AppError::NotFound(format!("Object '{object_key}'")));
    }
    let mut results = Vec::with_capacity(operations.len());
    for (index, operation) in operations.iter().enumerate() {
        conn.execute_batch("SAVEPOINT bulk_item")?;
        let outcome =
            apply_bulk_operation(conn, workspace_id, object_key, operation, actor_user_id);
        match &outcome {
            Ok(_) => conn.execute_batch("RELEASE bulk_item")?,
            Err(_) => conn.execute_batch("ROLLBACK TO bulk_item; RELEASE bulk_item")?,
        }
        results.push(match outcome {
            Ok(id) => ApiBulkItemResult {
                index,
                ok: true,
                id: Some(id),
                error: None,
            },
            Err(e) => ApiBulkItemResult {
                index,
                ok: false,
                id: operation.id.clone(),
                error: Some(e.to_string()),
            },
        });
    }
    let succeeded = results.iter().filter(|r| r.ok).count();
    Ok(ApiBulkResult {
        failed: results.len() - succeeded,
        succeeded,
        results,
    })
}

/// One bulk operation; returns the id of the record it wrote.
fn apply_bulk_operation(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    operation: &ApiBulkOperation,
    actor_user_id: Option<&str>,
) -> AppResult<String> {
    let need_id = || {
        operation
            .id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| AppError::Validation(format!("'{}' needs an id", operation.op)))
    };
    let need_data = || {
        operation
            .data
            .as_ref()
            .ok_or_else(|| AppError::Validation(format!("'{}' needs data", operation.op)))
    };
    match operation.op.as_str() {
        "create" => {
            let record =
                create_record(conn, workspace_id, object_key, need_data()?, actor_user_id)?;
            record
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| AppError::Validation("created record has no id".into()))
        }
        "update" => {
            let id = need_id()?;
            update_record(
                conn,
                workspace_id,
                object_key,
                id,
                need_data()?,
                actor_user_id,
            )?;
            Ok(id.to_string())
        }
        "archive" => {
            let id = need_id()?;
            archive_record(conn, workspace_id, object_key, id, actor_user_id)?;
            Ok(id.to_string())
        }
        other => Err(AppError::Validation(format!(
            "Unknown operation '{other}'; use create, update or archive"
        ))),
    }
}

pub fn get_record(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
) -> AppResult<Value> {
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

/// Every related record for one record, across every active Custom
/// Relationship it participates in, either direction - the same
/// `relationship_service::related_records_for` the desktop UI's own
/// "Related records" panel already calls, exposed generically here (no
/// existence check on `object_key`/`id` beyond what
/// `related_records_for` itself already does - an unknown pair simply
/// returns no rows, matching a record with no relationships at all,
/// since a relationship link is workspace-scoped by definition, not by
/// this dispatcher re-validating the record itself).
pub fn related_records(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
) -> AppResult<Vec<RelatedRecord>> {
    relationship_service::related_records_for(conn, workspace_id, object_key, id)
}

/// Runs a write in its own savepoint and refuses it, leaving nothing
/// behind, when a Business Rule blocks saving the record it produced.
/// The desktop editor applies rules through
/// `custom_field_service::set_entity_values` after its save; every
/// API-shaped write (REST, MCP, CSV import, bulk) comes through here, so it
/// is held to the same `block_save` rules (WP-0187 AC-6).
fn rule_checked_write(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    write: impl FnOnce() -> AppResult<Value>,
) -> AppResult<Value> {
    conn.execute_batch("SAVEPOINT api_write")?;
    let outcome = write().and_then(|record| {
        let id = record.get("id").and_then(Value::as_str).unwrap_or_default();
        let mut context = custom_field_repo::get_values(conn, id)?;
        context.extend(builtin_field_service::field_values(conn, object_key, id)?);
        let evaluation =
            business_rule_service::evaluate(conn, workspace_id, object_key, id, &context)?;
        match evaluation.blocked {
            Some(reason) => Err(AppError::Validation(reason)),
            None => Ok(record),
        }
    });
    match &outcome {
        Ok(_) => conn.execute_batch("RELEASE api_write")?,
        Err(_) => conn.execute_batch("ROLLBACK TO api_write; RELEASE api_write")?,
    }
    outcome
}

pub fn create_record(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    body: &Value,
    actor_user_id: Option<&str>,
) -> AppResult<Value> {
    rule_checked_write(conn, workspace_id, object_key, || {
        create_record_unchecked(conn, workspace_id, object_key, body, actor_user_id)
    })
}

pub fn update_record(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
    body: &Value,
    actor_user_id: Option<&str>,
) -> AppResult<Value> {
    rule_checked_write(conn, workspace_id, object_key, || {
        update_record_unchecked(conn, workspace_id, object_key, id, body, actor_user_id)
    })
}

fn not_writable(object_key: &str) -> AppError {
    AppError::Validation(format!(
        "'{object_key}' is read-only through the generic API for now - it's a compound document with its own conversion workflow, not a plain field-level create/update. See api_object_service's own doc comment."
    ))
}

fn create_record_unchecked(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    body: &Value,
    actor_user_id: Option<&str>,
) -> AppResult<Value> {
    if is_custom(conn, workspace_id, object_key)? {
        let mut merged = body.clone();
        if let Some(obj) = merged.as_object_mut() {
            obj.insert("object_key".to_string(), json!(object_key));
        }
        let input: CustomRecordInput = serde_json::from_value(merged)
            .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
        let record = custom_record_service::create(conn, workspace_id, &input, actor_user_id)?;
        return Ok(serde_json::to_value(record).unwrap_or(Value::Null));
    }
    if !WRITABLE_BUILTINS.contains(&object_key) {
        return Err(not_writable(object_key));
    }
    let value = match object_key {
        "Company" => {
            let input: CompanyInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(company_service::create(
                conn,
                workspace_id,
                &input,
                actor_user_id,
            )?)
        }
        "Contact" => {
            let input: ContactInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(contact_service::create(conn, &input, actor_user_id)?)
        }
        "Product" => {
            let input: ProductInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(product_service::create(
                conn,
                workspace_id,
                &input,
                actor_user_id,
            )?)
        }
        "Task" => {
            let input: TaskInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(task_service::create(
                conn,
                workspace_id,
                &input,
                actor_user_id,
            )?)
        }
        other => return Err(AppError::NotFound(format!("Object '{other}'"))),
    };
    value.map_err(|e| AppError::Validation(format!("could not serialize record: {e}")))
}

fn update_record_unchecked(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
    body: &Value,
    actor_user_id: Option<&str>,
) -> AppResult<Value> {
    if is_custom(conn, workspace_id, object_key)? {
        let input: CustomRecordUpdate = serde_json::from_value(body.clone())
            .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
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
            let input: CompanyInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(company_service::update(conn, id, &input, actor_user_id)?)
        }
        "Contact" => {
            let input: ContactInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(contact_service::update(conn, id, &input, actor_user_id)?)
        }
        "Product" => {
            let input: ProductInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(product_service::update(conn, id, &input, actor_user_id)?)
        }
        "Task" => {
            let input: TaskInput = serde_json::from_value(body.clone())
                .map_err(|e| AppError::Validation(format!("Invalid record body: {e}")))?;
            serde_json::to_value(task_service::update(
                conn,
                id,
                workspace_id,
                &input,
                actor_user_id,
            )?)
        }
        other => return Err(AppError::NotFound(format!("Object '{other}'"))),
    };
    value.map_err(|e| AppError::Validation(format!("could not serialize record: {e}")))
}

pub fn archive_record(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
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
