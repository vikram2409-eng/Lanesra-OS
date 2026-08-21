//! Integration Hub (spec §12/§13): generalizes the existing Companies/
//! Contacts-only *client-side* CSV import (`CsvImportDialog.tsx`, one
//! `create_*` call per row) into a real, object-agnostic, server-side
//! wizard - any built-in entity `api_object_service` can write, or an
//! active Custom Object - with column mapping, per-field transforms,
//! duplicate/upsert handling, a dry-run preview, and a real per-row
//! result. Every write goes through `api_object_service::create_record`/
//! `update_record`, the exact same dispatcher the inbound REST API uses,
//! so the same validation/business-rule/permission checks apply here too
//! (spec §9: "bulk writes must invoke the same checks as UI/API writes").
//!
//! `export_csv` is the mirror operation - any object's current
//! `list_records` result flattened to real RFC 4180 CSV text.

use std::collections::HashMap;
use std::time::Instant;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult};
use crate::models::integration::{ApiListQuery, CsvImportInput, CsvImportResult, CsvRowResult, FieldMapEntry};
use crate::services::integration_log_service::{self, FinishOutcome};
use crate::services::{api_object_service, mapping_service};

fn build_record_value(row: &HashMap<String, String>, field_map: &[FieldMapEntry]) -> Value {
    let mut obj = serde_json::Map::new();
    for entry in field_map {
        let raw = match &entry.constant {
            Some(constant) => constant.clone(),
            None => row.get(&entry.source_column).cloned().unwrap_or_default(),
        };
        let value = if raw.trim().is_empty() {
            entry.default_value.clone().unwrap_or(raw)
        } else {
            mapping_service::apply_transform(entry.transform.as_deref(), &raw)
        };
        obj.insert(entry.target_field.clone(), json!(value));
    }
    Value::Object(obj)
}

fn parse_csv(csv_text: &str) -> AppResult<Vec<HashMap<String, String>>> {
    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(csv_text.as_bytes());
    let headers = reader.headers().map_err(|e| AppError::Validation(format!("Invalid CSV: {e}")))?.clone();
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| AppError::Validation(format!("Invalid CSV row: {e}")))?;
        let row = headers.iter().enumerate().map(|(i, h)| (h.to_string(), record.get(i).unwrap_or("").to_string())).collect();
        rows.push(row);
    }
    Ok(rows)
}

/// Finds an existing record whose `match_key` field equals `value` - a
/// plain linear scan over `list_records`, fine at CSV-import scale
/// (hundreds to low thousands of rows), not meant for Bulk API volumes.
/// `pub(crate)` so `integration_job_service`'s upsert logic can reuse the
/// exact same lookup rather than re-implementing it.
pub(crate) fn find_existing(conn: &Connection, workspace_id: &str, object_key: &str, match_key: &str, value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    let page = api_object_service::list_records(conn, workspace_id, object_key, &ApiListQuery::default()).ok()?;
    page.records.iter().find(|r| r.get(match_key).and_then(|v| v.as_str()) == Some(value)).and_then(|r| r.get("id").and_then(|v| v.as_str()).map(String::from))
}

enum RowAction {
    Create,
    Update(String),
    Skip(String),
    Fail(String),
}

fn decide_action(operation: &str, duplicate_policy: &str, existing_id: Option<String>) -> RowAction {
    match existing_id {
        Some(id) if duplicate_policy == "skip" => RowAction::Skip(id),
        Some(id) => match operation {
            "update" | "upsert" => RowAction::Update(id),
            // "insert" with a match found and a non-skip duplicate policy:
            // still never silently overwrites an existing record under an
            // insert-only mapping - surfaced as a per-row failure instead.
            _ => RowAction::Fail(format!("A matching record already exists ({id}) - this mapping is insert-only")),
        },
        None => match operation {
            "update" => RowAction::Fail("No matching existing record - update requires a match".into()),
            _ => RowAction::Create,
        },
    }
}

