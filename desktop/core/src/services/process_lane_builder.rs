//! AI Agent Platform v2, Phase 7: WP-0718 - the visual lane builder's
//! core/service layer (REQ-LANE-024).
//!
//! The builder edits the same definition format the demo and the core
//! share (WP-0701); there is no builder-only format. Every function here
//! is the service the Tauri commands (`process_lane_commands`) and the
//! Team Workspace server dispatch arms call, so desktop and Team
//! Workspace behave the same from the same code (AC-4).
//!
//! - AC-1: `canonical_lane_json` parses a definition through the lane
//!   model and re-emits it with sorted keys, so data -> builder -> data
//!   is byte-identical modulo key order.
//! - AC-2: `builder_check` runs the WP-0714 publish checks
//!   (`check_publish_definition`, P-01..P-16) on the in-memory, unsaved
//!   definition - the same checks as publish, no builder-only rules.
//!   `builder_dry_run` runs the WP-0714 dry-run walk
//!   (`dry_run_definition`, shared with `dry_run_lane`) on the unsaved
//!   draft, and posts nothing.
//! - AC-4 (design §4): `builder_reads_writes` is the per-step
//!   reads/writes map plus the lane-level run-data plumbing table.
//! - AC-4 (design §6): `builder_action_types` is the Action-Type picker
//!   catalogue: the adopted WP-0705 catalog entries plus the provisional
//!   ids the stored lanes use, each marked provisional (never
//!   publishable - P-12).
//! - Save: `save_builder_lane` reuses `import_lane_definition` for a
//!   brand-new lane key and `create_version` for a new version of an
//!   existing lane.
//!
//! What this module deliberately does NOT do: no schema change (Q-33..
//! Q-36 are out of scope), no builder-only publish rule, no graph
//! runtime / execution_graph / graph_run change (ADR-0025).

use std::collections::{BTreeSet, HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::action_type::ActionParameterSpec;
use crate::models::process_lane::{parse_definition, ConditionOrGroup, LaneDefinitionFile};
use crate::repositories::process_lane_repo;
use crate::services::{action_type_service, process_lane_publish, process_lane_service};

fn classified_validation(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, code, message)
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Parses a definition file without running `validate_definition`, so
/// `builder_check` can report the failing checks instead of dying on
/// the first one - the same lenient parse the publish report uses.
fn parse_definition_lenient(definition_json: &str) -> AppResult<LaneDefinitionFile> {
    serde_json::from_str::<LaneDefinitionFile>(definition_json).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("lane definition is not valid JSON: {e}"),
        )
    })
}

// --- canonical JSON (AC-1) --------------------------------------------------

/// Recursively sorts an object's keys, so the canonical form is
/// deterministic regardless of the key order the author (or the demo)
/// wrote.
fn sort_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            for (key, inner) in map {
                sorted.insert(key, sort_keys(inner));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

/// Parses a definition through the lane model (so an invalid lane is
/// refused, not silently re-emitted) and re-emits it as canonical JSON:
/// the same fields, the same values, keys sorted. Data -> builder ->
/// data is byte-identical modulo key order.
pub fn canonical_lane_json(definition_json: &str) -> AppResult<String> {
    parse_definition(definition_json)?;
    let value: Value = serde_json::from_str(definition_json).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("lane definition is not valid JSON: {e}"),
        )
    })?;
    let canonical = sort_keys(value);
    serde_json::to_string(&canonical).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("lane definition is not valid JSON: {e}"),
        )
    })
}

// --- live publish checks on the unsaved draft (AC-2) ------------------------

