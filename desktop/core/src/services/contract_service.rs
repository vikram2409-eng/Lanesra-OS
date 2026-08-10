use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::numbering::{self, CONTRACT};
use crate::domain::{AppError, AppResult};
use crate::models::contract::{Contract, ContractInput, CONTRACT_STATUSES};
use crate::repositories::{audit_repo, company_repo, contact_repo, contract_repo, quote_repo};
use crate::services::workflow_service;

fn validate(conn: &Connection, input: &ContractInput) -> AppResult<String> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Contract title is required".into()));
    }
    if !CONTRACT_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid contract status '{}'",
            input.status
        )));
    }
    let company = company_repo::get(conn, &input.company_id)?
        .ok_or_else(|| AppError::Validation("Selected company does not exist".into()))?;

    if let Some(contact_id) = &input.contact_id {
        let contact = contact_repo::get(conn, contact_id)?
            .ok_or_else(|| AppError::Validation("Selected contact does not exist".into()))?;
        if contact.company_id != company.id {
            return Err(AppError::Validation(
                "Contact must belong to the selected company".into(),
            ));
        }
    }

    if let Some(quote_id) = &input.source_quote_id {
        let quote = quote_repo::get(conn, quote_id)?
            .ok_or_else(|| AppError::Validation("Selected quote does not exist".into()))?;
        if quote.company_id != company.id {
            return Err(AppError::Validation(
                "Source quote must belong to the selected company".into(),
            ));
        }
    }

    Ok(company.workspace_id)
}

pub fn create(
    conn: &Connection,
    input: &ContractInput,
    actor_user_id: Option<&str>,
) -> AppResult<Contract> {
    let workspace_id = validate(conn, input)?;
    let id = new_uuid();
    let contract_number = numbering::allocate_number(conn, &workspace_id, &CONTRACT)?;
    let contract = contract_repo::create(conn, &id, &workspace_id, &contract_number, input, actor_user_id)?;
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "create",
        Some("contract"),
        Some(&contract.id),
        &format!("Created contract {}", contract.contract_number),
        None,
    )?;
    Ok(contract)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Contract> {
    contract_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Contract".into()))
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Contract>> {
    Ok(contract_repo::list(conn, workspace_id)?)
}

pub fn list_by_company(conn: &Connection, company_id: &str) -> AppResult<Vec<Contract>> {
    Ok(contract_repo::list_by_company(conn, company_id)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &ContractInput,
    actor_user_id: Option<&str>,
) -> AppResult<Contract> {
    let workspace_id = validate(conn, input)?;
    let before = get(conn, id)?;
    let contract = contract_repo::update(conn, id, input, actor_user_id)?;
    let event_type = if before.status != contract.status { "status_change" } else { "update" };
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        event_type,
        Some("contract"),
        Some(id),
        &format!(
            "Updated contract {} (status: {})",
            contract.contract_number, contract.status
        ),
        None,
    )?;
    workflow_service::fire_transition(
        conn, &workspace_id, "Contract", id, &before.status, &contract.status, contract.owner_user_id.as_deref(), actor_user_id,
    )?;
    Ok(contract)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get(conn, id)?;
    contract_repo::archive(conn, id, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.workspace_id,
        actor_user_id,
        "archive",
        Some("contract"),
        Some(id),
        &format!("Archived contract {}", existing.contract_number),
        None,
    )?;
    Ok(())
}

pub struct RenewalAlerts {
    pub within_30_days: i64,
    pub within_60_days: i64,
    pub within_90_days: i64,
}

/// 30/60/90-day renewal alert counts (FR-CTR-05 / dashboard 9.1). Each
/// window is cumulative (within_90_days includes contracts already counted
/// in the 30- and 60-day windows), matching how the PRD's dashboard reads.
pub fn renewal_alerts(conn: &Connection, workspace_id: &str) -> AppResult<RenewalAlerts> {
    Ok(RenewalAlerts {
        within_30_days: contract_repo::count_renewing_within(conn, workspace_id, 30)?,
        within_60_days: contract_repo::count_renewing_within(conn, workspace_id, 60)?,
        within_90_days: contract_repo::count_renewing_within(conn, workspace_id, 90)?,
    })
}
