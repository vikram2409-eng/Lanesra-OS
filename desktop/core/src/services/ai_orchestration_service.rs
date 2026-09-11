//! AI & Agentic Layer, Phase 6b: orchestration on top of Phase 6a's
//! Agents - Pipeline/Trigger CRUD (Administrator-only, same shape every
//! other admin builder here uses), and the actual execution engine both
//! a manual "Run Now" and a drained schedule/webhook/workflow-fired run
//! go through.
//!
//! **Gating**: Pipeline/Trigger CRUD and `run_manual` (used by the
//! admin-action route and the admin chat assistant's own
//! `run_ai_agent`/`run_ai_agent_pipeline` tools) all require
//! Administrator up front - Orchestration itself is an admin-only
//! capability. The underlying `run_internal` execution has **no gate of
//! its own** beyond what each step's own agent already enforces
//! (`chat_service::agent_requires_admin`) - deliberately, since a
//! drained schedule/webhook/workflow-triggered run has no human actor at
//! all. In practice this means an admin-scoped agent or pipeline step
//! simply can't run unattended unless `actor` genuinely resolves to a
//! real Administrator (e.g. a webhook call authenticated by an API
//! client whose `owner_user_id` is one) - record-scoped-only automation
//! is the safe default, not an oversight.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::ai_agent_pipeline::{AiAgentPipeline, AiAgentPipelineInput, AiAgentRun, AiAgentTrigger, AiAgentTriggerInput, PIPELINE_TOPOLOGIES, TRIGGER_TARGET_TYPES, TRIGGER_TYPES};
use crate::repositories::{ai_agent_pending_run_repo, ai_agent_pipeline_repo, ai_agent_repo, ai_agent_run_repo};
use crate::services::chat_service;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn validate_pipeline_input(conn: &Connection, workspace_id: &str, input: &AiAgentPipelineInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Pipeline name is required".into()));
    }
    if !PIPELINE_TOPOLOGIES.contains(&input.topology.as_str()) {
        return Err(AppError::Validation(format!("Invalid topology '{}'", input.topology)));
    }
    if input.steps.is_empty() {
        return Err(AppError::Validation("A pipeline needs at least one step".into()));
    }
    match input.topology.as_str() {
        "consensus" if input.steps.len() < 2 => {
            return Err(AppError::Validation("A consensus pipeline needs at least one candidate step plus a synthesizer step".into()));
        }
        "peer_review" if input.steps.len() != 2 => {
            return Err(AppError::Validation("A peer-review pipeline needs exactly two steps - a drafter and a reviewer".into()));
        }
        _ => {}
    }
    // Phase 7e: a human-approval gate pauses and resumes at a specific
    // step index. "sequential" allows it on any step - there's always
    // exactly one well-defined next step. Phase 7g extends this to the
    // one step each of the other two topologies has a well-defined pause
    // point for: consensus's candidates run independently (pausing
    // mid-way has no single "next" step among peers), but its last
    // (synthesizer) step is a normal single resume point once every
    // candidate has already run; peer_review's drafter is mid-loop
    // rather than a resumable checkpoint, but its reviewer step already
    // is the loop's own decision point.
    match input.topology.as_str() {
        "sequential" => {}
        "consensus" => {
            let last = input.steps.len() - 1;
            if input.steps.iter().enumerate().any(|(i, s)| s.requires_approval && i != last) {
                return Err(AppError::Validation("On a consensus pipeline, a human-approval gate is only supported on the last (synthesizer) step".into()));
            }
        }
        "peer_review" if input.steps[0].requires_approval => {
            return Err(AppError::Validation("On a peer-review pipeline, a human-approval gate is only supported on the second (reviewer) step".into()));
        }
        _ => {}
    }
    for step in &input.steps {
        let agent = ai_agent_repo::get(conn, &step.agent_id)?.ok_or_else(|| AppError::Validation("Selected agent does not exist".into()))?;
        if agent.workspace_id != workspace_id || !agent.is_active {
            return Err(AppError::Validation("Selected agent does not exist".into()));
        }
    }
    Ok(())
}