/// Runs the WP-0714 publish checks (P-01..P-16) on an in-memory,
/// unsaved definition and returns the full report - the same checks as
/// publish, no builder-only rules. P-16 ("only a draft publishes")
/// passes: the definition being edited IS the draft, and saving it
/// creates the draft version, so a draft in the builder is a draft to
/// publish.
pub fn builder_check(
    conn: &Connection,
    workspace_id: &str,
    definition_json: &str,
    actor_user_id: Option<&str>,
) -> AppResult<process_lane_publish::PublishReport> {
    require_admin(conn, actor_user_id)?;
    let file = parse_definition_lenient(definition_json)?;
    process_lane_publish::check_publish_definition(
        conn,
        workspace_id,
        &file.lane,
        true,
        Some(file.lane.version),
    )
}

/// Dry-runs an unsaved draft against an anchor record: the shared
/// WP-0714 definition-level walk (`dry_run_definition`), so the
/// builder's preview is the same walk the stored-lane dry run runs.
/// Posts nothing and changes nothing.
pub fn builder_dry_run(
    conn: &Connection,
    workspace_id: &str,
    definition_json: &str,
    anchor_record_id: &str,
    fixtures: &process_lane_publish::DryRunFixtures,
    expected_outcome: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<process_lane_publish::DryRunReport> {
    require_admin(conn, actor_user_id)?;
    let file = parse_definition(definition_json)?;
    process_lane_publish::dry_run_definition(
        conn,
        workspace_id,
        &file.lane,
        file.lane.version,
        anchor_record_id,
        fixtures,
        expected_outcome,
        actor_user_id,
    )
}

// --- reads/writes map (design §4, AC-4) -------------------------------------

/// One `$`-reference a step reads, with where it reads it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRead {
    /// The reference as written, `$`-prefixed (`$record.supplier_id`,
    /// `$run_data.damage_approved`, `$item.qty_picked`).
    pub reference: String,
    /// Where it was found: `check`, `validation`, `write input '<key>'`,
    /// `field '<key>' binds to`, `child lane anchor`, `stage '<id>'
    /// entry/skip condition, `start_condition`.
    pub context: String,
}

/// One Action-Type write a step declares.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepWrite {
    pub action_type: String,
    /// The write's input keys, in definition order.
    pub inputs: Vec<String>,
}

/// What one step reads and writes (design §4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepReadsWrites {
    pub step_id: String,
    pub stage_id: String,
    pub kind: String,
    pub reads: Vec<StepRead>,
    pub writes: Vec<StepWrite>,
    /// The run-data fields this step sets: a `run_data.set` write's
    /// `key` input, and a form field bound to `run_data.<field>`.
    pub run_data_writes: Vec<String>,
}

/// One cell of the lane-level plumbing table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlumbingCell {
    pub step_id: String,
    /// `read`, `write` or `both`.
    pub access: String,
}

/// One row of the lane-level plumbing table: a run-data field and, for
/// every step that touches it, whether it reads, writes, or both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlumbingRow {
    pub field: String,
    pub field_type: Option<String>,
    pub per_child: bool,
    pub steps: Vec<PlumbingCell>,
}

/// The per-step reads/writes map plus the lane-level plumbing table
/// (design §4). Pure function over the definition: no workspace state,
/// no database read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadsWritesReport {
    pub lane_key: String,
    pub steps: Vec<StepReadsWrites>,
    /// References the lane itself (not one step) reads: the start
    /// condition and the stage entry/skip conditions.
    pub lane_reads: Vec<StepRead>,
    pub plumbing: Vec<PlumbingRow>,
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
    context: &str,
    out: &mut Vec<StepRead>,
) {
    match condition {
        ConditionOrGroup::Condition(c) => {
            if split_ref(&c.field).is_some() {
                out.push(StepRead {
                    reference: format!("${}", c.field.trim_start_matches('$')),
                    context: context.to_string(),
                });
            }
            if let Some(other) = &c.compare_field {
                if split_ref(other).is_some() {
                    out.push(StepRead {
                        reference: format!("${}", other.trim_start_matches('$')),
                        context: context.to_string(),
                    });
                }
            }
        }
        ConditionOrGroup::Group(g) => {
            for inner in &g.conditions {
                collect_refs_from_condition(inner, context, out);
            }
        }
    }
}

