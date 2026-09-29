//! AI Agent Platform v2, Phase 3: the Shared Execution Graph Runtime's
//! step-by-step executor. Walks a published `ExecutionGraph` one node at a
//! time from `ai_runs.current_node_id`, checkpointing (`graph_run_repo::
//! checkpoint`/`set_waiting_approval`/`set_waiting_scheduled`) after every
//! node transition so a run surviving a process restart is just calling
//! `resume_run` again - it re-reads `current_node_id`/`context_json` off
//! `ai_runs` and continues from exactly there, the same durability
//! property `ai_agent_runs` already gives a paused Pipeline, generalized
//! to an arbitrary graph position.
//!
//! Node-type execution:
//! - `trigger`: no-op, immediately continues to its one outgoing edge.
//! - `condition`/`router`: evaluates `models::workflow`-shaped conditions
//!   (via `domain::conditions::conditions_match`, the exact primitive
//!   `workflow_service`/`business_rule_service` already share) against
//!   `context` + `trigger_input`, and follows the matching branch-labeled
//!   edge.
//! - `action`: calls `workflow_service::apply_action` directly - the exact
//!   same dispatch a Workflow's own action list already uses, so an
//!   Action node and a Workflow action can never drift onto two
//!   implementations of the same `action_type`. Requires the run to carry
//!   `source_entity_type`/`source_entity_id` (i.e. this run was triggered
//!   from a record event), the same coupling a Workflow's own actions have.
//! - `agent`: resolves `{{trigger_input}}`/`{{node_key}}`/`{{node_key.
//!   field}}` in `input_template` against `context`, then calls
//!   `chat_service::run_agent_once_with_text` - the same Tool-Call
//!   Firewall path (policy checks, audit) every other agent invocation in
//!   this codebase already goes through.
//! - `approval`: creates a durable `ai_approvals` row (Phase 1's
//!   `approval_service`) and pauses the run in `waiting_approval` -
//!   `resolve_approval` is the only way out, continuing down the
//!   `approved`/`rejected` edge. This is the one real approval primitive
//!   this engine consolidates onto; see this module's own note in
//!   `services::ai_orchestration_service`'s doc comment about the three
//!   separate, unmerged pause-for-human mechanisms that predate this
//!   engine - a graph's Approval node is deliberately the only one of the
//!   three built on the durable, generic `ai_approvals` table.
//! - `delay`: pauses in `waiting_scheduled` until `resume_due_delays`'s
//!   sweep (mirroring `workflow_service::run_scheduled`'s own pattern)
//!   finds it past its `resume_at`.
//! - `transform`: resolves a small `{"set": {"field": "{{template}}"}}`
//!   config against `context`, writing the resolved object under its own
//!   `node_key`.
//! - `loop`: bounded iteration - each re-entry (via the `false`/back edge
//!   of whatever the loop body's own condition is) increments an attempt
//!   counter derived from how many `ai_run_nodes` rows already exist for
//!   this `(run_id, node_id)`; once that exceeds `max_iterations`, the
//!   `exit` edge is taken instead of `body`.
//! - `parallel_split`/`join`: **this phase's one explicit scope
//!   simplification** - the desktop runtime is one local worker (see this
//!   issue's own "Concurrency" scope note), so a Parallel Split's branches
//!   are walked one at a time, in order, down to their shared Join node
//!   (each branch must be a plain non-branching chain of `action`/`agent`/
//!   `delay`/`transform` nodes - a branch containing its own nested
//!   `condition`/`router`/`parallel_split`/`loop`/`approval` fails the run
//!   at execution time with a clear error, not silently). Real
//!   concurrency has no observable effect here since nothing races a
//!   wall clock; only `join`'s `mode` (`all`/`first_successful`/`n_of_m`)
//!   changes which combination of already-completed branch outcomes counts
//!   as a pass. `timeout_partial` has no real deadline to race without a
//!   worker pool, so it degrades to `first_successful` - documented here,
//!   not silently reinterpreted.
//! - `end`: terminal - the run completes without dispatching an `end`
//!   node's own executor (there is nothing to execute).

