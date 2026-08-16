//! Global search (spec §5.3/§9.3, roadmap "Global search & list-view
//! filtering"): a lightweight substring search across every core entity's
//! natural display fields, plus any custom field an admin has flagged
//! `is_searchable` - the first real use of a capability flag that's
//! existed since the Phase E custom-field-extensibility round but done
//! nothing until now (see `CustomFieldDefinition::is_searchable`'s doc
//! comment).
//!
//! Deliberately not a full-text search engine: no ranking beyond "which
//! table it came from", no tokenization, just a case-insensitive `LIKE`
//! per field - a "jump to a record I'm thinking of" tool, matching the
//! online demo's equally simple `runSearch` (see app.js), not a report.
//! Every core entity gets a hand-picked, per-type field list - whichever
//! columns a user would actually recognize a record by (name/number/
//! email/phone), not e.g. internal ids or money amounts.

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::domain::AppResult;
use crate::services::custom_object_service;

/// Results are capped so a broad query (a single common letter) can't
/// return hundreds of rows - matches the online demo's identical `slice(0,
/// 12)` cap in spirit, a little roomier since desktop has no dropdown
/// scroll-height constraint forcing a tighter number.
const MAX_RESULTS: usize = 25;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    /// A short secondary line - set when the match came from something
    /// other than the title itself (an email/phone, or a matched custom
    /// field's "label: value"), so a match on something other than the
    /// name is still explainable at a glance instead of looking like a
    /// false positive.
    pub subtitle: Option<String>,
}

fn like_pattern(query: &str) -> String {
    // Escapes the two characters SQLite's LIKE treats specially so a
    // query containing a literal "%" or "_" searches for that literal
    // text instead of being interpreted as a wildcard.
    format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
}

/// Runs `sql` (already scoped to `workspace_id` and non-archived rows) and
/// maps each row via `row_fn`, appending into `out` up to `MAX_RESULTS`
/// total across every entity type's contribution - shared by every
/// per-entity block below so the cap and the row-mapping boilerplate is
/// written once instead of nine times.
fn collect(
    conn: &Connection,
    out: &mut Vec<SearchResult>,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
    row_fn: impl Fn(&rusqlite::Row) -> rusqlite::Result<SearchResult>,
) -> AppResult<()> {
    if out.len() >= MAX_RESULTS {
        return Ok(());
    }
    let remaining = (MAX_RESULTS - out.len()) as i64;
    let sql = format!("{sql} LIMIT {remaining}");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params, row_fn)?;
    for row in rows {
        out.push(row?);
    }
    Ok(())
}