fn collect_refs_from_expression(expression: &str, context: &str, out: &mut Vec<StepRead>) {
    for token in expression.split_whitespace() {
        if split_ref(token).is_some() {
            out.push(StepRead {
                reference: format!("${}", token.trim_start_matches('$')),
                context: context.to_string(),
            });
        }
    }
}

fn collect_refs_from_rule(
    condition: &Option<ConditionOrGroup>,
    expression: &Option<String>,
    context: &str,
    out: &mut Vec<StepRead>,
) {
    if let Some(condition) = condition {
        collect_refs_from_condition(condition, context, out);
    }
    if let Some(expression) = expression {
        collect_refs_from_expression(expression, context, out);
    }
}

fn run_data_field_of(reference: &str) -> Option<String> {
    let rest = reference.strip_prefix("$run_data.")?;
    Some(rest.split('.').next().unwrap_or("").to_string())
}

/// The per-step reads/writes map and the lane-level plumbing table for
/// one parsed definition.
pub fn builder_reads_writes(definition_json: &str) -> AppResult<ReadsWritesReport> {
    let file = parse_definition(definition_json)?;
    let lane = file.lane;
    let mut steps: Vec<StepReadsWrites> = Vec::new();
    let mut lane_reads: Vec<StepRead> = Vec::new();

    if let Some(start) = &lane.start_condition {
        collect_refs_from_condition(start, "start_condition", &mut lane_reads);
    }

    // Declared run-data fields, in definition order, plus any
    // referenced-but-undeclared field (P-11's territory - shown here
    // with no declared type, never hidden).
    let mut declared: Vec<(String, Option<String>, bool)> = Vec::new();
    if let Some(run_data) = &lane.run_data {
        for field in &run_data.fields {
            declared.push((
                field.key.clone(),
                Some(field.field_type.clone()),
                field.per_child,
            ));
        }
    }

    for stage in &lane.stages {
        for (which, condition) in [
            ("entry_condition", &stage.entry_condition),
            ("skip_condition", &stage.skip_condition),
        ] {
            if let Some(condition) = condition {
                collect_refs_from_condition(
                    condition,
                    &format!("stage '{}' {which}", stage.id),
                    &mut lane_reads,
                );
            }
        }
        for step in &stage.steps {
            let mut reads: Vec<StepRead> = Vec::new();
            let mut writes: Vec<StepWrite> = Vec::new();
            let mut run_data_writes: Vec<String> = Vec::new();

            if let Some(fields) = &step.fields {
                for field in fields {
                    // A `binds_to` is a read of the bound value - and a
                    // `run_data.<field>` binding is also a write: the
                    // person fills it in.
                    reads.push(StepRead {
                        reference: format!("${}", field.binds_to.trim_start_matches('$')),
                        context: format!("field '{}' binds to", field.key),
                    });
                    if let Some(target) = field.binds_to.strip_prefix("run_data.") {
                        run_data_writes.push(target.to_string());
                    }
                }
            }
            if let Some(step_writes) = &step.writes {
                for write in step_writes {
                    let mut input_keys: Vec<String> = Vec::new();
                    for (key, value) in &write.inputs {
                        input_keys.push(key.clone());
                        if let Value::String(s) = value {
                            if split_ref(s).is_some() {
                                reads.push(StepRead {
                                    reference: format!("${}", s.trim_start_matches('$')),
                                    context: format!("write input '{key}'"),
                                });
                            }
                        }
                    }
                    writes.push(StepWrite {
                        action_type: write.action_type.clone(),
                        inputs: input_keys,
                    });
                    // The provisional `run_data.set` write sets the
                    // run-data field its `key` input names.
                    if write.action_type == "run_data.set" {
                        if let Some(Value::String(key)) = write.inputs.get("key") {
                            run_data_writes.push(key.clone());
                        }
                    }
                }
            }
            if let Some(check) = &step.check {
                collect_refs_from_rule(&check.condition, &check.expression, "check", &mut reads);
            }
            if let Some(validation) = &step.validation {
                collect_refs_from_rule(
                    &validation.condition,
                    &validation.expression,
                    "validation",
                    &mut reads,
                );
            }
            if let Some(child) = &step.child_lane {
                if split_ref(&child.anchor_record).is_some() {
                    reads.push(StepRead {
                        reference: format!("${}", child.anchor_record.trim_start_matches('$')),
                        context: "child lane anchor".to_string(),
                    });
                }
            }

            // De-duplicate reads (same reference, same context) while
            // keeping definition order.
            let mut seen: HashSet<(String, String)> = HashSet::new();
            reads.retain(|r| seen.insert((r.reference.clone(), r.context.clone())));
            run_data_writes.sort();
            run_data_writes.dedup();
            steps.push(StepReadsWrites {
                step_id: step.id.clone(),
                stage_id: stage.id.clone(),
                kind: step.kind.clone(),
                reads,
                writes,
                run_data_writes,
            });
        }
    }

    // The plumbing table: one row per declared run-data field (plus any
    // referenced-but-undeclared one), one cell per step that touches it.
    let mut field_order: Vec<String> = declared.iter().map(|(k, _, _)| k.clone()).collect();
    for step in &steps {
        for read in &step.reads {
            if let Some(field) = run_data_field_of(&read.reference) {
                if !field.is_empty() && !field_order.contains(&field) {
                    field_order.push(field);
                }
            }
        }
        for field in &step.run_data_writes {
            if !field_order.contains(field) {
                field_order.push(field.clone());
            }
        }
    }
    let declared_map: HashMap<&str, (Option<String>, bool)> = declared
        .iter()
        .map(|(k, t, p)| (k.as_str(), (t.clone(), *p)))
        .collect();
    let mut plumbing: Vec<PlumbingRow> = Vec::new();
    for field in field_order {
        let mut cells: Vec<PlumbingCell> = Vec::new();
        for step in &steps {
            let reads_it = step
                .reads
                .iter()
                .any(|r| run_data_field_of(&r.reference).as_deref() == Some(field.as_str()));
            let writes_it = step.run_data_writes.iter().any(|f| f == &field);
            let access = match (reads_it, writes_it) {
                (true, true) => "both",
                (true, false) => "read",
                (false, true) => "write",
                (false, false) => continue,
            };
            cells.push(PlumbingCell {
                step_id: step.step_id.clone(),
                access: access.to_string(),
            });
        }
        if cells.is_empty() {
            continue;
        }
        let (field_type, per_child) = declared_map
            .get(field.as_str())
            .cloned()
            .unwrap_or((None, false));
        plumbing.push(PlumbingRow {
            field,
            field_type,
            per_child,
            steps: cells,
        });
    }

    Ok(ReadsWritesReport {
        lane_key: lane.id,
        steps,
        lane_reads,
        plumbing,
    })
}

