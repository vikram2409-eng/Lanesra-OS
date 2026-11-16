//! AI Agent Platform v2, Phase 7: WP-0714 - dry run, publish checks,
//! lane-case framework and Solution export/import (REQ-LANE-021).
//!
//! Publish checks (WP-0714 §2): `check_publish` runs the full P-01..P-16
//! enumeration in report mode - one entry per check, every failure named -
//! and `publish_lane` (in `process_lane_service`) refuses with the whole
//! list when any REFUSE check fails. P-01..P-08 and the P-09 reachability
//! half are the existing validators, called as-is and mapped into named
//! entries (they are first-failure-only, so a failure there is reported
//! once, with the other P-01..P-08 entries marked `warn / not evaluated`
//! rather than guessed at). The new checks are P-09 outcome coverage,
//! P-10 (WARN), P-11, P-12, P-13, P-14 (WARN), P-15 and P-16.
//!
//! The dry run (§3) and the lane-case runner (§4) live here too: both
//! walk a lane in definition order against *live* values at evaluation
//! time - the seed record, the fixtures, and values written by earlier
//! simulated steps (RAID I-34, dry-run half DECIDED). The engine half of
//! I-34 stays open: live runs still read the run-start snapshot and
//! per-item checks still compile to zero conditions, so nothing here
//! touches `graph_runtime_service`, `execution_graph` or `graph_run`.
//!
//! What this module deliberately does NOT do:
//! - Adopt Action Types (P-12's catalog is read, never written here;
//!   adoption is WP-0705's follow-up). A provisional id refuses publish.
//! - Change the engine: the dry run simulates, the case runner drives
//!   the real runner through its public service functions.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::domain::builtin_fields;
use crate::domain::conditions::{condition_matches, CONDITION_OPERATORS};
use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::process_lane::{
    parse_definition, validate_definition, ConditionOrGroup, LaneDefinition, LaneDefinitionFile,
    LANE_REPEAT_MAX_ITERATIONS,
};
use crate::repositories::{custom_field_repo, process_lane_repo};
use crate::services::{
    action_type_service, builtin_field_service, custom_field_service, custom_object_service,
    custom_record_service, entity_registry, process_lane_service, relationship_service,
    workflow_service,
};

fn classified_validation(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, code, message)
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Parses a definition file without running `validate_definition`, so the
/// publish report can name the failing checks instead of dying on the
/// first one (WP-0714 §2: the report lists every failure).
fn parse_definition_lenient(definition_json: &str) -> AppResult<LaneDefinitionFile> {
    serde_json::from_str::<LaneDefinitionFile>(definition_json).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("lane definition is not valid JSON: {e}"),
        )
    })
}

// --- publish report -------------------------------------------------------

/// One publish-check entry (WP-0714 §2): the check id, the step it belongs
/// to (None for lane-level checks), and `pass` / `refuse` / `warn` with
/// the named reason. A `warn` entry whose reason starts with
/// "not evaluated" means the check could not run because an earlier
/// first-failure validator stopped first - it is not a pass, and it never
/// blocks publish on its own.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishCheckEntry {
    pub id: String,
    pub step: Option<String>,
    pub status: String,
    pub reason: Option<String>,
}

/// The publish report (WP-0714 §2): one entry per P-01..P-16, `refused`
/// when any entry is `refuse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishReport {
    pub lane_key: String,
    pub version: Option<i64>,
    pub refused: bool,
    pub checks: Vec<PublishCheckEntry>,
    /// Gap 9 (builder gaps fix): set when the lane's `object_key` is not
    /// a known object in this workspace. The P-11/P-13 refusals below
    /// trace to the missing object, not to broken steps; the checks
    /// themselves stay as they are.
    #[serde(default)]
    pub framing: Option<String>,
}

fn entry(
    id: &str,
    step: Option<String>,
    status: &str,
    reason: Option<String>,
) -> PublishCheckEntry {
    PublishCheckEntry {
        id: id.to_string(),
        step,
        status: status.to_string(),
        reason,
    }
}

fn pass(id: &str, step: Option<String>) -> PublishCheckEntry {
    entry(id, step, "pass", None)
}

fn pass_reason(id: &str, step: Option<String>, reason: &str) -> PublishCheckEntry {
    entry(id, step, "pass", Some(reason.to_string()))
}

fn refuse(id: &str, step: Option<String>, reason: String) -> PublishCheckEntry {
    entry(id, step, "refuse", Some(reason))
}

fn warn(id: &str, step: Option<String>, reason: String) -> PublishCheckEntry {
    entry(id, step, "warn", Some(reason))
}

fn not_evaluated(id: &str, why: &str) -> PublishCheckEntry {
    warn(id, None, format!("not evaluated: {why}"))
}

/// Maps a `validate_definition` failure message onto the P-01..P-09 check
/// it belongs to (the validator is first-failure-only, so this is the one
/// named failure). Anything unrecognised is a definition-shape failure
/// and maps to P-01 with the raw reason.
fn classify_definition_error(message: &str) -> (&'static str, Option<String>) {
    let step = message
        .split("step '")
        .nth(1)
        .and_then(|rest| rest.split('\'').next())
        .map(String::from);
    let id = if message.contains("has no way out") {
        "P-09"
    } else if message.starts_with("lane id must start")
        || message.starts_with("lane name is required")
        || message.starts_with("lane version must be at least 1")
        || message.starts_with("lane status must be")
        || message.starts_with("an excluded lane id must start")
    {
        "P-01"
    } else if message.starts_with("a lane needs at least one stage")
        || message.starts_with("stage id")
        || message.starts_with("stage '")
        || message.starts_with("step id")
    {
        "P-02"
    } else if message.contains("needs a role") || message.contains("must not have a role") {
        "P-03"
    } else if message.contains("must declare its branches")
        || message.contains("only check and approval steps declare branches")
    {
        "P-04"
    } else if message.contains("must declare its writes")
        || message.contains("only record and approval steps declare writes")
    {
        "P-05"
    } else if message.contains("branches to unknown step")
        || message.contains("targets the step itself")
    {
        "P-06"
    } else if message.starts_with("run_data declares field")
        || message.starts_with("run_data field")
    {
        "P-08"
    } else {
        // Condition / expression shape failures (P-07), and anything else
        // that is still a definition-shape failure (P-01 fallback).
        if message.contains("must be a $record")
            || message.contains("is not a Business Rules operator")
            || message.contains("needs a value or a compare_field")
            || message.contains("group match must be")
            || message.contains("group needs at least one condition")
            || message.contains("must carry exactly one of condition or expression")
            || message.contains("must carry a condition or an expression")
            || message.contains("is not a sum comparison")
            || message.contains("needs one comparator")
            || message.contains("has a bad operand")
            || message.contains("only admits '+'")
            || message.contains("ends with '+'")
            || message.contains("has a bad right-hand side")
            || message.contains("takes no value")
        {
            "P-07"
        } else {
            "P-01"
        }
    };
    (id, step)
}

// --- field resolution (P-11 / P-15) -----------------------------------------

/// The fields a `$record` reference may name on `object_key`: the
/// built-in fields, the record's own `id`, and its active custom fields.
fn record_field_names(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
) -> AppResult<HashSet<String>> {
    let mut names: HashSet<String> = builtin_fields::builtin_fields_for(object_key)
        .iter()
        .map(|f| f.key.to_string())
        .collect();
    names.insert("id".to_string());
    for def in custom_field_service::list_definitions(conn, workspace_id, object_key, true)? {
        names.insert(def.key);
    }
    Ok(names)
}

/// Resolves a `repeat_over` collection name to the child object it names:
/// the relationship out of the anchor object whose key or forward label
/// is the collection name. `None` when no such relationship exists - the
/// caller reports that as a named failure, never a silent pass.
fn child_object_for_collection(
    conn: &Connection,
    workspace_id: &str,
    anchor_object_key: &str,
    collection: &str,
) -> AppResult<Option<String>> {
    for def in relationship_service::list(conn, workspace_id, true)? {
        if def.source_entity_type == anchor_object_key
            && (def.key == collection || def.forward_label == collection)
        {
            return Ok(Some(def.target_entity_type.clone()));
        }
    }
    Ok(None)
}

