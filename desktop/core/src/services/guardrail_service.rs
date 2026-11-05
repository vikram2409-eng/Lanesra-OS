//! WP-0133: output guardrails as versioned policy objects (REQ-GOV-007).
//!
//! A guardrail checks some output and passes or fails. The check itself is a
//! pure function ([`evaluate`]) so every built-in has a straightforward
//! positive/negative test; the `judge` kind takes the model's verdict as an
//! argument, so the pure logic needs no live model. Fetching that verdict
//! from the configured provider, and rendering its prompt through the
//! catalog (ADR-0011), is the follow-up that wires guardrails into the agent
//! run loop; this module owns the objects, their versions, the built-in
//! library, and the evaluation record a run keeps (AC-2).

use rusqlite::Connection;
use serde_json::Value;

use crate::domain::{AppError, AppResult};
use crate::models::ai_guardrail::{Guardrail, GuardrailEdit, GuardrailInput, GuardrailOutcome};
use crate::repositories::ai_guardrail_repo as repo;

pub const CHECKER_TYPES: &[&str] = &["rule", "pii", "schema", "judge"];

/// WP-0128: where a guardrail runs in the agent run loop. `off` is not
/// wired in (the default); the three loop points are `step`, `model` and
/// `tool`. See `policy_engine_service::HookPoint`.
pub const HOOK_POINTS: &[&str] = &["off", "step", "model", "tool"];

fn validate_hook_point(hook_point: &str) -> AppResult<()> {
    if HOOK_POINTS.contains(&hook_point) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "hook point must be one of {HOOK_POINTS:?}"
        )))
    }
}

fn require_admin(conn: &Connection, actor: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor)
}

fn pass(detail: impl Into<String>) -> GuardrailOutcome {
    GuardrailOutcome {
        passed: true,
        detail: detail.into(),
    }
}
fn fail(detail: impl Into<String>) -> GuardrailOutcome {
    GuardrailOutcome {
        passed: false,
        detail: detail.into(),
    }
}

/// Evaluates `output` against a guardrail of `checker_type` configured by
/// `config`. For a `judge` guardrail, `judge_reply` is the model's verdict
/// (`None` is an error for that kind only).
pub fn evaluate(
    checker_type: &str,
    config: &Value,
    output: &str,
    judge_reply: Option<&str>,
) -> AppResult<GuardrailOutcome> {
    match checker_type {
        "pii" => {
            let found = super::dlp_service::scan(output);
            Ok(if found.is_empty() {
                pass("No PII detected")
            } else {
                fail(format!("PII detected: {}", found.join(", ")))
            })
        }
        "rule" => evaluate_rule(config, output),
        "schema" => evaluate_schema(config, output),
        "judge" => {
            let reply = judge_reply.ok_or_else(|| {
                AppError::Validation("a judge guardrail needs a model verdict".into())
            })?;
            Ok(evaluate_judge(config, reply))
        }
        other => Err(AppError::Validation(format!(
            "unknown guardrail checker '{other}'"
        ))),
    }
}

fn evaluate_rule(config: &Value, output: &str) -> AppResult<GuardrailOutcome> {
    let mode = config
        .get("mode")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Validation("rule guardrail needs a \"mode\"".into()))?;
    match mode {
        "must_contain" => {
            let text = config.get("text").and_then(Value::as_str).unwrap_or("");
            Ok(if output.contains(text) {
                pass("Required text present")
            } else {
                fail(format!("Missing required text: {text:?}"))
            })
        }
        "forbid_substrings" => {
            let list = config
                .get("substrings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let hit = list
                .iter()
                .filter_map(Value::as_str)
                .find(|s| output.contains(*s));
            Ok(match hit {
                Some(s) => fail(format!("Forbidden text present: {s:?}")),
                None => pass("No forbidden text"),
            })
        }
        "domain_allowlist" => {
            let allowed: Vec<String> = config
                .get("domains")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(|s| s.to_lowercase())
                        .collect()
                })
                .unwrap_or_default();
            let offending: Vec<String> = hosts_in(output)
                .into_iter()
                .filter(|h| {
                    !allowed
                        .iter()
                        .any(|d| h == d || h.ends_with(&format!(".{d}")))
                })
                .collect();
            Ok(if offending.is_empty() {
                pass("All linked domains are allowed")
            } else {
                fail(format!(
                    "Domains not on the allow-list: {}",
                    offending.join(", ")
                ))
            })
        }
        other => Err(AppError::Validation(format!("unknown rule mode '{other}'"))),
    }
}