pub fn import_csv(conn: &Connection, workspace_id: &str, input: &CsvImportInput, actor_user_id: Option<&str>) -> AppResult<CsvImportResult> {
    let started = Instant::now();
    let rows = parse_csv(&input.csv_text)?;
    let mut row_results = Vec::with_capacity(rows.len());
    let (mut successful, mut failed, mut skipped_duplicates) = (0usize, 0usize, 0usize);

    // A dry run never writes, so it never needs its own execution log
    // entry either - nothing actually happened for the unified log to
    // describe (spec §12.1 step 8: "preview... nothing is written").
    let execution_id =
        (!input.dry_run).then(|| integration_log_service::start(conn, workspace_id, "csv_import", None, None, "inbound", actor_user_id));

    for (index, row) in rows.iter().enumerate() {
        let value = build_record_value(row, &input.field_map);
        let existing_id = input.match_key.as_ref().and_then(|key| {
            let match_value = value.get(key).and_then(|v| v.as_str()).unwrap_or_default();
            find_existing(conn, workspace_id, &input.target_object_key, key, match_value)
        });

        let (status, record_id, error) = match decide_action(&input.operation, &input.duplicate_policy, existing_id) {
            RowAction::Skip(id) => ("skipped".to_string(), Some(id), None),
            RowAction::Fail(msg) => ("failed".to_string(), None, Some(msg)),
            RowAction::Create if input.dry_run => ("would_create".to_string(), None, None),
            RowAction::Update(id) if input.dry_run => ("would_update".to_string(), Some(id), None),
            RowAction::Create => match api_object_service::create_record(conn, workspace_id, &input.target_object_key, &value, actor_user_id) {
                Ok(record) => ("created".to_string(), record.get("id").and_then(|v| v.as_str()).map(String::from), None),
                Err(e) => ("failed".to_string(), None, Some(e.to_string())),
            },
            RowAction::Update(id) => match api_object_service::update_record(conn, workspace_id, &input.target_object_key, &id, &value, actor_user_id) {
                Ok(_) => ("updated".to_string(), Some(id), None),
                Err(e) => ("failed".to_string(), None, Some(e.to_string())),
            },
        };

        match status.as_str() {
            "failed" => failed += 1,
            "skipped" => skipped_duplicates += 1,
            _ => successful += 1,
        }
        row_results.push(CsvRowResult { row_index: index, status, record_id, error });
    }

    if let Some(execution_id) = execution_id {
        integration_log_service::finish(
            conn,
            &execution_id,
            &FinishOutcome {
                status: if failed == 0 { "success".into() } else { "partial".into() },
                records_written: successful as i64,
                records_skipped: skipped_duplicates as i64,
                records_failed: failed as i64,
                ..Default::default()
            },
        );
    }

    Ok(CsvImportResult { total_rows: rows.len(), successful, failed, skipped_duplicates, row_results, duration_ms: started.elapsed().as_millis() as u64 })
}

/// Any object's current records, flattened to CSV text - column order
/// follows `query.select` if given, otherwise every key present on the
/// first record (an empty result set exports just a blank file, not an
/// error - nothing to flatten is a legitimate outcome, not a failure).
pub fn export_csv(conn: &Connection, workspace_id: &str, object_key: &str, query: &ApiListQuery) -> AppResult<String> {
    let page = api_object_service::list_records(conn, workspace_id, object_key, query)?;
    let columns: Vec<String> = match &query.select {
        Some(cols) => cols.clone(),
        None => page.records.first().and_then(|r| r.as_object()).map(|o| o.keys().cloned().collect()).unwrap_or_default(),
    };
    let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
    if !columns.is_empty() {
        writer.write_record(&columns).map_err(|e| AppError::Validation(format!("could not write CSV header: {e}")))?;
    }
    for record in &page.records {
        let row: Vec<String> = columns.iter().map(|c| record.get(c).map(value_to_csv_cell).unwrap_or_default()).collect();
        writer.write_record(&row).map_err(|e| AppError::Validation(format!("could not write CSV row: {e}")))?;
    }
    let bytes = writer.into_inner().map_err(|e| AppError::Validation(format!("could not finalize CSV: {e}")))?;
    String::from_utf8(bytes).map_err(|e| AppError::Validation(format!("CSV output was not valid UTF-8: {e}")))
}

fn value_to_csv_cell(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
