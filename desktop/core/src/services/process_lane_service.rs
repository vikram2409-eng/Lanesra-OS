//! AI Agent Platform v2, Phase 7: Process Lane definitions in the core
//! (WP-0703) - store, lifecycle and compile to a graph (ADR-0023, ADR-0024).
//!
//! A lane is defined as data (one JSON Schema, WP-0701) and stored in
//! `process_lanes`/`process_lane_versions` (migration 0095). Lifecycle:
//! import a definition as a draft version, then publish it - publishing
//! compiles the lane to an execution graph (`source_kind = "lane"`,
//! `source_id = "{lane_id}@v{version}"`) and flips the version to
//! published. A published version is never edited in place; a change
//! publishes a new version, and a run keeps the version it started with
//! because its `graph_id` points at the compiled graph of that version.
//!
//! Compile (ADR-0024 §3): one step compiles to exactly one node - form to
//! the new `human_step` node, record to `action`, check to `condition`,
//! approval to `approval`, agent to `agent` - plus one `condition` node per
//! stage entry/skip condition, one `loop` node per repeating step, and the
//! `trigger`/`end` nodes the engine requires. Every compiled node records
//! its lane position in its config (`lane_stage_id` / `lane_step_id`);
//! stages are grouping only and contribute no node of their own. A check
//! step's `pass`/`fail` outcomes map onto the condition node's
//! `true`/`false` edges (the engine's condition vocabulary); the lane rule
//! itself is kept verbatim in the node's `lane_check` config.
//!
//! What this package deliberately does NOT do (later Phase 7 packages):
//! - `human_step` execution - waiting for a person, validating the
//!   submission (WP-0704). A compiled graph containing one passes
//!   `validate_for_publish` but its run waits on WP-0704 to execute it.
//! - Record-step writes: the compiled `action` node carries the step's
//!   `writes`; `$`-reference resolution and all-or-nothing, repeat-safe
//!   posting are WP-0705's. A single-write step also carries the
//!   `action_type`/`params_json` the runtime's action arm already reads.
//! - Expression and `$item` condition evaluation, and item-driven loop
//!   exit (WP-0710). The compile keeps those rules verbatim in the node
//!   config and derives the runtime's `conditions` shape only when the
//!   rule is expressible in it (literal values on `$record`/`$run_data`
//!   fields, at most one level of OR-group nesting); otherwise the node's
//!   `conditions` is empty and the runtime evaluates it false until
//!   WP-0710 lands. Nothing is silently dropped: the full rule is always
//!   in `lane_check` / `lane_condition`.
//! - The interrupted-run sweep's exclusion of the new waiting statuses
//!   (WP-0704); the statuses themselves stay out of `RUN_STATUSES` until
//!   that package wires the transition table.
//! - Installing a `process_lanes` entry from a Solution manifest
//!   (follow-up): the export arm in `industry_package_service` writes the
//!   lane's published definition; import of it is a new draft version here.

use std::collections::HashMap;

use rusqlite::Connection;
use serde_json::{json, Map, Value};

use crate::domain::conditions::{condition_matches, CONDITION_OPERATORS};
use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::execution_graph::{
    ExecutionGraph, ExecutionGraphInput, GraphEdgeInput, GraphNodeInput,
};
use crate::models::graph_run::GraphRun;
use crate::models::process_lane::{
    parse_definition, validate_definition, ConditionOrGroup, LaneDefinition, LaneStep, ProcessLane,
    ProcessLaneVersion, LANE_REPEAT_MAX_ITERATIONS,
};
use crate::repositories::{audit_repo, graph_run_repo, process_lane_repo};
use crate::services::{
    builtin_field_service, custom_record_service, execution_graph_service, graph_runtime_service,
};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn classified_validation(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, code, message)
}

/// The `source_id` a compiled lane graph carries: `{lane_id}@v{version}`
/// (ADR-0024 §3). A run's `graph_id` points at this graph, which is how a
/// run keeps its version.
pub fn lane_graph_source_id(lane_key: &str, version: i64) -> String {
    format!("{lane_key}@v{version}")
}

// --- lifecycle -----------------------------------------------------------