use std::collections::HashMap;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::ids::now_iso;
use crate::domain::{AppError, AppResult};
use crate::models::ai_approval::AiApprovalInput;
use crate::models::execution_graph::{is_single_unconditional_outgoing, ExecutionGraph, GraphNode};
use crate::models::graph_run::GraphRun;
use crate::repositories::{ai_agent_repo, execution_graph_repo, graph_run_repo};
use crate::services::{approval_service, chat_service, workflow_service};

/// Policy-configurable in a later phase (see issue #168's own "Concurrency"
/// note); a fixed default for this one, applied uniformly regardless of
/// workspace.
const MAX_STEPS_DEFAULT: i64 = 50;

enum NodeOutcome {
    Continue(String),
    WaitingApproval(String),
    /// Unlike `WaitingApproval` (whose resolution, `resolve_approval`,
    /// explicitly picks the `approved`/`rejected` branch to continue on),
    /// a Delay node has exactly one way out - its own single unconditional
    /// outgoing edge - so that target is resolved once, here, and carried
    /// forward as `next_node_id` rather than left for `resume_run` to
    /// re-derive. `run_from` stores `next_node_id` as the run's
    /// `current_node_id`, not the Delay node's own id - resuming re-enters
    /// at the node *after* the delay, not the delay node itself (which
    /// would otherwise recompute a fresh `resume_at` and pause forever,
    /// never actually advancing).
    WaitingScheduled { resume_at: String, next_node_id: String },
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn parse_context(context_json: &str) -> HashMap<String, Value> {
    serde_json::from_str(context_json).unwrap_or_default()
}

fn dump_context(context: &HashMap<String, Value>) -> String {
    serde_json::to_string(context).unwrap_or_else(|_| "{}".to_string())
}

/// Looks up a `{{...}}` token against `trigger_input` (the literal token
/// `trigger_input`) or a prior node's recorded output (`node_key` for the
/// whole thing, `node_key.field` for one field of it if it's an object) -
/// the graph-shaped generalization of `ai_orchestration_service::
/// resolve_template`'s fixed `{{previous_output}}`/`{{trigger_input}}`
/// placeholders.
fn lookup_token(token: &str, trigger_input: &str, context: &HashMap<String, Value>) -> String {
    if token == "trigger_input" {
        return trigger_input.to_string();
    }
    let (key, field) = match token.split_once('.') {
        Some((k, f)) => (k, Some(f)),
        None => (token, None),
    };
    let Some(value) = context.get(key) else { return String::new() };
    let target = match field {
        Some(f) => value.get(f),
        None => Some(value),
    };
    match target {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn resolve_run_template(template: &str, trigger_input: &str, context: &HashMap<String, Value>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let token = after[..end].trim();
        out.push_str(&lookup_token(token, trigger_input, context));
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// A flattened, string-valued view of `context` plus `trigger_input`,
/// suitable for `domain::conditions::conditions_match`'s
/// `HashMap<String, String>` context argument - a condition's `field_key`
/// addresses either `trigger_input` or `node_key`/`node_key.field` exactly
/// like an Agent node's template does.
fn condition_ctx(trigger_input: &str, context: &HashMap<String, Value>) -> HashMap<String, String> {
    let mut ctx: HashMap<String, String> = HashMap::new();
    ctx.insert("trigger_input".to_string(), trigger_input.to_string());
    for (key, value) in context {
        match value {
            Value::String(s) => {
                ctx.insert(key.clone(), s.clone());
            }
            Value::Object(map) => {
                for (field, v) in map {
                    let s = match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    ctx.insert(format!("{key}.{field}"), s);
                }
                ctx.insert(key.clone(), value.to_string());
            }
            other => {
                ctx.insert(key.clone(), other.to_string());
            }
        }
    }
    ctx
}

fn single_outgoing<'a>(graph: &'a ExecutionGraph, node_id: &str) -> AppResult<&'a str> {
    graph
        .edges
        .iter()
        .find(|e| e.from_node_id == node_id && e.branch_label.is_none())
        .map(|e| e.to_node_id.as_str())
        .ok_or_else(|| AppError::Validation("graph shape error: expected exactly one unconditional outgoing edge".into()))
}

fn branch_outgoing<'a>(graph: &'a ExecutionGraph, node_id: &str, label: &str) -> AppResult<&'a str> {
    graph
        .edges
        .iter()
        .find(|e| e.from_node_id == node_id && e.branch_label.as_deref() == Some(label))
        .map(|e| e.to_node_id.as_str())
        .ok_or_else(|| AppError::Validation(format!("graph shape error: expected an outgoing edge labeled '{label}'")))
}