pub fn create_pipeline(conn: &Connection, workspace_id: &str, input: &AiAgentPipelineInput, actor_user_id: Option<&str>) -> AppResult<AiAgentPipeline> {
    require_admin(conn, actor_user_id)?;
    validate_pipeline_input(conn, workspace_id, input)?;
    let id = new_uuid();
    Ok(ai_agent_pipeline_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn update_pipeline(conn: &Connection, id: &str, workspace_id: &str, input: &AiAgentPipelineInput, actor_user_id: Option<&str>) -> AppResult<AiAgentPipeline> {
    require_admin(conn, actor_user_id)?;
    ai_agent_pipeline_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
    validate_pipeline_input(conn, workspace_id, input)?;
    Ok(ai_agent_pipeline_repo::update(conn, id, input, actor_user_id)?)
}

pub fn get_pipeline(conn: &Connection, id: &str) -> AppResult<Option<AiAgentPipeline>> {
    Ok(ai_agent_pipeline_repo::get(conn, id)?)
}

pub fn list_pipelines(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<AiAgentPipeline>> {
    Ok(ai_agent_pipeline_repo::list(conn, workspace_id, active_only)?)
}

pub fn set_pipeline_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<AiAgentPipeline> {
    require_admin(conn, actor_user_id)?;
    ai_agent_pipeline_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
    Ok(ai_agent_pipeline_repo::set_active(conn, id, is_active, actor_user_id)?)
}

// --- Triggers ---------------------------------------------------------

/// `pub(crate)` so `ai_eval_service` (Phase 7d) can validate an Eval
/// Suite's own agent/pipeline target with the identical rule, rather
/// than duplicating it.
pub(crate) fn validate_target_exists(conn: &Connection, workspace_id: &str, target_type: &str, target_id: &str) -> AppResult<()> {
    if !TRIGGER_TARGET_TYPES.contains(&target_type) {
        return Err(AppError::Validation(format!("Invalid target type '{target_type}'")));
    }
    let exists = match target_type {
        "agent" => ai_agent_repo::get(conn, target_id)?.map(|a| a.workspace_id == workspace_id).unwrap_or(false),
        "pipeline" => ai_agent_pipeline_repo::get(conn, target_id)?.map(|p| p.workspace_id == workspace_id).unwrap_or(false),
        _ => false,
    };
    if !exists {
        return Err(AppError::Validation(format!("The selected {target_type} does not exist")));
    }
    Ok(())
}

pub fn create_trigger(conn: &Connection, workspace_id: &str, input: &AiAgentTriggerInput, actor_user_id: Option<&str>) -> AppResult<AiAgentTrigger> {
    require_admin(conn, actor_user_id)?;
    if !TRIGGER_TYPES.contains(&input.trigger_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid trigger type '{}'", input.trigger_type)));
    }
    validate_target_exists(conn, workspace_id, &input.target_type, &input.target_id)?;
    if input.trigger_type == "schedule" && input.interval_minutes.unwrap_or(0) < 1 {
        return Err(AppError::Validation("A schedule trigger needs an interval of at least 1 minute".into()));
    }
    let id = new_uuid();
    Ok(ai_agent_pipeline_repo::create_trigger(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn list_triggers(conn: &Connection, target_type: &str, target_id: &str) -> AppResult<Vec<AiAgentTrigger>> {
    Ok(ai_agent_pipeline_repo::list_triggers_for_target(conn, target_type, target_id)?)
}

pub fn set_trigger_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_pipeline_repo::set_trigger_active(conn, id, is_active)?)
}

pub fn delete_trigger(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    Ok(ai_agent_pipeline_repo::delete_trigger(conn, id)?)
}

// --- Execution ----------------------------------------------------------

const PREVIOUS_OUTPUT_PLACEHOLDER: &str = "{{previous_output}}";
const TRIGGER_INPUT_PLACEHOLDER: &str = "{{trigger_input}}";
/// Consensus topology only - resolved into every candidate step's final
/// answer, numbered, on the synthesizer step's own `input_template`.
const CANDIDATE_OUTPUTS_PLACEHOLDER: &str = "{{candidate_outputs}}";
/// Peer-review topology only - a reviewer's answer approves the current
/// draft by starting with this marker (case-insensitive); anything else
/// is treated as feedback for another drafting round.
const PEER_REVIEW_APPROVAL_MARKER: &str = "APPROVED";
/// A drafter/reviewer exchange this many rounds without approval fails
/// the run rather than looping forever - the same "stop and surface
/// distinctly" guard Phase 7c's loop-detection already established for a
/// single agent's own tool-calling loop.
const MAX_PEER_REVIEW_ROUNDS: u8 = 3;

fn resolve_template(template: &str, previous_output: &str, trigger_input: &str) -> String {
    template.replace(PREVIOUS_OUTPUT_PLACEHOLDER, previous_output).replace(TRIGGER_INPUT_PLACEHOLDER, trigger_input)
}

/// Runs one agent against `input_text`, appends the one
/// `ai_agent_run_steps` row this step gets regardless of topology
/// (timed - `started_at`/`finished_at` are this phase's basic tracing
/// primitive, see `run_to_otlp_json`), and returns its final text - the
/// single place every topology's execution funnels through, so a missing
/// agent or a provider error is recorded identically everywhere.
#[allow(clippy::too_many_arguments)]
async fn run_step(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    run_id: &str,
    agent_id: &str,
    step_order: i64,
    input_text: &str,
    missing_agent_msg: &str,
) -> AppResult<String> {
    let started_at = crate::domain::ids::now_iso();
    let agent = match ai_agent_repo::get(conn, agent_id)? {
        Some(a) => a,
        None => {
            let finished_at = crate::domain::ids::now_iso();
            ai_agent_run_repo::append_run_step(conn, run_id, agent_id, step_order, input_text, None, Some(missing_agent_msg), 0, &started_at, &finished_at)?;
            return Err(AppError::Validation(missing_agent_msg.to_string()));
        }
    };
    match chat_service::run_agent_once_with_text(conn, workspace_id, master_key, actor, &agent, input_text).await {
        Ok(outcome) => {
            let finished_at = crate::domain::ids::now_iso();
            let tool_calls = outcome.produced.iter().filter(|m| m.role == "tool").count() as i64;
            ai_agent_run_repo::append_run_step(conn, run_id, agent_id, step_order, input_text, Some(&outcome.final_text), None, tool_calls, &started_at, &finished_at)?;
            Ok(outcome.final_text)
        }
        Err(e) => {
            let finished_at = crate::domain::ids::now_iso();
            let msg = e.to_string();
            ai_agent_run_repo::append_run_step(conn, run_id, agent_id, step_order, input_text, None, Some(&msg), 0, &started_at, &finished_at)?;
            Err(AppError::Validation(msg))
        }
    }
}

/// What a topology's execution actually did - `run_internal` (a fresh
/// run) and `approve_pending_step` (resuming one) both act on this the
/// same way. `run_sequential_from`, `run_consensus`, and
/// `run_peer_review_from` can each produce `Paused`, at whichever single
/// point their own topology has a well-defined resume step (see
/// `validate_pipeline_input`'s own doc comment).
enum RunOutcome {
    Completed,
    Paused { resume_at_step: i64, previous_output: String },
}

/// The original (and still default) topology: a fixed chain, each step's
/// `{{previous_output}}` resolving to the prior step's answer - except a
/// step flagged `requires_approval` (Phase 7e), which pauses the run
/// right there instead of continuing into the next step automatically.
/// `start_index`/`previous_output` let `approve_pending_step` resume a
/// paused run from exactly where it left off, reusing this same loop.
async fn run_sequential_from(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    run_id: &str,
    steps: &[(String, String, bool)],
    start_index: usize,
    mut previous_output: String,
    trigger_input: &str,
) -> AppResult<RunOutcome> {
    for (step_order, (agent_id, template, requires_approval)) in steps.iter().enumerate().skip(start_index) {
        let input_text = resolve_template(template, &previous_output, trigger_input);
        let msg = format!("Step {} names an agent that no longer exists", step_order + 1);
        previous_output = run_step(conn, workspace_id, master_key, actor, run_id, agent_id, step_order as i64, &input_text, &msg).await?;
        if *requires_approval {
            return Ok(RunOutcome::Paused { resume_at_step: (step_order + 1) as i64, previous_output });
        }
    }
    Ok(RunOutcome::Completed)
}

/// Every step but the last ("candidates") runs independently against the
/// same trigger input - never chained to each other, since there is no
/// single "previous" among peers. The last step ("synthesizer") can then
/// reference every candidate's answer via `{{candidate_outputs}}`.
/// `validate_pipeline_input` already guarantees at least 2 steps for this
/// topology. Phase 7g: if the synthesizer step is flagged
/// `requires_approval`, pauses right there - after every candidate has
/// run, before the synthesizer does - with the joined `candidate_outputs`
/// (exactly the text `{{candidate_outputs}}` would otherwise resolve to)
/// as the resumable value; `run_consensus_synthesizer` below is the
/// shared tail end both this function and `approve_pending_step` run once
/// that's settled.
async fn run_consensus(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], actor: Option<&str>, run_id: &str, mut steps: Vec<(String, String, bool)>, trigger_input: &str) -> AppResult<RunOutcome> {
    let (synth_agent_id, synth_template, synth_requires_approval) = steps.pop().expect("validate_pipeline_input guarantees >= 2 steps for consensus");
    let synth_step_order = steps.len() as i64;

    let mut candidate_texts = Vec::with_capacity(steps.len());
    for (step_order, (agent_id, template, _)) in steps.into_iter().enumerate() {
        let input_text = resolve_template(&template, "", trigger_input);
        let msg = format!("Candidate {} names an agent that no longer exists", step_order + 1);
        let output = run_step(conn, workspace_id, master_key, actor, run_id, &agent_id, step_order as i64, &input_text, &msg).await?;
        candidate_texts.push(format!("Candidate {}: {}", step_order + 1, output));
    }

    let candidate_outputs = candidate_texts.join("\n\n");
    if synth_requires_approval {
        return Ok(RunOutcome::Paused { resume_at_step: synth_step_order, previous_output: candidate_outputs });
    }
    run_consensus_synthesizer(conn, workspace_id, master_key, actor, run_id, &synth_agent_id, &synth_template, synth_step_order, &candidate_outputs, trigger_input).await
}

/// Runs just the synthesizer step of a consensus pipeline against an
/// already-resolved `candidate_outputs` string - the shared tail end of
/// both a fresh, unapproved-gate run (`run_consensus` above, called
/// inline) and an approved resume (`approve_pending_step`, where
/// `candidate_outputs` may be an admin's edited text rather than the
/// candidates' own joined answers).
async fn run_consensus_synthesizer(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    run_id: &str,
    synth_agent_id: &str,
    synth_template: &str,
    synth_step_order: i64,
    candidate_outputs: &str,
    trigger_input: &str,
) -> AppResult<RunOutcome> {
    let input_text = resolve_template(synth_template, "", trigger_input).replace(CANDIDATE_OUTPUTS_PLACEHOLDER, candidate_outputs);
    let msg = "The synthesizer step names an agent that no longer exists".to_string();
    run_step(conn, workspace_id, master_key, actor, run_id, synth_agent_id, synth_step_order, &input_text, &msg).await?;
    Ok(RunOutcome::Completed)
}

/// Exactly two steps - a drafter and a reviewer - looping: the reviewer's
/// answer becomes the drafter's `{{previous_output}}` for the next round
/// (feedback to revise from), and the drafter's own latest answer becomes
/// the reviewer's `{{previous_output}}` (the draft to critique).
/// Approved the moment the reviewer's answer starts with "APPROVED"
/// (case-insensitive); fails clearly, rather than looping forever, once
/// `MAX_PEER_REVIEW_ROUNDS` is reached without approval.
/// `validate_pipeline_input` already guarantees exactly 2 steps for this
/// topology.
///
/// Phase 7g: if the reviewer step is flagged `requires_approval`, pauses
/// after every round's reviewer verdict - before it's checked for the
/// "APPROVED" marker - with the verdict itself as the resumable value, so
/// an admin can edit a reviewer's feedback (or force approval by editing
/// it to start with "APPROVED") before it's acted on. `start_step_order`/
/// `review_override` let `approve_pending_step` resume from exactly the
/// round it paused on, reusing this same loop - and, with both at their
/// defaults (`0`/`None`), this is also how a fresh run starts.
async fn run_peer_review_from(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    actor: Option<&str>,
    run_id: &str,
    steps: Vec<(String, String, bool)>,
    start_step_order: i64,
    review_override: Option<String>,
    trigger_input: &str,
) -> AppResult<RunOutcome> {
    let (drafter_id, drafter_template, _) = steps[0].clone();
    let (reviewer_id, reviewer_template, reviewer_requires_approval) = steps[1].clone();

    // Two steps (drafter + reviewer) per round, so the step_order a round
    // resumes at always lands on a round boundary.
    let rounds_done = (start_step_order / 2) as u8;
    let mut step_order = start_step_order;
    let mut review_feedback = String::new();
    if let Some(review) = review_override {
        if review.trim_start().to_uppercase().starts_with(PEER_REVIEW_APPROVAL_MARKER) {
            return Ok(RunOutcome::Completed);
        }
        review_feedback = review;
    }

    for _ in rounds_done..MAX_PEER_REVIEW_ROUNDS {
        let draft_input = resolve_template(&drafter_template, &review_feedback, trigger_input);
        let draft = run_step(conn, workspace_id, master_key, actor, run_id, &drafter_id, step_order, &draft_input, "The drafter step names an agent that no longer exists").await?;
        step_order += 1;

        let review_input = resolve_template(&reviewer_template, &draft, trigger_input);
        let review = run_step(conn, workspace_id, master_key, actor, run_id, &reviewer_id, step_order, &review_input, "The reviewer step names an agent that no longer exists").await?;
        step_order += 1;

        if reviewer_requires_approval {
            return Ok(RunOutcome::Paused { resume_at_step: step_order, previous_output: review });
        }
        if review.trim_start().to_uppercase().starts_with(PEER_REVIEW_APPROVAL_MARKER) {
            return Ok(RunOutcome::Completed);
        }
        review_feedback = review;
    }
    Err(AppError::Validation(format!(
        "Peer review did not reach approval after {MAX_PEER_REVIEW_ROUNDS} rounds - see the reviewer's latest feedback in the run steps above."
    )))
}

/// A manual "Run Now" (the admin action route, and the admin chat
/// assistant's own `run_ai_agent`/`run_ai_agent_pipeline` tools) -
/// Administrator-gated up front, then delegates to `run_internal`.
#[allow(clippy::too_many_arguments)]
pub async fn run_manual(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], target_type: &str, target_id: &str, input_text: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentRun> {
    require_admin(conn, actor_user_id)?;
    run_internal(conn, workspace_id, master_key, target_type, target_id, actor_user_id, input_text, Some("manual"), None, None).await
}

/// The entry point for a run with no human actor clicking anything - the
/// inline webhook Trigger route (`server/src/agent_v1.rs`) and
/// `drain_pending_runs` below both call this directly, skipping
/// `run_manual`'s own top-level admin gate. See this module's own doc
/// comment for why: each step's own agent-level gating
/// (`chat_service::agent_requires_admin`) is still fully enforced by
/// `run_internal`, just with no blanket "must be an Administrator" check
/// on top of it.
#[allow(clippy::too_many_arguments)]
pub async fn run_triggered(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    target_type: &str,
    target_id: &str,
    actor: Option<&str>,
    trigger_input: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> AppResult<AiAgentRun> {
    run_internal(conn, workspace_id, master_key, target_type, target_id, actor, trigger_input, triggered_by, source_entity_type, source_entity_id).await
}

/// The execution engine itself - runs a lone Agent (a 1-step run) or a
/// Pipeline in order, writing one `ai_agent_runs` row plus one
/// `ai_agent_run_steps` row per step. See this module's own doc comment
/// for why this has no admin gate of its own.
#[allow(clippy::too_many_arguments)]
async fn run_internal(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    target_type: &str,
    target_id: &str,
    actor: Option<&str>,
    trigger_input: &str,
    triggered_by: Option<&str>,
    source_entity_type: Option<&str>,
    source_entity_id: Option<&str>,
) -> AppResult<AiAgentRun> {
    let (topology, steps): (String, Vec<(String, String, bool)>) = match target_type {
        "agent" => {
            let agent = ai_agent_repo::get(conn, target_id)?.ok_or_else(|| AppError::NotFound("Agent".into()))?;
            ("sequential".to_string(), vec![(agent.id, TRIGGER_INPUT_PLACEHOLDER.to_string(), false)])
        }
        "pipeline" => {
            let pipeline = ai_agent_pipeline_repo::get(conn, target_id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
            if pipeline.steps.is_empty() {
                return Err(AppError::Validation("This pipeline has no steps".into()));
            }
            (pipeline.topology.clone(), pipeline.steps.into_iter().map(|s| (s.agent_id, s.input_template, s.requires_approval)).collect())
        }
        other => return Err(AppError::Validation(format!("Unknown target type '{other}'"))),
    };

    let run_id = new_uuid();
    ai_agent_run_repo::start_run(conn, &run_id, workspace_id, target_type, target_id, triggered_by, source_entity_type, source_entity_id, trigger_input)?;

    let result = match topology.as_str() {
        "consensus" => run_consensus(conn, workspace_id, master_key, actor, &run_id, steps, trigger_input).await,
        "peer_review" => run_peer_review_from(conn, workspace_id, master_key, actor, &run_id, steps, 0, None, trigger_input).await,
        _ => run_sequential_from(conn, workspace_id, master_key, actor, &run_id, &steps, 0, String::new(), trigger_input).await,
    };
    finalize_run(conn, &run_id, result)
}

/// Shared by a fresh run (`run_internal`) and a resumed one
/// (`approve_pending_step`) - applies whichever `RunOutcome` the
/// execution produced to the run row, then returns it fully hydrated.
fn finalize_run(conn: &Connection, run_id: &str, result: AppResult<RunOutcome>) -> AppResult<AiAgentRun> {
    match result {
        Ok(RunOutcome::Completed) => ai_agent_run_repo::finish_run(conn, run_id, "succeeded", None)?,
        Ok(RunOutcome::Paused { resume_at_step, previous_output }) => ai_agent_run_repo::pause_for_approval(conn, run_id, resume_at_step, &previous_output)?,
        Err(e) => ai_agent_run_repo::finish_run(conn, run_id, "failed", Some(&e.to_string()))?,
    }
    Ok(ai_agent_run_repo::get_run(conn, run_id)?.expect("just finished or paused"))
}

/// Human-in-the-loop (Phase 7e; extended to consensus/peer_review in
/// Phase 7g): approves the step a run is currently paused on and resumes
/// execution from there - optionally with `edited_output` standing in for
/// that step's own real output, the same "approve, or approve with
/// changes" latitude a real reviewer needs. Re-reads the pipeline fresh
/// (not a frozen copy from when the run paused) - an edit to the
/// pipeline in the meantime takes effect on resume, the same way a
/// changed Business Rule takes effect on its next evaluation rather than
/// being pinned to whatever existed when a record was first opened.
/// Dispatches by the pipeline's own `topology`, since each one resumes
/// through a different function - `run_sequential_from` continues into
/// the next step, `run_consensus_synthesizer` runs just the synthesizer
/// against the (possibly edited) `candidate_outputs`, and
/// `run_peer_review_from` checks the (possibly edited) reviewer verdict
/// and continues the round-robin loop if it isn't approval.
pub async fn approve_pending_step(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], run_id: &str, edited_output: Option<&str>, actor_user_id: Option<&str>) -> AppResult<AiAgentRun> {
    require_admin(conn, actor_user_id)?;
    let run = ai_agent_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))?;
    if run.status != "awaiting_approval" {
        return Err(AppError::Validation("This run is not awaiting approval".into()));
    }
    let pipeline = ai_agent_pipeline_repo::get(conn, &run.target_id)?.ok_or_else(|| AppError::NotFound("Pipeline".into()))?;
    let topology = pipeline.topology.clone();
    let steps: Vec<(String, String, bool)> = pipeline.steps.into_iter().map(|s| (s.agent_id, s.input_template, s.requires_approval)).collect();
    let resume_at_step = run.paused_at_step_order.unwrap_or(0);
    let previous_output = edited_output.map(str::to_string).unwrap_or_else(|| run.resume_previous_output.clone().unwrap_or_default());

    let result = match topology.as_str() {
        "consensus" => {
            let mut steps = steps;
            let (synth_agent_id, synth_template, _) = steps.pop().expect("validate_pipeline_input guarantees >= 2 steps for consensus");
            let synth_step_order = steps.len() as i64;
            run_consensus_synthesizer(conn, workspace_id, master_key, actor_user_id, run_id, &synth_agent_id, &synth_template, synth_step_order, &previous_output, &run.trigger_input).await
        }
        "peer_review" => run_peer_review_from(conn, workspace_id, master_key, actor_user_id, run_id, steps, resume_at_step, Some(previous_output), &run.trigger_input).await,
        _ => run_sequential_from(conn, workspace_id, master_key, actor_user_id, run_id, &steps, resume_at_step as usize, previous_output, &run.trigger_input).await,
    };
    finalize_run(conn, run_id, result)
}

/// Human-in-the-loop (Phase 7e): rejects a paused run outright - `reason`
/// is stored as the run's own error, the same field a genuine execution
/// failure already uses, so a rejected run reads the same way a failed
/// one does everywhere it's shown.
pub fn reject_pending_run(conn: &Connection, run_id: &str, reason: &str, actor_user_id: Option<&str>) -> AppResult<AiAgentRun> {
    require_admin(conn, actor_user_id)?;
    let run = ai_agent_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))?;
    if run.status != "awaiting_approval" {
        return Err(AppError::Validation("This run is not awaiting approval".into()));
    }
    ai_agent_run_repo::finish_run(conn, run_id, "rejected", Some(reason))?;
    Ok(ai_agent_run_repo::get_run(conn, run_id)?.expect("just updated"))
}