/// Imports a lane definition as a new lane's first (draft) version. The
/// definition JSON is stored verbatim - a lane from the demo imports
/// unchanged (WP-0703 AC-3) - and the version row's status is the lifecycle
/// truth, independent of the `status` inside the definition.
pub fn import_lane_definition(
    conn: &Connection,
    workspace_id: &str,
    definition_json: &str,
    actor_user_id: Option<&str>,
) -> AppResult<ProcessLane> {
    require_admin(conn, actor_user_id)?;
    let file = parse_definition(definition_json)?;
    if process_lane_repo::get_lane_by_key(conn, workspace_id, &file.lane.id)?.is_some() {
        return Err(classified_validation(
            "validation.arguments",
            format!(
                "lane '{}' already exists - import a new version instead",
                file.lane.id
            ),
        ));
    }
    let lane = process_lane_repo::create_lane(
        conn,
        &crate::domain::ids::new_uuid(),
        workspace_id,
        &file.lane.id,
        &file.lane.name,
        file.lane.description.as_deref(),
        &file.lane.object_key,
        actor_user_id,
    )?;
    process_lane_repo::create_version(
        conn,
        &crate::domain::ids::new_uuid(),
        &lane.id,
        workspace_id,
        file.lane.version,
        "draft",
        definition_json,
        actor_user_id,
    )?;
    super::solution_component_service::tag_local(
        conn,
        workspace_id,
        "process_lane",
        &lane.id,
        actor_user_id,
    )?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "create",
        Some("process_lane"),
        Some(&lane.id),
        &format!("Imported lane definition '{}'", file.lane.id),
        None,
    )?;
    Ok(process_lane_repo::get_lane(conn, &lane.id)?.expect("just created"))
}

pub fn get_lane(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    actor_user_id: Option<&str>,
) -> AppResult<ProcessLane> {
    require_admin(conn, actor_user_id)?;
    process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))
}

pub fn list_lanes(
    conn: &Connection,
    workspace_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<ProcessLane>> {
    require_admin(conn, actor_user_id)?;
    Ok(process_lane_repo::list_lanes(conn, workspace_id)?)
}

/// Creates a new draft version of an existing lane from an edited
/// definition. Refused while a draft version already exists (one draft at
/// a time), and refused when the definition disagrees with the lane (id,
/// object) or names a different version than the next free one.
pub fn create_version(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    definition_json: &str,
    actor_user_id: Option<&str>,
) -> AppResult<ProcessLaneVersion> {
    require_admin(conn, actor_user_id)?;
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let file = parse_definition(definition_json)?;
    if file.lane.id != lane.lane_key {
        return Err(classified_validation(
            "validation.arguments",
            "the definition's id does not match this lane",
        ));
    }
    if file.lane.object_key != lane.object_key {
        return Err(classified_validation(
            "validation.arguments",
            "a lane stays on the object it was created for",
        ));
    }
    let latest = process_lane_repo::latest_version(conn, &lane.id)?;
    if let Some(latest) = &latest {
        if latest.status == "draft" {
            return Err(classified_validation(
                "validation.arguments",
                "this lane already has a draft version - publish it first",
            ));
        }
    }
    let next_version = latest.map(|v| v.version + 1).unwrap_or(1);
    if file.lane.version != next_version {
        return Err(classified_validation(
            "validation.arguments",
            format!("the definition's version must be {next_version} for this lane",),
        ));
    }
    let created = process_lane_repo::create_version(
        conn,
        &crate::domain::ids::new_uuid(),
        &lane.id,
        workspace_id,
        next_version,
        "draft",
        definition_json,
        actor_user_id,
    )?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "create",
        Some("process_lane_version"),
        Some(&created.id),
        &format!("Created draft version {next_version} of lane '{lane_key}'"),
        None,
    )?;
    Ok(created)
}

/// Publishes the lane's latest draft version: compiles it to an execution
/// graph (which must pass `validate_for_publish`), publishes that graph,
/// and flips the version to published with the graph's id. The graph's
/// `source_id` is `{lane_id}@v{version}`, so runs started from now on use
/// this version and runs already started keep theirs (WP-0703 AC-2).
pub fn publish_lane(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    actor_user_id: Option<&str>,
) -> AppResult<ProcessLaneVersion> {
    require_admin(conn, actor_user_id)?;
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let draft = process_lane_repo::latest_version(conn, &lane.id)?
        .filter(|v| v.status == "draft")
        .ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                format!("lane '{lane_key}' has no draft version to publish"),
            )
        })?;
    let file = parse_definition(&draft.definition_json)?;
    let input = compile_lane(&file.lane)?;
    validate_compiled(&input, &file.lane)?;
    let graph = execution_graph_service::create_from_source(
        conn,
        workspace_id,
        input,
        "lane",
        &lane_graph_source_id(&lane.lane_key, draft.version),
        actor_user_id,
    )?;
    process_lane_repo::set_version_published(conn, &draft.id, &graph.id, actor_user_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "publish",
        Some("process_lane"),
        Some(&lane.id),
        &format!(
            "Published lane '{}' version {} as graph '{}'",
            lane_key, draft.version, graph.id
        ),
        None,
    )?;
    Ok(process_lane_repo::get_version_by_id(conn, &draft.id)?.expect("just published"))
}

