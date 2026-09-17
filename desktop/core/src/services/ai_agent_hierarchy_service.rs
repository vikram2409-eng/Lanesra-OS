//! AI & Agentic Layer: the Agent Hierarchy - resolves and narrates, for one
//! Pipeline, the full picture of which agents run and which agents they
//! may delegate to. Two things this codebase already tracks separately are
//! combined into one view here, nothing new is invented:
//!
//! - The Pipeline's own `steps` (deterministic, ordered - "who runs, and
//!   in what shape", per `AiAgentPipeline::topology`).
//! - Each step's agent's own `delegate_agent_ids` (dynamic, runtime-decided
//!   - "who that agent *may* call", the same field `chat_service`'s
//!   `delegate_to_agent` tool actually uses).
//!
//! `resolve` builds the tree; `describe_pipeline` narrates it in plain
//! language, one paragraph per step plus a topology-level summary -
//! mirrors the "live rule-summary panel" Business Rules/Workflow
//! Automation already give an admin, applied to orchestration instead.
//! Both read the exact same data a run itself would use (no separate,
//! divergent notion of "the hierarchy") - the same "one evaluator, every
//! caller" principle `access_service::explain_access` established for the
//! Access Inspector, applied here to orchestration instead of authorization.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ai_agent_pipeline::{AgentHierarchyNode, PipelineHierarchy};
use crate::repositories::{ai_agent_pipeline_repo, ai_agent_repo};
use crate::services::chat_service::MAX_DELEGATION_DEPTH;
use crate::services::ai_orchestration_service::MAX_PEER_REVIEW_ROUNDS;

/// One agent's node in the tree, expanding its own `delegate_agent_ids`
/// recursively. `path` is every agent id from the tree's root down to (not
/// including) this node - used to catch a delegation cycle (an edge back
/// to an agent already on the path to it), which the create/update save
/// path already warns about but doesn't hard-block, so the tree must
/// handle one gracefully rather than recursing forever.
fn resolve_node(
    conn: &Connection,
    agent_id: &str,
    step_order: Option<i64>,
    requires_approval: bool,
    path: &mut Vec<String>,
    depth: u8,
) -> AppResult<AgentHierarchyNode> {
    let agent = ai_agent_repo::get(conn, agent_id)?;
    let (agent_name, agent_icon, agent_is_active, delegate_ids) = match &agent {
        Some(a) => (a.name.clone(), a.icon.clone(), a.is_active, a.delegate_agent_ids.clone()),
        // An agent referenced by a step or a delegate list can still be
        // deleted out from under it in principle (nothing here cascades) -
        // render a clearly-labeled stub rather than erroring the whole tree.
        None => ("(deleted agent)".to_string(), "⚠".to_string(), false, Vec::new()),
    };

    if path.contains(&agent_id.to_string()) {
        return Ok(AgentHierarchyNode {
            agent_id: agent_id.to_string(),
            agent_name,
            agent_icon,
            agent_is_active,
            step_order,
            requires_approval,
            delegates: Vec::new(),
            truncated: Some("Delegation cycle - this agent already appears earlier on this same path".to_string()),
        });
    }
    if depth >= MAX_DELEGATION_DEPTH {
        return Ok(AgentHierarchyNode {
            agent_id: agent_id.to_string(),
            agent_name,
            agent_icon,
            agent_is_active,
            step_order,
            requires_approval,
            delegates: Vec::new(),
            truncated: Some(format!("Delegation depth limit ({MAX_DELEGATION_DEPTH}) reached - matches the runtime guard, so a run could never actually go deeper than this either")),
        });
    }

    path.push(agent_id.to_string());
    let mut delegates = Vec::with_capacity(delegate_ids.len());
    for delegate_id in &delegate_ids {
        delegates.push(resolve_node(conn, delegate_id, None, false, path, depth + 1)?);
    }
    path.pop();

    Ok(AgentHierarchyNode { agent_id: agent_id.to_string(), agent_name, agent_icon, agent_is_active, step_order, requires_approval, delegates, truncated: None })
}

/// Builds the full hierarchy for one Pipeline - one root per step, each
/// expanded through that step's agent's own delegate graph.
pub fn resolve(conn: &Connection, pipeline_id: &str) -> AppResult<PipelineHierarchy> {
    let pipeline = ai_agent_pipeline_repo::get(conn, pipeline_id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;

    let mut roots = Vec::with_capacity(pipeline.steps.len());
    for step in &pipeline.steps {
        let mut path = Vec::new();
        roots.push(resolve_node(conn, &step.agent_id, Some(step.step_order), step.requires_approval, &mut path, 0)?);
    }

    let description_md = describe_pipeline(&pipeline.name, &pipeline.topology, &roots);

    Ok(PipelineHierarchy { pipeline_id: pipeline.id, pipeline_name: pipeline.name, topology: pipeline.topology, roots, description_md })
}

fn delegate_summary(node: &AgentHierarchyNode) -> String {
    if node.delegates.is_empty() {
        return String::new();
    }
    let names: Vec<&str> = node.delegates.iter().map(|d| d.agent_name.as_str()).collect();
    format!(" (which may delegate to {})", names.join(", "))
}

/// Narrates a resolved hierarchy in plain language - one paragraph per
/// step (naming its agent, whether it pauses for approval, and a one-level
/// summary of who it may delegate to) plus a topology-level opening
/// sentence explaining how the steps relate to each other. Only ever reads
/// `roots` (already-resolved data), never re-queries the database, so this
/// can never disagree with what `resolve` itself just built.
pub fn describe_pipeline(pipeline_name: &str, topology: &str, roots: &[AgentHierarchyNode]) -> String {
    if roots.is_empty() {
        return format!("**{pipeline_name}** has no steps yet - add at least one to see how it runs.");
    }

    let mut lines = Vec::new();
    match topology {
        "consensus" if roots.len() >= 2 => {
            let (candidates, synthesizer) = roots.split_at(roots.len() - 1);
            let synthesizer = &synthesizer[0];
            lines.push(format!(
                "**{pipeline_name}** runs {} candidate step(s) independently against the same input, then a synthesizer combines every candidate's answer into one.",
                candidates.len()
            ));
            for c in candidates {
                lines.push(format!("- Candidate: **{}**{}", c.agent_name, delegate_summary(c)));
            }
            lines.push(format!("- Synthesizer: **{}**{}, reads every candidate's answer via `{{{{candidate_outputs}}}}`.", synthesizer.agent_name, delegate_summary(synthesizer)));
        }
        "peer_review" if roots.len() == 2 => {
            let drafter = &roots[0];
            let reviewer = &roots[1];
            lines.push(format!(
                "**{pipeline_name}** loops a drafter and a reviewer: **{}**{} drafts, **{}**{} critiques, the drafter revises from that critique - up to {MAX_PEER_REVIEW_ROUNDS} rounds, until the reviewer's answer starts with \"APPROVED\".",
                drafter.agent_name, delegate_summary(drafter), reviewer.agent_name, delegate_summary(reviewer)
            ));
        }
        _ => {
            lines.push(format!("**{pipeline_name}** runs {} step(s) in order, each one's output feeding the next via `{{{{previous_output}}}}`.", roots.len()));
            for r in roots {
                let step_label = r.step_order.map(|n| format!("Step {}", n + 1)).unwrap_or_default();
                let pause = if r.requires_approval { " - pauses for an Administrator's approval before continuing" } else { "" };
                lines.push(format!("- {step_label}: **{}**{}{}", r.agent_name, delegate_summary(r), pause));
            }
        }
    }
    lines.join("\n")
}
