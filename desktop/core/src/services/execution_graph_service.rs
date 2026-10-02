//! AI Agent Platform v2, Phase 3: CRUD + publish-lifecycle for
//! `ai_execution_graphs`, and the compatibility-shape mapping that lets an
//! existing Workflow or Pipeline be expressed as an equivalent graph
//! *shape* on this engine without moving their own storage or execution
//! path (`workflow_service.rs`/`ai_orchestration_service.rs` keep running
//! completely unchanged - see this module's own tests in
//! `tests/execution_graph_topologies.rs` for the parity checks that prove
//! the mapping is faithful). See `services::graph_runtime_service` for
//! execution once a graph is published.
//!
//! A graph is a directed graph over `NODE_TYPES` (`models::execution_graph`).
//! `validate_for_publish` enforces the shape invariants
//! `graph_runtime_service`'s executor relies on:
//! - exactly one `trigger` node, reachable-from-trigger covers every node,
//!   at least one `end` node is reachable;
//! - a `trigger`/`action`/`agent`/`delay`/`transform` node has exactly one
//!   unconditional outgoing edge; `condition` has exactly `true`+`false`;
//!   `approval` has exactly `approved`+`rejected`; `loop` has exactly
//!   `body`+`exit`; `parallel_split` has 2+ unconditional edges; `join` has
//!   2+ incoming edges; `end` has none;
//! - every `loop` node's `config_json` declares a positive `max_iterations`
//!   (the bound `graph_runtime_service` enforces per-iteration);
//! - no cycle exists other than one that re-enters a `loop` node (that
//!   node's own bounded iteration, not a runaway loop) - see
//!   `find_unbounded_cycle`'s own doc comment for exactly how this is
//!   distinguished from a genuine unbounded cycle.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde_json::json;

use crate::domain::{AppError, AppResult};
use crate::models::ai_agent_pipeline::AiAgentPipeline;
use crate::models::execution_graph::{is_single_unconditional_outgoing, ExecutionGraph, ExecutionGraphInput, GraphEdgeInput, GraphNodeInput, NODE_TYPES};
use crate::models::workflow::WorkflowDefinition;
use crate::repositories::execution_graph_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_input(input: &ExecutionGraphInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }
    if input.nodes.is_empty() {
        return Err(AppError::Validation("a graph needs at least one node".into()));
    }
    let mut seen_keys = HashSet::new();
    for node in &input.nodes {
        if node.node_key.trim().is_empty() {
            return Err(AppError::Validation("every node needs a non-empty node_key".into()));
        }
        if !seen_keys.insert(node.node_key.as_str()) {
            return Err(AppError::Validation(format!("duplicate node_key '{}'", node.node_key)));
        }
        if !NODE_TYPES.contains(&node.node_type.as_str()) {
            return Err(AppError::Validation(format!("'{}' is not a valid node_type", node.node_type)));
        }
        serde_json::from_str::<serde_json::Value>(&node.config_json)
            .map_err(|e| AppError::Validation(format!("node '{}' has invalid config_json: {e}", node.node_key)))?;
    }
    for edge in &input.edges {
        if !seen_keys.contains(edge.from_node_key.as_str()) {
            return Err(AppError::Validation(format!("edge references unknown node_key '{}'", edge.from_node_key)));
        }
        if !seen_keys.contains(edge.to_node_key.as_str()) {
            return Err(AppError::Validation(format!("edge references unknown node_key '{}'", edge.to_node_key)));
        }
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &ExecutionGraphInput, actor_user_id: Option<&str>) -> AppResult<ExecutionGraph> {
    require_admin(conn, actor_user_id)?;
    validate_input(input)?;
    Ok(execution_graph_repo::create(conn, &crate::domain::ids::new_uuid(), workspace_id, input, actor_user_id)?)
}

fn get_owned(conn: &Connection, id: &str, workspace_id: &str) -> AppResult<ExecutionGraph> {
    let graph = execution_graph_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Execution graph".into()))?;
    if graph.workspace_id != workspace_id {
        return Err(AppError::NotFound("Execution graph".into()));
    }
    Ok(graph)
}

pub fn get(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<ExecutionGraph> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, id, workspace_id)
}