/// Builds the in-memory graph for a compiled input and runs the engine's
/// own publish validator over it, before anything is written - so a lane
/// that cannot compile fails the publish with the validator's reason and
/// leaves no orphan draft graph behind.
fn validate_compiled(input: &ExecutionGraphInput, lane: &LaneDefinition) -> AppResult<()> {
    let mut key_to_id: HashMap<String, String> = HashMap::new();
    let mut nodes = Vec::new();
    for node in &input.nodes {
        let id = crate::domain::ids::new_uuid();
        key_to_id.insert(node.node_key.clone(), id.clone());
        nodes.push(crate::models::execution_graph::GraphNode {
            id,
            graph_id: String::new(),
            node_key: node.node_key.clone(),
            node_type: node.node_type.clone(),
            config_json: node.config_json.clone(),
            position_x: node.position_x,
            position_y: node.position_y,
            sort_order: node.sort_order,
        });
    }
    let mut edges = Vec::new();
    for edge in &input.edges {
        let from = key_to_id.get(&edge.from_node_key).ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                format!(
                    "compiled lane edge references unknown node '{}'",
                    edge.from_node_key
                ),
            )
        })?;
        let to = key_to_id.get(&edge.to_node_key).ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                format!(
                    "compiled lane edge references unknown node '{}'",
                    edge.to_node_key
                ),
            )
        })?;
        edges.push(crate::models::execution_graph::GraphEdge {
            id: crate::domain::ids::new_uuid(),
            graph_id: String::new(),
            from_node_id: from.clone(),
            to_node_id: to.clone(),
            branch_label: edge.branch_label.clone(),
            sort_order: edge.sort_order,
        });
    }
    let graph = ExecutionGraph {
        id: String::new(),
        workspace_id: String::new(),
        name: lane.name.clone(),
        description: lane.description.clone(),
        status: "draft".to_string(),
        version: lane.version,
        source_kind: Some("lane".to_string()),
        source_id: Some(lane_graph_source_id(&lane.id, lane.version)),
        created_at: String::new(),
        created_by: None,
        updated_at: String::new(),
        updated_by: None,
        nodes,
        edges,
    };
    execution_graph_service::validate_for_publish(&graph)
}

// --- compile -------------------------------------------------------------

/// One compiled step's node config: the step's own JSON (role, fields,
/// writes, check, assistance, `repeat_over`, ...) with its lane position
/// added, so the runner and the audit trail can always say which step a
/// node is. The step JSON is stored verbatim - no lane field is renamed.
fn step_config(step: &LaneStep, stage_id: &str) -> AppResult<Value> {
    let mut config = serde_json::to_value(step).map_err(|e| {
        classified_validation(
            "validation.arguments",
            format!("lane step is not valid JSON: {e}"),
        )
    })?;
    let object = config.as_object_mut().ok_or_else(|| {
        classified_validation("validation.arguments", "lane step is not an object")
    })?;
    object.insert("lane_stage_id".into(), json!(stage_id));
    object.insert("lane_step_id".into(), json!(step.id));
    Ok(config)
}

/// A runtime condition as the engine's condition executor reads it:
/// `(group_id, field_key, operator, value)` (see
/// `domain::conditions::conditions_match`).
type RuntimeCondition = (Option<String>, String, String, String);

