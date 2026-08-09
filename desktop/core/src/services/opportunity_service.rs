use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::numbering::{self, OPPORTUNITY};
use crate::domain::{AppError, AppResult};
use crate::models::opportunity::{
    Opportunity, OpportunityInput, OpportunityProduct, OpportunityProductInput,
    OPPORTUNITY_STAGES, OPPORTUNITY_STATUSES,
};
use crate::repositories::{audit_repo, company_repo, contact_repo, opportunity_repo};
use crate::services::workflow_service;

fn validate(conn: &Connection, input: &OpportunityInput) -> AppResult<String> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Opportunity name is required".into()));
    }
    if !OPPORTUNITY_STAGES.contains(&input.stage.as_str()) {
        return Err(AppError::Validation(format!("Invalid stage '{}'", input.stage)));
    }
    if !OPPORTUNITY_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid opportunity status '{}'",
            input.status
        )));
    }
    let company = company_repo::get(conn, &input.company_id)?
        .ok_or_else(|| AppError::Validation("Selected company does not exist".into()))?;

    if let Some(contact_id) = &input.primary_contact_id {
        let contact = contact_repo::get(conn, contact_id)?
            .ok_or_else(|| AppError::Validation("Selected contact does not exist".into()))?;
        if contact.company_id != company.id {
            return Err(AppError::Validation(
                "Primary contact must belong to the selected company".into(),
            ));
        }
    }

    Ok(company.workspace_id)
}

pub fn create(
    conn: &Connection,
    input: &OpportunityInput,
    actor_user_id: Option<&str>,
) -> AppResult<Opportunity> {
    let workspace_id = validate(conn, input)?;
    let id = new_uuid();
    let opportunity_number = numbering::allocate_number(conn, &workspace_id, &OPPORTUNITY)?;
    let opportunity = opportunity_repo::create(
        conn,
        &id,
        &workspace_id,
        &opportunity_number,
        input,
        actor_user_id,
    )?;
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "create",
        Some("opportunity"),
        Some(&opportunity.id),
        &format!("Created opportunity {}", opportunity.opportunity_number),
        None,
    )?;
    Ok(opportunity)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Opportunity> {
    opportunity_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Opportunity".into()))
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Opportunity>> {
    Ok(opportunity_repo::list(conn, workspace_id)?)
}

pub fn list_by_company(conn: &Connection, company_id: &str) -> AppResult<Vec<Opportunity>> {
    Ok(opportunity_repo::list_by_company(conn, company_id)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &OpportunityInput,
    actor_user_id: Option<&str>,
) -> AppResult<Opportunity> {
    let workspace_id = validate(conn, input)?;
    let before = get(conn, id)?;
    let opportunity = opportunity_repo::update(conn, id, input, actor_user_id)?;

    let event_type = if before.stage != opportunity.stage {
        "status_change"
    } else {
        "update"
    };
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        event_type,
        Some("opportunity"),
        Some(id),
        &format!(
            "Updated opportunity {} (stage: {})",
            opportunity.opportunity_number, opportunity.stage
        ),
        None,
    )?;

    // FR-WFL: fire any workflow rules whose trigger stage the opportunity
    // just moved into (e.g. auto-create a follow-up task on Won).
    workflow_service::fire_transition(
        conn,
        &workspace_id,
        "Opportunity",
        id,
        &before.stage,
        &opportunity.stage,
        opportunity.owner_user_id.as_deref(),
        actor_user_id,
    )?;

    Ok(opportunity)
}

pub fn set_products(
    conn: &Connection,
    opportunity_id: &str,
    products: &[OpportunityProductInput],
) -> AppResult<Vec<OpportunityProduct>> {
    get(conn, opportunity_id)?;
    Ok(opportunity_repo::set_products(conn, opportunity_id, products)?)
}

pub fn list_products(conn: &Connection, opportunity_id: &str) -> AppResult<Vec<OpportunityProduct>> {
    Ok(opportunity_repo::list_products(conn, opportunity_id)?)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get(conn, id)?;
    opportunity_repo::archive(conn, id, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.workspace_id,
        actor_user_id,
        "archive",
        Some("opportunity"),
        Some(id),
        &format!("Archived opportunity {}", existing.opportunity_number),
        None,
    )?;
    Ok(())
}