/// The one enqueue point for a genuinely-async trigger firing from a
/// context that must never block on it - a due schedule (`enqueue_due_
/// schedules` below), the new Workflow Automation action, or (were an
/// inbound trigger ever changed to defer instead of run inline) a
/// webhook. Resolves `{{trigger_input}}` immediately (from whatever the
/// caller already built) - `run_internal` still resolves
/// `{{previous_output}}` per step when this drains.
#[allow(clippy::too_many_arguments)]
pub fn enqueue(conn: &Connection, workspace_id: &str, target_type: &str, target_id: &str, resolved_input_text: &str, triggered_by: Option<&str>, source_entity_type: Option<&str>, source_entity_id: Option<&str>) -> AppResult<()> {
    let id = new_uuid();
    Ok(ai_agent_pending_run_repo::enqueue(conn, &id, workspace_id, target_type, target_id, resolved_input_text, triggered_by, source_entity_type, source_entity_id)?)
}

/// Finds schedule Triggers whose interval has elapsed and enqueues one
/// pending run each - called once per `job_scheduler` tick, right before
/// `drain_pending_runs`, so a newly-due schedule still drains in the
/// same tick it becomes due rather than waiting a full extra interval.
pub fn enqueue_due_schedules(conn: &Connection, workspace_id: &str) -> AppResult<usize> {
    let due = ai_agent_pipeline_repo::list_due_schedule_triggers(conn, workspace_id)?;
    let count = due.len();
    for trigger in &due {
        enqueue(conn, workspace_id, &trigger.target_type, &trigger.target_id, "", Some("schedule"), None, None)?;
        ai_agent_pipeline_repo::mark_trigger_run(conn, &trigger.id)?;
    }
    Ok(count)
}

