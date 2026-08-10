//! Admin flexibility: a simple report builder - pick an entity type, a
//! field to group by (its built-in status/stage, or any active custom
//! field), and an aggregate (count of records, or sum of a numeric custom
//! field) per group. Deliberately not a full drag-and-drop builder or
//! dashboard designer - see "Report builder" in the product backlog for
//! the fuller version this was scoped down from.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::custom_field::CUSTOM_FIELD_ENTITY_TYPES;
use crate::models::custom_report::{
    CustomReport, CustomReportInput, CustomReportRow, CustomReportUpdate, REPORT_AGGREGATES, REPORT_GROUP_BY_SOURCES,
};
use crate::models::field_rule::builtin_trigger_field_for;
use crate::repositories::{
    company_repo, contact_repo, contract_repo, custom_field_repo, custom_report_repo, invoice_repo, opportunity_repo,
    order_repo, product_repo, quote_repo, task_repo, user_repo,
};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage custom reports".into()));
    }
    Ok(())
}

fn validate_shape(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    name: &str,
    group_by_source: &str,
    group_by_field: &str,
    aggregate: &str,
    sum_field_key: Option<&str>,
) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("Report name is required".into()));
    }
    if !CUSTOM_FIELD_ENTITY_TYPES.contains(&entity_type) {
        return Err(AppError::Validation(format!("Invalid entity type '{entity_type}'")));
    }
    if !REPORT_GROUP_BY_SOURCES.contains(&group_by_source) {
        return Err(AppError::Validation(format!("Invalid group-by source '{group_by_source}'")));
    }
    if !REPORT_AGGREGATES.contains(&aggregate) {
        return Err(AppError::Validation(format!("Invalid aggregate '{aggregate}'")));
    }

    let active_defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?
        .into_iter()
        .filter(|d| d.is_active)
        .collect::<Vec<_>>();

    if group_by_source == "builtin" {
        let expected = builtin_trigger_field_for(entity_type);
        if group_by_field != expected {
            return Err(AppError::Validation(format!(
                "'{group_by_field}' is not the built-in field for {entity_type} (expected '{expected}')"
            )));
        }
    } else if !active_defs.iter().any(|d| d.key == group_by_field) {
        return Err(AppError::Validation(format!(
            "'{group_by_field}' is not an active custom field to group by"
        )));
    }

    if aggregate == "sum" {
        let key = sum_field_key.ok_or_else(|| AppError::Validation("A sum report needs a numeric field to sum".into()))?;
        if !active_defs.iter().any(|d| d.key == key && d.field_type == "number") {
            return Err(AppError::Validation(format!("'{key}' is not an active numeric custom field to sum")));
        }
    }

    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &CustomReportInput, actor_user_id: Option<&str>) -> AppResult<CustomReport> {
    require_admin(conn, actor_user_id)?;
    validate_shape(
        conn, workspace_id, &input.entity_type, &input.name, &input.group_by_source, &input.group_by_field,
        &input.aggregate, input.sum_field_key.as_deref(),
    )?;
    let id = crate::domain::ids::new_uuid();
    Ok(custom_report_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<CustomReport>> {
    Ok(custom_report_repo::list(conn, workspace_id)?)
}

pub fn update(conn: &Connection, id: &str, input: &CustomReportUpdate, actor_user_id: Option<&str>) -> AppResult<CustomReport> {
    require_admin(conn, actor_user_id)?;
    let existing = custom_report_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Custom report".into()))?;
    // entity_type is immutable after creation, same as custom field definitions - it defines
    // which group-by/sum fields are even valid, so changing it invalidates the rest of the report.
    validate_shape(
        conn, &existing.workspace_id, &existing.entity_type, &input.name, &input.group_by_source, &input.group_by_field,
        &input.aggregate, input.sum_field_key.as_deref(),
    )?;
    Ok(custom_report_repo::update(conn, id, input, actor_user_id)?)
}

pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    custom_report_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Custom report".into()))?;
    Ok(custom_report_repo::delete(conn, id)?)
}

/// (entity_id, builtin_trigger_value) for every record of `entity_type` in
/// the workspace - the same per-entity status/stage/is_active convention
/// `custom_field_service::resolve_entity_workspace` uses for one record at
/// a time, just listed in bulk here.
fn list_builtin_values(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<(String, String)>> {
    Ok(match entity_type {
        "Company" => company_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Contact" => contact_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Opportunity" => opportunity_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Quote" => quote_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Order" => order_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Invoice" => invoice_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Contract" => contract_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Task" => task_repo::list(conn, workspace_id)?.into_iter().map(|r| (r.id, r.status)).collect(),
        "Product" => product_repo::list(conn, workspace_id)?
            .into_iter()
            .map(|r| (r.id, if r.is_active { "true".to_string() } else { "false".to_string() }))
            .collect(),
        other => return Err(AppError::Validation(format!("Unsupported report entity type '{other}'"))),
    })
}

/// Runs a saved report against live data - grouped counts or sums, one row
/// per distinct group value seen (a record with no value for the group-by
/// custom field is bucketed under "(none)").
pub fn run(conn: &Connection, report: &CustomReport) -> AppResult<Vec<CustomReportRow>> {
    let entity_ids = list_builtin_values(conn, &report.workspace_id, &report.entity_type)?;

    // group_key(entity_id, builtin_value) -> group label
    let group_of: Box<dyn Fn(&str, &str, &HashMap<String, String>) -> String> = if report.group_by_source == "builtin" {
        Box::new(|_id, builtin_value, _values| builtin_value.to_string())
    } else {
        let field = report.group_by_field.clone();
        Box::new(move |_id, _builtin_value, values: &HashMap<String, String>| {
            values.get(&field).filter(|v| !v.is_empty()).cloned().unwrap_or_else(|| "(none)".to_string())
        })
    };

    let mut totals: HashMap<String, f64> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for (entity_id, builtin_value) in &entity_ids {
        let values = if report.group_by_source == "custom" || report.aggregate == "sum" {
            custom_field_repo::get_values(conn, entity_id)?
        } else {
            HashMap::new()
        };

        let group = group_of(entity_id, builtin_value, &values);
        let contribution = if report.aggregate == "sum" {
            report
                .sum_field_key
                .as_ref()
                .and_then(|k| values.get(k))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0)
        } else {
            1.0
        };

        if !totals.contains_key(&group) {
            order.push(group.clone());
        }
        *totals.entry(group).or_insert(0.0) += contribution;
    }

    Ok(order.into_iter().map(|group| {
        let value = totals[&group];
        CustomReportRow { group, value }
    }).collect())
}