fn evaluate_conditions_config(config: &Value, trigger_input: &str, context: &HashMap<String, Value>) -> bool {
    let match_type = config.get("match_type").and_then(|v| v.as_str()).unwrap_or("all");
    let conditions = config.get("conditions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let ctx = condition_ctx(trigger_input, context);
    let resolved: Vec<(Option<String>, String, String, String)> = conditions
        .iter()
        .map(|c| {
            (
                c.get("group_id").and_then(|v| v.as_str()).map(String::from),
                c.get("field_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                c.get("operator").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                c.get("value").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            )
        })
        .collect();
    crate::domain::conditions::conditions_match(match_type, resolved.iter().map(|(g, f, o, v)| (g.as_deref(), f.as_str(), o.as_str(), v.as_str())), &ctx)
}

#[allow(clippy::too_many_arguments)]
async fn execute_node(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    graph: &ExecutionGraph,
    node: &GraphNode,
    run: &GraphRun,
    context: &mut HashMap<String, Value>,
) -> AppResult<NodeOutcome> {
    let config: Value = serde_json::from_str(&node.config_json).unwrap_or(json!({}));
    let started_at = now_iso();
    let attempt = graph_run_repo::list_run_nodes(conn, &run.id)?.iter().filter(|n| n.node_id == node.id).count() as i64 + 1;

    macro_rules! record {
        ($status:expr, $input:expr, $output:expr, $err:expr) => {
            graph_run_repo::append_run_node(
                conn,
                &run.id,
                &node.id,
                &node.node_key,
                &node.node_type,
                attempt,
                $status,
                $input,
                $output,
                $err,
                &started_at,
                Some(&now_iso()),
            )?;
        };
    }

    match node.node_type.as_str() {
        "trigger" => {
            record!("completed", None, None, None);
            Ok(NodeOutcome::Continue(single_outgoing(graph, &node.id)?.to_string()))
        }
        "condition" => {
            let result = evaluate_conditions_config(&config, &run.trigger_input, context);
            record!("completed", Some(&node.config_json), Some(&json!({"result": result}).to_string()), None);
            let label = if result { "true" } else { "false" };
            Ok(NodeOutcome::Continue(branch_outgoing(graph, &node.id, label)?.to_string()))
        }
        "router" => {
            let branches = config.get("branches").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut chosen: Option<String> = None;
            for branch in &branches {
                let label = branch.get("branch_label").and_then(|v| v.as_str()).unwrap_or_default();
                if evaluate_conditions_config(branch, &run.trigger_input, context) {
                    chosen = Some(label.to_string());
                    break;
                }
            }
            let label = chosen.or_else(|| graph.edges.iter().any(|e| e.from_node_id == node.id && e.branch_label.as_deref() == Some("default")).then(|| "default".to_string()));
            let Some(label) = label else {
                record!("failed", Some(&node.config_json), None, Some("no router branch matched and no 'default' edge exists"));
                return Err(AppError::Validation("no router branch matched and no 'default' edge exists".into()));
            };
            record!("completed", Some(&node.config_json), Some(&json!({"branch_label": label}).to_string()), None);
            Ok(NodeOutcome::Continue(branch_outgoing(graph, &node.id, &label)?.to_string()))
        }
        "action" => {
            let (Some(entity_type), Some(entity_id)) = (run.source_entity_type.as_deref(), run.source_entity_id.as_deref()) else {
                let msg = "Action node requires the run to carry a source entity (source_entity_type/source_entity_id)";
                record!("failed", Some(&node.config_json), None, Some(msg));
                return Err(AppError::Validation(msg.into()));
            };
            let action_type = config.get("action_type").and_then(|v| v.as_str()).unwrap_or_default();
            let params_json = config.get("params_json").and_then(|v| v.as_str()).unwrap_or("{}");
            match workflow_service::apply_action(conn, workspace_id, action_type, params_json, entity_type, entity_id, None, actor) {
                Ok(summary) => {
                    context.insert(node.node_key.clone(), json!({"summary": summary}));
                    record!("completed", Some(&node.config_json), Some(&json!({"summary": summary}).to_string()), None);
                    Ok(NodeOutcome::Continue(single_outgoing(graph, &node.id)?.to_string()))
                }
                Err(e) => {
                    let msg = e.to_string();
                    record!("failed", Some(&node.config_json), None, Some(&msg));
                    Err(e)
                }
            }
        }
        "agent" => {
            let agent_id = config.get("agent_id").and_then(|v| v.as_str()).unwrap_or_default();
            let input_template = config.get("input_template").and_then(|v| v.as_str()).unwrap_or_default();
            let input_text = resolve_run_template(input_template, &run.trigger_input, context);
            let Some(agent) = ai_agent_repo::get(conn, agent_id)? else {
                let msg = format!("agent '{agent_id}' not found");
                record!("failed", Some(&input_text), None, Some(&msg));
                return Err(AppError::Validation(msg));
            };
            match chat_service::run_agent_once_with_text(conn, workspace_id, master_key, actor, &agent, &input_text).await {
                Ok(outcome) => {
                    context.insert(node.node_key.clone(), json!({"output": outcome.final_text}));
                    record!("completed", Some(&input_text), Some(&json!({"output": outcome.final_text}).to_string()), None);
                    Ok(NodeOutcome::Continue(single_outgoing(graph, &node.id)?.to_string()))
                }
                Err(e) => {
                    let msg = e.to_string();
                    record!("failed", Some(&input_text), None, Some(&msg));
                    Err(e)
                }
            }
        }
        "approval" => {
            let subject_type = config.get("subject_type").and_then(|v| v.as_str()).unwrap_or("execution_graph_node");
            let proposal = json!({"run_id": run.id, "node_key": node.node_key, "context_snapshot": context});
            let approval = approval_service::create(conn, workspace_id, &AiApprovalInput { subject_type: subject_type.to_string(), subject_id: run.id.clone(), proposal }, actor)?;
            record!("waiting_approval", Some(&node.config_json), None, None);
            Ok(NodeOutcome::WaitingApproval(approval.id))
        }
        "delay" => {
            let delay_seconds = config.get("delay_seconds").and_then(|v| v.as_i64()).unwrap_or(0).max(0);
            let resume_at = (chrono::Utc::now() + chrono::Duration::seconds(delay_seconds)).to_rfc3339();
            let next_node_id = single_outgoing(graph, &node.id)?.to_string();
            record!("completed", Some(&node.config_json), Some(&json!({"resume_at": resume_at}).to_string()), None);
            Ok(NodeOutcome::WaitingScheduled { resume_at, next_node_id })
        }
        "transform" => {
            let sets = config.get("set").and_then(|v| v.as_object()).cloned().unwrap_or_default();
            let mut resolved = serde_json::Map::new();
            for (key, template) in &sets {
                let template_str = template.as_str().unwrap_or_default();
                resolved.insert(key.clone(), Value::String(resolve_run_template(template_str, &run.trigger_input, context)));
            }
            context.insert(node.node_key.clone(), Value::Object(resolved.clone()));
            record!("completed", Some(&node.config_json), Some(&Value::Object(resolved).to_string()), None);
            Ok(NodeOutcome::Continue(single_outgoing(graph, &node.id)?.to_string()))
        }
        "loop" => {
            let max_iterations = config.get("max_iterations").and_then(|v| v.as_i64()).unwrap_or(1).max(1);
            if attempt > max_iterations {
                record!("completed", Some(&node.config_json), Some(&json!({"result": "max_iterations_exceeded"}).to_string()), None);
                Ok(NodeOutcome::Continue(branch_outgoing(graph, &node.id, "exit")?.to_string()))
            } else {
                record!("completed", Some(&node.config_json), Some(&json!({"result": "continue", "attempt": attempt}).to_string()), None);
                Ok(NodeOutcome::Continue(branch_outgoing(graph, &node.id, "body")?.to_string()))
            }
        }
        "parallel_split" => {
            let branch_starts: Vec<String> = graph.edges.iter().filter(|e| e.from_node_id == node.id).map(|e| e.to_node_id.clone()).collect();
            let (join_id, branch_oks) = Box::pin(run_parallel_branches(conn, workspace_id, master_key, actor, graph, &branch_starts, run, context)).await?;
            record!("completed", None, Some(&json!({"branch_results": branch_oks}).to_string()), None);
            context.insert("__parallel_join_results".to_string(), json!(branch_oks));
            Ok(NodeOutcome::Continue(join_id))
        }
        "join" => {
            let mode = config.get("mode").and_then(|v| v.as_str()).unwrap_or("all");
            let results: Vec<bool> = context.remove("__parallel_join_results").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();
            let required_count = config.get("required_count").and_then(|v| v.as_i64()).unwrap_or(1).max(1) as usize;
            let ok_count = results.iter().filter(|b| **b).count();
            let passes = match mode {
                "all" => !results.is_empty() && ok_count == results.len(),
                "n_of_m" => ok_count >= required_count,
                // `first_successful` and `timeout_partial` both degrade to
                // "at least one branch succeeded" - see this module's own
                // top doc comment on why a true race-against-a-deadline
                // isn't meaningful without a concurrent worker pool.
                _ => ok_count >= 1,
            };
            if !passes {
                let msg = format!("Join '{}' (mode={mode}) did not pass: {ok_count}/{} branches succeeded", node.node_key, results.len());
                record!("failed", None, None, Some(&msg));
                return Err(AppError::Validation(msg));
            }
            record!("completed", None, Some(&json!({"mode": mode, "ok_count": ok_count, "total": results.len()}).to_string()), None);
            Ok(NodeOutcome::Continue(single_outgoing(graph, &node.id)?.to_string()))
        }
        other => Err(AppError::Validation(format!("unsupported node_type '{other}'"))),
    }
}

/// Walks each Parallel Split branch, one at a time (see this module's top
/// doc comment for why sequential is correct, not just convenient, on a
/// one-worker desktop runtime), down to the Join node they all share.
/// A branch is only permitted to contain `action`/`agent`/`delay`/
/// `transform` nodes (checked here at execution time, not at publish time
/// in this phase - see the top doc comment's explicit scope note); hitting
/// anything else fails the run with a clear message rather than behaving
/// unpredictably.
#[allow(clippy::too_many_arguments)]
async fn run_parallel_branches(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    graph: &ExecutionGraph,
    branch_starts: &[String],
    run: &GraphRun,
    context: &mut HashMap<String, Value>,
) -> AppResult<(String, Vec<bool>)> {
    let mut join_id: Option<String> = None;
    let mut branch_oks = Vec::with_capacity(branch_starts.len());

    for start_id in branch_starts {
        let mut current = start_id.clone();
        let ok = loop {
            let Some(node) = graph.nodes.iter().find(|n| n.id == current) else {
                break false;
            };
            if node.node_type == "join" {
                if join_id.get_or_insert_with(|| node.id.clone()) != &node.id {
                    return Err(AppError::Validation("a Parallel Split's branches must all converge on the same Join node in this phase".into()));
                }
                break true;
            }
            if !is_single_unconditional_outgoing(&node.node_type) {
                return Err(AppError::Validation(format!(
                    "Parallel Split branch hit unsupported node type '{}' ('{}') - only action/agent/delay/transform nodes are supported inside a parallel branch in this phase",
                    node.node_type, node.node_key
                )));
            }
            match execute_node(conn, workspace_id, master_key, actor, graph, node, run, context).await {
                Ok(NodeOutcome::Continue(next)) => current = next,
                Ok(_) => return Err(AppError::Validation("a Parallel Split branch node cannot pause the run in this phase".into())),
                Err(_) => break false,
            }
        };
        branch_oks.push(ok);
    }

    let join_id = join_id.ok_or_else(|| AppError::Validation("Parallel Split's branches never reached a Join node".into()))?;
    Ok((join_id, branch_oks))
}

async fn run_from(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor: Option<&str>, graph: &ExecutionGraph, run_id: &str, start_node_id: &str) -> AppResult<GraphRun> {
    let mut current_id = start_node_id.to_string();
    loop {
        let run = graph_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))?;
        if run.steps_executed >= MAX_STEPS_DEFAULT {
            graph_run_repo::finish_run(conn, run_id, "failed", Some("step limit exceeded"))?;
            break;
        }
        let Some(node) = graph.nodes.iter().find(|n| n.id == current_id) else {
            graph_run_repo::finish_run(conn, run_id, "failed", Some("graph shape error: current node not found"))?;
            break;
        };
        if node.node_type == "end" {
            let mut context = parse_context(&run.context_json);
            context.remove("__parallel_join_results");
            graph_run_repo::checkpoint(conn, run_id, "running", &dump_context(&context), None, run.steps_executed + 1)?;
            graph_run_repo::finish_run(conn, run_id, "completed", None)?;
            break;
        }
        let mut context = parse_context(&run.context_json);
        match execute_node(conn, workspace_id, master_key, actor, graph, node, &run, &mut context).await {
            Ok(NodeOutcome::Continue(next_id)) => {
                graph_run_repo::checkpoint(conn, run_id, "running", &dump_context(&context), Some(&next_id), run.steps_executed + 1)?;
                current_id = next_id;
            }
            Ok(NodeOutcome::WaitingApproval(approval_id)) => {
                graph_run_repo::set_waiting_approval(conn, run_id, &dump_context(&context), &node.id, &approval_id)?;
                break;
            }
            Ok(NodeOutcome::WaitingScheduled { resume_at, next_node_id }) => {
                graph_run_repo::set_waiting_scheduled(conn, run_id, &dump_context(&context), &next_node_id, &resume_at)?;
                break;
            }
            Err(e) => {
                graph_run_repo::finish_run(conn, run_id, "failed", Some(&e.to_string()))?;
                break;
            }
        }
    }
    graph_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))
}