/// Translates a lane condition (or group) into the runtime's flat
/// conditions shape, when it can be expressed there at all: literal values
/// on `$record`/`$run_data` fields, at most one level of OR-group nesting
/// (the engine groups are always OR-ed). Returns `None` when the rule
/// needs something the runtime cannot express yet - an `expression`, a
/// `compare_field`, a `$item` reference, a nested AND group, or deeper
/// nesting. `None` is not an error: the caller keeps the rule verbatim in
/// `lane_check` / `lane_condition` and leaves `conditions` empty, and
/// WP-0710 owns evaluating it.
fn runtime_conditions(rule: &ConditionOrGroup) -> Option<(String, Vec<RuntimeCondition>)> {
    fn leaf(
        field: &str,
        operator: &str,
        value: &Option<Value>,
        compare_field: &Option<String>,
        group: Option<String>,
        out: &mut Vec<RuntimeCondition>,
    ) -> bool {
        if compare_field.is_some() {
            return false;
        }
        let Some(value) = value else {
            return false;
        };
        let key = match field.strip_prefix("$record.") {
            Some(rest) => rest.to_string(),
            None => match field.strip_prefix("$run_data.") {
                Some(rest) => format!("run_data.{rest}"),
                None => return false, // $item.* or anything else
            },
        };
        if !CONDITION_OPERATORS.contains(&operator) {
            return false;
        }
        let value = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        out.push((group, key, operator.to_string(), value));
        true
    }
    fn walk(
        rule: &ConditionOrGroup,
        group: Option<String>,
        out: &mut Vec<RuntimeCondition>,
    ) -> Option<String> {
        match rule {
            ConditionOrGroup::Condition(c) => {
                if leaf(
                    &c.field,
                    &c.operator,
                    &c.value,
                    &c.compare_field,
                    group,
                    out,
                ) {
                    Some(String::new()) // no group of its own
                } else {
                    None
                }
            }
            ConditionOrGroup::Group(g) => {
                // The engine can only express a nested group as an OR
                // group; a nested AND group has no runtime shape.
                if group.is_some() {
                    return None; // deeper than one level of nesting
                }
                let mut nested = Vec::new();
                for child in &g.conditions {
                    match child {
                        ConditionOrGroup::Condition(c) => {
                            let child_group = if g.match_type == "any" {
                                Some("g0".to_string())
                            } else {
                                None
                            };
                            if !leaf(
                                &c.field,
                                &c.operator,
                                &c.value,
                                &c.compare_field,
                                child_group,
                                &mut nested,
                            ) {
                                return None;
                            }
                        }
                        ConditionOrGroup::Group(_) => return None,
                    }
                }
                out.extend(nested);
                Some(g.match_type.clone())
            }
        }
    }
    let mut out = Vec::new();
    let match_type = walk(rule, None, &mut out)?;
    if match_type.is_empty() {
        Some(("all".to_string(), out))
    } else {
        Some((match_type, out))
    }
}

/// The node type a lane step compiles to (ADR-0024 §3).
fn step_node_type(step: &LaneStep) -> &'static str {
    match step.kind.as_str() {
        "form" => "human_step",
        "record" => "action",
        "check" => "condition",
        "approval" => "approval",
        "agent" => "agent",
        _ => unreachable!("validate_definition rejects unknown step kinds"),
    }
}

/// The config a compiled step node carries, on top of its own JSON.
fn step_node_config(step: &LaneStep, stage_id: &str) -> AppResult<Value> {
    let mut config = step_config(step, stage_id)?;
    let object = config
        .as_object_mut()
        .expect("step_config returns an object");
    match step.kind.as_str() {
        "record" => {
            // The runtime's action arm reads `action_type` + `params_json`
            // for a single-write step; multi-write steps keep the `writes`
            // array, which WP-0705 resolves and posts all-or-nothing.
            if let Some(writes) = &step.writes {
                if writes.len() == 1 {
                    let write = &writes[0];
                    object.insert("action_type".into(), json!(write.action_type));
                    object.insert(
                        "params_json".into(),
                        json!(serde_json::to_string(&write.inputs).unwrap_or_default()),
                    );
                }
            }
        }
        "check" => {
            let rule = step
                .check
                .as_ref()
                .expect("validated: check steps have a check");
            object.insert(
                "lane_check".into(),
                serde_json::to_value(rule).unwrap_or(json!(null)),
            );
            let (match_type, conditions) = match &rule.condition {
                Some(condition) => {
                    runtime_conditions(condition).unwrap_or_else(|| ("all".to_string(), Vec::new()))
                }
                None => ("all".to_string(), Vec::new()), // expression: WP-0710
            };
            object.insert("scope".into(), json!(rule.scope));
            object.insert("aggregate".into(), json!(rule.aggregate));
            object.insert("match_type".into(), json!(match_type));
            object.insert(
                "conditions".into(),
                json!(conditions
                    .iter()
                    .map(|(group, field_key, operator, value)| {
                        json!({
                            "group_id": group,
                            "field_key": field_key,
                            "operator": operator,
                            "value": value,
                        })
                    })
                    .collect::<Vec<_>>()),
            );
        }
        "approval" => {
            object.insert("subject_type".into(), json!("lane_step"));
        }
        _ => {}
    }
    Ok(config)
}