/// The async drain for everything `enqueue`/`enqueue_due_schedules`
/// queued - called from `job_scheduler.rs`'s tick (server) and a
/// matching desktop poll, same "enqueue now, drain later" shape
/// `connector_execution_service::drain_pending_actions` already
/// established for `call_connector_action`. A single item's failure
/// (a bad target, a provider error) is recorded on its own
/// `ai_agent_runs` row via `run_internal`'s own error handling and never
/// aborts the batch.
pub async fn drain_pending_runs(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], limit: i64) -> AppResult<usize> {
    let batch = ai_agent_pending_run_repo::list_batch(conn, workspace_id, limit)?;
    let mut drained = 0;
    for item in &batch {
        let _ = run_internal(
            conn, workspace_id, master_key, &item.target_type, &item.target_id, None, &item.resolved_input_text,
            item.triggered_by.as_deref(), item.source_entity_type.as_deref(), item.source_entity_id.as_deref(),
        )
        .await;
        ai_agent_pending_run_repo::delete(conn, &item.id)?;
        drained += 1;
    }
    Ok(drained)
}

pub fn list_runs(conn: &Connection, target_type: &str, target_id: &str, limit: i64) -> AppResult<Vec<AiAgentRun>> {
    Ok(ai_agent_run_repo::list_runs_for_target(conn, target_type, target_id, limit)?)
}

