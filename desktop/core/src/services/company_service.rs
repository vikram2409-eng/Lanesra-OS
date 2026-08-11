use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::numbering::{self, COMPANY};
use crate::domain::{AppError, AppResult};
use crate::models::company::{Company, CompanyInput, COMPANY_STATUSES};
use crate::repositories::{audit_repo, company_repo};
use crate::services::{builtin_field_service, workflow_service};

fn validate(input: &CompanyInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Company name is required".into()));
    }
    if !COMPANY_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid company status '{}'",
            input.status
        )));
    }
    Ok(())
}

/// Names that are likely duplicates, so the UI can warn before saving
/// (FR-COM-04). Does not block the save itself.
pub fn check_duplicates(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    exclude_id: Option<&str>,
) -> AppResult<Vec<Company>> {
    Ok(company_repo::find_duplicates_by_name(
        conn,
        workspace_id,
        name,
        exclude_id,
    )?)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &CompanyInput,
    actor_user_id: Option<&str>,
) -> AppResult<Company> {
    validate(input)?;
    let id = new_uuid();
    let customer_number = numbering::allocate_number(conn, workspace_id, &COMPANY)?;
    let company = company_repo::create(conn, &id, workspace_id, &customer_number, input, actor_user_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "create",
        Some("company"),
        Some(&company.id),
        &format!("Created company {}", company.customer_number),
        None,
    )?;
    workflow_service::fire_event(conn, workspace_id, "Company", &company.id, None, &company.status, company.owner_user_id.as_deref(), actor_user_id)?;
    Ok(company)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Company> {
    company_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Company".into()))
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Company>> {
    Ok(company_repo::list(conn, workspace_id)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &CompanyInput,
    actor_user_id: Option<&str>,
) -> AppResult<Company> {
    validate(input)?;
    let before = get(conn, id)?;
    let before_fields = builtin_field_service::field_values(conn, "Company", id)?;
    let company = company_repo::update(conn, id, input, actor_user_id)?;
    audit_repo::record(
        conn,
        &before.workspace_id,
        actor_user_id,
        "update",
        Some("company"),
        Some(id),
        &format!("Updated company {}", company.customer_number),
        None,
    )?;
    workflow_service::fire_event(
        conn,
        &before.workspace_id,
        "Company",
        id,
        Some(&before.status),
        &company.status,
        company.owner_user_id.as_deref(),
        actor_user_id,
    )?;
    let after_fields = builtin_field_service::field_values(conn, "Company", id)?;
    let changed = workflow_service::changed_builtin_keys(&before_fields, &after_fields);
    workflow_service::fire_field_changed(conn, &before.workspace_id, "Company", id, "builtin", &changed, company.owner_user_id.as_deref(), actor_user_id)?;
    Ok(company)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get(conn, id)?;
    company_repo::archive(conn, id, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.workspace_id,
        actor_user_id,
        "archive",
        Some("company"),
        Some(id),
        &format!("Archived company {}", existing.customer_number),
        None,
    )?;
    Ok(())
}