// --- Action-Type picker catalogue (design §6, AC-4) ------------------------

/// One entry of the Action-Type picker catalogue (design §6): an
/// adopted WP-0705 catalog entry, or a provisional id the stored lanes
/// use. A provisional entry is never publishable (P-12) - the builder
/// shows it struck-through with the P-12 reason, and never lets it
/// through to publish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderActionType {
    pub name: String,
    pub target_object_type: String,
    pub adopted: bool,
    pub provisional: bool,
    pub effect: Option<String>,
    pub description: String,
    pub parameters: Vec<ActionParameterSpec>,
}

/// The Action-Type picker catalogue: the adopted WP-0705 catalog
/// entries (active `action_type_definitions` rows, read from the same
/// source P-12 reads) plus the provisional ids used by the lanes
/// stored in this workspace, each marked provisional. `object_key`
/// scopes the catalogue to one object; `None` lists every object.
/// Provisional ids only exist once a lane using them is stored - a
/// fresh workspace drafting its first lane lists the adopted catalog
/// only.
pub fn builder_action_types(
    conn: &Connection,
    workspace_id: &str,
    object_key: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BuilderActionType>> {
    require_admin(conn, actor_user_id)?;
    let adopted = action_type_service::list(conn, workspace_id, object_key, true)?;
    let mut adopted_keys: HashSet<(String, String)> = HashSet::new();
    let mut out: Vec<BuilderActionType> = Vec::new();
    for def in adopted {
        adopted_keys.insert((def.name.clone(), def.target_object_type.clone()));
        out.push(BuilderActionType {
            name: def.name,
            target_object_type: def.target_object_type,
            adopted: true,
            provisional: false,
            effect: Some(def.effect),
            description: def.description,
            parameters: def.parameters,
        });
    }

    // Provisional ids: every write Action Type the stored lanes use
    // that has no adopted active row on the same object.
    let mut provisional: BTreeSet<(String, String)> = BTreeSet::new();
    for lane in process_lane_repo::list_lanes(conn, workspace_id)? {
        if let Some(object) = object_key {
            if lane.object_key != object {
                continue;
            }
        }
        let Some(version) = process_lane_repo::latest_version(conn, &lane.id)? else {
            continue;
        };
        let Ok(file) = parse_definition_lenient(&version.definition_json) else {
            continue;
        };
        for stage in &file.lane.stages {
            for step in &stage.steps {
                if let Some(writes) = &step.writes {
                    for write in writes {
                        let key = (write.action_type.clone(), file.lane.object_key.clone());
                        if !adopted_keys.contains(&key) {
                            provisional.insert(key);
                        }
                    }
                }
            }
        }
    }
    for (name, object) in provisional {
        out.push(BuilderActionType {
            name,
            target_object_type: object,
            adopted: false,
            provisional: true,
            effect: None,
            description: "Provisional - not in the adopted catalog, cannot publish (P-12)"
                .to_string(),
            parameters: Vec::new(),
        });
    }
    out.sort_by(|a, b| {
        a.target_object_type
            .cmp(&b.target_object_type)
            .then(a.name.cmp(&b.name))
    });
    Ok(out)
}

// --- save path --------------------------------------------------------------

/// What saving a builder lane produced: the lane key and the draft
/// version the builder can now keep editing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderSaveResult {
    pub lane_key: String,
    pub version: i64,
    pub status: String,
}