// --- Observability (Phase 7e) ---------------------------------------------

/// A deterministic 32-hex-char id from a seed string - `sha2` (already a
/// dependency, used elsewhere for HMAC signing) rather than a new
/// dependency just for this. Same seed always yields the same id, so a
/// re-export of the same run produces byte-identical span/trace ids.
fn hex_id_32(seed: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(seed.as_bytes());
    digest.iter().take(16).map(|b| format!("{b:02x}")).collect()
}

/// Same idea, truncated to 16 hex chars - OTLP span ids are 8 bytes,
/// trace ids are 16.
fn hex_id_16(seed: &str) -> String {
    hex_id_32(seed)[..16].to_string()
}

fn unix_nanos(rfc3339: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(rfc3339).ok().and_then(|dt| dt.timestamp_nanos_opt()).unwrap_or(0)
}

/// Renders one run as an OTLP-shaped trace (the standard `resourceSpans`
/// JSON wire format - see the "Explicitly deferred" note in this phase's
/// PR body for why this stops at *shaping* the trace rather than also
/// pushing it to a configured collector): one root span for the whole
/// run, one child span per step, both timed from this phase's new
/// `started_at`/`finished_at` columns. Trace/span ids are deterministic
/// (`hex_id_32`/`hex_id_16` above), not random, so re-exporting the same
/// already-finished run is idempotent.
pub fn run_to_otlp_json(run: &AiAgentRun) -> serde_json::Value {
    let trace_id = hex_id_32(&run.id);
    let root_span_id = hex_id_16(&format!("{}:root", run.id));
    let root_start = unix_nanos(&run.started_at);
    let root_end = run.finished_at.as_deref().map(unix_nanos).unwrap_or(root_start);

    let mut spans = vec![serde_json::json!({
        "traceId": trace_id,
        "spanId": root_span_id,
        "name": format!("{} run", run.target_type),
        "startTimeUnixNano": root_start.to_string(),
        "endTimeUnixNano": root_end.to_string(),
        "attributes": [
            {"key": "lanesra.run_id", "value": {"stringValue": run.id}},
            {"key": "lanesra.target_type", "value": {"stringValue": run.target_type}},
            {"key": "lanesra.target_id", "value": {"stringValue": run.target_id}},
            {"key": "lanesra.status", "value": {"stringValue": run.status}},
            {"key": "lanesra.triggered_by", "value": {"stringValue": run.triggered_by.clone().unwrap_or_default()}},
        ],
    })];

    for step in &run.steps {
        let span_id = hex_id_16(&format!("{}:{}", run.id, step.step_order));
        let start = step.started_at.as_deref().map(unix_nanos).unwrap_or(root_start);
        let end = step.finished_at.as_deref().map(unix_nanos).unwrap_or(start);
        spans.push(serde_json::json!({
            "traceId": trace_id,
            "spanId": span_id,
            "parentSpanId": root_span_id,
            "name": format!("step {}: agent {}", step.step_order + 1, step.agent_id),
            "startTimeUnixNano": start.to_string(),
            "endTimeUnixNano": end.to_string(),
            "attributes": [
                {"key": "lanesra.agent_id", "value": {"stringValue": step.agent_id}},
                {"key": "lanesra.step_order", "value": {"intValue": step.step_order.to_string()}},
                {"key": "lanesra.tool_calls_count", "value": {"intValue": step.tool_calls_count.to_string()}},
                {"key": "lanesra.error", "value": {"stringValue": step.error.clone().unwrap_or_default()}},
            ],
        }));
    }

    serde_json::json!({
        "resourceSpans": [{
            "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "lanesra-os"}}]},
            "scopeSpans": [{
                "scope": {"name": "lanesra_core.ai_orchestration_service"},
                "spans": spans,
            }],
        }],
    })
}