pub fn list(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<ExecutionGraph>> {
    require_admin(conn, actor_user_id)?;
    Ok(execution_graph_repo::list(conn, workspace_id)?)
}

/// Only a `draft` graph may have its nodes/edges replaced - the same
/// immutable-once-published rule `agent_version_service` enforces for a
/// Published agent version, so a `ai_runs` row's `graph_id` always points
/// at the exact shape that produced it.
pub fn update(conn: &Connection, id: &str, workspace_id: &str, input: &ExecutionGraphInput, actor_user_id: Option<&str>) -> AppResult<ExecutionGraph> {
    require_admin(conn, actor_user_id)?;
    let graph = get_owned(conn, id, workspace_id)?;
    if graph.status != "draft" {
        return Err(AppError::Validation("only a draft graph can be edited - disable it and create a new version instead".into()));
    }
    validate_input(input)?;
    Ok(execution_graph_repo::update(conn, id, input, actor_user_id)?)
}

pub fn set_disabled(conn: &Connection, id: &str, workspace_id: &str, disabled: bool, actor_user_id: Option<&str>) -> AppResult<ExecutionGraph> {
    require_admin(conn, actor_user_id)?;
    let graph = get_owned(conn, id, workspace_id)?;
    let new_status = if disabled { "disabled".to_string() } else if graph.status == "disabled" { "published".to_string() } else { graph.status.clone() };
    execution_graph_repo::set_status(conn, id, &new_status, actor_user_id)?;
    get_owned(conn, id, workspace_id)
}

/// Validates the graph's shape and, if it passes, flips `status` to
/// `published` (immutable from then on - see `update`'s own doc comment).
pub fn publish(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<ExecutionGraph> {
    require_admin(conn, actor_user_id)?;
    let graph = get_owned(conn, id, workspace_id)?;
    validate_for_publish(&graph)?;
    execution_graph_repo::set_status(conn, id, "published", actor_user_id)?;
    get_owned(conn, id, workspace_id)
}

/// Runs every shape check this module's own doc comment lists. Returns the
/// first violation found rather than accumulating all of them - a
/// malformed graph is corrected one problem at a time via the same
/// draft-edit-republish loop any other validated draft resource in this
/// codebase already uses (a Business Rule, a Screen Layout).
pub fn validate_for_publish(graph: &ExecutionGraph) -> AppResult<()> {
    if graph.nodes.is_empty() {
        return Err(AppError::Validation("a graph needs at least one node".into()));
    }
    let triggers: Vec<&str> = graph.nodes.iter().filter(|n| n.node_type == "trigger").map(|n| n.id.as_str()).collect();
    if triggers.len() != 1 {
        return Err(AppError::Validation(format!("a graph must have exactly one trigger node (found {})", triggers.len())));
    }
    let trigger_id = triggers[0];

    let mut outgoing: HashMap<&str, Vec<(&str, Option<&str>)>> = HashMap::new();
    let mut incoming_count: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        outgoing.entry(edge.from_node_id.as_str()).or_default().push((edge.to_node_id.as_str(), edge.branch_label.as_deref()));
        *incoming_count.entry(edge.to_node_id.as_str()).or_insert(0) += 1;
    }

    let node_type_of: HashMap<&str, &str> = graph.nodes.iter().map(|n| (n.id.as_str(), n.node_type.as_str())).collect();

    for node in &graph.nodes {
        let outs = outgoing.get(node.id.as_str()).cloned().unwrap_or_default();
        match node.node_type.as_str() {
            "end" => {
                if !outs.is_empty() {
                    return Err(AppError::Validation(format!("End node '{}' must have no outgoing edges", node.node_key)));
                }
            }
            t if is_single_unconditional_outgoing(t) => {
                if outs.len() != 1 || outs[0].1.is_some() {
                    return Err(AppError::Validation(format!("'{}' node '{}' must have exactly one unconditional outgoing edge", t, node.node_key)));
                }
            }
            "condition" => {
                let labels: HashSet<&str> = outs.iter().filter_map(|(_, l)| *l).collect();
                if outs.len() != 2 || !labels.contains("true") || !labels.contains("false") {
                    return Err(AppError::Validation(format!("Condition node '{}' must have exactly a 'true' and a 'false' outgoing edge", node.node_key)));
                }
            }
            "evaluate_result" => {
                let labels: HashSet<&str> = outs.iter().filter_map(|(_, l)| *l).collect();
                if outs.len() != 2 || !labels.contains("pass") || !labels.contains("fail") {
                    return Err(AppError::Validation(format!("Evaluate Result node '{}' must have exactly a 'pass' and a 'fail' outgoing edge", node.node_key)));
                }
            }
            "approval" => {
                let labels: HashSet<&str> = outs.iter().filter_map(|(_, l)| *l).collect();
                if outs.len() != 2 || !labels.contains("approved") || !labels.contains("rejected") {
                    return Err(AppError::Validation(format!("Approval node '{}' must have exactly an 'approved' and a 'rejected' outgoing edge", node.node_key)));
                }
            }
            "loop" => {
                let labels: HashSet<&str> = outs.iter().filter_map(|(_, l)| *l).collect();
                if outs.len() != 2 || !labels.contains("body") || !labels.contains("exit") {
                    return Err(AppError::Validation(format!("Loop node '{}' must have exactly a 'body' and an 'exit' outgoing edge", node.node_key)));
                }
                let config: serde_json::Value = serde_json::from_str(&node.config_json).unwrap_or(json!({}));
                let max_iterations = config.get("max_iterations").and_then(|v| v.as_i64());
                if !matches!(max_iterations, Some(n) if n > 0) {
                    return Err(AppError::Validation(format!("Loop node '{}' needs a positive integer max_iterations in its config", node.node_key)));
                }
            }
            "router" => {
                if outs.is_empty() || outs.iter().any(|(_, l)| l.is_none()) {
                    return Err(AppError::Validation(format!("Router node '{}' needs one or more branch-labeled outgoing edges", node.node_key)));
                }
            }
            "parallel_split" => {
                if outs.len() < 2 {
                    return Err(AppError::Validation(format!("Parallel Split node '{}' needs at least two outgoing edges", node.node_key)));
                }
            }
            "join" => {
                if incoming_count.get(node.id.as_str()).copied().unwrap_or(0) < 2 {
                    return Err(AppError::Validation(format!("Join node '{}' needs at least two incoming edges", node.node_key)));
                }
            }
            _ => {}
        }
    }

    // Reachability: every node must be reachable from the trigger by
    // forward traversal (using every edge, including a Loop's own
    // re-entry back-edge, which is a perfectly normal forward path from
    // whichever node points into it).
    let mut reachable: HashSet<&str> = HashSet::new();
    let mut queue = vec![trigger_id];
    reachable.insert(trigger_id);
    while let Some(cur) = queue.pop() {
        for (to, _) in outgoing.get(cur).cloned().unwrap_or_default() {
            if reachable.insert(to) {
                queue.push(to);
            }
        }
    }
    let unreachable: Vec<&str> = graph.nodes.iter().map(|n| n.node_key.as_str()).zip(graph.nodes.iter().map(|n| n.id.as_str())).filter(|(_, id)| !reachable.contains(id)).map(|(key, _)| key).collect();
    if !unreachable.is_empty() {
        return Err(AppError::Validation(format!("unreachable node(s) from the trigger: {}", unreachable.join(", "))));
    }
    if !graph.nodes.iter().any(|n| n.node_type == "end" && reachable.contains(n.id.as_str())) {
        return Err(AppError::Validation("a graph must have at least one reachable End node".into()));
    }

    if let Some(cycle) = find_unbounded_cycle(&graph.nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), &outgoing, &node_type_of) {
        return Err(AppError::Validation(format!("unbounded cycle detected (not passing through a Loop node's own re-entry): {}", cycle.join(" -> "))));
    }

    Ok(())
}