fn get_published_graph(conn: &Connection, graph_id: &str, workspace_id: &str) -> AppResult<ExecutionGraph> {
    let graph = execution_graph_repo::get(conn, graph_id)?.ok_or_else(|| AppError::NotFound("Execution graph".into()))?;
    if graph.workspace_id != workspace_id {
        return Err(AppError::NotFound("Execution graph".into()));
    }
    if graph.status != "published" {
        return Err(AppError::Validation("graph must be published before it can be run".into()));
    }
    Ok(graph)
}

#[allow(clippy::too_many_arguments)]
pub async fn start_run(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    graph_id: &str,
    trigger_input: &str,
    actor: Option<&str>,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> AppResult<GraphRun> {
    let graph = get_published_graph(conn, graph_id, workspace_id)?;
    let trigger = graph.nodes.iter().find(|n| n.node_type == "trigger").ok_or_else(|| AppError::Validation("graph has no trigger node".into()))?;
    let run_id = crate::domain::ids::new_uuid();
    graph_run_repo::start_run(conn, &run_id, workspace_id, graph_id, trigger_input, triggered_by, source_entity_type, source_entity_id)?;
    graph_run_repo::checkpoint(conn, &run_id, "running", "{}", Some(&trigger.id), 0)?;
    run_from(conn, workspace_id, master_key, actor, &graph, &run_id, &trigger.id).await
}

fn get_owned_run(conn: &Connection, run_id: &str, workspace_id: &str) -> AppResult<GraphRun> {
    let run = graph_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))?;
    if run.workspace_id != workspace_id {
        return Err(AppError::NotFound("Run".into()));
    }
    Ok(run)
}