/// Saves a builder lane. A brand-new lane key (`lane_key` is `None`)
/// reuses the import path (`import_lane_definition`): the definition's
/// own id becomes the lane, stored as its first draft version. An
/// existing lane key reuses `create_version`: the definition must name
/// the next free version, and the lane must have no draft already (the
/// same rules the version command enforces).
pub fn save_builder_lane(
    conn: &Connection,
    workspace_id: &str,
    lane_key: Option<&str>,
    definition_json: &str,
    actor_user_id: Option<&str>,
) -> AppResult<BuilderSaveResult> {
    require_admin(conn, actor_user_id)?;
    match lane_key {
        None => {
            let lane = process_lane_service::import_lane_definition(
                conn,
                workspace_id,
                definition_json,
                actor_user_id,
            )?;
            let version = process_lane_repo::latest_version(conn, &lane.id)?
                .ok_or_else(|| AppError::NotFound("Process lane version".into()))?;
            Ok(BuilderSaveResult {
                lane_key: lane.lane_key,
                version: version.version,
                status: version.status,
            })
        }
        Some(key) => {
            let version = process_lane_service::create_version(
                conn,
                workspace_id,
                key,
                definition_json,
                actor_user_id,
            )?;
            Ok(BuilderSaveResult {
                lane_key: key.to_string(),
                version: version.version,
                status: version.status,
            })
        }
    }
}