enum GateKind {
    Entry,
    Skip,
}

enum Pos {
    Trigger,
    Gate {
        key: String,
        stage: usize,
        gate: GateKind,
    },
    Step {
        entry: String,
        node: String,
        loop_key: Option<String>,
        stage: usize,
        kind: String,
        ends_run: bool,
        branches: Vec<crate::models::process_lane::Branch>,
    },
    End,
}

impl Pos {
    fn stage(&self) -> Option<usize> {
        match self {
            Pos::Gate { stage, .. } => Some(*stage),
            Pos::Step { stage, .. } => Some(*stage),
            _ => None,
        }
    }
    fn entry_key(&self) -> String {
        match self {
            Pos::Trigger => "trigger".to_string(),
            Pos::Gate { key, .. } => key.clone(),
            Pos::Step { entry, .. } => entry.clone(),
            Pos::End => "end".to_string(),
        }
    }
}

fn unconditional(from: &str, to: &str, sort: i64) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.to_string(),
        to_node_key: to.to_string(),
        branch_label: None,
        sort_order: sort,
    }
}

fn branched(from: &str, to: &str, label: &str, sort: i64) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.to_string(),
        to_node_key: to.to_string(),
        branch_label: Some(label.to_string()),
        sort_order: sort,
    }
}

/// Resolves a branch target (`{stage, step}` or `end`) to the node key an
/// outgoing edge should point at, or the step's linear successor when the
/// outcome is not declared.
fn resolve_branch(
    branches: &[crate::models::process_lane::Branch],
    outcome: &str,
    fallback: &str,
    step_entry: &HashMap<String, String>,
) -> String {
    for branch in branches {
        if branch.on == outcome {
            if let Some(target) = &branch.target {
                if let Some(entry) = step_entry.get(&format!("{}:{}", target.stage, target.step)) {
                    return entry.clone();
                }
            }
            if branch.end.is_some() {
                return "end".to_string();
            }
        }
    }
    fallback.to_string()
}

