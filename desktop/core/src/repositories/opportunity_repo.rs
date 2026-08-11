use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::opportunity::{
    Opportunity, OpportunityInput, OpportunityProduct, OpportunityProductInput,
};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Opportunity> {
    Ok(Opportunity {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        opportunity_number: row.get("opportunity_number")?,
        company_id: row.get("company_id")?,
        primary_contact_id: row.get("primary_contact_id")?,
        name: row.get("name")?,
        stage: row.get("stage")?,
        status: row.get("status")?,
        value_cents: row.get("value_cents")?,
        currency_code: row.get("currency_code")?,
        probability_bp: row.get("probability_bp")?,
        expected_close_date: row.get("expected_close_date")?,
        owner_user_id: row.get("owner_user_id")?,
        lost_reason: row.get("lost_reason")?,
        next_step: row.get("next_step")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
    })
}

fn map_product_row(row: &rusqlite::Row) -> rusqlite::Result<OpportunityProduct> {
    Ok(OpportunityProduct {
        id: row.get("id")?,
        opportunity_id: row.get("opportunity_id")?,
        product_id: row.get("product_id")?,
        quantity_milli: row.get("quantity_milli")?,
        unit_price_cents: row.get("unit_price_cents")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    opportunity_number: &str,
    input: &OpportunityInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Opportunity> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO opportunities (id, workspace_id, opportunity_number, company_id, primary_contact_id, name, stage, status, value_cents, currency_code, probability_bp, expected_close_date, owner_user_id, lost_reason, next_step, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?16, ?17)",
        rusqlite::params![
            id,
            workspace_id,
            opportunity_number,
            &input.company_id,
            &input.primary_contact_id,
            &input.name,
            &input.stage,
            &input.status,
            input.value_cents,
            &input.currency_code,
            input.probability_bp,
            &input.expected_close_date,
            &input.owner_user_id,
            &input.lost_reason,
            &input.next_step,
            &now,
            actor_user_id,
        ],
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Opportunity>> {
    conn.query_row("SELECT * FROM opportunities WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Opportunity>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM opportunities WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn list_by_company(conn: &Connection, company_id: &str) -> rusqlite::Result<Vec<Opportunity>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM opportunities WHERE company_id = ?1 AND archived_at IS NULL ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([company_id], map_row)?;
    rows.collect()
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &OpportunityInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Opportunity> {
    let now = now_iso();
    conn.execute(
        "UPDATE opportunities SET company_id = ?1, primary_contact_id = ?2, name = ?3, stage = ?4,
            status = ?5, value_cents = ?6, currency_code = ?7, probability_bp = ?8, expected_close_date = ?9,
            owner_user_id = ?10, lost_reason = ?11, next_step = ?12, updated_at = ?13, updated_by = ?14
         WHERE id = ?15",
        (
            &input.company_id,
            &input.primary_contact_id,
            &input.name,
            &input.stage,
            &input.status,
            input.value_cents,
            &input.currency_code,
            input.probability_bp,
            &input.expected_close_date,
            &input.owner_user_id,
            &input.lost_reason,
            &input.next_step,
            &now,
            actor_user_id,
            id,
        ),
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE opportunities SET status = 'Archived', archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}

pub fn set_products(
    conn: &Connection,
    opportunity_id: &str,
    products: &[OpportunityProductInput],
) -> rusqlite::Result<Vec<OpportunityProduct>> {
    conn.execute(
        "DELETE FROM opportunity_products WHERE opportunity_id = ?1",
        [opportunity_id],
    )?;
    for p in products {
        conn.execute(
            "INSERT INTO opportunity_products (id, opportunity_id, product_id, quantity_milli, unit_price_cents)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (new_uuid(), opportunity_id, &p.product_id, p.quantity_milli, p.unit_price_cents),
        )?;
    }
    list_products(conn, opportunity_id)
}

pub fn list_products(
    conn: &Connection,
    opportunity_id: &str,
) -> rusqlite::Result<Vec<OpportunityProduct>> {
    let mut stmt =
        conn.prepare("SELECT * FROM opportunity_products WHERE opportunity_id = ?1")?;
    let rows = stmt.query_map([opportunity_id], map_product_row)?;
    rows.collect()
}

/// See company_repo::set_owner's comment - same reasoning.
pub fn set_owner(conn: &Connection, id: &str, owner_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE opportunities SET owner_user_id = ?1, updated_at = ?2 WHERE id = ?3",
        (owner_user_id, now_iso(), id),
    )?;
    Ok(())
}