/// Every `repeat_over` collection in the lane, with the child object each
/// resolves to (or the named failure when one does not resolve).
fn lane_child_objects(
    conn: &Connection,
    workspace_id: &str,
    lane: &LaneDefinition,
) -> AppResult<HashMap<String, String>> {
    let mut out = HashMap::new();
    for stage in &lane.stages {
        for step in &stage.steps {
            if let Some(repeat) = &step.repeat_over {
                if !out.contains_key(&repeat.field) {
                    match child_object_for_collection(
                        conn,
                        workspace_id,
                        &lane.object_key,
                        &repeat.field,
                    )? {
                        Some(child) => {
                            out.insert(repeat.field.clone(), child);
                        }
                        None => {
                            return Err(classified_validation(
                                "validation.arguments",
                                format!(
                                    "step '{}' repeats over '{}', which names no relationship out of '{}'",
                                    step.id, repeat.field, lane.object_key
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

fn run_data_fields(lane: &LaneDefinition) -> HashMap<String, String> {
    lane.run_data
        .as_ref()
        .map(|r| {
            r.fields
                .iter()
                .map(|f| (f.key.clone(), f.field_type.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// One `$`-reference found in a definition, with where it was found.
struct RefUse {
    /// `run_data`, `record` or `item`.
    kind: String,
    /// The dotted path after the `$` (e.g. `lines[].qty_received`).
    path: String,
    step: Option<String>,
    context: String,
}

/// Splits a `$`-reference (or a bare `run_data.`/`record.` binding) into
/// its kind and path. Returns `None` for a literal.
fn split_ref(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix('$').unwrap_or(value);
    let mut parts = rest.splitn(2, '.');
    match (parts.next(), parts.next()) {
        (Some(kind @ ("record" | "run_data" | "item")), Some(path)) => {
            Some((kind.to_string(), path.to_string()))
        }
        _ => None,
    }
}

fn collect_refs_from_condition(
    condition: &ConditionOrGroup,
    step: Option<String>,
    context: &str,
    out: &mut Vec<RefUse>,
) {
    match condition {
        ConditionOrGroup::Condition(c) => {
            if let Some((kind, path)) = split_ref(&c.field) {
                out.push(RefUse {
                    kind,
                    path,
                    step: step.clone(),
                    context: context.to_string(),
                });
            }
            if let Some(other) = &c.compare_field {
                if let Some((kind, path)) = split_ref(other) {
                    out.push(RefUse {
                        kind,
                        path,
                        step: step.clone(),
                        context: context.to_string(),
                    });
                }
            }
        }
        ConditionOrGroup::Group(g) => {
            for inner in &g.conditions {
                collect_refs_from_condition(inner, step.clone(), context, out);
            }
        }
    }
}

/// The `$`-references in an arithmetic expression (`$a + $b <= $c`): every
/// whitespace-separated token that starts with `$`.
fn collect_refs_from_expression(
    expression: &str,
    step: Option<String>,
    context: &str,
    out: &mut Vec<RefUse>,
) {
    for token in expression.split_whitespace() {
        if let Some((kind, path)) = split_ref(token) {
            out.push(RefUse {
                kind,
                path,
                step: step.clone(),
                context: context.to_string(),
            });
        }
    }
}

fn collect_refs_from_rule(
    condition: &Option<ConditionOrGroup>,
    expression: &Option<String>,
    step: Option<String>,
    context: &str,
    out: &mut Vec<RefUse>,
) {
    if let Some(condition) = condition {
        collect_refs_from_condition(condition, step.clone(), context, out);
    }
    if let Some(expression) = expression {
        collect_refs_from_expression(expression, step, context, out);
    }
}

/// Every `$`-reference in the lane's steps: form-field bindings, write
/// inputs, check rules, validations, child-lane anchors.
fn collect_step_refs(lane: &LaneDefinition) -> Vec<RefUse> {
    let mut out = Vec::new();
    for stage in &lane.stages {
        for step in &stage.steps {
            let step_id = Some(step.id.clone());
            if let Some(fields) = &step.fields {
                for field in fields {
                    if let Some((kind, path)) = split_ref(&field.binds_to) {
                        out.push(RefUse {
                            kind,
                            path,
                            step: step_id.clone(),
                            context: format!("field '{}' binds to", field.key),
                        });
                    }
                    // Q-35 (builder gaps fix): a `resolve.where` value
                    // that is a `$`-reference resolves against the
                    // anchor / run data as usual. `resolve.field` is a
                    // field name on the resolve target object, not a
                    // reference, so it is never collected here.
                    if let Some(resolve) = &field.resolve {
                        if let Some(where_clause) = &resolve.where_clause {
                            for (key, value) in where_clause {
                                if let Value::String(s) = value {
                                    if let Some((kind, path)) = split_ref(s) {
                                        out.push(RefUse {
                                            kind,
                                            path,
                                            step: step_id.clone(),
                                            context: format!(
                                                "field '{}' resolve where '{key}'",
                                                field.key
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(writes) = &step.writes {
                for write in writes {
                    for (key, value) in &write.inputs {
                        if let Value::String(s) = value {
                            if let Some((kind, path)) = split_ref(s) {
                                out.push(RefUse {
                                    kind,
                                    path,
                                    step: step_id.clone(),
                                    context: format!("write input '{key}'"),
                                });
                            }
                        }
                    }
                }
            }
            if let Some(check) = &step.check {
                collect_refs_from_rule(
                    &check.condition,
                    &check.expression,
                    step_id.clone(),
                    "check",
                    &mut out,
                );
            }
            if let Some(validation) = &step.validation {
                collect_refs_from_rule(
                    &validation.condition,
                    &validation.expression,
                    step_id.clone(),
                    "validation",
                    &mut out,
                );
            }
            if let Some(child) = &step.child_lane {
                if let Some((kind, path)) = split_ref(&child.anchor_record) {
                    out.push(RefUse {
                        kind,
                        path,
                        step: step_id.clone(),
                        context: "child lane anchor".to_string(),
                    });
                }
            }
            // Q-34 (builder gaps fix): a skip filter's references resolve
            // like any other step condition ($item / $record / $run_data).
            if let Some(skip_when) = &step.skip_when {
                collect_refs_from_condition(skip_when, step_id.clone(), "skip_when", &mut out);
            }
        }
    }
    out
}

/// Resolves one `$`-reference against the declared fields. Returns the
/// named failure, or `None` when it resolves. `step_repeat` is the
/// repeating step's own collection (for `$item` inside that step);
/// `lane_children` maps every collection in the lane to its child object.
fn resolve_ref(
    conn: &Connection,
    workspace_id: &str,
    lane: &LaneDefinition,
    anchor_fields: &HashSet<String>,
    lane_children: &HashMap<String, String>,
    step_repeat: Option<&str>,
    r: &RefUse,
) -> AppResult<Option<String>> {
    let declared = run_data_fields(lane);
    match r.kind.as_str() {
        "run_data" => {
            let field = r.path.split('.').next().unwrap_or("");
            if declared.contains_key(field) {
                Ok(None)
            } else {
                Ok(Some(format!(
                    "step '{}' references undeclared run-data field '{field}' ({})",
                    r.step.as_deref().unwrap_or("?"),
                    r.context,
                )))
            }
        }
        "record" => {
            let segments: Vec<&str> = r.path.split('.').collect();
            if segments.len() == 1 {
                let field = segments[0].strip_suffix("[]").unwrap_or(segments[0]);
                if anchor_fields.contains(field) {
                    Ok(None)
                } else {
                    Ok(Some(format!(
                        "step '{}' references unknown record field '{field}' on '{}' ({})",
                        r.step.as_deref().unwrap_or("?"),
                        lane.object_key,
                        r.context
                    )))
                }
            } else {
                // `record.<collection>[].<field>`: the collection must
                // resolve to a child object, the field to one of its fields.
                let collection = segments[0].strip_suffix("[]").unwrap_or(segments[0]);
                let child_object = match lane_children.get(collection) {
                    Some(child) => child.clone(),
                    None => {
                        return Ok(Some(format!(
                            "step '{}' references unknown child collection '{collection}' on '{}' ({})",
                            r.step.as_deref().unwrap_or("?"),
                            lane.object_key,
                            r.context
                        )));
                    }
                };
                let child_fields = record_field_names(conn, workspace_id, &child_object)?;
                let field = segments[1].strip_suffix("[]").unwrap_or(segments[1]);
                if child_fields.contains(field) {
                    Ok(None)
                } else {
                    Ok(Some(format!(
                        "step '{}' references unknown child field '{field}'",
                        r.step.as_deref().unwrap_or("?"),
                    )))
                }
            }
        }
        "item" => {
            let field = r.path.split('.').next().unwrap_or("");
            // Inside a repeating step, `$item` is that step's children;
            // a per-item check reads the lane's repeat collections.
            let candidates: Vec<String> = match step_repeat {
                Some(collection) => lane_children.get(collection).cloned().into_iter().collect(),
                None => lane_children.values().cloned().collect(),
            };
            if candidates.is_empty() {
                return Ok(Some(format!(
                    "step '{}' references $item.{field}, but no step in the lane repeats over a child collection",
                    r.step.as_deref().unwrap_or("?"),
                )));
            }
            for child_object in &candidates {
                let child_fields = record_field_names(conn, workspace_id, child_object)?;
                if child_fields.contains(field) {
                    return Ok(None);
                }
            }
            Ok(Some(format!(
                "step '{}' references unknown child field '{field}'",
                r.step.as_deref().unwrap_or("?"),
            )))
        }
        _ => Ok(None),
    }
}

/// The repeating step a step belongs to, for `$item` resolution: its own
/// `repeat_over`, or `None` (a check step never repeats - it reads the
/// lane's collections).
fn step_repeat_field(lane: &LaneDefinition, step_id: &str) -> Option<String> {
    lane.stages
        .iter()
        .flat_map(|s| &s.steps)
        .find(|s| s.id == step_id)
        .and_then(|s| s.repeat_over.as_ref().map(|r| r.field.clone()))
}

/// Runs P-11 over a set of collected references, returning one refuse
/// entry per unresolvable reference.
fn check_refs(
    conn: &Connection,
    workspace_id: &str,
    lane: &LaneDefinition,
    anchor_fields: &HashSet<String>,
    lane_children: &HashMap<String, String>,
    refs: &[RefUse],
    check_id: &str,
) -> AppResult<Vec<PublishCheckEntry>> {
    let mut entries = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for r in refs {
        let step_repeat = r.step.as_deref().and_then(|id| step_repeat_field(lane, id));
        if let Some(reason) = resolve_ref(
            conn,
            workspace_id,
            lane,
            anchor_fields,
            lane_children,
            step_repeat.as_deref(),
            r,
        )? {
            let key = format!("{}|{}", r.step.as_deref().unwrap_or(""), reason);
            if seen.insert(key) {
                entries.push(refuse(check_id, r.step.clone(), reason));
            }
        }
    }
    Ok(entries)
}

// --- the publish checks -----------------------------------------------------

/// Runs P-01..P-16 in report mode over an already-parsed definition.
/// `has_draft` is whether a draft version exists to publish (P-16);
/// `version` is that draft's version number, for the report header. The
/// import path calls this with `has_draft = true`: the install itself
/// creates the draft version, so P-16 is the install's promise, not a
/// pre-condition.
pub fn check_publish_definition(
    conn: &Connection,
    workspace_id: &str,
    lane: &LaneDefinition,
    has_draft: bool,
    version: Option<i64>,
) -> AppResult<PublishReport> {
    let mut checks: Vec<PublishCheckEntry> = Vec::new();
    let validation = validate_definition(lane);
    match &validation {
        Ok(()) => {
            for id in [
                "P-01", "P-02", "P-03", "P-04", "P-05", "P-06", "P-07", "P-08",
            ] {
                checks.push(pass(id, None));
            }
            checks.push(pass_reason(
                "P-09",
                None,
                "reachability: every step reaches an end",
            ));
        }
        Err(e) => {
            let message = e.to_string();
            let (id, step) = classify_definition_error(&message);
            // Gap 9 (builder gaps fix): the P-09 halves are labelled, so
            // the two P-09 entries read as reachability vs outcome
            // coverage. The `validate_definition stopped at P-0Y`
            // substring stays intact inside any not_evaluated reason
            // (the frontend collapse parser depends on it).
            let message = if id == "P-09" {
                format!("reachability: {message}")
            } else {
                message
            };
            checks.push(refuse(id, step, message.clone()));
            for other in [
                "P-01", "P-02", "P-03", "P-04", "P-05", "P-06", "P-07", "P-08",
            ] {
                if other != id {
                    checks.push(not_evaluated(
                        other,
                        &format!("validate_definition stopped at {id} - {message}"),
                    ));
                }
            }
            if id != "P-09" {
                // The reachability half of P-09 never ran: the failure was
                // an earlier check.
                checks.push(warn(
                    "P-09",
                    None,
                    format!(
                        "reachability: not evaluated: validate_definition stopped at {id} - {message}"
                    ),
                ));
            }
        }
    }

    // P-09, outcome coverage: every check declares pass and fail, every
    // approval declares approve and reject, and every branch declares a
    // way out (a target step or an end outcome) - a branch with neither
    // falls through in the reachability walk, but it refuses publish
    // (WP-0714 §2, P-09).
    let mut outcome_failures = 0;
    for stage in &lane.stages {
        for step in &stage.steps {
            let branches = step.branches.clone().unwrap_or_default();
            for branch in &branches {
                if branch.target.is_none() && branch.end.is_none() {
                    checks.push(refuse(
                        "P-09",
                        Some(step.id.clone()),
                        format!(
                            "outcome coverage: branch '{}' of step '{}' has no exit — declare a target step or an end outcome",
                            branch.on, step.id
                        ),
                    ));
                    outcome_failures += 1;
                }
            }
            let has = |on: &str| branches.iter().any(|b| b.on == on);
            match step.kind.as_str() {
                "check" => {
                    for (on, missing) in [
                        ("pass", "has no pass branch"),
                        ("fail", "has no fail branch"),
                    ] {
                        if !has(on) {
                            checks.push(refuse(
                                "P-09",
                                Some(step.id.clone()),
                                format!("outcome coverage: check '{}' {missing}", step.id),
                            ));
                            outcome_failures += 1;
                        }
                    }
                }
                "approval" => {
                    for (on, missing) in [
                        ("approved", "has no approve branch"),
                        ("rejected", "has no reject branch"),
                    ] {
                        if !has(on) {
                            checks.push(refuse(
                                "P-09",
                                Some(step.id.clone()),
                                format!("outcome coverage: approval '{}' {missing}", step.id),
                            ));
                            outcome_failures += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if outcome_failures == 0 {
        checks.push(pass_reason(
            "P-09",
            None,
            "outcome coverage: every check declares pass and fail; every approval declares approve and reject",
        ));
    }

    // P-10 (WARN): a check step that is not the last step of its stage.
    let mut p10_warns = 0;
    for stage in &lane.stages {
        for (i, step) in stage.steps.iter().enumerate() {
            if step.kind == "check" && i + 1 < stage.steps.len() {
                checks.push(warn(
                    "P-10",
                    Some(step.id.clone()),
                    format!(
                        "check '{}' is not the last step of stage '{}' - the step after it runs on the pass path; move the check to the end of the stage, or confirm the next step is the pass continuation",
                        step.id, stage.id
                    ),
                ));
                p10_warns += 1;
            }
        }
    }
    if p10_warns == 0 {
        checks.push(pass("P-10", None));
    }

    // P-11 / P-15 need the anchor's fields and the lane's child objects.
    // When a repeat collection cannot resolve to a relationship, that is
    // itself a named P-11 failure, not a reason to stop the report.
    let anchor_fields = record_field_names(conn, workspace_id, &lane.object_key)?;
    let lane_children = match lane_child_objects(conn, workspace_id, lane) {
        Ok(children) => children,
        Err(e) => {
            checks.push(refuse("P-11", None, e.to_string()));
            HashMap::new()
        }
    };
    let step_refs = collect_step_refs(lane);
    let p11 = check_refs(
        conn,
        workspace_id,
        lane,
        &anchor_fields,
        &lane_children,
        &step_refs,
        "P-11",
    )?;
    if p11.is_empty() {
        checks.push(pass("P-11", None));
    } else {
        checks.extend(p11);
    }

    // P-12: every write's Action Type must be adopted - an active
    // `action_type_definitions` row with this name on the anchor object.
    let mut p12_failures = 0;
    for stage in &lane.stages {
        for step in &stage.steps {
            if let Some(writes) = &step.writes {
                for write in writes {
                    let adopted = action_type_service::get_by_name(
                        conn,
                        workspace_id,
                        &lane.object_key,
                        &write.action_type,
                    )?
                    .map(|d| d.is_active)
                    .unwrap_or(false);
                    if !adopted {
                        checks.push(refuse(
                            "P-12",
                            Some(step.id.clone()),
                            format!(
                                "step '{}' uses provisional Action Type '{}' - not in the adopted catalog",
                                step.id, write.action_type
                            ),
                        ));
                        p12_failures += 1;
                    }
                }
            }
        }
    }
    if p12_failures == 0 {
        checks.push(pass("P-12", None));
    }

    // P-13: every `replaces` entry names a real workflow id and a real
    // action index on it (looked up by id, no name fallback).
    let mut p13_failures = 0;
    for stage in &lane.stages {
        for step in &stage.steps {
            if let Some(replaces) = &step.replaces {
                for r in replaces {
                    match workflow_service::get_rule(conn, &r.workflow_id)? {
                        None => {
                            checks.push(refuse(
                                "P-13",
                                Some(step.id.clone()),
                                format!(
                                    "step '{}' replaces unknown workflow '{}'",
                                    step.id, r.workflow_id
                                ),
                            ));
                            p13_failures += 1;
                        }
                        Some(workflow) => {
                            if r.action_index < 0
                                || r.action_index as usize >= workflow.actions.len()
                            {
                                checks.push(refuse(
                                    "P-13",
                                    Some(step.id.clone()),
                                    format!(
                                        "step '{}' replaces action {} of workflow '{}', which has no such action",
                                        step.id, r.action_index, r.workflow_id
                                    ),
                                ));
                                p13_failures += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    if p13_failures == 0 {
        checks.push(pass("P-13", None));
    }

    // P-14 (WARN): a run-data field used as a number must be declared a
    // number; a form field bound to a run-data field must match its
    // declared type. Record-bound uses are not judged: the core stores
    // custom-field values as strings, so there is no declared record
    // type to compare against.
    let declared = run_data_fields(lane);
    let mut p14_warns: Vec<PublishCheckEntry> = Vec::new();
    let mut warn_seen: HashSet<String> = HashSet::new();
    // A numeric use of a run-data field: a numeric comparison, or any
    // reference inside an arithmetic expression. `context` names the use
    // the way the spec's reason does ("by step '<id>'" / "by the start
    // condition" / "by stage '<id>'").
    let numeric_use_warn = |field: &str,
                            step: Option<String>,
                            context: &str,
                            out: &mut Vec<PublishCheckEntry>,
                            seen: &mut HashSet<String>| {
        let Some(declared_type) = declared.get(field) else {
            return;
        };
        if matches!(declared_type.as_str(), "integer" | "number") {
            return;
        }
        let entry = warn(
            "P-14",
            step,
            format!("run-data field '{field}' is '{declared_type}' but is used as 'number' by {context}"),
        );
        let key = format!(
            "{}|{}",
            entry.step.as_deref().unwrap_or(""),
            entry.reason.as_deref().unwrap_or("")
        );
        if seen.insert(key) {
            out.push(entry);
        }
    };
    // Scans one rule (condition and/or expression) for numeric uses of
    // run-data fields.
    let scan_rule = |condition: &Option<ConditionOrGroup>,
                     expression: &Option<String>,
                     step: Option<String>,
                     context: String,
                     out: &mut Vec<PublishCheckEntry>,
                     seen: &mut HashSet<String>| {
        if expression.is_some() {
            let mut refs = Vec::new();
            collect_refs_from_expression(
                expression.as_deref().unwrap_or(""),
                step.clone(),
                "expression",
                &mut refs,
            );
            for r in refs {
                if r.kind == "run_data" {
                    let field = r.path.split('.').next().unwrap_or("").to_string();
                    numeric_use_warn(&field, step.clone(), &context, out, seen);
                }
            }
        }
        if let Some(condition) = condition {
            let mut leaves = Vec::new();
            collect_condition_leaves(condition, &mut leaves);
            for (field_ref, operator) in leaves {
                if !matches!(operator.as_str(), "greater_than" | "less_than") {
                    continue;
                }
                if let Some((kind, path)) = split_ref(&field_ref) {
                    if kind != "run_data" {
                        continue;
                    }
                    let field = path.split('.').next().unwrap_or("").to_string();
                    numeric_use_warn(&field, step.clone(), &context, out, seen);
                }
            }
        }
    };
    for stage in &lane.stages {
        for step in &stage.steps {
            let context = format!("step '{}'", step.id);
            if let Some(check) = &step.check {
                scan_rule(
                    &check.condition,
                    &check.expression,
                    Some(step.id.clone()),
                    context.clone(),
                    &mut p14_warns,
                    &mut warn_seen,
                );
            }
            if let Some(validation) = &step.validation {
                scan_rule(
                    &validation.condition,
                    &validation.expression,
                    Some(step.id.clone()),
                    context.clone(),
                    &mut p14_warns,
                    &mut warn_seen,
                );
            }
            // A form field bound to a run-data field must match its type.
            if let Some(fields) = &step.fields {
                for field in fields {
                    if let Some(target) = field.binds_to.strip_prefix("run_data.") {
                        if let Some(declared_type) = declared.get(target) {
                            let compatible = field.field_type == *declared_type
                                || (matches!(field.field_type.as_str(), "integer" | "number")
                                    && matches!(declared_type.as_str(), "integer" | "number"));
                            if !compatible {
                                let entry = warn(
                                    "P-14",
                                    Some(step.id.clone()),
                                    format!(
                                        "run-data field '{target}' is '{declared_type}' but is used as '{}' by step '{}'",
                                        field.field_type, step.id
                                    ),
                                );
                                let key = format!(
                                    "{}|{}",
                                    entry.step.as_deref().unwrap_or(""),
                                    entry.reason.as_deref().unwrap_or("")
                                );
                                if warn_seen.insert(key) {
                                    p14_warns.push(entry);
                                }
                            }
                        }
                    }
                }
            }
        }
        for condition in [&stage.entry_condition, &stage.skip_condition]
            .into_iter()
            .flatten()
        {
            scan_rule(
                &Some(condition.clone()),
                &None,
                None,
                format!("stage '{}'", stage.id),
                &mut p14_warns,
                &mut warn_seen,
            );
        }
    }
    if let Some(start) = &lane.start_condition {
        scan_rule(
            &Some(start.clone()),
            &None,
            None,
            "the start condition".to_string(),
            &mut p14_warns,
            &mut warn_seen,
        );
    }
    if p14_warns.is_empty() {
        checks.push(pass("P-14", None));
    } else {
        checks.extend(p14_warns);
    }

    // P-15: the start condition may only reference declared run-data
    // fields and anchor-object fields (the P-11 rule, named separately).
    if let Some(start) = &lane.start_condition {
        let mut refs = Vec::new();
        collect_refs_from_condition(start, None, "start_condition", &mut refs);
        let mut p15 = check_refs(
            conn,
            workspace_id,
            lane,
            &anchor_fields,
            &lane_children,
            &refs,
            "P-15",
        )?;
        if p15.is_empty() {
            checks.push(pass("P-15", None));
        } else {
            // P-15 reasons name the start condition, not a step.
            for entry in &mut p15 {
                if let Some(reason) = &entry.reason {
                    entry.reason = Some(reason.replacen("step '?'", "start_condition", 1));
                }
                entry.step = None;
            }
            checks.extend(p15);
        }
    } else {
        checks.push(pass("P-15", None));
    }

    // P-16: only a draft publishes.
    if has_draft {
        checks.push(pass("P-16", None));
    } else {
        checks.push(refuse(
            "P-16",
            None,
            format!("lane '{}' has no draft version to publish", lane.id),
        ));
    }

    let refused = checks.iter().any(|c| c.status == "refuse");
    // Gap 9 (builder gaps fix): when the lane's object does not exist
    // in this workspace, frame the P-11/P-13 refusals below as tracing
    // to the missing object, not to broken steps. The checks themselves
    // stay as they are. Known means a core entity type or a dynamic
    // entity type - the same lookup P-11's resolver uses.
    let object_known = entity_registry::CORE_ENTITY_TYPES.contains(&lane.object_key.as_str())
        || custom_object_service::is_valid_dynamic_entity_type(
            conn,
            workspace_id,
            &lane.object_key,
        )
        .unwrap_or(false);
    let framing = if object_known {
        None
    } else {
        Some(format!(
            "This lane's object '{}' doesn't exist in this workspace — the P-11/P-13 refusals below trace to the missing object, not to broken steps.",
            lane.object_key
        ))
    };
    Ok(PublishReport {
        lane_key: lane.id.clone(),
        version,
        refused,
        checks,
        framing,
    })
}

fn collect_condition_leaves(condition: &ConditionOrGroup, out: &mut Vec<(String, String)>) {
    match condition {
        ConditionOrGroup::Condition(c) => {
            out.push((c.field.clone(), c.operator.clone()));
        }
        ConditionOrGroup::Group(g) => {
            for inner in &g.conditions {
                collect_condition_leaves(inner, out);
            }
        }
    }
}

/// The publish report for a stored lane: the latest draft version when
/// there is one, else a report that refuses P-16 and marks the checks
/// that need a definition as not evaluated.
pub fn check_publish(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    actor_user_id: Option<&str>,
) -> AppResult<PublishReport> {
    require_admin(conn, actor_user_id)?;
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let draft = process_lane_repo::latest_version(conn, &lane.id)?.filter(|v| v.status == "draft");
    match draft {
        Some(draft) => {
            let file = parse_definition_lenient(&draft.definition_json)?;
            check_publish_definition(conn, workspace_id, &file.lane, true, Some(draft.version))
        }
        None => {
            let mut checks = vec![refuse(
                "P-16",
                None,
                format!("lane '{lane_key}' has no draft version to publish"),
            )];
            for id in [
                "P-01", "P-02", "P-03", "P-04", "P-05", "P-06", "P-07", "P-08", "P-09", "P-10",
                "P-11", "P-12", "P-13", "P-14", "P-15",
            ] {
                checks.push(not_evaluated(id, "lane has no draft version to check"));
            }
            Ok(PublishReport {
                lane_key: lane_key.to_string(),
                version: None,
                refused: true,
                checks,
                framing: None,
            })
        }
    }
}

// --- dry run (WP-0714 §3) ---------------------------------------------------

/// The fixtures a dry run walks with (WP-0714 §3): the values a person
/// would type - run-data fields, per-child values keyed by child record
/// id and leaf field name, approval decisions by step id, and any other
/// form values by step id and field key. A check that depends on a
/// fixture the author did not supply reports `unevaluated (needs
/// fixture)` instead of guessing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DryRunFixtures {
    #[serde(default)]
    pub run_data: Map<String, Value>,
    #[serde(default)]
    pub per_child: HashMap<String, Map<String, Value>>,
    #[serde(default)]
    pub approvals: HashMap<String, String>,
    #[serde(default)]
    pub form_values: HashMap<String, Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunStep {
    pub stage_id: String,
    pub step_id: String,
    pub kind: String,
    pub outcome: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunCheckChild {
    pub child_id: String,
    pub outcome: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunCheck {
    pub step_id: String,
    pub outcome: String,
    pub reason: Option<String>,
    #[serde(default)]
    pub per_child: Vec<DryRunCheckChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunBranch {
    pub step_id: String,
    pub on: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunWrite {
    pub step_id: String,
    pub child_id: Option<String>,
    pub action_type: String,
    pub inputs: Map<String, Value>,
    pub zero_quantity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunZeroQuantity {
    pub step_id: String,
    pub action_type: String,
    pub skipped_children: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunUnevaluated {
    pub step_id: String,
    pub reason: String,
}

/// The dry-run report (WP-0714 §3): the path walked, the checks, the
/// branches taken, the writes simulated (never posted), the zero-quantity
/// flags, and the end outcome against the author's expectation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunReport {
    pub lane_key: String,
    pub lane_version: i64,
    pub anchor_object_key: String,
    pub anchor_record_id: String,
    pub would_start: bool,
    pub refusal_reason: Option<String>,
    pub stages_visited: Vec<String>,
    pub steps: Vec<DryRunStep>,
    pub checks: Vec<DryRunCheck>,
    pub branches_taken: Vec<DryRunBranch>,
    pub writes: Vec<DryRunWrite>,
    pub zero_quantity_flags: Vec<DryRunZeroQuantity>,
    pub unevaluated: Vec<DryRunUnevaluated>,
    pub end_outcome: Option<String>,
    pub expected_outcome: Option<String>,
    pub matches_expected: bool,
    pub completed: bool,
    pub stopped_reason: Option<String>,
}

/// One live value lookup: present (with its string form), missing (a
/// fixture the author did not supply), or unknown (a field the lane
/// should have declared - P-11's territory, reported here as
/// unevaluated, never guessed).
enum RefLookup {
    Present(String),
    Missing(String),
    Unknown(String),
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// The live state a dry run evaluates against: the seed record, the
/// fixtures, and the values earlier simulated steps wrote.
struct DryRunState {
    record_fields: HashMap<String, String>,
    run_data: Map<String, Value>,
    /// Per-child run-data values set by simulated steps, by child id.
    sim_per_child: HashMap<String, Map<String, Value>>,
    /// The lane's child records, by repeat collection.
    children: HashMap<String, Vec<DryChild>>,
    anchor_fields: HashSet<String>,
    run_data_keys: HashSet<String>,
    child_fields: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
struct DryChild {
    id: String,
    object_key: String,
    fields: HashMap<String, String>,
}

/// What a `$`-reference resolves to in the live dry-run state. `item`
/// is the child the reference is evaluated for, if any.
fn lookup_ref(
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    kind: &str,
    path: &str,
    item: Option<&DryChild>,
) -> RefLookup {
    match kind {
        "record" => {
            let segments: Vec<&str> = path.split('.').collect();
            if segments.len() == 1 {
                let field = segments[0].strip_suffix("[]").unwrap_or(segments[0]);
                if !state.anchor_fields.contains(field) {
                    return RefLookup::Unknown(format!(
                        "references unknown record field '${field}'"
                    ));
                }
                RefLookup::Present(state.record_fields.get(field).cloned().unwrap_or_default())
            } else {
                RefLookup::Unknown(format!(
                    "references '${path}', a collection - it cannot be evaluated as one value"
                ))
            }
        }
        "run_data" => {
            let field = path.split('.').next().unwrap_or("");
            if !state.run_data_keys.contains(field) {
                return RefLookup::Unknown(format!(
                    "references undeclared run-data field '${field}'"
                ));
            }
            if let Some(child) = item {
                if let Some(values) = fixtures.per_child.get(&child.id) {
                    if let Some(value) = values.get(field) {
                        return RefLookup::Present(value_to_string(value));
                    }
                }
                if let Some(values) = state.sim_per_child.get(&child.id) {
                    if let Some(value) = values.get(field) {
                        return RefLookup::Present(value_to_string(value));
                    }
                }
            }
            match state.run_data.get(field) {
                Some(value) => RefLookup::Present(value_to_string(value)),
                None => RefLookup::Missing(format!("needs fixture: run_data.{field}")),
            }
        }
        "item" => {
            let Some(child) = item else {
                return RefLookup::Missing(format!(
                    "references $item.{path}, but there is no repeating step here"
                ));
            };
            let field = path.split('.').next().unwrap_or("");
            let known = state
                .child_fields
                .get(&child.object_key)
                .map(|f| f.contains(field))
                .unwrap_or(false);
            if !known {
                return RefLookup::Unknown(format!("references unknown child field '${field}'"));
            }
            // A per-child fixture (or a value an earlier simulated step
            // wrote for this child) shadows the seed record's value - the
            // same precedence the `$run_data` arm uses.
            if let Some(values) = fixtures.per_child.get(&child.id) {
                if let Some(value) = values.get(field) {
                    return RefLookup::Present(value_to_string(value));
                }
            }
            if let Some(values) = state.sim_per_child.get(&child.id) {
                if let Some(value) = values.get(field) {
                    return RefLookup::Present(value_to_string(value));
                }
            }
            RefLookup::Present(child.fields.get(field).cloned().unwrap_or_default())
        }
        _ => RefLookup::Unknown(format!("references unknown source '${kind}'")),
    }
}

/// The field key the shared condition matcher reads a `$`-reference
/// under: `$record.<f>` as the bare name, `$run_data.<f>` as
/// `run_data.<f>`, `$item.<f>` as `item.<f>`.
fn ref_field_key(kind: &str, path: &str) -> Option<String> {
    let field = path.split('.').next().unwrap_or("");
    match kind {
        "record" => Some(field.to_string()),
        "run_data" => Some(format!("run_data.{field}")),
        "item" => Some(format!("item.{field}")),
        _ => None,
    }
}

/// Evaluates a lane condition against the live state. `Err` is the
/// unevaluated reason - a missing fixture or an unknown field - never a
/// guess.
fn eval_condition(
    rule: &ConditionOrGroup,
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    item: Option<&DryChild>,
) -> Result<bool, String> {
    match rule {
        ConditionOrGroup::Condition(c) => {
            if !CONDITION_OPERATORS.contains(&c.operator.as_str()) {
                return Err(format!("unknown condition operator '{}'", c.operator));
            }
            let (kind, path) = split_ref(&c.field)
                .ok_or_else(|| format!("condition field '{}' is not a $ reference", c.field))?;
            let field_key = ref_field_key(&kind, &path)
                .ok_or_else(|| format!("condition field '{}' is not a $ reference", c.field))?;
            let actual = match lookup_ref(state, fixtures, &kind, &path, item) {
                RefLookup::Present(v) => v,
                RefLookup::Missing(reason) => return Err(reason),
                RefLookup::Unknown(reason) => return Err(reason),
            };
            let value = match &c.compare_field {
                Some(other) => {
                    let (other_kind, other_path) = split_ref(other)
                        .ok_or_else(|| format!("compare_field '{other}' is not a $ reference"))?;
                    match lookup_ref(state, fixtures, &other_kind, &other_path, item) {
                        RefLookup::Present(v) => v,
                        RefLookup::Missing(reason) => return Err(reason),
                        RefLookup::Unknown(reason) => return Err(reason),
                    }
                }
                None => match &c.value {
                    Some(v) => value_to_string(v),
                    None => {
                        if matches!(c.operator.as_str(), "is_empty" | "is_not_empty") {
                            String::new()
                        } else {
                            return Err(format!(
                                "condition on '{}' has neither a value nor a compare_field",
                                c.field
                            ));
                        }
                    }
                },
            };
            let mut ctx = HashMap::new();
            ctx.insert(field_key.clone(), actual);
            Ok(condition_matches(&field_key, &c.operator, &value, &ctx))
        }
        ConditionOrGroup::Group(g) => {
            let mut results = Vec::new();
            for child in &g.conditions {
                results.push(eval_condition(child, state, fixtures, item)?);
            }
            Ok(match g.match_type.as_str() {
                "any" => results.iter().any(|r| *r),
                _ => results.iter().all(|r| *r),
            })
        }
    }
}

/// Evaluates the narrow sum-comparison expression grammar (ADR-0024 §5)
/// against the live state: a sum of `$`-references and numbers, one
/// comparator, one `$`-reference or number.
fn eval_expression(
    expression: &str,
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    item: Option<&DryChild>,
) -> Result<bool, String> {
    let parts: Vec<&str> = expression.split_whitespace().collect();
    let comparator_at = parts
        .iter()
        .position(|p| matches!(*p, "<=" | "<" | ">=" | ">" | "==" | "!="))
        .ok_or_else(|| format!("expression '{expression}' needs one comparator"))?;
    let resolve_operand = |token: &str| -> Result<f64, String> {
        if let Some((kind, path)) = split_ref(token) {
            match lookup_ref(state, fixtures, &kind, &path, item) {
                RefLookup::Present(v) => v.parse::<f64>().map_err(|_| {
                    format!("'{token}' is '{v}', not a number, in expression '{expression}'")
                }),
                RefLookup::Missing(reason) => Err(reason),
                RefLookup::Unknown(reason) => Err(reason),
            }
        } else {
            token
                .parse::<f64>()
                .map_err(|_| format!("'{token}' is not a number in expression '{expression}'"))
        }
    };
    let mut lhs = 0.0;
    for token in &parts[..comparator_at] {
        if *token == "+" {
            continue;
        }
        lhs += resolve_operand(token)?;
    }
    let rhs = resolve_operand(parts[comparator_at + 1])?;
    let comparator = parts[comparator_at];
    Ok(match comparator {
        "<=" => lhs <= rhs,
        "<" => lhs < rhs,
        ">=" => lhs >= rhs,
        ">" => lhs > rhs,
        "==" => (lhs - rhs).abs() < f64::EPSILON,
        "!=" => (lhs - rhs).abs() >= f64::EPSILON,
        _ => unreachable!(),
    })
}

/// Evaluates a check rule or a step validation (exactly one of condition
/// or expression) against the live state.
fn eval_rule(
    condition: &Option<ConditionOrGroup>,
    expression: &Option<String>,
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    item: Option<&DryChild>,
) -> Result<bool, String> {
    match (condition, expression) {
        (Some(condition), None) => eval_condition(condition, state, fixtures, item),
        (None, Some(expression)) => eval_expression(expression, state, fixtures, item),
        _ => Err("the rule carries neither a condition nor an expression".to_string()),
    }
}

/// Resolves one write input against the live state. A literal passes
/// through; a `$` reference resolves to its string value.
fn resolve_write_input(
    value: &Value,
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    item: Option<&DryChild>,
) -> Result<Value, String> {
    match value {
        Value::String(s) => match split_ref(s) {
            Some((kind, path)) => match lookup_ref(state, fixtures, &kind, &path, item) {
                RefLookup::Present(v) => Ok(Value::String(v)),
                RefLookup::Missing(reason) => Err(reason),
                RefLookup::Unknown(reason) => Err(reason),
            },
            None => Ok(value.clone()),
        },
        other => Ok(other.clone()),
    }
}

fn load_record_fields(
    conn: &Connection,
    object_key: &str,
    record_id: &str,
) -> AppResult<HashMap<String, String>> {
    let mut fields = builtin_field_service::field_values(conn, object_key, record_id)?;
    for (key, value) in custom_field_repo::get_values(conn, record_id)? {
        fields.insert(key, value);
    }
    fields.insert("id".to_string(), record_id.to_string());
    Ok(fields)
}

/// Walks a lane against a seed record and writes nothing (WP-0714 §3).
/// The walk uses the lane's latest version (draft when one exists - the
/// thing being authored - else the latest published one) and evaluates
/// every condition against live values at evaluation time: the seed
/// record, the fixtures, and the values earlier simulated steps wrote.
pub fn dry_run_lane(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    anchor_record_id: &str,
    fixtures: &DryRunFixtures,
    expected_outcome: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<DryRunReport> {
    require_admin(conn, actor_user_id)?;
    let lane_row = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let version_row = process_lane_repo::latest_version(conn, &lane_row.id)?.ok_or_else(|| {
        classified_validation(
            "validation.arguments",
            format!("lane '{lane_key}' has no version to dry-run"),
        )
    })?;
    let file = parse_definition_lenient(&version_row.definition_json)?;
    let lane = file.lane;
    dry_run_definition(
        conn,
        workspace_id,
        &lane,
        version_row.version,
        anchor_record_id,
        fixtures,
        expected_outcome,
        actor_user_id,
    )
}

/// The definition-level dry-run walk (WP-0718, REQ-LANE-024): the same
/// walk `dry_run_lane` runs, but over an already-parsed definition
/// instead of a stored lane - so the visual builder can dry-run an
/// unsaved draft. The stored-lane wrapper above is the only other
/// caller; the walk itself is shared, never duplicated.
pub fn dry_run_definition(
    conn: &Connection,
    workspace_id: &str,
    lane: &LaneDefinition,
    lane_version: i64,
    anchor_record_id: &str,
    fixtures: &DryRunFixtures,
    expected_outcome: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<DryRunReport> {
    require_admin(conn, actor_user_id)?;
    let record = custom_record_service::get(conn, anchor_record_id)?;
    if record.workspace_id != workspace_id || record.object_key != lane.object_key {
        return Err(classified_validation(
            "validation.arguments",
            "the anchor record is not a record of this lane's object",
        ));
    }

    // The live state: the seed record's fields, the fixtures' run data,
    // and one child list per repeat collection in the lane.
    let anchor_fields = record_field_names(conn, workspace_id, &lane.object_key)?;
    let run_data_keys: HashSet<String> = run_data_fields(lane).keys().cloned().collect();
    let mut children: HashMap<String, Vec<DryChild>> = HashMap::new();
    let mut child_fields: HashMap<String, HashSet<String>> = HashMap::new();
    let mut collections: Vec<String> = Vec::new();
    for stage in &lane.stages {
        for step in &stage.steps {
            if let Some(repeat) = &step.repeat_over {
                if !collections.contains(&repeat.field) {
                    collections.push(repeat.field.clone());
                }
            }
        }
    }
    let mut state = DryRunState {
        record_fields: load_record_fields(conn, &lane.object_key, anchor_record_id)?,
        run_data: fixtures.run_data.clone(),
        sim_per_child: HashMap::new(),
        children: HashMap::new(),
        anchor_fields,
        run_data_keys,
        child_fields: HashMap::new(),
    };
    for collection in &collections {
        let defs = relationship_service::list(conn, workspace_id, true)?;
        let def = defs
            .iter()
            .find(|d| {
                d.source_entity_type == lane.object_key
                    && (d.key == *collection || d.forward_label == *collection)
            })
            .ok_or_else(|| {
                classified_validation(
                    "validation.arguments",
                    format!(
                        "lane repeats over '{collection}', which names no relationship out of '{}'",
                        lane.object_key
                    ),
                )
            })?;
        let child_object = def.target_entity_type.clone();
        child_fields.entry(child_object.clone()).or_insert_with(|| {
            record_field_names(conn, workspace_id, &child_object).unwrap_or_default()
        });
        let mut kids = Vec::new();
        for related in relationship_service::related_records_for(
            conn,
            workspace_id,
            &lane.object_key,
            anchor_record_id,
        )? {
            if related.relationship_key != def.key {
                continue;
            }
            kids.push(DryChild {
                id: related.entity_id.clone(),
                object_key: related.entity_type.clone(),
                fields: load_record_fields(conn, &related.entity_type, &related.entity_id)?,
            });
        }
        children.insert(collection.clone(), kids);
    }
    state.children = children;
    state.child_fields = child_fields;

    let mut report = DryRunReport {
        lane_key: lane.id.clone(),
        lane_version,
        anchor_object_key: lane.object_key.clone(),
        anchor_record_id: anchor_record_id.to_string(),
        would_start: true,
        refusal_reason: None,
        stages_visited: Vec::new(),
        steps: Vec::new(),
        checks: Vec::new(),
        branches_taken: Vec::new(),
        writes: Vec::new(),
        zero_quantity_flags: Vec::new(),
        unevaluated: Vec::new(),
        end_outcome: None,
        expected_outcome: expected_outcome.map(String::from),
        matches_expected: true,
        completed: false,
        stopped_reason: None,
    };

    // The start condition gates the whole walk.
    if let Some(start) = &lane.start_condition {
        match eval_condition(start, &state, fixtures, None) {
            Ok(true) => {}
            Ok(false) => {
                report.would_start = false;
                report.refusal_reason =
                    Some("would refuse to start, start condition is not met".to_string());
                return Ok(report);
            }
            Err(reason) => {
                report.would_start = false;
                report.refusal_reason = Some(format!(
                    "would refuse to start, start condition is unevaluated ({reason})"
                ));
                return Ok(report);
            }
        }
    }

    // The flat walk: definition order, with branch jumps. A jump forward
    // marks the steps it passes as skipped, with the branch that skipped
    // them; a jump back re-walks steps (a hold branch returning to an
    // earlier step). The walk is bounded by LANE_REPEAT_MAX_ITERATIONS.
    let flat: Vec<(usize, usize)> = lane
        .stages
        .iter()
        .enumerate()
        .flat_map(|(si, s)| s.steps.iter().enumerate().map(move |(ti, _)| (si, ti)))
        .collect();
    let find_flat = |stage: &str, step: &str| -> Option<usize> {
        flat.iter().position(|&(si, ti)| {
            lane.stages[si].id == stage && lane.stages[si].steps[ti].id == step
        })
    };
    let mut pos = 0usize;
    let mut visited: i64 = 0;
    while pos < flat.len() {
        visited += 1;
        if visited > LANE_REPEAT_MAX_ITERATIONS {
            report.stopped_reason = Some(format!(
                "walk exceeded {LANE_REPEAT_MAX_ITERATIONS} steps - a branch may loop forever"
            ));
            break;
        }
        let (stage_idx, step_idx) = flat[pos];
        let stage = &lane.stages[stage_idx];
        let step = &stage.steps[step_idx];
        if !report.stages_visited.contains(&stage.id) {
            report.stages_visited.push(stage.id.clone());
        }

        // Stage gates run when the walk reaches a stage's first step.
        if step_idx == 0 {
            let mut skip_stage: Option<String> = None;
            if let Some(entry) = &stage.entry_condition {
                match eval_condition(entry, &state, fixtures, None) {
                    Ok(true) => {}
                    Ok(false) => skip_stage = Some("stage entry condition not met".to_string()),
                    Err(reason) => {
                        report.unevaluated.push(DryRunUnevaluated {
                            step_id: step.id.clone(),
                            reason: format!("stage entry condition is unevaluated ({reason})"),
                        });
                        report.stopped_reason =
                            Some("stage entry condition could not be evaluated".to_string());
                        break;
                    }
                }
            }
            if skip_stage.is_none() {
                if let Some(skip) = &stage.skip_condition {
                    match eval_condition(skip, &state, fixtures, None) {
                        Ok(true) => skip_stage = Some("stage skip condition met".to_string()),
                        Ok(false) => {}
                        Err(reason) => {
                            report.unevaluated.push(DryRunUnevaluated {
                                step_id: step.id.clone(),
                                reason: format!("stage skip condition is unevaluated ({reason})"),
                            });
                            report.stopped_reason =
                                Some("stage skip condition could not be evaluated".to_string());
                            break;
                        }
                    }
                }
            }
            if let Some(reason) = skip_stage {
                for s in &stage.steps {
                    report.steps.push(DryRunStep {
                        stage_id: stage.id.clone(),
                        step_id: s.id.clone(),
                        kind: s.kind.clone(),
                        outcome: "skipped".to_string(),
                        reason: Some(reason.clone()),
                    });
                }
                report.stages_visited.retain(|s| s != &stage.id);
                pos = flat
                    .iter()
                    .position(|&(si, _)| si > stage_idx)
                    .unwrap_or(flat.len());
                continue;
            }
        }

        let mut next_pos: Option<usize> = None;
        let mut end_outcome: Option<String> = None;
        match step.kind.as_str() {
            "form" => {
                match simulate_form_step(&mut state, fixtures, lane, stage, step)? {
                    FormOutcome::Executed => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "executed".to_string(),
                            reason: None,
                        });
                    }
                    FormOutcome::Unevaluated(reason) => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "unevaluated".to_string(),
                            reason: Some(reason.clone()),
                        });
                        report.unevaluated.push(DryRunUnevaluated {
                            step_id: step.id.clone(),
                            reason,
                        });
                        report.stopped_reason =
                            Some("a form step could not be simulated".to_string());
                        break;
                    }
                    FormOutcome::Failed(reason) => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "failed".to_string(),
                            reason: Some(reason.clone()),
                        });
                        report.stopped_reason = Some(reason);
                        break;
                    }
                }
                if let Some(ends) = &step.ends_run {
                    end_outcome = Some(ends.outcome.clone());
                }
            }
            "record" => {
                match simulate_record_step(&mut state, fixtures, lane, stage, step, &mut report)? {
                    RecordOutcome::Executed => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "executed".to_string(),
                            reason: None,
                        });
                    }
                    RecordOutcome::Failed(reason) => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "failed".to_string(),
                            reason: Some(reason.clone()),
                        });
                        report.stopped_reason = Some(reason);
                        break;
                    }
                }
                if let Some(ends) = &step.ends_run {
                    end_outcome = Some(ends.outcome.clone());
                }
            }
            "check" => {
                let rule = step
                    .check
                    .as_ref()
                    .expect("validated: check steps have a check");
                let (outcome, reason, per_child) =
                    evaluate_check(&state, fixtures, rule, &collections);
                match outcome.as_str() {
                    "unevaluated" => {
                        report.checks.push(DryRunCheck {
                            step_id: step.id.clone(),
                            outcome: outcome.clone(),
                            reason: reason.clone(),
                            per_child,
                        });
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "unevaluated".to_string(),
                            reason: reason.clone(),
                        });
                        if let Some(reason) = reason {
                            report.unevaluated.push(DryRunUnevaluated {
                                step_id: step.id.clone(),
                                reason,
                            });
                        }
                        report.stopped_reason = Some("a check could not be evaluated".to_string());
                        break;
                    }
                    _ => {
                        report.checks.push(DryRunCheck {
                            step_id: step.id.clone(),
                            outcome: outcome.clone(),
                            reason: reason.clone(),
                            per_child,
                        });
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "executed".to_string(),
                            reason: reason.clone(),
                        });
                        let branches = step.branches.clone().unwrap_or_default();
                        // The check outcome is "passed"/"failed"; the
                        // branch it selects is "pass"/"fail".
                        let branch_on = if outcome == "passed" { "pass" } else { "fail" };
                        if let Some(branch) = branches.iter().find(|b| b.on == branch_on) {
                            if let Some(target) = &branch.target {
                                report.branches_taken.push(DryRunBranch {
                                    step_id: step.id.clone(),
                                    on: branch_on.to_string(),
                                    target: format!("{}/{}", target.stage, target.step),
                                });
                                next_pos = find_flat(&target.stage, &target.step);
                            } else if let Some(end) = &branch.end {
                                report.branches_taken.push(DryRunBranch {
                                    step_id: step.id.clone(),
                                    on: branch_on.to_string(),
                                    target: format!("end:{}", end.outcome),
                                });
                                end_outcome = Some(end.outcome.clone());
                            }
                        }
                    }
                }
            }
            "approval" => {
                match simulate_approval_step(&mut state, fixtures, lane, stage, step, &mut report)?
                {
                    ApprovalOutcome::Executed { decision, target } => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "executed".to_string(),
                            reason: Some(format!("approval decision: {decision}")),
                        });
                        match target {
                            BranchTarget::Step {
                                stage,
                                step: target_step,
                            } => {
                                report.branches_taken.push(DryRunBranch {
                                    step_id: step.id.clone(),
                                    on: decision.clone(),
                                    target: format!("{stage}/{target_step}"),
                                });
                                next_pos = find_flat(&stage, &target_step);
                            }
                            BranchTarget::End { outcome } => {
                                report.branches_taken.push(DryRunBranch {
                                    step_id: step.id.clone(),
                                    on: decision.clone(),
                                    target: format!("end:{outcome}"),
                                });
                                end_outcome = Some(outcome);
                            }
                            BranchTarget::Linear => {}
                        }
                    }
                    ApprovalOutcome::Unevaluated(reason) => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "unevaluated".to_string(),
                            reason: Some(reason.clone()),
                        });
                        report.unevaluated.push(DryRunUnevaluated {
                            step_id: step.id.clone(),
                            reason,
                        });
                        report.stopped_reason =
                            Some("an approval has no fixture decision".to_string());
                        break;
                    }
                    ApprovalOutcome::Failed(reason) => {
                        report.steps.push(DryRunStep {
                            stage_id: stage.id.clone(),
                            step_id: step.id.clone(),
                            kind: step.kind.clone(),
                            outcome: "failed".to_string(),
                            reason: Some(reason.clone()),
                        });
                        report.stopped_reason = Some(reason);
                        break;
                    }
                }
            }
            "agent" => {
                // An agent step cannot be simulated: no agent runs, and
                // its outputs are unavailable, so a later check that reads
                // them reports unevaluated. The walk continues linearly.
                report.steps.push(DryRunStep {
                    stage_id: stage.id.clone(),
                    step_id: step.id.clone(),
                    kind: step.kind.clone(),
                    outcome: "executed".to_string(),
                    reason: Some(
                        "agent step simulated as completed without invoking the agent; its outputs are unavailable".to_string(),
                    ),
                });
                if let Some(ends) = &step.ends_run {
                    end_outcome = Some(ends.outcome.clone());
                }
            }
            _ => {
                report.stopped_reason = Some(format!("unknown step kind '{}'", step.kind));
                break;
            }
        }
        if let Some(outcome) = end_outcome {
            report.end_outcome = Some(outcome);
            report.completed = true;
            break;
        }
        match next_pos {
            Some(target) => {
                // A forward jump marks the steps it passes as skipped.
                if target > pos + 1 {
                    for &(si, ti) in flat.iter().take(target).skip(pos + 1) {
                        let skipped_step = &lane.stages[si].steps[ti];
                        report.steps.push(DryRunStep {
                            stage_id: lane.stages[si].id.clone(),
                            step_id: skipped_step.id.clone(),
                            kind: skipped_step.kind.clone(),
                            outcome: "skipped".to_string(),
                            reason: Some(format!(
                                "branch of step '{}' jumped to '{}'",
                                step.id,
                                report
                                    .branches_taken
                                    .last()
                                    .map(|b| b.target.clone())
                                    .unwrap_or_default()
                            )),
                        });
                    }
                }
                pos = target;
            }
            None => {
                pos += 1;
            }
        }
    }
    if report.end_outcome.is_none() && report.stopped_reason.is_none() && pos >= flat.len() {
        report.end_outcome = Some("completed".to_string());
        report.completed = true;
    }
    // Zero-quantity flags: one per step, counting the simulated rows whose
    // quantity resolved to zero.
    let mut by_step: HashMap<String, (String, i64)> = HashMap::new();
    for write in &report.writes {
        if write.zero_quantity {
            let slot = by_step
                .entry(write.step_id.clone())
                .or_insert_with(|| (write.action_type.clone(), 0));
            slot.1 += 1;
        }
    }
    let mut flags: Vec<DryRunZeroQuantity> = by_step
        .into_iter()
        .map(|(step_id, (action_type, n))| DryRunZeroQuantity {
            step_id,
            action_type,
            skipped_children: n,
            message: format!("would skip {n} children with nothing to post"),
        })
        .collect();
    flags.sort_by(|a, b| a.step_id.cmp(&b.step_id));
    report.zero_quantity_flags = flags;
    if let Some(expected) = &report.expected_outcome {
        report.matches_expected = report.end_outcome.as_deref() == Some(expected.as_str());
    }
    Ok(report)
}

enum BranchTarget {
    Step { stage: String, step: String },
    End { outcome: String },
    Linear,
}

enum FormOutcome {
    Executed,
    Unevaluated(String),
    Failed(String),
}

enum RecordOutcome {
    Executed,
    Failed(String),
}

enum ApprovalOutcome {
    Executed {
        decision: String,
        target: BranchTarget,
    },
    Unevaluated(String),
    Failed(String),
}

/// The children a per-item check evaluates over: the lane's one repeat
/// collection. A lane repeating over several collections cannot pick one,
/// so its per-item checks report unevaluated instead of guessing.
fn check_children<'a>(
    state: &'a DryRunState,
    collections: &[String],
) -> Result<&'a Vec<DryChild>, String> {
    if collections.len() != 1 {
        return Err(
            "the lane repeats over several collections, so a per-item check cannot pick one"
                .to_string(),
        );
    }
    state
        .children
        .get(&collections[0])
        .ok_or_else(|| "the repeat collection has no children loaded".to_string())
}

fn evaluate_check(
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    rule: &crate::models::process_lane::CheckRule,
    collections: &[String],
) -> (String, Option<String>, Vec<DryRunCheckChild>) {
    match rule.scope.as_str() {
        "per_item" => {
            let children = match check_children(state, collections) {
                Ok(children) => children,
                Err(reason) => return ("unevaluated".to_string(), Some(reason), Vec::new()),
            };
            let mut per_child = Vec::new();
            let mut results = Vec::new();
            for child in children {
                match eval_rule(
                    &rule.condition,
                    &rule.expression,
                    state,
                    fixtures,
                    Some(child),
                ) {
                    Ok(passed) => {
                        results.push(passed);
                        per_child.push(DryRunCheckChild {
                            child_id: child.id.clone(),
                            outcome: if passed {
                                "passed".to_string()
                            } else {
                                "failed".to_string()
                            },
                            reason: None,
                        });
                    }
                    Err(reason) => {
                        per_child.push(DryRunCheckChild {
                            child_id: child.id.clone(),
                            outcome: "unevaluated".to_string(),
                            reason: Some(reason.clone()),
                        });
                        return (
                            "unevaluated".to_string(),
                            Some(format!(
                                "check depends on a fixture or value that is not available for child {} ({reason})",
                                child.id
                            )),
                            per_child,
                        );
                    }
                }
            }
            let passed = match rule.aggregate.as_deref() {
                Some("any") => results.iter().any(|r| *r),
                _ => results.iter().all(|r| *r),
            };
            (
                if passed {
                    "passed".to_string()
                } else {
                    "failed".to_string()
                },
                if passed {
                    None
                } else {
                    Some(rule.message.clone())
                },
                per_child,
            )
        }
        _ => match eval_rule(&rule.condition, &rule.expression, state, fixtures, None) {
            Ok(true) => ("passed".to_string(), None, Vec::new()),
            Ok(false) => ("failed".to_string(), Some(rule.message.clone()), Vec::new()),
            Err(reason) => ("unevaluated".to_string(), Some(reason), Vec::new()),
        },
    }
}

/// Simulates a form step: resolves every declared field from the fixtures
/// (and the values earlier steps wrote), stores the entered values in the
/// live state, and evaluates the step's validation. A required field with
/// no fixture value stops the walk as unevaluated - the dry run never
/// invents a value a person would type.
fn simulate_form_step(
    state: &mut DryRunState,
    fixtures: &DryRunFixtures,
    lane: &LaneDefinition,
    _stage: &crate::models::process_lane::LaneStage,
    step: &crate::models::process_lane::LaneStep,
) -> AppResult<FormOutcome> {
    let declared = run_data_fields(lane);
    let per_child_keys: HashSet<&str> = lane
        .run_data
        .as_ref()
        .map(|r| {
            r.fields
                .iter()
                .filter(|f| f.per_child)
                .map(|f| f.key.as_str())
                .collect()
        })
        .unwrap_or_default();
    let fields = step.fields.clone().unwrap_or_default();
    let repeat_collection = step.repeat_over.as_ref().map(|r| r.field.clone());
    // The children this step runs over, if it repeats.
    let children: Vec<DryChild> = match &repeat_collection {
        Some(collection) => state
            .children
            .get(collection)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|c| DryChild {
                id: c.id,
                object_key: c.object_key,
                fields: c.fields,
            })
            .collect(),
        None => Vec::new(),
    };
    let rounds: Vec<Option<DryChild>> = if step.repeat_over.is_some() {
        children.into_iter().map(Some).collect()
    } else {
        vec![None]
    };
    for round_child in &rounds {
        for field in &fields {
            let value = form_field_value(
                state,
                fixtures,
                step,
                field,
                round_child.as_ref(),
                &per_child_keys,
            );
            let Some(value) = value else {
                if field.required {
                    return Ok(FormOutcome::Unevaluated(format!(
                        "needs fixture: field '{}' of step '{}'",
                        field.key, step.id
                    )));
                }
                continue;
            };
            apply_form_value(
                state,
                lane,
                step,
                field,
                round_child.as_ref(),
                &value,
                &declared,
            );
        }
        // The step's validation runs against the values just entered.
        if let Some(validation) = &step.validation {
            match eval_rule(
                &validation.condition,
                &validation.expression,
                state,
                fixtures,
                round_child.as_ref(),
            ) {
                Ok(true) => {}
                Ok(false) => return Ok(FormOutcome::Failed(validation.message.clone())),
                Err(reason) => {
                    return Ok(FormOutcome::Unevaluated(format!(
                        "validation of step '{}' is unevaluated ({reason})",
                        step.id
                    )))
                }
            }
        }
    }
    Ok(FormOutcome::Executed)
}

/// Resolves one form field's value for one round (one child, or the
/// anchor round). Fixture precedence: an explicit `form_values` entry,
/// then the per-child fixture, then the run data the walk has collected.
fn form_field_value(
    state: &DryRunState,
    fixtures: &DryRunFixtures,
    step: &crate::models::process_lane::LaneStep,
    field: &crate::models::process_lane::FormField,
    child: Option<&DryChild>,
    per_child_keys: &HashSet<&str>,
) -> Option<Value> {
    if let Some(values) = fixtures.form_values.get(&step.id) {
        if let Some(value) = values.get(&field.key) {
            return Some(value.clone());
        }
    }
    match field.binds_to.strip_prefix("run_data.") {
        Some(key) => {
            if let Some(child) = child {
                if per_child_keys.contains(key) {
                    if let Some(values) = fixtures.per_child.get(&child.id) {
                        if let Some(value) = values.get(key) {
                            return Some(value.clone());
                        }
                    }
                    if let Some(values) = state.sim_per_child.get(&child.id) {
                        if let Some(value) = values.get(key) {
                            return Some(value.clone());
                        }
                    }
                    return None;
                }
            }
            state.run_data.get(key).cloned()
        }
        None => match field.binds_to.strip_prefix("record.") {
            Some(path) => {
                let segments: Vec<&str> = path.split('.').collect();
                if segments.len() == 1 {
                    // A record-bound field with no fixture keeps the
                    // record's current value - the person left it alone.
                    state
                        .record_fields
                        .get(segments[0])
                        .map(|v| Value::String(v.clone()))
                } else if let Some(child) = child {
                    let leaf = segments[1].strip_suffix("[]").unwrap_or(segments[1]);
                    if let Some(values) = fixtures.per_child.get(&child.id) {
                        if let Some(value) = values.get(leaf) {
                            return Some(value.clone());
                        }
                        if let Some(value) = values.get(&field.key) {
                            return Some(value.clone());
                        }
                    }
                    child.fields.get(leaf).map(|v| Value::String(v.clone()))
                } else {
                    None
                }
            }
            None => None,
        },
    }
}

/// Stores an entered form value in the live state, where later steps
/// (and checks) read it.
fn apply_form_value(
    state: &mut DryRunState,
    _lane: &LaneDefinition,
    step: &crate::models::process_lane::LaneStep,
    field: &crate::models::process_lane::FormField,
    child: Option<&DryChild>,
    value: &Value,
    _declared: &HashMap<String, String>,
) {
    let text = value_to_string(value);
    match field.binds_to.strip_prefix("run_data.") {
        Some(key) => {
            if let Some(child) = child {
                if step.repeat_over.is_some()
                    && _lane
                        .run_data
                        .as_ref()
                        .map(|r| r.fields.iter().any(|f| f.key == key && f.per_child))
                        .unwrap_or(false)
                {
                    state
                        .sim_per_child
                        .entry(child.id.clone())
                        .or_default()
                        .insert(key.to_string(), value.clone());
                    return;
                }
            }
            state.run_data.insert(key.to_string(), value.clone());
        }
        None => {
            if let Some(path) = field.binds_to.strip_prefix("record.") {
                let segments: Vec<&str> = path.split('.').collect();
                if segments.len() == 1 {
                    state.record_fields.insert(segments[0].to_string(), text);
                } else if let Some(child) = child {
                    let leaf = segments[1].strip_suffix("[]").unwrap_or(segments[1]);
                    if let Some(collection) = step.repeat_over.as_ref().map(|r| r.field.clone()) {
                        if let Some(kids) = state.children.get_mut(&collection) {
                            if let Some(kid) = kids.iter_mut().find(|k| k.id == child.id) {
                                kid.fields.insert(leaf.to_string(), text);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Simulates a record step: resolves every write's inputs against the
/// live state and records the exact writes it would post, without
/// posting anything. A write whose `$` reference does not resolve fails
/// the step. The demo's provisional `run_data.set` write is applied to
/// the simulated run data, so a later check that reads the flag the
/// approval set can evaluate - it is a simulation-only effect.
fn simulate_record_step(
    state: &mut DryRunState,
    fixtures: &DryRunFixtures,
    _lane: &LaneDefinition,
    stage: &crate::models::process_lane::LaneStage,
    step: &crate::models::process_lane::LaneStep,
    report: &mut DryRunReport,
) -> AppResult<RecordOutcome> {
    let _ = stage;
    let writes = step.writes.clone().unwrap_or_default();
    let children: Vec<Option<DryChild>> = match &step.repeat_over {
        Some(repeat) => state
            .children
            .get(&repeat.field)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(Some)
            .collect(),
        None => vec![None],
    };
    for child in &children {
        for write in &writes {
            let mut inputs: Map<String, Value> = Map::new();
            for (key, raw) in &write.inputs {
                match resolve_write_input(raw, state, fixtures, child.as_ref()) {
                    Ok(resolved) => {
                        inputs.insert(key.clone(), resolved);
                    }
                    Err(reason) => {
                        return Ok(RecordOutcome::Failed(format!(
                            "write input '{key}' references a value that does not resolve ({reason})"
                        )));
                    }
                }
            }
            let zero_quantity = inputs
                .get("quantity")
                .and_then(|v| {
                    v.as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64())
                })
                .map(|q| q == 0.0)
                .unwrap_or(false);
            report.writes.push(DryRunWrite {
                step_id: step.id.clone(),
                child_id: child.as_ref().map(|c| c.id.clone()),
                action_type: write.action_type.clone(),
                inputs: inputs.clone(),
                zero_quantity,
            });
            if write.action_type == "run_data.set" {
                if let (Some(Value::String(key)), Some(value)) =
                    (inputs.get("key"), inputs.get("value"))
                {
                    state.run_data.insert(key.clone(), value.clone());
                }
            }
        }
    }
    Ok(RecordOutcome::Executed)
}

/// Simulates an approval step: the decision comes from the fixtures, and
/// the step's writes are simulated exactly like a record step's. A
/// missing fixture decision stops the walk as unevaluated.
fn simulate_approval_step(
    state: &mut DryRunState,
    fixtures: &DryRunFixtures,
    lane: &LaneDefinition,
    stage: &crate::models::process_lane::LaneStage,
    step: &crate::models::process_lane::LaneStep,
    report: &mut DryRunReport,
) -> AppResult<ApprovalOutcome> {
    let Some(decision) = fixtures.approvals.get(&step.id) else {
        return Ok(ApprovalOutcome::Unevaluated(format!(
            "needs fixture: approval decision for step '{}'",
            step.id
        )));
    };
    if decision != "approved" && decision != "rejected" {
        return Ok(ApprovalOutcome::Failed(format!(
            "approval fixture for step '{}' must be 'approved' or 'rejected', not '{decision}'",
            step.id
        )));
    }
    // The approval's writes are simulated (and reported) like a record
    // step's, before the branch is followed.
    match simulate_record_step(state, fixtures, lane, stage, step, report)? {
        RecordOutcome::Executed => {}
        RecordOutcome::Failed(reason) => return Ok(ApprovalOutcome::Failed(reason)),
    }
    let branches = step.branches.clone().unwrap_or_default();
    let branch = branches.iter().find(|b| b.on == *decision);
    let target = match branch {
        Some(branch) => {
            if let Some(t) = &branch.target {
                BranchTarget::Step {
                    stage: t.stage.clone(),
                    step: t.step.clone(),
                }
            } else if let Some(end) = &branch.end {
                BranchTarget::End {
                    outcome: end.outcome.clone(),
                }
            } else {
                BranchTarget::Linear
            }
        }
        None => BranchTarget::Linear,
    };
    Ok(ApprovalOutcome::Executed {
        decision: decision.clone(),
        target,
    })
}

// --- lane cases (WP-0714 §4) ---------------------------------------------

/// One stored lane case: the case JSON verbatim, with the lane version it
/// tests and the name from the case itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneCaseRecord {
    pub id: String,
    pub lane_key: String,
    pub version: i64,
    pub name: String,
    pub case_json: String,
}

/// One lane-case assertion's evidence, as the runner observed it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseCheck {
    pub step_id: String,
    pub outcome: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseWrite {
    pub step_id: String,
    pub action_type: String,
    pub count: i64,
}

/// The result of running one lane case (WP-0714 §4): whether every
/// assertion held, each failure with expected vs actual, and the
/// evidence the assertions read. The run itself is always rolled back,
/// pass or fail, so a case leaves no trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneCaseResult {
    pub case: String,
    pub lane: String,
    pub version: i64,
    pub passed: bool,
    pub failures: Vec<String>,
    pub end_outcome: Option<String>,
    pub steps_completed: Vec<String>,
    pub steps_skipped: Vec<String>,
    pub checks: Vec<CaseCheck>,
    pub writes_posted: Vec<CaseWrite>,
}

/// One lane's import result (WP-0714 §5): a version imported as a new
/// draft, or skipped with the reason (it already exists here), plus the
/// publish checks in report mode for the imported version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLaneImportEntry {
    pub lane_key: String,
    pub version: i64,
    pub status: String,
    pub reason: String,
    pub checks: Vec<PublishCheckEntry>,
}

fn parse_case(case_json: &str) -> AppResult<Value> {
    let case: Value = serde_json::from_str(case_json).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("lane case is not valid JSON: {e}"),
        )
    })?;
    for key in ["case", "lane", "version", "anchor", "fixtures", "assert"] {
        if case.get(key).is_none() {
            return Err(classified_validation(
                "validation.arguments",
                format!("lane case is missing '{key}'"),
            ));
        }
    }
    if !case["assert"].is_array() {
        return Err(classified_validation(
            "validation.arguments",
            "lane case 'assert' must be an array",
        ));
    }
    Ok(case)
}

/// Stores a lane case with the lane version it tests. The case names its
/// lane and version; both must match the stored-against lane and version.
/// Re-storing a case of the same name replaces it.
pub fn store_lane_case(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    version: i64,
    case_json: &str,
    actor_user_id: Option<&str>,
) -> AppResult<LaneCaseRecord> {
    require_admin(conn, actor_user_id)?;
    let case = parse_case(case_json)?;
    let name = case["case"]
        .as_str()
        .ok_or_else(|| {
            classified_validation("validation.arguments", "lane case 'case' must be a string")
        })?
        .to_string();
    if case["lane"].as_str() != Some(lane_key) {
        return Err(classified_validation(
            "validation.arguments",
            "the case's lane does not match the lane it is stored against",
        ));
    }
    if case["version"].as_i64() != Some(version) {
        return Err(classified_validation(
            "validation.arguments",
            "the case's version does not match the version it is stored against",
        ));
    }
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    if process_lane_repo::get_version(conn, &lane.id, version)?.is_none() {
        return Err(classified_validation(
            "validation.arguments",
            format!("lane '{lane_key}' has no version {version} to attach a case to"),
        ));
    }
    let id = crate::domain::ids::new_uuid();
    process_lane_repo::upsert_case(
        conn,
        &id,
        &lane.id,
        workspace_id,
        version,
        &name,
        case_json,
        actor_user_id,
    )?;
    Ok(LaneCaseRecord {
        id,
        lane_key: lane_key.to_string(),
        version,
        name,
        case_json: case_json.to_string(),
    })
}

/// The cases stored against one lane (all versions, or one version).
pub fn list_lane_cases(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    version: Option<i64>,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<LaneCaseRecord>> {
    require_admin(conn, actor_user_id)?;
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let rows = match version {
        Some(version) => process_lane_repo::list_cases_for_version(conn, &lane.id, version)?
            .into_iter()
            .map(|(id, name, case_json)| (id, version, name, case_json))
            .collect(),
        None => process_lane_repo::list_cases_for_lane(conn, &lane.id)?,
    };
    Ok(rows
        .into_iter()
        .map(|(id, version, name, case_json)| LaneCaseRecord {
            id,
            lane_key: lane_key.to_string(),
            version,
            name,
            case_json,
        })
        .collect())
}

/// Every stored lane case in the workspace - what the Unified Test Run
/// discovers (WP-0714 §4).
pub fn discover_lane_cases(
    conn: &Connection,
    workspace_id: &str,
) -> AppResult<Vec<LaneCaseRecord>> {
    let lanes = process_lane_repo::list_lanes(conn, workspace_id)?;
    let by_id: HashMap<String, String> = lanes
        .iter()
        .map(|l| (l.id.clone(), l.lane_key.clone()))
        .collect();
    let mut out = Vec::new();
    for (id, lane_id, version, name, case_json) in
        process_lane_repo::list_all_cases(conn, workspace_id)?
    {
        if let Some(lane_key) = by_id.get(&lane_id) {
            out.push(LaneCaseRecord {
                id,
                lane_key: lane_key.clone(),
                version,
                name,
                case_json,
            });
        }
    }
    Ok(out)
}

/// Runs every stored case in the workspace and returns one result per
/// case (WP-0714 §4). A case that cannot run (its lane or version is
/// gone, its version is not published) reports a failed result with the
/// named reason instead of aborting the run.
pub async fn run_all_lane_cases(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor_user_id: Option<&str>,
) -> AppResult<Vec<LaneCaseResult>> {
    let mut results = Vec::new();
    for record in discover_lane_cases(conn, workspace_id)? {
        match run_lane_case(
            conn,
            workspace_id,
            master_key,
            &record.case_json,
            actor_user_id,
        )
        .await
        {
            Ok(result) => results.push(result),
            Err(e) => results.push(LaneCaseResult {
                case: record.name.clone(),
                lane: record.lane_key.clone(),
                version: record.version,
                passed: false,
                failures: vec![format!("case could not run: {e}")],
                end_outcome: None,
                steps_completed: Vec::new(),
                steps_skipped: Vec::new(),
                checks: Vec::new(),
                writes_posted: Vec::new(),
            }),
        }
    }
    Ok(results)
}

/// Runs one lane case for real against the seeded workspace, in a
/// savepoint that is always rolled back, so the case leaves no trace:
/// no run, no posted write, no changed record (WP-0714 §4). Form steps
/// are driven with the fixture values, record and check steps run
/// through the real runner, and an approval step takes its decision
/// from `fixtures.approvals[step_id]` - an approval reached without a
/// fixture decision fails the case with a named reason.
///
/// The case's version must be published: a draft has no compiled graph
/// to run. The run outcome is the `ends_run` outcome of the step or
/// branch that ended the run, or `completed` when the run reaches the
/// end node with no named outcome.
pub async fn run_lane_case(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    case_json: &str,
    actor_user_id: Option<&str>,
) -> AppResult<LaneCaseResult> {
    let case = parse_case(case_json)?;
    let case_name = case["case"].as_str().unwrap_or("").to_string();
    let lane_key = case["lane"].as_str().unwrap_or("").to_string();
    let version = case["version"].as_i64().unwrap_or(0);
    let mut result = LaneCaseResult {
        case: case_name.clone(),
        lane: lane_key.clone(),
        version,
        passed: false,
        failures: Vec::new(),
        end_outcome: None,
        steps_completed: Vec::new(),
        steps_skipped: Vec::new(),
        checks: Vec::new(),
        writes_posted: Vec::new(),
    };
    // A savepoint, not a transaction: the caller may already hold one
    // (an install runs inside `unchecked_transaction`), and a savepoint
    // rolls back cleanly inside it. Every exit below - pass, fail or
    // error - rolls back before returning.
    conn.execute("SAVEPOINT lane_case_run", [])
        .map_err(AppError::from)?;
    let drive = drive_case(
        conn,
        workspace_id,
        master_key,
        &case,
        actor_user_id,
        &mut result,
    )
    .await;
    conn.execute("ROLLBACK TO SAVEPOINT lane_case_run", [])
        .map_err(AppError::from)?;
    conn.execute("RELEASE SAVEPOINT lane_case_run", [])
        .map_err(AppError::from)?;
    drive?;
    Ok(result)
}

/// Drives one case inside the caller's savepoint: starts the run, walks
/// it to completion through the real runner, collects the evidence, and
/// evaluates the assertions. The savepoint rollback is the caller's job.
async fn drive_case(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    case: &Value,
    actor_user_id: Option<&str>,
    result: &mut LaneCaseResult,
) -> AppResult<()> {
    let lane_key = case["lane"].as_str().unwrap_or("");
    let version = case["version"].as_i64().unwrap_or(0);
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let version_row =
        process_lane_repo::get_version(conn, &lane.id, version)?.ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                format!("lane '{lane_key}' has no version {version}"),
            )
        })?;
    if version_row.status != "published" {
        result.failures.push(format!(
            "version {version} of lane '{lane_key}' is '{}', not published - a case runs the published version",
            version_row.status
        ));
        return Ok(());
    }
    let definition = parse_definition(&version_row.definition_json)?.lane;
    let anchor = &case["anchor"];
    let anchor_object = anchor["object"].as_str().unwrap_or("");
    let anchor_id = anchor["id"].as_str().unwrap_or("");
    if anchor_object != lane.object_key {
        result.failures.push(format!(
            "the case anchors on '{anchor_object}', but lane '{lane_key}' is on '{}'",
            lane.object_key
        ));
        return Ok(());
    }
    let fixtures = case["fixtures"].clone();
    let fixture_run_data: Map<String, Value> = fixtures
        .get("run_data")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let fixture_form: HashMap<String, Map<String, Value>> = fixtures
        .get("form_values")
        .and_then(|v| v.as_object())
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_object().cloned().map(|m| (k.clone(), m)))
                .collect()
        })
        .unwrap_or_default();
    let approvals: Map<String, Value> = fixtures
        .get("approvals")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let mut run = match process_lane_service::start_lane_run(
        conn,
        workspace_id,
        master_key,
        lane_key,
        anchor_id,
        actor_user_id,
    )
    .await
    {
        Ok(run) => run,
        Err(e) => {
            result
                .failures
                .push(format!("the run could not start: {e}"));
            return Ok(());
        }
    };

    // Drive the run to a terminal status. Each iteration reads the live
    // run (the runner advances it), acts on the step it waits on, and
    // records the branch outcome of any check or approval it passes.
    let mut end_outcome: Option<String> = None;
    let mut iterations = 0;
    loop {
        iterations += 1;
        if iterations > LANE_REPEAT_MAX_ITERATIONS as usize {
            result
                .failures
                .push("the run did not finish within the step bound".to_string());
            return Ok(());
        }
        let live = crate::repositories::graph_run_repo::get_run(conn, &run.id)?
            .ok_or_else(|| AppError::NotFound("Run".into()))?;
        match live.status.as_str() {
            "completed" | "completed_partial" => {
                break;
            }
            "failed" | "cancelled" => {
                result.failures.push(format!(
                    "the run ended '{}': {}",
                    live.status,
                    live.error_message.as_deref().unwrap_or("")
                ));
                return Ok(());
            }
            "waiting_person" => {
                let view = process_lane_service::get_run_view(
                    conn,
                    workspace_id,
                    &live.id,
                    actor_user_id,
                )?;
                let Some(current) = view.current else {
                    result
                        .failures
                        .push("the run waits on a person but has no current step".to_string());
                    return Ok(());
                };
                let step = definition
                    .stages
                    .iter()
                    .flat_map(|s| &s.steps)
                    .find(|s| s.id == current.step_id)
                    .ok_or_else(|| {
                        classified_validation(
                            "validation.state",
                            format!(
                                "current step '{}' is not in the definition",
                                current.step_id
                            ),
                        )
                    })?;
                let mut values: Map<String, Value> = Map::new();
                let mut missing: Vec<String> = Vec::new();
                for field in current.fields.iter() {
                    let value = fixture_form
                        .get(&current.step_id)
                        .and_then(|m| m.get(&field.key).cloned())
                        .or_else(|| {
                            field
                                .binds_to
                                .strip_prefix("run_data.")
                                .and_then(|k| fixture_run_data.get(k).cloned())
                        });
                    match value {
                        Some(value) => {
                            values.insert(field.key.clone(), value);
                        }
                        None => {
                            if field.required {
                                missing.push(field.key.clone());
                            }
                        }
                    }
                }
                if !missing.is_empty() {
                    result.failures.push(format!(
                        "step '{}' needs fixture values for required fields: {}",
                        current.step_id,
                        missing.join(", ")
                    ));
                    return Ok(());
                }
                let _ = step;
                match process_lane_service::submit_person_step(
                    conn,
                    workspace_id,
                    master_key,
                    &live.id,
                    &values,
                    actor_user_id,
                )
                .await
                {
                    Ok(next) => run = next,
                    Err(e) => {
                        result
                            .failures
                            .push(format!("submitting step '{}' failed: {e}", current.step_id));
                        return Ok(());
                    }
                }
                // A form step that ends the run names its outcome.
                if let Some(ends) = step_ends_run(&definition, &current.step_id) {
                    end_outcome = Some(ends);
                }
            }
            "waiting_approval" => {
                let view = process_lane_service::get_run_view(
                    conn,
                    workspace_id,
                    &live.id,
                    actor_user_id,
                )?;
                let Some(waiting) = view.waiting else {
                    result
                        .failures
                        .push("the run waits on an approval but has no inbox state".to_string());
                    return Ok(());
                };
                let Some(decision) = approvals.get(&waiting.step_id).and_then(|v| v.as_str())
                else {
                    result.failures.push(format!(
                        "approval '{}' was reached without a fixture decision - add fixtures.approvals['{}'] = 'approved' or 'rejected'",
                        waiting.step_id, waiting.step_id
                    ));
                    return Ok(());
                };
                let approve = match decision {
                    "approved" => true,
                    "rejected" => false,
                    other => {
                        result.failures.push(format!(
                            "approval fixture for step '{}' must be 'approved' or 'rejected', not '{other}'",
                            waiting.step_id
                        ));
                        return Ok(());
                    }
                };
                // The branch the decision takes, for the end outcome.
                if let Some(step) = definition
                    .stages
                    .iter()
                    .flat_map(|s| &s.steps)
                    .find(|s| s.id == waiting.step_id)
                {
                    let on = if approve { "approved" } else { "rejected" };
                    if let Some(branch) = step
                        .branches
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .find(|b| b.on == on)
                    {
                        if let Some(end) = &branch.end {
                            end_outcome = Some(end.outcome.clone());
                        }
                    }
                }
                match process_lane_service::resolve_lane_approval(
                    conn,
                    workspace_id,
                    master_key,
                    &live.id,
                    approve,
                    None,
                    actor_user_id,
                )
                .await
                {
                    Ok(next) => run = next,
                    Err(e) => {
                        result.failures.push(format!(
                            "resolving approval '{}' failed: {e}",
                            waiting.step_id
                        ));
                        return Ok(());
                    }
                }
            }
            other => {
                result
                    .failures
                    .push(format!("the run is in an unexpected status '{other}'"));
                return Ok(());
            }
        }
        // A check that completed on this advance records its branch
        // outcome in the run's end-outcome tracking below (read after the
        // loop from the run nodes).
    }

    // Evidence, read before the caller rolls back: the step list, the
    // check outcomes, the writes posted, the run data and the anchor
    // record's end state.
    let run = crate::repositories::graph_run_repo::get_run(conn, &run.id)?
        .ok_or_else(|| AppError::NotFound("Run".into()))?;
    let view = process_lane_service::get_run_view(conn, workspace_id, &run.id, actor_user_id)?;
    for step in &view.steps {
        match step.status.as_str() {
            "completed" => result.steps_completed.push(step.step_id.clone()),
            "skipped" => result.steps_skipped.push(step.step_id.clone()),
            _ => {}
        }
    }
    // Check outcomes come from the condition nodes' outputs
    // (`{"result": bool, "reason"?}`), keyed back to their lane step.
    let graph = crate::repositories::execution_graph_repo::get(conn, &run.graph_id)?
        .ok_or_else(|| AppError::NotFound("Execution graph".into()))?;
    let mut node_step: HashMap<String, String> = HashMap::new();
    for node in &graph.nodes {
        if node.node_type == "condition" {
            if let Ok(Value::Object(config)) = serde_json::from_str::<Value>(&node.config_json) {
                if let Some(Value::String(step_id)) = config.get("lane_step_id") {
                    node_step.insert(node.id.clone(), step_id.clone());
                }
            }
        }
    }
    for row in crate::repositories::graph_run_repo::list_run_nodes(conn, &run.id)? {
        if row.node_type != "condition" || row.status != "completed" {
            continue;
        }
        let Some(step_id) = node_step.get(&row.node_id) else {
            continue;
        };
        let output: Value = row
            .output_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(Value::Null);
        let passed = output
            .get("result")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        result.checks.push(CaseCheck {
            step_id: step_id.clone(),
            outcome: if passed {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            reason: output
                .get("reason")
                .and_then(|v| v.as_str())
                .map(String::from),
        });
        // A check branch that ends the run names its outcome; the
        // definition lookup below resolves it from the branch taken.
        if end_outcome.is_none() {
            let on = if passed { "pass" } else { "fail" };
            if let Some(step) = definition
                .stages
                .iter()
                .flat_map(|s| &s.steps)
                .find(|s| &s.id == step_id)
            {
                if let Some(branch) = step
                    .branches
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .find(|b| b.on == on)
                {
                    if let Some(end) = &branch.end {
                        end_outcome = Some(end.outcome.clone());
                    }
                }
            }
        }
    }
    // Writes posted: the completed action nodes' `applied` lists.
    let mut action_nodes: HashMap<String, String> = HashMap::new();
    for node in &graph.nodes {
        if node.node_type == "action" {
            if let Ok(Value::Object(config)) = serde_json::from_str::<Value>(&node.config_json) {
                if let Some(Value::String(step_id)) = config.get("lane_step_id") {
                    action_nodes.insert(node.id.clone(), step_id.clone());
                }
            }
        }
    }
    let mut write_counts: HashMap<(String, String), i64> = HashMap::new();
    for row in crate::repositories::graph_run_repo::list_run_nodes(conn, &run.id)? {
        if row.node_type != "action" || row.status != "completed" {
            continue;
        }
        let Some(step_id) = action_nodes.get(&row.node_id) else {
            continue;
        };
        let applied: Vec<String> = row
            .output_json
            .as_deref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|o| o.get("applied").cloned())
            .map(|a| match a {
                Value::Array(items) => items
                    .into_iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
                Value::String(s) => vec![s],
                _ => Vec::new(),
            })
            .unwrap_or_default();
        for action_type in applied {
            *write_counts
                .entry((step_id.clone(), action_type))
                .or_insert(0) += 1;
        }
    }
    let mut writes: Vec<CaseWrite> = write_counts
        .into_iter()
        .map(|((step_id, action_type), count)| CaseWrite {
            step_id,
            action_type,
            count,
        })
        .collect();
    writes.sort_by_key(|a| (a.step_id.clone(), a.action_type.clone()));
    result.writes_posted = writes;
    // A run that reaches the end node with no named outcome ends as
    // `completed`; a step that ends the run names its outcome.
    if end_outcome.is_none() {
        for step_id in &result.steps_completed {
            if let Some(outcome) = step_ends_run(&definition, step_id) {
                end_outcome = Some(outcome);
                break;
            }
        }
    }
    result.end_outcome = Some(end_outcome.unwrap_or_else(|| "completed".to_string()));

    // The assertions, each naming expected vs actual on failure.
    let run_data: Map<String, Value> = run
        .run_data_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let assertions = case["assert"].as_array().cloned().unwrap_or_default();
    for assertion in &assertions {
        check_assertion(conn, case, assertion, result, &run_data, &definition);
    }
    result.passed = result.failures.is_empty();
    Ok(())
}

fn step_ends_run(definition: &LaneDefinition, step_id: &str) -> Option<String> {
    definition
        .stages
        .iter()
        .flat_map(|s| &s.steps)
        .find(|s| s.id == step_id)
        .and_then(|s| s.ends_run.as_ref().map(|e| e.outcome.clone()))
}

/// Evaluates one case assertion against the collected evidence. Every
/// failure names expected vs actual. Both assertion spellings are
/// accepted: `{"step.completed": "<id>"}` and `{"step.<id>.completed"}`,
/// and the same pair for checks.
#[allow(clippy::too_many_arguments)]
fn check_assertion(
    conn: &Connection,
    case: &Value,
    assertion: &Value,
    result: &mut LaneCaseResult,
    run_data: &Map<String, Value>,
    definition: &LaneDefinition,
) {
    let Some(object) = assertion.as_object() else {
        result
            .failures
            .push(format!("assertion is not an object: {assertion}"));
        return;
    };
    if object.len() != 1 {
        result
            .failures
            .push(format!("assertion must have exactly one key: {assertion}"));
        return;
    }
    let (key, value) = object.iter().next().unwrap();
    // Normalise the dotted spellings (`step.<id>.completed`,
    // `check.<id>.passed`) into the flat form (`step.completed` with the
    // id alongside).
    let (kind, id_from_key): (String, Option<String>) =
        if let Some(rest) = key.strip_prefix("step.") {
            if let Some(id) = rest.strip_suffix(".completed") {
                ("step.completed".to_string(), Some(id.to_string()))
            } else if let Some(id) = rest.strip_suffix(".skipped") {
                ("step.skipped".to_string(), Some(id.to_string()))
            } else {
                (key.clone(), None)
            }
        } else if let Some(rest) = key.strip_prefix("check.") {
            if let Some(id) = rest.strip_suffix(".passed") {
                ("check.passed".to_string(), Some(id.to_string()))
            } else if let Some(id) = rest.strip_suffix(".failed") {
                ("check.failed".to_string(), Some(id.to_string()))
            } else {
                (key.clone(), None)
            }
        } else {
            (key.clone(), None)
        };
    match kind.as_str() {
        "run.ends_with" => {
            let expected = value.as_str().unwrap_or("");
            if result.end_outcome.as_deref() != Some(expected) {
                result.failures.push(format!(
                    "run.ends_with: expected '{expected}', actual '{}'",
                    result.end_outcome.as_deref().unwrap_or("")
                ));
            }
        }
        "step.completed" => {
            let step_id = id_from_key
                .or_else(|| value.as_str().map(String::from))
                .unwrap_or_default();
            if !result.steps_completed.contains(&step_id) {
                result.failures.push(format!(
                    "step '{step_id}' did not complete (completed: {:?})",
                    result.steps_completed
                ));
            }
        }
        "step.skipped" => {
            let (step_id, reason) = match (id_from_key, value) {
                (Some(id), Value::String(reason)) => (id, Some(reason.clone())),
                (Some(id), _) => (id, None),
                (None, Value::String(id)) => (id.clone(), None),
                (None, Value::Object(_)) => (
                    value
                        .get("step")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    value
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                ),
                _ => (String::new(), None),
            };
            if !result.steps_skipped.contains(&step_id) {
                result.failures.push(format!(
                    "step '{step_id}' was not skipped (skipped: {:?})",
                    result.steps_skipped
                ));
            } else if let Some(reason) = reason {
                let _ = reason;
            }
        }
        "check.passed" | "check.failed" => {
            let step_id = id_from_key
                .clone()
                .or_else(|| value.as_str().map(String::from))
                .unwrap_or_default();
            let expected = if kind == "check.passed" {
                "passed"
            } else {
                "failed"
            };
            // In the dotted form (`check.<id>.failed: "<reason>"`) the
            // value is the reason the check reported - the check's
            // message, which the runtime records on the run node when the
            // check fails (WP-0705). In the flat form the value is the
            // step id, and there is no reason to compare.
            let expected_reason = if id_from_key.is_some() && kind == "check.failed" {
                value.as_str().map(String::from)
            } else {
                None
            };
            match result.checks.iter().find(|c| c.step_id == step_id) {
                Some(check) if check.outcome == expected => {
                    if let Some(expected_reason) = expected_reason {
                        match &check.reason {
                            Some(actual) if actual == &expected_reason => {}
                            Some(actual) => {
                                result.failures.push(format!(
                                    "check '{step_id}' failed with reason '{actual}', expected '{expected_reason}'"
                                ));
                            }
                            None => {
                                result.failures.push(format!(
                                    "check '{step_id}' reported no reason, expected '{expected_reason}'"
                                ));
                            }
                        }
                    }
                }
                Some(check) => {
                    result.failures.push(format!(
                        "check '{step_id}' expected '{expected}', actual '{}'{}",
                        check.outcome,
                        check
                            .reason
                            .as_ref()
                            .map(|r| format!(" ({r})"))
                            .unwrap_or_default()
                    ));
                }
                None => {
                    result.failures.push(format!(
                        "check '{step_id}' did not run (checks: {:?})",
                        result.checks.iter().map(|c| &c.step_id).collect::<Vec<_>>()
                    ));
                }
            }
        }
        "write.posted" => {
            let action_type = value
                .get("action_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let expected_count = value.get("count").and_then(|v| v.as_i64());
            let posted: i64 = result
                .writes_posted
                .iter()
                .filter(|w| w.action_type == action_type)
                .map(|w| w.count)
                .sum();
            if posted == 0 {
                result.failures.push(format!(
                    "write '{action_type}' was not posted (posted: {:?})",
                    result.writes_posted
                ));
            } else if let Some(expected) = expected_count {
                if posted != expected {
                    result.failures.push(format!(
                        "write '{action_type}' expected {expected} rows, actual {posted}"
                    ));
                }
            }
            if let Some(inputs) = value.get("inputs").and_then(|v| v.as_object()) {
                // The runtime records which Action Types a step applied,
                // not the inputs it resolved; the inputs assertion
                // re-resolves the step's declared writes against the
                // run's end state (the same resolution the runtime did).
                for step in definition.stages.iter().flat_map(|s| &s.steps) {
                    let Some(writes) = &step.writes else {
                        continue;
                    };
                    for write in writes {
                        if write.action_type != action_type {
                            continue;
                        }
                        for (input_key, expected_value) in inputs {
                            let declared = write.inputs.get(input_key);
                            let resolved = declared
                                .map(|raw| resolve_declared_input(raw, conn, case, run_data));
                            match (resolved, expected_value) {
                                (Some(Ok(actual)), expected) => {
                                    if actual != *expected {
                                        result.failures.push(format!(
                                            "write '{action_type}' input '{input_key}': expected {expected}, actual {actual}"
                                        ));
                                    }
                                }
                                (Some(Err(reason)), _) => {
                                    result.failures.push(format!(
                                        "write '{action_type}' input '{input_key}' did not resolve ({reason})"
                                    ));
                                }
                                (None, _) => {
                                    result.failures.push(format!(
                                        "write '{action_type}' declares no input '{input_key}'"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        "write.not_posted" => {
            let action_type = value
                .get("action_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let reason = value.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            let posted: i64 = result
                .writes_posted
                .iter()
                .filter(|w| w.action_type == action_type)
                .map(|w| w.count)
                .sum();
            if posted != 0 {
                result.failures.push(format!(
                    "write '{action_type}' was posted {posted} times, expected none ({reason})"
                ));
            }
        }
        "record.equals" => {
            let object = value.get("object").and_then(|v| v.as_str()).unwrap_or("");
            let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let field = value.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let expected = value.get("value").cloned().unwrap_or(Value::Null);
            match read_record_field(conn, object, id, field) {
                Ok(actual) => {
                    if actual != expected {
                        result.failures.push(format!(
                            "record {object}/{id} field '{field}': expected {expected}, actual {actual}"
                        ));
                    }
                }
                Err(e) => {
                    result.failures.push(format!(
                        "record {object}/{id} field '{field}' could not be read: {e}"
                    ));
                }
            }
        }
        "run_data.equals" => {
            let field = value.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let expected = value.get("value").cloned().unwrap_or(Value::Null);
            let actual = run_data.get(field).cloned().unwrap_or(Value::Null);
            if actual != expected {
                result.failures.push(format!(
                    "run_data field '{field}': expected {expected}, actual {actual}"
                ));
            }
        }
        other => {
            result
                .failures
                .push(format!("unknown assertion kind '{other}'"));
        }
    }
}

/// Resolves one declared write input against the case's end state: the
/// anchor record's fields plus the run data. `$item` inputs cannot be
/// resolved here (no per-item context survives the run) and fail the
/// assertion with a named reason.
fn resolve_declared_input(
    raw: &Value,
    conn: &Connection,
    case: &Value,
    run_data: &Map<String, Value>,
) -> AppResult<Value> {
    let Value::String(s) = raw else {
        return Ok(raw.clone());
    };
    let Some((kind, path)) = split_ref(s) else {
        return Ok(raw.clone());
    };
    match kind.as_str() {
        "run_data" => {
            let field = path.split('.').next().unwrap_or("");
            Ok(run_data.get(field).cloned().unwrap_or(Value::Null))
        }
        "record" => {
            let anchor = &case["anchor"];
            let object = anchor["object"].as_str().unwrap_or("");
            let id = anchor["id"].as_str().unwrap_or("");
            let field = path.split('.').next().unwrap_or("");
            let fields = load_record_fields(conn, object, id)?;
            Ok(fields
                .get(field)
                .map(|v| Value::String(v.clone()))
                .unwrap_or(Value::Null))
        }
        _ => Err(classified_validation(
            "validation.arguments",
            format!("write input '{s}' references $item, which has no end-state value to compare against"),
        )),
    }
}

fn read_record_field(conn: &Connection, object: &str, id: &str, field: &str) -> AppResult<Value> {
    let fields = load_record_fields(conn, object, id)?;
    Ok(fields
        .get(field)
        .map(|v| Value::String(v.clone()))
        .unwrap_or(Value::Null))
}