/// Lowercased hostnames of every http(s) URL in the text.
fn hosts_in(text: &str) -> Vec<String> {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"https?://([^/\s:]+)").unwrap());
    let mut hosts: Vec<String> = re
        .captures_iter(text)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_lowercase())
        .collect();
    hosts.sort();
    hosts.dedup();
    hosts
}

fn evaluate_schema(config: &Value, output: &str) -> AppResult<GuardrailOutcome> {
    let schema = config
        .get("schema")
        .ok_or_else(|| AppError::Validation("schema guardrail needs a \"schema\"".into()))?;
    let parsed: Value = match serde_json::from_str(output.trim()) {
        Ok(v) => v,
        Err(e) => return Ok(fail(format!("Output is not valid JSON: {e}"))),
    };
    Ok(match json_conforms(schema, &parsed) {
        Ok(()) => pass("Output conforms to the schema"),
        Err(why) => fail(why),
    })
}

/// A small, dependency-free JSON-schema check: `type`, `required`,
/// `properties`, `enum` and `items`. Enough for a conformance guardrail
/// without pulling in a validator crate.
fn json_conforms(schema: &Value, value: &Value) -> Result<(), String> {
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            return Err("value is not one of the allowed enum values".into());
        }
    }
    if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        let ok = match ty {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => return Err(format!("unsupported schema type '{ty}'")),
        };
        if !ok {
            return Err(format!("expected {ty}"));
        }
    }
    if let Some(req) = schema.get("required").and_then(Value::as_array) {
        let obj = value
            .as_object()
            .ok_or("expected an object for required keys")?;
        for key in req.iter().filter_map(Value::as_str) {
            if !obj.contains_key(key) {
                return Err(format!("missing required property '{key}'"));
            }
        }
    }
    if let Some(props) = schema.get("properties").and_then(Value::as_object) {
        if let Some(obj) = value.as_object() {
            for (key, sub) in props {
                if let Some(v) = obj.get(key) {
                    json_conforms(sub, v).map_err(|e| format!("{key}: {e}"))?;
                }
            }
        }
    }
    if let Some(items) = schema.get("items") {
        if let Some(arr) = value.as_array() {
            for (i, v) in arr.iter().enumerate() {
                json_conforms(items, v).map_err(|e| format!("[{i}]: {e}"))?;
            }
        }
    }
    Ok(())
}

fn evaluate_judge(config: &Value, reply: &str) -> GuardrailOutcome {
    let pass_token = config
        .get("pass_token")
        .and_then(Value::as_str)
        .unwrap_or("PASS");
    let first = reply.trim().lines().next().unwrap_or("").to_uppercase();
    if first.contains(&pass_token.to_uppercase()) {
        pass(reply.trim().chars().take(200).collect::<String>())
    } else {
        fail(reply.trim().chars().take(200).collect::<String>())
    }
}

// ---- CRUD + versioning + built-ins ----------------------------------------

/// `(builtin_key, name, description, checker_type, config_json)`.
pub const BUILTINS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "pii_leak",
        "No PII leak",
        "Fails when the output contains an email, phone number, card or other PII.",
        "pii",
        "{}",
    ),
    (
        "required_disclaimer",
        "Required disclaimer",
        "The output must contain a disclaimer. Edit the text for your workspace.",
        "rule",
        r#"{"mode":"must_contain","text":"This is not professional advice."}"#,
    ),
    (
        "domain_allowlist",
        "External domain allow-list",
        "Every linked domain must be on the allow-list. Edit the domains for your workspace.",
        "rule",
        r#"{"mode":"domain_allowlist","domains":["example.com"]}"#,
    ),
    (
        "json_schema",
        "Schema conformance",
        "The output must be JSON matching the configured schema.",
        "schema",
        r#"{"schema":{"type":"object","required":["answer"]}}"#,
    ),
    (
        "tone",
        "Professional tone",
        "An LLM judge checks the output is professional and courteous.",
        "judge",
        r#"{"instruction":"The response must be professional and courteous.","pass_token":"PASS"}"#,
    ),
];

