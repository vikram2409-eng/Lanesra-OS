//! AI & Agentic Layer, Phase 4: Agent Actions. The product backlog names
//! four - meeting-prep briefing, commitment capture, record-hygiene
//! suggestions, and natural-language reporting - as four *separate*
//! scoping decisions, not one pass. Only **natural-language reporting**
//! is built here: the backlog's own reasoning is that it's "the
//! lowest-risk starting point since it reuses shipped infrastructure
//! directly and needs no new data model," while the other three "get
//! more useful once at least one live Activity channel exists" (none
//! does yet - each live channel, email/call/Slack, is still its own
//! separately-tracked proposed item). This module is where those three
//! land too, once each is its own scoped phase - not a reason to build
//! them speculatively now.
//!
//! `ask_report` translates a plain-English question into the exact
//! shape `custom_report_service` already runs (`CustomReportInput` -
//! one object, one group-by field, count-or-sum) via a real call to the
//! workspace's configured LLM (`ai_service::complete`) - not a new
//! reporting engine, a thin translation layer in front of the one that
//! already exists. The model's output is validated and executed through
//! `custom_report_service::preview`, the same `validate_shape` a
//! human's manual report creation already goes through - an
//! ill-formed or out-of-scope reply is rejected exactly the same way a
//! person's own mistake would be, not trusted specially because an LLM
//! produced it.

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::{AppError, AppResult};
use crate::models::agent::{NlReportQuery, NlReportResult};
use crate::models::business_rule::builtin_trigger_field_for;
use crate::models::custom_field::CUSTOM_FIELD_ENTITY_TYPES;
use crate::models::custom_report::CustomReportInput;
use crate::repositories::custom_field_repo;

const MAX_QUESTION_LEN: usize = 500;

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() } else { format!("{}...", s.chars().take(max).collect::<String>()) }
}

/// One line per reportable object: its key, its builtin group-by field,
/// and any active+reportable custom fields (numeric ones flagged
/// summable) - exactly the set `custom_report_service::validate_shape`
/// already treats as legal, described in plain language for the model.
fn describe_catalog(conn: &Connection, workspace_id: &str) -> AppResult<String> {
    let mut lines = Vec::new();
    let mut describe_one = |key: &str, label: &str| -> AppResult<()> {
        let builtin = builtin_trigger_field_for(key);
        let fields: Vec<String> = custom_field_repo::list_definitions(conn, workspace_id, key)?
            .into_iter()
            .filter(|d| d.is_active && d.is_reportable)
            .map(|d| if d.field_type == "number" { format!("{} (key: \"{}\", numeric, can be summed)", d.label, d.key) } else { format!("{} (key: \"{}\")", d.label, d.key) })
            .collect();
        let extra = if fields.is_empty() { String::new() } else { format!(" Custom fields: {}.", fields.join(", ")) };
        lines.push(format!("- \"{key}\" ({label}): built-in group-by field is \"{builtin}\".{extra}"));
        Ok(())
    };
    for key in CUSTOM_FIELD_ENTITY_TYPES {
        describe_one(key, key)?;
    }
    for def in super::custom_object_service::list(conn, workspace_id, true)? {
        describe_one(&def.key, &def.singular_label)?;
    }
    Ok(lines.join("\n"))
}

fn system_prompt(catalog: &str) -> String {
    format!(
        "You translate a plain-English reporting question into a strict JSON directive for an \
         existing, deliberately simple report engine. The engine can only do ONE thing: group \
         every record of ONE object type by ONE field (its built-in group-by field, or one active \
         custom field), and either count the records in each group or sum one numeric custom field. \
         It has no filters, no date ranges, no multiple group-bys, and no joins across objects.\n\n\
         Objects this workspace has, and their valid group-by options:\n{catalog}\n\n\
         Respond with ONLY a JSON object - no markdown fences, no other text - matching exactly:\n\
         {{\"entity_type\": \"<one of the object keys above, exactly as written>\", \
         \"group_by_source\": \"builtin\" or \"custom\", \
         \"group_by_field\": \"<the built-in field name, or a listed custom field's key>\", \
         \"aggregate\": \"count\" or \"sum\", \
         \"sum_field_key\": \"<a listed numeric custom field's key, or null if aggregate is count>\"}}\n\n\
         If the question cannot be answered with this shape - it needs a filter, a date range, \
         multiple group-bys, a field not listed above, or anything else this engine can't do - \
         respond with ONLY: {{\"error\": \"<a short, plain-English reason>\"}}"
    )
}

fn strip_code_fence(s: &str) -> &str {
    let t = s.trim();
    let Some(rest) = t.strip_prefix("```") else { return t };
    let rest = rest.strip_prefix("json").unwrap_or(rest).trim_start();
    rest.strip_suffix("```").unwrap_or(rest).trim()
}

fn required_str<'a>(value: &'a Value, key: &str) -> AppResult<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| AppError::Validation(format!("The model's response is missing '{key}'")))
}

fn parse_directive(raw: &str, question: &str) -> AppResult<CustomReportInput> {
    let cleaned = strip_code_fence(raw);
    let value: Value = serde_json::from_str(cleaned)
        .map_err(|_| AppError::Validation(format!("The model didn't return valid JSON: {}", truncate(raw, 300))))?;
    if let Some(reason) = value.get("error").and_then(Value::as_str) {
        return Err(AppError::Validation(format!("That question doesn't fit the report engine: {reason}")));
    }
    Ok(CustomReportInput {
        name: truncate(question, 60),
        entity_type: required_str(&value, "entity_type")?.to_string(),
        group_by_source: required_str(&value, "group_by_source")?.to_string(),
        group_by_field: required_str(&value, "group_by_field")?.to_string(),
        aggregate: required_str(&value, "aggregate")?.to_string(),
        sum_field_key: value.get("sum_field_key").and_then(Value::as_str).map(String::from),
    })
}

/// No admin gate - same convention `ai_service::complete` and
/// `custom_report_service::preview` (which this calls) already
/// document: asking a question needs no more privilege than viewing an
/// existing report's numbers already does.
pub async fn ask_report(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], query: &NlReportQuery) -> AppResult<NlReportResult> {
    let question = query.question.trim();
    if question.is_empty() {
        return Err(AppError::Validation("Ask a question first".into()));
    }
    if question.chars().count() > MAX_QUESTION_LEN {
        return Err(AppError::Validation(format!("Keep the question under {MAX_QUESTION_LEN} characters")));
    }
    let catalog = describe_catalog(conn, workspace_id)?;
    let prompt = system_prompt(&catalog);
    let raw = super::ai_service::complete(conn, workspace_id, master_key, &prompt, question).await?;
    let report = parse_directive(&raw, question)?;
    let rows = super::custom_report_service::preview(conn, workspace_id, &report)?;
    Ok(NlReportResult { report, rows })
}