/// No admin gate - exporting an already-visible run's own trace needs no
/// more privilege than `list_runs` (which has none) already grants.
pub fn export_run_as_otlp(conn: &Connection, run_id: &str) -> AppResult<serde_json::Value> {
    let run = ai_agent_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))?;
    Ok(run_to_otlp_json(&run))
}

/// Phase 7f: pushes one run's trace to the workspace's configured OTLP
/// collector, on demand - the manual counterpart to `export_run_as_otlp`,
/// same "Test Connection"-style shape `webhook_service::attempt_delivery`
/// already uses (a real outbound call, timed out, no silent retry - a
/// one-off push, not a delivery queue, so there's no delivery history to
/// persist here the way a real webhook subscription earns one). Admin-
/// gated since it makes a real outbound call to an admin-configured
/// endpoint, unlike the no-gate JSON-only export above.
pub async fn push_run_trace_to_otlp(conn: &Connection, workspace_id: &str, run_id: &str, actor_user_id: Option<&str>) -> AppResult<String> {
    require_admin(conn, actor_user_id)?;
    let settings = super::ai_service::get_settings(conn, workspace_id)?;
    let endpoint = settings.otlp_endpoint.ok_or_else(|| AppError::Validation("No OTLP collector endpoint is configured (Admin -> LLM & MCP -> Gateway)".into()))?;
    let run = ai_agent_run_repo::get_run(conn, run_id)?.ok_or_else(|| AppError::NotFound("Run".into()))?;
    let payload = run_to_otlp_json(&run);
    let body = serde_json::to_string(&payload).map_err(|e| AppError::Validation(format!("could not serialize trace: {e}")))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::Validation(format!("could not reach the collector: {e}")))?;

    let status = response.status();
    if status.is_success() {
        Ok(format!("Collector responded {status}"))
    } else {
        let snippet: String = response.text().await.unwrap_or_default().chars().take(200).collect();
        Err(AppError::Validation(format!("Collector responded {status}: {snippet}")))
    }
}
