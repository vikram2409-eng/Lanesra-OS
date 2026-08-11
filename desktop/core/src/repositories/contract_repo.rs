use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::contract::{Contract, ContractInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Contract> {
    Ok(Contract {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        contract_number: row.get("contract_number")?,
        company_id: row.get("company_id")?,
        contact_id: row.get("contact_id")?,
        source_quote_id: row.get("source_quote_id")?,
        title: row.get("title")?,
        r#type: row.get("type")?,
        value_cents: row.get("value_cents")?,
        currency_code: row.get("currency_code")?,
        owner_user_id: row.get("owner_user_id")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        renewal_date: row.get("renewal_date")?,
        notice_period_days: row.get("notice_period_days")?,
        status: row.get("status")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    contract_number: &str,
    input: &ContractInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Contract> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO contracts (id, workspace_id, contract_number, company_id, contact_id, source_quote_id, title, type, value_cents, currency_code, owner_user_id, start_date, end_date, renewal_date, notice_period_days, status, notes, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?18, ?19)",
        rusqlite::params![
            id,
            workspace_id,
            contract_number,
            &input.company_id,
            &input.contact_id,
            &input.source_quote_id,
            &input.title,
            &input.r#type,
            input.value_cents,
            &input.currency_code,
            &input.owner_user_id,
            &input.start_date,
            &input.end_date,
            &input.renewal_date,
            input.notice_period_days,
            &input.status,
            &input.notes,
            &now,
            actor_user_id,
        ],
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Contract>> {
    conn.query_row("SELECT * FROM contracts WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Contract>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contracts WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn list_by_company(conn: &Connection, company_id: &str) -> rusqlite::Result<Vec<Contract>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contracts WHERE company_id = ?1 AND archived_at IS NULL ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([company_id], map_row)?;
    rows.collect()
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: &str,
    input: &ContractInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Contract> {
    let now = now_iso();
    conn.execute(
        "UPDATE contracts SET company_id = ?1, contact_id = ?2, source_quote_id = ?3, title = ?4, type = ?5,
            value_cents = ?6, currency_code = ?7, owner_user_id = ?8, start_date = ?9, end_date = ?10,
            renewal_date = ?11, notice_period_days = ?12, status = ?13, notes = ?14, updated_at = ?15, updated_by = ?16
         WHERE id = ?17",
        rusqlite::params![
            &input.company_id,
            &input.contact_id,
            &input.source_quote_id,
            &input.title,
            &input.r#type,
            input.value_cents,
            &input.currency_code,
            &input.owner_user_id,
            &input.start_date,
            &input.end_date,
            &input.renewal_date,
            input.notice_period_days,
            &input.status,
            &input.notes,
            &now,
            actor_user_id,
            id,
        ],
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE contracts SET archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}

/// Count of active, non-archived contracts whose renewal_date falls within
/// the next `days` days (used for the 30/60/90-day renewal alerts, 9.1/FR-CTR-05).
pub fn count_renewing_within(conn: &Connection, workspace_id: &str, days: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM contracts
         WHERE workspace_id = ?1 AND archived_at IS NULL
           AND status NOT IN ('Expired', 'Terminated', 'Renewed')
           AND renewal_date IS NOT NULL
           AND date(renewal_date) BETWEEN date('now') AND date('now', ?2 || ' days')",
        (workspace_id, days.to_string()),
        |row| row.get(0),
    )
}

/// See company_repo::set_owner's comment - same reasoning.
pub fn set_owner(conn: &Connection, id: &str, owner_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE contracts SET owner_user_id = ?1, updated_at = ?2 WHERE id = ?3",
        (owner_user_id, now_iso(), id),
    )?;
    Ok(())
}