/// Cycle detection that treats an edge landing on a `loop`-type node as
/// that loop's own legitimate re-entry point, not a graph-structure cycle -
/// exactly how `graph_from_pipeline`'s own peer_review mapping uses one
/// (see this module's top doc comment). Implementation: run standard
/// DFS white/gray/black cycle detection, but never traverse an edge whose
/// *target* is a `loop` node as part of extending the current path being
/// explored - so the only way such an edge can close a cycle is if the
/// loop node itself is already on the stack (its own forward `body` path
/// having been explored earlier), which is precisely the bounded-iteration
/// shape this engine allows. A cycle that doesn't route back through a
/// `loop` node's own stack frame is reported.
fn find_unbounded_cycle<'a>(
    node_ids: &[&'a str],
    outgoing: &HashMap<&'a str, Vec<(&'a str, Option<&'a str>)>>,
    node_type_of: &HashMap<&'a str, &'a str>,
) -> Option<Vec<&'a str>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: HashMap<&str, Color> = node_ids.iter().map(|id| (*id, Color::White)).collect();
    let mut stack: Vec<&str> = Vec::new();

    fn visit<'a>(
        node: &'a str,
        outgoing: &HashMap<&'a str, Vec<(&'a str, Option<&'a str>)>>,
        node_type_of: &HashMap<&'a str, &'a str>,
        color: &mut HashMap<&'a str, Color>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<&'a str>> {
        color.insert(node, Color::Gray);
        stack.push(node);
        for (to, _) in outgoing.get(node).cloned().unwrap_or_default() {
            match color.get(to).copied().unwrap_or(Color::White) {
                Color::White => {
                    if let Some(cycle) = visit(to, outgoing, node_type_of, color, stack) {
                        return Some(cycle);
                    }
                }
                Color::Gray => {
                    // A back-edge landing on a `loop` node closes a
                    // legitimate bounded iteration, not an unbounded cycle.
                    if node_type_of.get(to).copied() != Some("loop") {
                        let start = stack.iter().position(|n| n == &to).unwrap_or(0);
                        let mut cycle: Vec<&str> = stack[start..].to_vec();
                        cycle.push(to);
                        return Some(cycle);
                    }
                }
                Color::Black => {}
            }
        }
        stack.pop();
        color.insert(node, Color::Black);
        None
    }

    for &id in node_ids {
        if color.get(id).copied().unwrap_or(Color::White) == Color::White {
            if let Some(cycle) = visit(id, outgoing, node_type_of, &mut color, &mut stack) {
                return Some(cycle);
            }
        }
    }
    None
}