/// Compiles a lane definition to an execution graph input (ADR-0024 §3).
/// One step compiles to exactly one node; stage entry/skip conditions
/// compile to `condition` nodes placed before the stage's first step; a
/// repeating step is wrapped in a `loop` node (the loop's `body` edge
/// enters the step, its `exit` edge leaves it, the step returns to the
/// loop) with `max_iterations` set to the safety bound
/// `LANE_REPEAT_MAX_ITERATIONS` - the per-run item count is WP-0710's.
/// The graph is validated by `publish_lane` before it is written.
pub fn compile_lane(lane: &LaneDefinition) -> AppResult<ExecutionGraphInput> {
    validate_definition(lane)?;
    let mut nodes: Vec<GraphNodeInput> = Vec::new();
    let mut positions: Vec<Pos> = Vec::new();
    let mut step_entry: HashMap<String, String> = HashMap::new();
    let mut sort: i64 = 0;

    nodes.push(GraphNodeInput {
        node_key: "trigger".to_string(),
        node_type: "trigger".to_string(),
        config_json: json!({
            "source": "lane",
            "lane_id": lane.id,
            "lane_version": lane.version,
        })
        .to_string(),
        position_x: None,
        position_y: Some(0.0),
        sort_order: sort,
    });
    sort += 1;
    positions.push(Pos::Trigger);

    for (stage_idx, stage) in lane.stages.iter().enumerate() {
        for (gate, condition) in [
            (GateKind::Entry, &stage.entry_condition),
            (GateKind::Skip, &stage.skip_condition),
        ] {
            let Some(condition) = condition else {
                continue;
            };
            let gate_name = match gate {
                GateKind::Entry => "entry",
                GateKind::Skip => "skip",
            };
            let key = format!("stage:{}:{gate_name}", stage.id);
            let (match_type, conditions) =
                runtime_conditions(condition).unwrap_or_else(|| ("all".to_string(), Vec::new()));
            let mut config = Map::new();
            config.insert("lane_stage_id".into(), json!(stage.id));
            config.insert(
                "lane_condition".into(),
                serde_json::to_value(condition).unwrap_or(json!(null)),
            );
            config.insert("match_type".into(), json!(match_type));
            config.insert(
                "conditions".into(),
                json!(conditions
                    .iter()
                    .map(|(group, field_key, operator, value)| {
                        json!({
                            "group_id": group,
                            "field_key": field_key,
                            "operator": operator,
                            "value": value,
                        })
                    })
                    .collect::<Vec<_>>()),
            );
            nodes.push(GraphNodeInput {
                node_key: key.clone(),
                node_type: "condition".to_string(),
                config_json: Value::Object(config).to_string(),
                position_x: None,
                position_y: Some(sort as f64),
                sort_order: sort,
            });
            sort += 1;
            positions.push(Pos::Gate {
                key,
                stage: stage_idx,
                gate,
            });
        }
        for step in &stage.steps {
            let node_key = format!("step:{}:{}", stage.id, step.id);
            let node_type = step_node_type(step);
            let config = step_node_config(step, &stage.id)?;
            let loop_key = if let Some(repeat_over) = &step.repeat_over {
                let key = format!("loop:{}:{}", stage.id, step.id);
                let mut loop_config = Map::new();
                loop_config.insert("max_iterations".into(), json!(LANE_REPEAT_MAX_ITERATIONS));
                loop_config.insert("lane_stage_id".into(), json!(stage.id));
                loop_config.insert("lane_step_id".into(), json!(step.id));
                loop_config.insert(
                    "repeat_over".into(),
                    serde_json::to_value(repeat_over).unwrap_or(json!(null)),
                );
                nodes.push(GraphNodeInput {
                    node_key: key.clone(),
                    node_type: "loop".to_string(),
                    config_json: Value::Object(loop_config).to_string(),
                    position_x: None,
                    position_y: Some(sort as f64),
                    sort_order: sort,
                });
                sort += 1;
                Some(key)
            } else {
                None
            };
            nodes.push(GraphNodeInput {
                node_key: node_key.clone(),
                node_type: node_type.to_string(),
                config_json: config.to_string(),
                position_x: None,
                position_y: Some(sort as f64),
                sort_order: sort,
            });
            sort += 1;
            let entry = loop_key.clone().unwrap_or_else(|| node_key.clone());
            step_entry.insert(format!("{}:{}", stage.id, step.id), entry.clone());
            positions.push(Pos::Step {
                entry,
                node: node_key,
                loop_key,
                stage: stage_idx,
                kind: step.kind.clone(),
                ends_run: step.ends_run.is_some(),
                branches: step.branches.clone().unwrap_or_default(),
            });
        }
    }

    nodes.push(GraphNodeInput {
        node_key: "end".to_string(),
        node_type: "end".to_string(),
        config_json: "{}".to_string(),
        position_x: None,
        position_y: Some(sort as f64),
        sort_order: sort,
    });
    positions.push(Pos::End);

    // The entry key of each stage's first position, for entry/skip routing.
    let stage_first: Vec<String> = (0..lane.stages.len())
        .map(|i| {
            positions
                .iter()
                .find(|p| p.stage() == Some(i))
                .map(|p| p.entry_key())
                .unwrap_or_else(|| "end".to_string())
        })
        .collect();

    let mut edges: Vec<GraphEdgeInput> = Vec::new();
    let mut edge_sort: i64 = 0;
    for (i, pos) in positions.iter().enumerate() {
        let next = if i + 1 < positions.len() {
            positions[i + 1].entry_key()
        } else {
            "end".to_string()
        };
        match pos {
            Pos::Trigger => {
                edges.push(unconditional("trigger", &next, edge_sort));
                edge_sort += 1;
            }
            Pos::Gate { key, stage, gate } => {
                let other_stage = stage_first
                    .get(stage + 1)
                    .cloned()
                    .unwrap_or_else(|| "end".to_string());
                let (true_to, false_to) = match gate {
                    GateKind::Entry => (next.clone(), other_stage),
                    GateKind::Skip => (other_stage, next.clone()),
                };
                edges.push(branched(key, &true_to, "true", edge_sort));
                edge_sort += 1;
                edges.push(branched(key, &false_to, "false", edge_sort));
                edge_sort += 1;
            }
            Pos::Step {
                entry: _,
                node,
                loop_key,
                kind,
                ends_run,
                branches,
                ..
            } => {
                let after = if *ends_run {
                    "end".to_string()
                } else {
                    next.clone()
                };
                match kind.as_str() {
                    "check" => {
                        let pass = resolve_branch(branches, "pass", &after, &step_entry);
                        let fail = resolve_branch(branches, "fail", &after, &step_entry);
                        edges.push(branched(node, &pass, "true", edge_sort));
                        edge_sort += 1;
                        edges.push(branched(node, &fail, "false", edge_sort));
                        edge_sort += 1;
                    }
                    "approval" => {
                        for outcome in ["approved", "rejected"] {
                            let target = branches
                                .iter()
                                .find(|b| b.on == outcome)
                                .and_then(|b| {
                                    b.target
                                        .as_ref()
                                        .and_then(|t| {
                                            step_entry
                                                .get(&format!("{}:{}", t.stage, t.step))
                                                .cloned()
                                        })
                                        .or_else(|| b.end.as_ref().map(|_| "end".to_string()))
                                })
                                .ok_or_else(|| {
                                    classified_validation(
                                        "validation.arguments",
                                        format!("approval step must declare an '{outcome}' branch"),
                                    )
                                })?;
                            edges.push(branched(node, &target, outcome, edge_sort));
                            edge_sort += 1;
                        }
                    }
                    _ => {
                        if let Some(loop_key) = loop_key {
                            edges.push(branched(loop_key, node, "body", edge_sort));
                            edge_sort += 1;
                            edges.push(branched(loop_key, &after, "exit", edge_sort));
                            edge_sort += 1;
                            edges.push(unconditional(node, loop_key, edge_sort));
                            edge_sort += 1;
                        } else {
                            edges.push(unconditional(node, &after, edge_sort));
                            edge_sort += 1;
                        }
                    }
                }
            }
            Pos::End => {}
        }
    }

    Ok(ExecutionGraphInput {
        name: lane.name.clone(),
        description: lane.description.clone(),
        nodes,
        edges,
    })
}

