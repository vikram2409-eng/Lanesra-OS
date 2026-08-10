use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::numbering::{self, CONTACT};
use crate::domain::{AppError, AppResult};
use crate::models::contact::{Contact, ContactInput, CONTACT_STATUSES};
use crate::repositories::{audit_repo, company_repo, contact_repo};
use crate::services::workflow_service;

fn validate(conn: &Connection, input: &ContactInput) -> AppResult<String> {
    if input.first_name.trim().is_empty() || input.last_name.trim().is_empty() {
        return Err(AppError::Validation(
            "Contact first and last name are required".into(),
        ));
    }
    if !CONTACT_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid contact status '{}'",
            input.status
        )));
    }
    let company = company_repo::get(conn, &input.company_id)?
        .ok_or_else(|| AppError::Validation("Selected company does not exist".into()))?;
    Ok(company.workspace_id)
}

pub fn check_duplicates(
    conn: &Connection,
    company_id: &str,
    email: &str,
    exclude_id: Option<&str>,
) -> AppResult<Vec<Contact>> {
    if email.trim().is_empty() {
        return Ok(vec![]);
    }
    Ok(contact_repo::find_duplicates_by_email(
        conn, company_id, email, exclude_id,
    )?)
}

pub fn create(
    conn: &Connection,
    input: &ContactInput,
    actor_user_id: Option<&str>,
) -> AppResult<Contact> {
    let workspace_id = validate(conn, input)?;
    let id = new_uuid();
    let contact_number = numbering::allocate_number(conn, &workspace_id, &CONTACT)?;
    let contact = contact_repo::create(conn, &id, &workspace_id, &contact_number, input, actor_user_id)?;
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "create",
        Some("contact"),
        Some(&contact.id),
        &format!("Created contact {}", contact.contact_number),
        None,
    )?;
    Ok(contact)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Contact> {
    contact_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Contact".into()))
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Contact>> {
    Ok(contact_repo::list(conn, workspace_id)?)
}

pub fn list_by_company(conn: &Connection, company_id: &str) -> AppResult<Vec<Contact>> {
    Ok(contact_repo::list_by_company(conn, company_id)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &ContactInput,
    actor_user_id: Option<&str>,
) -> AppResult<Contact> {
    let workspace_id = validate(conn, input)?;
    let before = get(conn, id)?;
    let contact = contact_repo::update(conn, id, input, actor_user_id)?;
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "update",
        Some("contact"),
        Some(id),
        &format!("Updated contact {}", contact.contact_number),
        None,
    )?;
    workflow_service::fire_transition(
        conn, &workspace_id, "Contact", id, &before.status, &contact.status, None, actor_user_id,
    )?;
    Ok(contact)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get(conn, id)?;
    contact_repo::archive(conn, id, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.workspace_id,
        actor_user_id,
        "archive",
        Some("contact"),
        Some(id),
        &format!("Archived contact {}", existing.contact_number),
        None,
    )?;
    Ok(())
}
