//! AI & Agentic Layer: the Agent Hierarchy - resolving, for one Pipeline,
//! the tree of which agents run (the pipeline's own step order) and which
//! agents they may in turn delegate to (each step's agent's own
//! `delegate_agent_ids`), plus the plain-language description generated
//! from that same resolved tree. See `ai_agent_hierarchy_service`'s own
//! doc comment for the design.

use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_hierarchy_service, ai_agent_service, ai_orchestration_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Hierarchy Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str, delegate_agent_ids: Vec<String>) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids, skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

fn step(agent_id: &str, requires_approval: bool) -> PipelineStepInput {
    PipelineStepInput { agent_id: agent_id.into(), input_template: "{{trigger_input}}".into(), requires_approval }
}

#[test]
fn hierarchy_resolves_sequential_steps_and_each_steps_delegates() {
    let (conn, ws, admin) = setup_workspace();
    let researcher = make_agent(&conn, &ws, &admin, "Researcher", vec![]);
    let drafter = make_agent(&conn, &ws, &admin, "Drafter", vec![researcher.id.clone()]);
    let reviewer = make_agent(&conn, &ws, &admin, "Reviewer", vec![]);

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput { name: "Content Pipeline".into(), description: None, topology: "sequential".into(), steps: vec![step(&drafter.id, false), step(&reviewer.id, true)] },
        Some(&admin),
    )
    .unwrap();

    let hierarchy = ai_agent_hierarchy_service::resolve(&conn, &pipeline.id).unwrap();
    assert_eq!(hierarchy.roots.len(), 2);

    let drafter_node = &hierarchy.roots[0];
    assert_eq!(drafter_node.agent_id, drafter.id);
    assert_eq!(drafter_node.step_order, Some(0));
    assert!(!drafter_node.requires_approval);
    assert_eq!(drafter_node.delegates.len(), 1, "Drafter's own delegate_agent_ids must appear as its child in the tree");
    assert_eq!(drafter_node.delegates[0].agent_id, researcher.id);
    assert!(drafter_node.delegates[0].delegates.is_empty());

    let reviewer_node = &hierarchy.roots[1];
    assert_eq!(reviewer_node.agent_id, reviewer.id);
    assert_eq!(reviewer_node.step_order, Some(1));
    assert!(reviewer_node.requires_approval, "the step's own requires_approval flag must carry through to its node");
    assert!(reviewer_node.delegates.is_empty());

    assert!(hierarchy.description_md.contains("Drafter"));
    assert!(hierarchy.description_md.contains("Researcher"));
    assert!(hierarchy.description_md.contains("Reviewer"));
    assert!(hierarchy.description_md.contains("pauses for an Administrator's approval"));
}

#[test]
fn hierarchy_describes_consensus_and_peer_review_topologies_distinctly() {
    let (conn, ws, admin) = setup_workspace();
    let candidate_a = make_agent(&conn, &ws, &admin, "Candidate A", vec![]);
    let candidate_b = make_agent(&conn, &ws, &admin, "Candidate B", vec![]);
    let synthesizer = make_agent(&conn, &ws, &admin, "Synthesizer", vec![]);

    let consensus_pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput { name: "Consensus Pipeline".into(), description: None, topology: "consensus".into(), steps: vec![step(&candidate_a.id, false), step(&candidate_b.id, false), step(&synthesizer.id, false)] },
        Some(&admin),
    )
    .unwrap();
    let consensus_hierarchy = ai_agent_hierarchy_service::resolve(&conn, &consensus_pipeline.id).unwrap();
    assert!(consensus_hierarchy.description_md.contains("independently"));
    assert!(consensus_hierarchy.description_md.contains("Synthesizer"));
    assert!(consensus_hierarchy.description_md.contains("candidate_outputs"));

    let drafter = make_agent(&conn, &ws, &admin, "PR Drafter", vec![]);
    let peer_reviewer = make_agent(&conn, &ws, &admin, "PR Reviewer", vec![]);
    let peer_review_pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput { name: "Peer Review Pipeline".into(), description: None, topology: "peer_review".into(), steps: vec![step(&drafter.id, false), step(&peer_reviewer.id, false)] },
        Some(&admin),
    )
    .unwrap();
    let peer_review_hierarchy = ai_agent_hierarchy_service::resolve(&conn, &peer_review_pipeline.id).unwrap();
    assert!(peer_review_hierarchy.description_md.contains("loops a drafter and a reviewer"));
    assert!(peer_review_hierarchy.description_md.contains("APPROVED"));
}

#[test]
fn hierarchy_truncates_at_the_same_depth_the_runtime_delegation_guard_uses() {
    let (conn, ws, admin) = setup_workspace();
    // A chain of 6 agents, each delegating to the next - deeper than
    // MAX_DELEGATION_DEPTH (4), so the tree must truncate rather than
    // resolve (or recurse) past where a real run ever could.
    let mut deepest = make_agent(&conn, &ws, &admin, "Link 5", vec![]);
    for i in (0..5).rev() {
        deepest = make_agent(&conn, &ws, &admin, &format!("Link {i}"), vec![deepest.id.clone()]);
    }
    let root_agent = deepest;

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput { name: "Deep Delegation Pipeline".into(), description: None, topology: "sequential".into(), steps: vec![step(&root_agent.id, false)] },
        Some(&admin),
    )
    .unwrap();

    let hierarchy = ai_agent_hierarchy_service::resolve(&conn, &pipeline.id).unwrap();
    let mut node = &hierarchy.roots[0];
    let mut depth = 0;
    while node.truncated.is_none() {
        assert!(!node.delegates.is_empty(), "must keep descending until truncation, not stop early");
        node = &node.delegates[0];
        depth += 1;
        assert!(depth <= 10, "must not recurse indefinitely if truncation somehow never triggers");
    }
    assert!(node.truncated.as_ref().unwrap().contains("depth limit"));
}

#[test]
fn hierarchy_handles_a_delegation_cycle_without_recursing_forever() {
    let (conn, ws, admin) = setup_workspace();
    let agent_a = make_agent(&conn, &ws, &admin, "Agent A", vec![]);
    let agent_b = make_agent(&conn, &ws, &admin, "Agent B", vec![agent_a.id.clone()]);
    // Update A after B exists, so A now delegates back to B - a genuine
    // cycle (A -> B -> A -> ...), the kind normal usage would only create
    // by editing two agents in sequence, not something the save path
    // itself hard-blocks.
    let agent_a = ai_agent_service::update(
        &conn, &agent_a.id, &ws,
        &AiAgentInput { name: agent_a.name.clone(), description: None, icon: "🤖".into(), system_prompt: agent_a.system_prompt.clone(), action_names: vec![], delegate_agent_ids: vec![agent_b.id.clone()], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput { name: "Cyclical Pipeline".into(), description: None, topology: "sequential".into(), steps: vec![step(&agent_a.id, false)] },
        Some(&admin),
    )
    .unwrap();

    let hierarchy = ai_agent_hierarchy_service::resolve(&conn, &pipeline.id).unwrap();
    let root = &hierarchy.roots[0];
    assert_eq!(root.agent_id, agent_a.id);
    assert!(root.truncated.is_none());
    assert_eq!(root.delegates.len(), 1);
    assert_eq!(root.delegates[0].agent_id, agent_b.id);
    let cycle_node = &root.delegates[0].delegates[0];
    assert_eq!(cycle_node.agent_id, agent_a.id);
    assert!(cycle_node.truncated.as_ref().unwrap().contains("cycle"), "the second visit to Agent A must be flagged as a cycle, not silently repeated or hung");
}