// --- run start -----------------------------------------------------------

/// Evaluates a lane condition (or group) against a context of field values
/// (`$record.<field>` as the bare field name, `$run_data.<field>` as
/// `run_data.<field>` - the same keys the compiled runtime shape uses).
/// An `expression` cannot be evaluated here - that is WP-0710's grammar -
/// and a `$item` reference has no meaning outside a repeating step, so
/// both are refused with a clear error instead of silently passing.
fn evaluate_condition(rule: &ConditionOrGroup, ctx: &HashMap<String, String>) -> AppResult<bool> {
    match rule {
        ConditionOrGroup::Condition(c) => {
            let field_key = match c.field.strip_prefix("$record.") {
                Some(rest) => rest.to_string(),
                None => match c.field.strip_prefix("$run_data.") {
                    Some(rest) => format!("run_data.{rest}"),
                    None => {
                        return Err(classified_validation(
                            "validation.arguments",
                            "a start condition cannot reference $item (no repeating step yet)",
                        ))
                    }
                },
            };
            if !CONDITION_OPERATORS.contains(&c.operator.as_str()) {
                return Err(classified_validation(
                    "validation.arguments",
                    format!("unknown condition operator '{}'", c.operator),
                ));
            }
            let value = match &c.compare_field {
                Some(other) => {
                    let other_key = match other.strip_prefix("$record.") {
                        Some(rest) => rest.to_string(),
                        None => match other.strip_prefix("$run_data.") {
                            Some(rest) => format!("run_data.{rest}"),
                            None => {
                                return Err(classified_validation(
                                    "validation.arguments",
                                    "a start condition cannot compare against $item",
                                ))
                            }
                        },
                    };
                    ctx.get(&other_key).cloned().unwrap_or_default()
                }
                None => match c.value.as_ref() {
                    Some(Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => {
                        return Err(classified_validation(
                            "validation.arguments",
                            "a start condition has neither a value nor a compare_field",
                        ))
                    }
                },
            };
            Ok(condition_matches(&field_key, &c.operator, &value, ctx))
        }
        ConditionOrGroup::Group(g) => {
            let mut results = Vec::new();
            for child in &g.conditions {
                results.push(evaluate_condition(child, ctx)?);
            }
            Ok(match g.match_type.as_str() {
                "any" => results.iter().any(|r| *r),
                _ => results.iter().all(|r| *r),
            })
        }
    }
}

/// The display label of a run (ADR-0024 §11): `{anchor display label} ·
/// {lane name} · stage {n} of {N}`. The anchor's display label is its
/// display number where it has one (a purchase order's `PO-2026-001`),
/// else its primary name. A finished run shows its outcome after the
/// label; the runner (WP-0707) composes that.
pub fn lane_label(
    lane: &ProcessLane,
    anchor_display_label: &str,
    stage_index: usize,
    stage_count: usize,
) -> String {
    format!(
        "{anchor_display_label} · {} · stage {} of {stage_count}",
        lane.name,
        stage_index + 1
    )
}

/// Starts a run of the lane's latest published version on one anchor
/// record. Refused when the lane has no published version, when the record
/// is not of the lane's object, when the lane's start condition fails, or
/// when the record already has an active run of this lane - or of a lane
/// this one excludes (ADR-0024 §6, one active run per lane per record).
/// The run keeps the version it starts with: its `graph_id` is the
/// published version's compiled graph. The anchor columns and the empty
/// run data (`{}`) are stamped on the run row right after the graph
/// runtime creates it, in the same call - the runtime's own `start_run`
/// does not know about lanes.
pub async fn start_lane_run(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    lane_key: &str,
    anchor_record_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<GraphRun> {
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let version =
        process_lane_repo::latest_published_version(conn, &lane.id)?.ok_or_else(|| {
            classified_validation(
                "validation.arguments",
                format!("lane '{lane_key}' has no published version to run"),
            )
        })?;
    let graph_id = version.graph_id.clone().ok_or_else(|| {
        classified_validation(
            "validation.arguments",
            format!(
                "published version {} of lane '{lane_key}' has no compiled graph",
                version.version
            ),
        )
    })?;
    let record = custom_record_service::get(conn, anchor_record_id)?;
    if record.workspace_id != workspace_id || record.object_key != lane.object_key {
        return Err(classified_validation(
            "validation.arguments",
            "the anchor record is not a record of this lane's object",
        ));
    }
    if let Some(active) = process_lane_repo::find_active_run_for_lane_record(
        conn,
        workspace_id,
        lane_key,
        anchor_record_id,
    )? {
        return Err(AppError::Conflict(format!(
            "lane '{lane_key}' already has an active run ({active}) on this record"
        )));
    }
    let definition = parse_definition(&version.definition_json)?;
    for excluded in definition.lane.excludes.as_deref().unwrap_or_default() {
        if let Some(active) = process_lane_repo::find_active_run_for_lane_record(
            conn,
            workspace_id,
            excluded,
            anchor_record_id,
        )? {
            return Err(AppError::Conflict(format!(
                "lane '{lane_key}' excludes '{excluded}', which already has an active run ({active}) on this record"
            )));
        }
    }
    if let Some(start_condition) = &definition.lane.start_condition {
        let mut ctx =
            builtin_field_service::field_values(conn, &lane.object_key, anchor_record_id)?;
        for (key, value) in
            crate::repositories::custom_field_repo::get_values(conn, anchor_record_id)?
        {
            ctx.insert(key, value);
        }
        // Run data is empty at the start - no `run_data.*` keys yet.
        if !evaluate_condition(start_condition, &ctx)? {
            return Err(classified_validation(
                "validation.arguments",
                format!(
                    "lane '{lane_key}' cannot start on this record: its start condition is not met"
                ),
            ));
        }
    }
    let mut trigger_fields =
        builtin_field_service::field_values(conn, &lane.object_key, anchor_record_id)?;
    for (key, value) in crate::repositories::custom_field_repo::get_values(conn, anchor_record_id)?
    {
        trigger_fields.insert(key, value);
    }
    // Flat record fields at the top level (the condition executor looks
    // them up by their bare name), plus the empty run data it flattens to
    // `run_data.<field>` keys as values are entered.
    let mut trigger_object = Map::new();
    for (key, value) in trigger_fields {
        trigger_object.insert(key, json!(value));
    }
    trigger_object.insert("run_data".into(), json!({}));
    let trigger_input =
        serde_json::to_string(&Value::Object(trigger_object)).unwrap_or_else(|_| "{}".to_string());
    let run = graph_runtime_service::start_run(
        conn,
        workspace_id,
        master_key,
        &graph_id,
        &trigger_input,
        actor_user_id,
        None,
        Some(&lane.object_key),
        Some(anchor_record_id),
    )
    .await?;
    graph_run_repo::set_lane_anchor(conn, &run.id, &lane.object_key, anchor_record_id, "{}")?;
    Ok(graph_run_repo::get_run(conn, &run.id)?.expect("just started"))
}