/// Every core entity's own table, then active custom objects, then
/// `is_searchable` custom field values - in roughly that priority order,
/// since a match on a record's own name/number is usually what the user
/// meant even when a custom field also happens to match.
pub fn global_search(conn: &Connection, workspace_id: &str, query: &str) -> AppResult<Vec<SearchResult>> {
    let query = query.trim();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }
    let like = like_pattern(query);
    let mut out = Vec::new();

    collect(
        conn, &mut out,
        "SELECT id, name, customer_number, email, phone FROM companies
         WHERE workspace_id = ?1 AND archived_at IS NULL
           AND (name LIKE ?2 ESCAPE '\\' OR customer_number LIKE ?2 ESCAPE '\\' OR email LIKE ?2 ESCAPE '\\'
                OR phone LIKE ?2 ESCAPE '\\' OR tax_number LIKE ?2 ESCAPE '\\' OR website LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let name: String = r.get("name")?;
            let email: Option<String> = r.get("email")?;
            let phone: Option<String> = r.get("phone")?;
            Ok(SearchResult { entity_type: "Company".into(), entity_id: r.get("id")?, title: name, subtitle: email.or(phone) })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, first_name, last_name, contact_number, email, phone, mobile FROM contacts
         WHERE workspace_id = ?1 AND archived_at IS NULL
           AND (first_name LIKE ?2 ESCAPE '\\' OR last_name LIKE ?2 ESCAPE '\\' OR contact_number LIKE ?2 ESCAPE '\\'
                OR email LIKE ?2 ESCAPE '\\' OR phone LIKE ?2 ESCAPE '\\' OR mobile LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let first: String = r.get("first_name")?;
            let last: String = r.get("last_name")?;
            let email: Option<String> = r.get("email")?;
            let phone: Option<String> = r.get("phone")?;
            Ok(SearchResult {
                entity_type: "Contact".into(), entity_id: r.get("id")?,
                title: format!("{first} {last}").trim().to_string(), subtitle: email.or(phone),
            })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name, opportunity_number FROM opportunities
         WHERE workspace_id = ?1 AND archived_at IS NULL AND (name LIKE ?2 ESCAPE '\\' OR opportunity_number LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| Ok(SearchResult { entity_type: "Opportunity".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: None }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name, sku, product_number, category FROM products
         WHERE workspace_id = ?1 AND archived_at IS NULL
           AND (name LIKE ?2 ESCAPE '\\' OR sku LIKE ?2 ESCAPE '\\' OR product_number LIKE ?2 ESCAPE '\\' OR category LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let sku: Option<String> = r.get("sku")?;
            Ok(SearchResult { entity_type: "Product".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: sku })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, quote_number FROM quotes WHERE workspace_id = ?1 AND archived_at IS NULL AND quote_number LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| Ok(SearchResult { entity_type: "Quote".into(), entity_id: r.get("id")?, title: r.get("quote_number")?, subtitle: None }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, order_number FROM orders WHERE workspace_id = ?1 AND archived_at IS NULL AND order_number LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| Ok(SearchResult { entity_type: "Order".into(), entity_id: r.get("id")?, title: r.get("order_number")?, subtitle: None }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, invoice_number FROM invoices WHERE workspace_id = ?1 AND archived_at IS NULL AND invoice_number LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| Ok(SearchResult { entity_type: "Invoice".into(), entity_id: r.get("id")?, title: r.get("invoice_number")?, subtitle: None }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, title, contract_number FROM contracts
         WHERE workspace_id = ?1 AND archived_at IS NULL AND (title LIKE ?2 ESCAPE '\\' OR contract_number LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| Ok(SearchResult { entity_type: "Contract".into(), entity_id: r.get("id")?, title: r.get("title")?, subtitle: None }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, title, task_number FROM tasks
         WHERE workspace_id = ?1 AND archived_at IS NULL AND (title LIKE ?2 ESCAPE '\\' OR task_number LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| Ok(SearchResult { entity_type: "Task".into(), entity_id: r.get("id")?, title: r.get("title")?, subtitle: None }),
    )?;

    // Active custom objects - one query per active object definition,
    // since each is its own logical entity_type sharing the one physical
    // custom_records table (see custom_record_repo's own doc comment).
    for def in custom_object_service::list(conn, workspace_id, true)? {
        if out.len() >= MAX_RESULTS {
            break;
        }
        collect(
            conn, &mut out,
            "SELECT id, primary_name, display_number FROM custom_records
             WHERE workspace_id = ?1 AND object_key = ?2 AND archived_at IS NULL
               AND (primary_name LIKE ?3 ESCAPE '\\' OR display_number LIKE ?3 ESCAPE '\\')",
            params![workspace_id, def.key, like],
            |r| Ok(SearchResult { entity_type: def.key.clone(), entity_id: r.get("id")?, title: r.get("primary_name")?, subtitle: None }),
        )?;
    }

    // is_searchable custom field values - joined back to the definition
    // for its label/entity_type/key, and to the owning record for a
    // display title via each entity's own resolve() dispatch (the same
    // one relationships/rules/workflows already use), since a custom
    // field match should still show the record's real name, not the
    // matched value itself, as the result title.
    if out.len() < MAX_RESULTS {
        let remaining = (MAX_RESULTS - out.len()) as i64;
        let mut stmt = conn.prepare(
            "SELECT v.entity_id, d.entity_type, d.label, v.value_text
             FROM custom_field_values v
             JOIN custom_field_definitions d ON d.id = v.definition_id
             WHERE d.workspace_id = ?1 AND d.is_active = 1 AND d.is_searchable = 1
               AND v.value_text LIKE ?2 ESCAPE '\\'
             LIMIT ?3",
        )?;
        let matches: Vec<(String, String, String, String)> = stmt
            .query_map(params![workspace_id, like, remaining], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        for (entity_id, entity_type, label, value) in matches {
            if out.len() >= MAX_RESULTS {
                break;
            }
            if let Some(resolved) = crate::services::entity_registry::resolve(conn, &entity_type, &entity_id)? {
                out.push(SearchResult {
                    entity_type, entity_id,
                    title: resolved.display_name,
                    subtitle: Some(format!("{label}: {value}")),
                });
            }
        }
    }

    Ok(out)
}