/// Seeds the built-in library for a workspace that has none yet (idempotent).
pub fn ensure_builtins(conn: &Connection, workspace_id: &str) -> AppResult<()> {
    for (key, name, desc, checker, config) in BUILTINS {
        if repo::get_by_builtin_key(conn, workspace_id, key)?.is_none() {
            repo::create(
                conn,
                workspace_id,
                Some(key),
                name,
                Some(desc),
                checker,
                true,
                config,
                "off",
                None,
            )?;
        }
    }
    Ok(())
}

pub fn list(
    conn: &Connection,
    workspace_id: &str,
    actor: Option<&str>,
) -> AppResult<Vec<Guardrail>> {
    require_admin(conn, actor)?;
    ensure_builtins(conn, workspace_id)?;
    Ok(repo::list(conn, workspace_id)?)
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Guardrail> {
    repo::get(conn, workspace_id, id)?.ok_or_else(|| AppError::NotFound("Guardrail".into()))
}

fn validate_config(checker_type: &str, config_json: &str) -> AppResult<Value> {
    let value: Value = serde_json::from_str(config_json)
        .map_err(|e| AppError::Validation(format!("config is not valid JSON: {e}")))?;
    match checker_type {
        "pii" => {}
        "rule" => {
            // A concrete output exercises the mode/shape before it is stored.
            evaluate_rule(&value, "")?;
        }
        "schema" => {
            value.get("schema").ok_or_else(|| {
                AppError::Validation("schema guardrail needs a \"schema\"".into())
            })?;
        }
        "judge" => {
            value
                .get("instruction")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    AppError::Validation("judge guardrail needs an \"instruction\"".into())
                })?;
        }
        other => return Err(AppError::Validation(format!("unknown checker '{other}'"))),
    }
    Ok(value)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &GuardrailInput,
    actor: Option<&str>,
) -> AppResult<Guardrail> {
    require_admin(conn, actor)?;
    if !CHECKER_TYPES.contains(&input.checker_type.as_str()) {
        return Err(AppError::Validation(format!(
            "checker must be one of {CHECKER_TYPES:?}"
        )));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    validate_config(&input.checker_type, &input.config_json)?;
    validate_hook_point(&input.hook_point)?;
    let id = repo::create(
        conn,
        workspace_id,
        None,
        input.name.trim(),
        input.description.as_deref(),
        &input.checker_type,
        false,
        &input.config_json,
        &input.hook_point,
        actor,
    )?;
    get(conn, workspace_id, &id)
}

/// Edits a guardrail. A changed `config_json` writes a new version and bumps
/// `current_version` (AC-2); name/description/enabled update in place.
pub fn edit(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    edit: &GuardrailEdit,
    actor: Option<&str>,
) -> AppResult<Guardrail> {
    require_admin(conn, actor)?;
    let current = get(conn, workspace_id, id)?;
    if let Some(new_config) = edit.config_json.as_deref() {
        let a: Value = serde_json::from_str(&current.config_json).unwrap_or(Value::Null);
        let b: Value = validate_config(&current.checker_type, new_config)?;
        if a != b {
            repo::add_version(conn, id, new_config, actor)?;
        }
    }
    let hook_point = edit.hook_point.as_deref().unwrap_or(&current.hook_point);
    validate_hook_point(hook_point)?;
    repo::update_meta(
        conn,
        id,
        edit.name.as_deref().unwrap_or(&current.name),
        edit.description
            .as_deref()
            .or(current.description.as_deref()),
        edit.enabled.unwrap_or(current.enabled),
        hook_point,
        actor,
    )?;
    get(conn, workspace_id, id)
}

/// Evaluates a guardrail in the context of a run and records which version it
/// checked, so the run carries that (AC-2). `judge_reply` is required only for
/// a `judge` guardrail.
pub fn evaluate_for_run(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
    guardrail_id: &str,
    output: &str,
    judge_reply: Option<&str>,
) -> AppResult<GuardrailOutcome> {
    let g = get(conn, workspace_id, guardrail_id)?;
    let config: Value = serde_json::from_str(&g.config_json)
        .map_err(|e| AppError::Validation(format!("stored config is invalid: {e}")))?;
    let outcome = evaluate(&g.checker_type, &config, output, judge_reply)?;
    repo::record_evaluation(
        conn,
        workspace_id,
        run_kind,
        run_id,
        guardrail_id,
        g.current_version,
        outcome.passed,
        &outcome.detail,
    )?;
    Ok(outcome)
}