// --- compatibility-shape mapping: existing Workflow -> graph shape ------

/// Builds the graph shape an existing `WorkflowDefinition` is equivalent
/// to: `trigger -> [conditions ->] action_1 -> action_2 -> ... -> end`.
/// `workflow_service.rs`'s own trigger/condition/action execution keeps
/// running completely unchanged - this produces a *second*, independent
/// representation of the same logic for `graph_runtime_service` to run,
/// so the parity test in `tests/execution_graph_topologies.rs` can prove
/// the two agree on the same input, not so production traffic switches
/// over to it.
pub fn graph_from_workflow(wf: &WorkflowDefinition) -> ExecutionGraphInput {
    let mut nodes = vec![GraphNodeInput { node_key: "trigger".into(), node_type: "trigger".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order: 0 }];
    let mut edges = Vec::new();
    let mut sort_order = 1i64;

    let mut prev_key = "trigger".to_string();
    if !wf.conditions.is_empty() {
        let cfg = json!({
            "match_type": wf.match_type,
            "conditions": wf.conditions.iter().map(|c| json!({
                "group_id": c.group_id, "field_key": c.field_key, "operator": c.operator, "value": c.value,
            })).collect::<Vec<_>>(),
        });
        nodes.push(GraphNodeInput { node_key: "conditions".into(), node_type: "condition".into(), config_json: cfg.to_string(), position_x: None, position_y: None, sort_order });
        edges.push(GraphEdgeInput { from_node_key: prev_key.clone(), to_node_key: "conditions".into(), branch_label: None, sort_order });
        sort_order += 1;
        edges.push(GraphEdgeInput { from_node_key: "conditions".into(), to_node_key: "end".into(), branch_label: Some("false".into()), sort_order });
        sort_order += 1;
        prev_key = "conditions".to_string();
    }

    let branch_from_conditions = prev_key == "conditions";
    for (i, action) in wf.actions.iter().enumerate() {
        let key = format!("action_{i}");
        let cfg = json!({"action_type": action.action_type, "params_json": action.params_json});
        nodes.push(GraphNodeInput { node_key: key.clone(), node_type: "action".into(), config_json: cfg.to_string(), position_x: None, position_y: None, sort_order });
        let label = if i == 0 && branch_from_conditions { Some("true".to_string()) } else { None };
        edges.push(GraphEdgeInput { from_node_key: prev_key.clone(), to_node_key: key.clone(), branch_label: label, sort_order });
        sort_order += 1;
        prev_key = key;
    }
    if wf.actions.is_empty() && branch_from_conditions {
        edges.push(GraphEdgeInput { from_node_key: prev_key.clone(), to_node_key: "end".into(), branch_label: Some("true".into()), sort_order });
        sort_order += 1;
    } else {
        edges.push(GraphEdgeInput { from_node_key: prev_key, to_node_key: "end".into(), branch_label: None, sort_order });
        sort_order += 1;
    }

    nodes.push(GraphNodeInput { node_key: "end".into(), node_type: "end".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order });
    ExecutionGraphInput { name: format!("{} (graph shape)", wf.name), description: wf.description.clone(), nodes, edges }
}