/// Picks a run interrupted mid-flight (a process restart, a panic) back
/// up from exactly `current_node_id`/`context_json` as last checkpointed -
/// the durability guarantee this whole module exists to provide (AI-AC-06).
pub async fn resume_run(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], run_id: &str, actor: Option<&str>) -> AppResult<GraphRun> {
    let run = get_owned_run(conn, run_id, workspace_id)?;
    if !crate::models::graph_run::is_waiting_status(&run.status) {
        return Err(AppError::Validation(format!("run is '{}', not resumable", run.status)));
    }
    let graph = execution_graph_repo::get(conn, &run.graph_id)?.ok_or_else(|| AppError::NotFound("Execution graph".into()))?;
    let current = run.current_node_id.clone().ok_or_else(|| AppError::Validation("run has no current_node_id to resume from".into()))?;
    run_from(conn, workspace_id, master_key, actor, &graph, run_id, &current).await
}

/// Resolves the durable `ai_approvals` row a `waiting_approval` run is
/// blocked on, then continues down the `approved`/`rejected` edge from
/// the paused Approval node - the graph engine's one consolidated
/// approval-gate primitive (see this module's top doc comment).
pub async fn resolve_approval(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], run_id: &str, approve: bool, notes: Option<&str>, actor: Option<&str>) -> AppResult<GraphRun> {
    let run = get_owned_run(conn, run_id, workspace_id)?;
    if run.status != "waiting_approval" {
        return Err(AppError::Validation(format!("run is '{}', not waiting on an approval", run.status)));
    }
    let approval_id = run.pending_approval_id.clone().ok_or_else(|| AppError::Validation("run has no pending approval".into()))?;
    approval_service::resolve(
        conn,
        &approval_id,
        workspace_id,
        &crate::models::ai_approval::AiApprovalResolution { approve, resolution_notes: notes.map(String::from) },
        actor,
    )?;
    let graph = execution_graph_repo::get(conn, &run.graph_id)?.ok_or_else(|| AppError::NotFound("Execution graph".into()))?;
    let approval_node_id = run.current_node_id.clone().ok_or_else(|| AppError::Validation("run has no current_node_id".into()))?;
    let label = if approve { "approved" } else { "rejected" };
    let next = branch_outgoing(&graph, &approval_node_id, label)?.to_string();
    graph_run_repo::clear_wait_markers(conn, run_id)?;
    graph_run_repo::checkpoint(conn, run_id, "running", &run.context_json, Some(&next), run.steps_executed + 1)?;
    run_from(conn, workspace_id, master_key, actor, &graph, run_id, &next).await
}

