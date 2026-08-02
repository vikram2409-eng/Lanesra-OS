//! Atomic, transaction-safe allocation of human-readable business document
//! numbers (Appendix B). Generated numbers are immutable and never reused
//! (BR-002); prefixes are configurable per workspace by an administrator,
//! this module just supplies the sensible defaults.

use chrono::Utc;
use rusqlite::Connection;

use super::ids::new_uuid;

pub struct NumberingConfig {
    pub entity_type: &'static str,
    pub default_prefix: &'static str,
    pub uses_year: bool,
    pub digits: usize,
}

pub const COMPANY: NumberingConfig = NumberingConfig {
    entity_type: "company",
    default_prefix: "CUS",
    uses_year: false,
    digits: 6,
};
pub const CONTACT: NumberingConfig = NumberingConfig {
    entity_type: "contact",
    default_prefix: "CON",
    uses_year: false,
    digits: 6,
};
pub const OPPORTUNITY: NumberingConfig = NumberingConfig {
    entity_type: "opportunity",
    default_prefix: "OPP",
    uses_year: true,
    digits: 6,
};
pub const PRODUCT: NumberingConfig = NumberingConfig {
    entity_type: "product",
    default_prefix: "PRD",
    uses_year: false,
    digits: 6,
};
pub const QUOTE: NumberingConfig = NumberingConfig {
    entity_type: "quote",
    default_prefix: "Q",
    uses_year: true,
    digits: 6,
};
pub const ORDER: NumberingConfig = NumberingConfig {
    entity_type: "order",
    default_prefix: "SO",
    uses_year: true,
    digits: 6,
};
pub const INVOICE: NumberingConfig = NumberingConfig {
    entity_type: "invoice",
    default_prefix: "INV",
    uses_year: true,
    digits: 6,
};
pub const CONTRACT: NumberingConfig = NumberingConfig {
    entity_type: "contract",
    default_prefix: "CTR",
    uses_year: true,
    digits: 6,
};
pub const TASK: NumberingConfig = NumberingConfig {
    entity_type: "task",
    default_prefix: "TSK",
    uses_year: false,
    digits: 6,
};

/// Atomically allocates and formats the next number for `config` within
/// `workspace_id`. Must be called within the same transaction as the record
/// insert it numbers, so a rolled-back insert does not burn a gap silently
/// (gaps from legitimate rollbacks are acceptable; reuse is not).
pub fn allocate_number(
    conn: &Connection,
    workspace_id: &str,
    config: &NumberingConfig,
) -> rusqlite::Result<String> {
    let period_key = if config.uses_year {
        Utc::now().format("%Y").to_string()
    } else {
        String::new()
    };

    let next_value: i64 = conn.query_row(
        "INSERT INTO number_sequences (id, workspace_id, entity_type, prefix, period_key, next_value)
         VALUES (?1, ?2, ?3, ?4, ?5, 1)
         ON CONFLICT (workspace_id, entity_type, period_key)
         DO UPDATE SET next_value = number_sequences.next_value + 1
         RETURNING next_value",
        (
            new_uuid(),
            workspace_id,
            config.entity_type,
            config.default_prefix,
            &period_key,
        ),
        |row| row.get(0),
    )?;

    Ok(if config.uses_year {
        format!(
            "{}-{}-{:0width$}",
            config.default_prefix,
            period_key,
            next_value,
            width = config.digits
        )
    } else {
        format!(
            "{}-{:0width$}",
            config.default_prefix,
            next_value,
            width = config.digits
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory_db;

    fn seed_workspace(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO workspaces (id, business_name, created_at, updated_at) VALUES (?1, 'Test', '2026-01-01', '2026-01-01')",
            [id],
        )
        .unwrap();
    }

    #[test]
    fn allocates_sequential_numbers() {
        let conn = open_in_memory_db().unwrap();
        seed_workspace(&conn, "ws1");

        let first = allocate_number(&conn, "ws1", &COMPANY).unwrap();
        let second = allocate_number(&conn, "ws1", &COMPANY).unwrap();
        let third = allocate_number(&conn, "ws1", &COMPANY).unwrap();

        assert_eq!(first, "CUS-000001");
        assert_eq!(second, "CUS-000002");
        assert_eq!(third, "CUS-000003");
    }

    #[test]
    fn includes_year_for_year_scoped_entities() {
        let conn = open_in_memory_db().unwrap();
        seed_workspace(&conn, "ws1");

        let number = allocate_number(&conn, "ws1", &QUOTE).unwrap();
        let year = Utc::now().format("%Y").to_string();
        assert_eq!(number, format!("Q-{}-000001", year));
    }

    #[test]
    fn sequences_are_isolated_per_workspace_and_entity_type() {
        let conn = open_in_memory_db().unwrap();
        seed_workspace(&conn, "ws1");
        seed_workspace(&conn, "ws2");

        assert_eq!(allocate_number(&conn, "ws1", &COMPANY).unwrap(), "CUS-000001");
        assert_eq!(allocate_number(&conn, "ws2", &COMPANY).unwrap(), "CUS-000001");
        assert_eq!(allocate_number(&conn, "ws1", &CONTACT).unwrap(), "CON-000001");
        assert_eq!(allocate_number(&conn, "ws1", &COMPANY).unwrap(), "CUS-000002");
    }
}