// --- compatibility-shape mapping: existing Pipeline -> graph shape ------

/// A real `PipelineStep.input_template` commonly contains the literal
/// token `{{previous_output}}`, which `ai_orchestration_service` resolves
/// specially (the immediately preceding step's final answer). The graph
/// engine's `graph_runtime_service::lookup_token` has no such special
/// token - it only understands `trigger_input` and `node_key`/`node_key.
/// field`. Left untranslated, every mapped step after the first would
/// silently resolve `{{previous_output}}` to an empty string instead of
/// the real prior output, breaking the compatibility-shape mapping's own
/// parity claim. This rewrites the literal token to the graph-relative
/// equivalent (`{{<prev_agent_key>.output}}`) before the node is built;
/// `prev_agent_key: None` (the first step in a chain) leaves the token
/// untouched, which resolves to the same empty string the original gives
/// a first step with no prior output - so the two behaviors agree without
/// a special case.
fn resolve_previous_output_token(template: &str, prev_agent_key: Option<&str>) -> String {
    match prev_agent_key {
        Some(key) => template.replace("{{previous_output}}", &format!("{{{{{key}.output}}}}")),
        None => template.to_string(),
    }
}

fn agent_node(key: &str, agent_id: &str, input_template: &str, sort_order: i64) -> GraphNodeInput {
    GraphNodeInput {
        node_key: key.into(),
        node_type: "agent".into(),
        config_json: json!({"agent_id": agent_id, "input_template": input_template}).to_string(),
        position_x: None,
        position_y: None,
        sort_order,
    }
}