pub async fn cancel_run(conn: &Connection, workspace_id: &str, run_id: &str, actor_user_id: Option<&str>) -> AppResult<GraphRun> {
    require_admin(conn, actor_user_id)?;
    let run = get_owned_run(conn, run_id, workspace_id)?;
    if crate::models::graph_run::is_terminal_status(&run.status) {
        return Err(AppError::Validation(format!("run is already '{}'", run.status)));
    }
    graph_run_repo::finish_run(conn, run_id, "cancelled", None)?;
    get_owned_run(conn, run_id, workspace_id)
}

/// Sweeps every `waiting_scheduled` run whose `resume_at` has passed and
/// continues each one - mirrors `workflow_service::run_scheduled`'s own
/// sweep pattern for the Workflow engine's `scheduled` trigger type,
/// generalized to a Delay node's resume point instead of a whole
/// workflow's next scheduled firing.
pub async fn resume_due_delays(conn: &Connection, workspace_id: &str, master_key: &[u8; 32]) -> AppResult<usize> {
    let due = graph_run_repo::list_due_delayed_runs(conn, workspace_id)?;
    let mut resumed = 0;
    for run in due {
        if resume_run(conn, workspace_id, master_key, &run.id, None).await.is_ok() {
            resumed += 1;
        }
    }
    Ok(resumed)
}

pub fn get_run(conn: &Connection, run_id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<GraphRun> {
    require_admin(conn, actor_user_id)?;
    get_owned_run(conn, run_id, workspace_id)
}

pub fn list_runs_for_graph(conn: &Connection, graph_id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<GraphRun>> {
    require_admin(conn, actor_user_id)?;
    let graph = execution_graph_repo::get(conn, graph_id)?.ok_or_else(|| AppError::NotFound("Execution graph".into()))?;
    if graph.workspace_id != workspace_id {
        return Err(AppError::NotFound("Execution graph".into()));
    }
    Ok(graph_run_repo::list_runs_for_graph(conn, graph_id, 50)?)
}