/// `sequential` -> a linear Agent-node chain, an `approval` node spliced in
/// after any step whose `requires_approval` is set (see
/// `ai_orchestration_service::run_sequential_from`'s own pause behavior,
/// which this mirrors). `consensus` -> `parallel_split` into every
/// candidate step, `join(all)`, then the synthesizer step (optionally
/// gated by its own `approval` node the same way). `peer_review` -> a
/// bounded `loop` (see this module's top doc comment for exactly how its
/// body/exit shape maps onto `ai_orchestration_service::run_peer_review_
/// from`'s drafter/reviewer/APPROVED-marker loop).
pub fn graph_from_pipeline(p: &AiAgentPipeline) -> AppResult<ExecutionGraphInput> {
    let mut nodes = vec![GraphNodeInput { node_key: "trigger".into(), node_type: "trigger".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order: 0 }];
    let mut edges = Vec::new();
    let mut sort_order = 1i64;

    match p.topology.as_str() {
        "sequential" => {
            let mut prev_key = "trigger".to_string();
            let mut prev_agent_key: Option<String> = None;
            for (i, step) in p.steps.iter().enumerate() {
                let key = format!("agent_{i}");
                let template = resolve_previous_output_token(&step.input_template, prev_agent_key.as_deref());
                nodes.push(agent_node(&key, &step.agent_id, &template, sort_order));
                edges.push(GraphEdgeInput { from_node_key: prev_key.clone(), to_node_key: key.clone(), branch_label: None, sort_order });
                sort_order += 1;
                prev_agent_key = Some(key.clone());
                if step.requires_approval {
                    let approval_key = format!("approval_{i}");
                    nodes.push(GraphNodeInput { node_key: approval_key.clone(), node_type: "approval".into(), config_json: json!({"subject_type": "pipeline_step"}).to_string(), position_x: None, position_y: None, sort_order });
                    edges.push(GraphEdgeInput { from_node_key: key.clone(), to_node_key: approval_key.clone(), branch_label: None, sort_order });
                    sort_order += 1;
                    edges.push(GraphEdgeInput { from_node_key: approval_key.clone(), to_node_key: "end".into(), branch_label: Some("rejected".into()), sort_order });
                    sort_order += 1;
                    prev_key = approval_key;
                } else {
                    prev_key = key;
                }
            }
            let last_label = if p.steps.last().is_some_and(|s| s.requires_approval) { Some("approved".to_string()) } else { None };
            edges.push(GraphEdgeInput { from_node_key: prev_key, to_node_key: "end".into(), branch_label: last_label, sort_order });
        }
        // A real consensus synthesizer step's `input_template` commonly
        // uses `{{candidate_outputs}}` - `ai_orchestration_service`'s own
        // special token that formats every candidate's answer into a
        // numbered list ("Candidate 1: ...\nCandidate 2: ..."). Unlike
        // `{{previous_output}}` (rewritten above to a plain `{{node_key.
        // field}}` lookup the graph engine already understands), there is
        // no graph-relative equivalent for an *aggregation-and-formatting*
        // token without a new runtime primitive - left as a known,
        // documented gap in this mapping rather than a silent
        // approximation. A synthesizer template using `{{candidate_
        // outputs}}` maps onto a graph whose synthesizer node receives the
        // literal, unresolved token text; the real Pipeline keeps running
        // unaffected on `ai_orchestration_service`. A template written
        // against `{{split.<i>.output}}`-style direct node references
        // would resolve correctly on this engine today.
        "consensus" => {
            if p.steps.len() < 2 {
                return Err(AppError::Validation("a consensus Pipeline needs at least a candidate and a synthesizer step".into()));
            }
            let (candidates, synth) = p.steps.split_at(p.steps.len() - 1);
            let synth = &synth[0];
            nodes.push(GraphNodeInput { node_key: "split".into(), node_type: "parallel_split".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order });
            edges.push(GraphEdgeInput { from_node_key: "trigger".into(), to_node_key: "split".into(), branch_label: None, sort_order });
            sort_order += 1;
            nodes.push(GraphNodeInput { node_key: "join".into(), node_type: "join".into(), config_json: json!({"mode": "all"}).to_string(), position_x: None, position_y: None, sort_order });
            for (i, step) in candidates.iter().enumerate() {
                let key = format!("candidate_{i}");
                nodes.push(agent_node(&key, &step.agent_id, &step.input_template, sort_order));
                edges.push(GraphEdgeInput { from_node_key: "split".into(), to_node_key: key.clone(), branch_label: None, sort_order });
                sort_order += 1;
                edges.push(GraphEdgeInput { from_node_key: key, to_node_key: "join".into(), branch_label: None, sort_order });
                sort_order += 1;
            }
            nodes.push(agent_node("synthesizer", &synth.agent_id, &synth.input_template, sort_order));
            edges.push(GraphEdgeInput { from_node_key: "join".into(), to_node_key: "synthesizer".into(), branch_label: None, sort_order });
            sort_order += 1;
            if synth.requires_approval {
                nodes.push(GraphNodeInput { node_key: "approval".into(), node_type: "approval".into(), config_json: json!({"subject_type": "pipeline_step"}).to_string(), position_x: None, position_y: None, sort_order });
                edges.push(GraphEdgeInput { from_node_key: "synthesizer".into(), to_node_key: "approval".into(), branch_label: None, sort_order });
                sort_order += 1;
                edges.push(GraphEdgeInput { from_node_key: "approval".into(), to_node_key: "end".into(), branch_label: Some("approved".into()), sort_order });
                sort_order += 1;
                edges.push(GraphEdgeInput { from_node_key: "approval".into(), to_node_key: "end".into(), branch_label: Some("rejected".into()), sort_order });
            } else {
                edges.push(GraphEdgeInput { from_node_key: "synthesizer".into(), to_node_key: "end".into(), branch_label: None, sort_order });
            }
        }
        "peer_review" => {
            if p.steps.len() != 2 {
                return Err(AppError::Validation("a peer_review Pipeline needs exactly a drafter and a reviewer step".into()));
            }
            let drafter = &p.steps[0];
            let reviewer = &p.steps[1];
            let max_iterations = super::ai_orchestration_service::MAX_PEER_REVIEW_ROUNDS;
            nodes.push(GraphNodeInput {
                node_key: "loop".into(),
                node_type: "loop".into(),
                config_json: json!({"max_iterations": max_iterations}).to_string(),
                position_x: None,
                position_y: None,
                sort_order,
            });
            edges.push(GraphEdgeInput { from_node_key: "trigger".into(), to_node_key: "loop".into(), branch_label: None, sort_order });
            sort_order += 1;
            // `{{previous_output}}` in the drafter's template is the prior
            // round's reviewer feedback (`ai_orchestration_service::
            // run_peer_review_from`'s own semantics); on the first round
            // there is no reviewer output yet, so leaving the token
            // untranslated (via `resolve_previous_output_token`'s `None`
            // case) correctly resolves to empty the same way. In the
            // reviewer's own template, `{{previous_output}}` is always the
            // latest drafter output - a fixed node_key since this is a
            // static graph re-entering the same `drafter`/`reviewer` nodes
            // each round, not a per-round distinct node.
            let drafter_template = resolve_previous_output_token(&drafter.input_template, Some("reviewer"));
            nodes.push(agent_node("drafter", &drafter.agent_id, &drafter_template, sort_order));
            edges.push(GraphEdgeInput { from_node_key: "loop".into(), to_node_key: "drafter".into(), branch_label: Some("body".into()), sort_order });
            sort_order += 1;
            let reviewer_template = resolve_previous_output_token(&reviewer.input_template, Some("drafter"));
            nodes.push(agent_node("reviewer", &reviewer.agent_id, &reviewer_template, sort_order));
            edges.push(GraphEdgeInput { from_node_key: "drafter".into(), to_node_key: "reviewer".into(), branch_label: None, sort_order });
            sort_order += 1;
            // An Agent node's recorded output is always `{"output": <raw
            // text>}` (see `graph_runtime_service`'s own doc comment) -
            // there is no structured "approved" field to check. This
            // mirrors `ai_orchestration_service::run_peer_review_from`'s
            // own check (the reviewer's answer, trimmed and uppercased,
            // starting with `PEER_REVIEW_APPROVAL_MARKER`) as closely as a
            // declarative condition can: `starts_with` is case-sensitive
            // here (`domain::conditions::condition_matches` does no case
            // folding), so a graph-mapped peer_review run expects the
            // reviewer's own literal text to start with "APPROVED" -
            // slightly stricter than the original's case-insensitive
            // check, a known, narrow difference from this mapping alone.
            nodes.push(GraphNodeInput {
                node_key: "review_check".into(),
                node_type: "condition".into(),
                config_json: json!({"match_type": "all", "conditions": [{"group_id": null, "field_key": "reviewer.output", "operator": "starts_with", "value": "APPROVED"}]}).to_string(),
                position_x: None,
                position_y: None,
                sort_order,
            });
            edges.push(GraphEdgeInput { from_node_key: "reviewer".into(), to_node_key: "review_check".into(), branch_label: None, sort_order });
            sort_order += 1;
            edges.push(GraphEdgeInput { from_node_key: "review_check".into(), to_node_key: "end".into(), branch_label: Some("true".into()), sort_order });
            sort_order += 1;
            edges.push(GraphEdgeInput { from_node_key: "review_check".into(), to_node_key: "loop".into(), branch_label: Some("false".into()), sort_order });
            sort_order += 1;
            if reviewer.requires_approval {
                // A pausing reviewer step is not modeled further in this
                // phase's mapping (an already-narrow edge case for a
                // peer_review topology); the mapped graph runs it
                // unattended. Real Pipelines using this combination
                // continue to run on `ai_orchestration_service` unchanged.
            }
            edges.push(GraphEdgeInput { from_node_key: "loop".into(), to_node_key: "end".into(), branch_label: Some("exit".into()), sort_order });
        }
        other => return Err(AppError::Validation(format!("unknown pipeline topology '{other}'"))),
    }

    nodes.push(GraphNodeInput { node_key: "end".into(), node_type: "end".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order: sort_order + 1 });
    Ok(ExecutionGraphInput { name: format!("{} (graph shape)", p.name), description: p.description.clone(), nodes, edges })
}

/// Persists the result of `graph_from_workflow`/`graph_from_pipeline` as a
/// new, already-published graph (the mapping is deterministic and
/// pre-validated by construction against a real saved Workflow/Pipeline,
/// so there's no draft-review step here the way a hand-authored graph
/// gets) with `source_kind`/`source_id` set, for the parity tests
/// (and, later, an explicit "run this Workflow/Pipeline on the new engine
/// instead" opt-in) to run against.
pub fn create_from_source(conn: &Connection, workspace_id: &str, input: ExecutionGraphInput, source_kind: &str, source_id: &str, actor_user_id: Option<&str>) -> AppResult<ExecutionGraph> {
    let id = crate::domain::ids::new_uuid();
    let graph = execution_graph_repo::create_with_source(conn, &id, workspace_id, &input, source_kind, source_id, actor_user_id)?;
    validate_for_publish(&graph)?;
    execution_graph_repo::set_status(conn, &id, "published", actor_user_id)?;
    Ok(execution_graph_repo::get(conn, &id)?.expect("just published"))
}
