//! Next-Gen program, Domain A (Intelligence Foundation), FND-03: the
//! Unified Test & Evaluation Framework. Covers each of the seven
//! `test_type` executors against the real, already-existing dry-run/
//! read-only function it wraps (not a re-implementation), the unified
//! runner's own acceptance criterion - a single `run_tests` call mixing a
//! deterministic case with a real AI case into one readiness result -
//! and a Solution Release's own validation job running only its curated
//! cases.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::domain::AppError;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::business_rule::{BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput};
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::integration::{FieldMapEntry, MappingInput};
use lanesra_core::models::screen_layout::{LayoutSection, LayoutTab, LayoutTabs, ScreenLayoutUpdate};
use lanesra_core::models::solution::{SolutionInput, SolutionMemberInput};
use lanesra_core::models::test_eval::TestCaseDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowConditionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, ai_agent_service, ai_service, business_rule_service, execution_graph_service, mapping_service, screen_layout_service, solution_service, system_graph_service,
    test_eval_service, user_service, workflow_service, workspace_service,
};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [91u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

/// Same raw-socket stub `ai_eval_harness.rs`/`execution_graph_runtime.rs`
/// already use - serves one canned response body per request in order
/// (repeating the last once exhausted).
fn spawn_sequence_stub(bodies: Vec<String>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut content_length: usize = 0;
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) => break,
                    Ok(_) => {
                        if l == "\r\n" || l.trim().is_empty() {
                            break;
                        }
                        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body_buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body_buf);
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() }
            };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn business_rule_case(name: &str, target_id: &str, dataset_json: &str) -> TestCaseDefinitionInput {
    TestCaseDefinitionInput { name: name.into(), description: None, test_type: "business_rule".into(), target_id: target_id.into(), dataset_json: dataset_json.into(), cost_threshold_usd: None, latency_threshold_ms: None }
}

#[tokio::test]
async fn business_rule_executor_checks_field_effects_against_the_real_rule_engine() {
    let (conn, ws, admin) = setup_workspace("Rule Exec Co");
    let admin = Some(admin.as_str());

    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            entity_type: "Company".into(), name: "Lead requires a name".into(), description: None, match_type: "all".into(), priority: 0,
            effective_start_date: None, effective_end_date: None, app_id: None,
            conditions: vec![BusinessRuleConditionInput { field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(), compare_field_source: None, compare_field_key: None, group_id: None, relationship_definition_id: None }],
            actions: vec![BusinessRuleActionInput { action_type: "require".into(), target_field_key: Some("name".into()), target_field_source: "builtin".into(), action_value: None, message: None }],
        },
        admin,
    )
    .unwrap();

    let case = test_eval_service::create(
        &conn, &ws,
        &business_rule_case("Lead requires a name", "Company", &serde_json::json!({"ctx": {"status": "Prospect"}, "expect_field_effects": {"name": "require"}}).to_string()),
        admin,
    )
    .unwrap();
    let run = test_eval_service::run_tests(&conn, &ws, &master_key(), &[case.id], None, "manual", admin).await.unwrap();
    assert_eq!(run.passed_count, 1, "{:?}", run.results);
    assert_eq!(run.failed_count, 0);
    assert!(run.results[0].passed);

    // A dataset asserting the wrong effect is correctly graded as failed,
    // not silently passed.
    let wrong_case = test_eval_service::create(
        &conn, &ws,
        &business_rule_case("Lead requires a name (wrong expectation)", "Company", &serde_json::json!({"ctx": {"status": "Prospect"}, "expect_field_effects": {"name": "hide"}}).to_string()),
        admin,
    )
    .unwrap();
    let run2 = test_eval_service::run_tests(&conn, &ws, &master_key(), &[wrong_case.id], None, "manual", admin).await.unwrap();
    assert_eq!(run2.failed_count, 1);
    assert!(run2.results[0].trace_text.as_deref().unwrap_or("").contains("expected effect 'hide'"));
}

#[tokio::test]
async fn workflow_executor_matches_active_workflows_against_synthetic_ctx() {
    let (conn, ws, admin) = setup_workspace("Workflow Exec Co");
    let admin = Some(admin.as_str());

    workflow_service::create_rule(
        &conn, &ws,
        &WorkflowDefinitionInput {
            entity_type: "Company".into(), name: "Notify on Lead".into(), description: None, trigger_type: "record_created".into(),
            trigger_status: None, trigger_field_key: None, trigger_field_source: "builtin".into(), trigger_offset_days: 0,
            match_type: "all".into(), priority: 0, app_id: None,
            conditions: vec![WorkflowConditionInput { field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(), compare_field_source: None, compare_field_key: None, group_id: None, relationship_definition_id: None }],
            actions: vec![WorkflowActionInput { action_type: "create_task".into(), params_json: serde_json::json!({"title": "Follow up", "due_in_days": 1}).to_string() }],
        },
        admin,
    )
    .unwrap();

    let case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput {
            name: "Lead triggers notify workflow".into(), description: None, test_type: "workflow".into(), target_id: "Company".into(),
            dataset_json: serde_json::json!({"ctx": {"status": "Prospect"}, "expect_matched_workflow_names": ["Notify on Lead"]}).to_string(),
            cost_threshold_usd: None, latency_threshold_ms: None,
        },
        admin,
    )
    .unwrap();
    let run = test_eval_service::run_tests(&conn, &ws, &master_key(), &[case.id], None, "manual", admin).await.unwrap();
    assert_eq!(run.passed_count, 1, "{:?}", run.results);
}

#[tokio::test]
async fn access_security_executor_uses_the_real_access_evaluation() {
    let (conn, ws, admin) = setup_workspace("Access Exec Co");
    let admin_ref = Some(admin.as_str());

    let rep = user_service::create(&conn, &ws, &NewUser { username: "rep".into(), display_name: "Rep".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] }, admin_ref).unwrap().id;
    let role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "Reader".into(), description: "".into() }, admin_ref).unwrap();
    access_role_service::upsert_grant(
        &conn, &role.id,
        &AccessRoleGrantInput { object_key: "Company".into(), can_create: false, can_read: true, can_update: false, can_delete: false, can_assign: false, record_scope: "ORGANIZATION".into() },
        admin_ref,
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &rep, &role.id, admin_ref).unwrap();

    let allowed_case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput { name: "Rep can read Companies".into(), description: None, test_type: "access_security".into(), target_id: "Company".into(), dataset_json: serde_json::json!({"actor_user_id": rep, "capability": "read", "expect_allowed": true}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None },
        admin_ref,
    )
    .unwrap();
    let denied_case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput { name: "Rep cannot assign Companies".into(), description: None, test_type: "access_security".into(), target_id: "Company".into(), dataset_json: serde_json::json!({"actor_user_id": rep, "capability": "assign", "expect_allowed": false}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None },
        admin_ref,
    )
    .unwrap();

    let run = test_eval_service::run_tests(&conn, &ws, &master_key(), &[allowed_case.id, denied_case.id], None, "manual", admin_ref).await.unwrap();
    assert_eq!(run.passed_count, 2, "{:?}", run.results);
}

#[tokio::test]
async fn screen_visibility_executor_checks_the_resolved_published_layout() {
    let (conn, ws, admin) = setup_workspace("Layout Exec Co");
    let admin = Some(admin.as_str());

    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let draft = LayoutTabs { tabs: vec![LayoutTab { id: "t1".into(), title: "Details".into(), sections: vec![LayoutSection { id: "s1".into(), title: "Details".into(), columns: 2, fields: vec!["industry".into()] }], related: vec![] }] };
    screen_layout_service::update_layout(&conn, &layout.id, &ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft }, admin).unwrap();
    screen_layout_service::publish_layout(&conn, &layout.id, admin).unwrap();

    let case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput { name: "Industry visible, city hidden".into(), description: None, test_type: "screen_visibility".into(), target_id: "Company".into(), dataset_json: serde_json::json!({"expect_visible_fields": ["industry"], "expect_hidden_fields": ["city"]}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None },
        admin,
    )
    .unwrap();
    let run = test_eval_service::run_tests(&conn, &ws, &master_key(), &[case.id], None, "manual", admin).await.unwrap();
    assert_eq!(run.passed_count, 1, "{:?}", run.results);
}

#[tokio::test]
async fn integration_mapping_executor_applies_the_real_field_transform() {
    let (conn, ws, admin) = setup_workspace("Mapping Exec Co");
    let admin = Some(admin.as_str());

    let mapping = mapping_service::create(
        &conn, &ws,
        &MappingInput {
            name: "Company Import".into(), target_object_key: "Company".into(), operation: "upsert".into(), match_key: Some("email".into()),
            field_map: vec![
                FieldMapEntry { source_column: "Company Name".into(), target_field: "name".into(), transform: Some("trim".into()), default_value: None, constant: None },
                FieldMapEntry { source_column: "Status".into(), target_field: "status".into(), transform: Some("uppercase".into()), default_value: Some("PROSPECT".into()), constant: None },
            ],
            duplicate_policy: "update_matched".into(),
        },
        admin,
    )
    .unwrap();

    let case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput {
            name: "Mapping transforms a row correctly".into(), description: None, test_type: "integration_mapping".into(), target_id: mapping.id.clone(),
            dataset_json: serde_json::json!({"source_row": {"Company Name": "  Acme  ", "Status": "lead"}, "expect_target_fields": {"name": "Acme", "status": "LEAD"}}).to_string(),
            cost_threshold_usd: None, latency_threshold_ms: None,
        },
        admin,
    )
    .unwrap();
    let run = test_eval_service::run_tests(&conn, &ws, &master_key(), &[case.id], None, "manual", admin).await.unwrap();
    assert_eq!(run.passed_count, 1, "{:?}", run.results);
}

#[test]
fn validation_rejects_unknown_test_type_and_unknown_target() {
    let (conn, ws, admin) = setup_workspace("Validation Co");
    let admin = Some(admin.as_str());

    let bad_type = TestCaseDefinitionInput { name: "Bad".into(), description: None, test_type: "not_a_real_type".into(), target_id: "Company".into(), dataset_json: "{}".into(), cost_threshold_usd: None, latency_threshold_ms: None };
    assert!(matches!(test_eval_service::create(&conn, &ws, &bad_type, admin), Err(AppError::Validation(_))));

    let bad_target = TestCaseDefinitionInput { name: "Bad target".into(), description: None, test_type: "business_rule".into(), target_id: "NotARealObject".into(), dataset_json: "{}".into(), cost_threshold_usd: None, latency_threshold_ms: None };
    assert!(matches!(test_eval_service::create(&conn, &ws, &bad_target, admin), Err(AppError::Validation(_))));

    let bad_dataset = TestCaseDefinitionInput { name: "Bad JSON".into(), description: None, test_type: "business_rule".into(), target_id: "Company".into(), dataset_json: "not json".into(), cost_threshold_usd: None, latency_threshold_ms: None };
    assert!(matches!(test_eval_service::create(&conn, &ws, &bad_dataset, admin), Err(AppError::Validation(_))));
}

#[tokio::test]
async fn running_a_mixed_run_grades_a_deterministic_and_a_real_ai_case_into_one_readiness_result() {
    let (conn, ws, admin) = setup_workspace("Mixed Run Co");
    let admin_ref = Some(admin.as_str());

    // A deterministic case that passes.
    business_rule_service::create_rule(
        &conn, &ws,
        &BusinessRuleInput {
            entity_type: "Company".into(), name: "Lead requires a name".into(), description: None, match_type: "all".into(), priority: 0,
            effective_start_date: None, effective_end_date: None, app_id: None,
            conditions: vec![BusinessRuleConditionInput { field_source: "builtin".into(), field_key: "status".into(), operator: "equals".into(), value: "Prospect".into(), compare_field_source: None, compare_field_key: None, group_id: None, relationship_definition_id: None }],
            actions: vec![BusinessRuleActionInput { action_type: "require".into(), target_field_key: Some("name".into()), target_field_source: "builtin".into(), action_value: None, message: None }],
        },
        admin_ref,
    )
    .unwrap();
    let rule_case = test_eval_service::create(&conn, &ws, &business_rule_case("Lead requires a name", "Company", &serde_json::json!({"ctx": {"status": "Prospect"}, "expect_field_effects": {"name": "require"}}).to_string()), admin_ref).unwrap();

    // A real agent_eval case - one stub response for the agent's own
    // reply, one for the judge call, same request shape
    // ai_eval_service::run_suite's own tests already prove.
    let agent = ai_agent_service::create(&conn, &ws, &AiAgentInput { name: "Answerer".into(), description: None, icon: "🤖".into(), system_prompt: "You are Answerer.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] }, admin_ref).unwrap();
    let agent_case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput { name: "Answerer greets correctly".into(), description: None, test_type: "agent_eval".into(), target_id: agent.id.clone(), dataset_json: serde_json::json!({"input_text": "hi", "success_criteria": "The response greets the user."}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None },
        admin_ref,
    )
    .unwrap();

    let port = spawn_sequence_stub(vec![anthropic_text_body("Hello there!"), anthropic_text_body("PASS\nThe response greets the user.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = test_eval_service::run_tests(&conn, &ws, &master_key(), &[rule_case.id, agent_case.id], None, "manual", admin_ref).await.unwrap();
    assert_eq!(run.status, "completed");
    assert_eq!(run.passed_count, 2, "{:?}", run.results);
    assert_eq!(run.failed_count, 0);
    assert!(run.is_ready(), "a run with zero failures must report ready");
    assert_eq!(run.results.iter().find(|r| r.test_type == "agent_eval").unwrap().policy_outcome, "clean");
}

#[tokio::test]
async fn run_for_solution_runs_only_its_own_curated_cases() {
    let (conn, ws, admin) = setup_workspace("Solution Validation Co");
    let admin_ref = Some(admin.as_str());

    let curated = test_eval_service::create(&conn, &ws, &TestCaseDefinitionInput { name: "Curated case".into(), description: None, test_type: "access_security".into(), target_id: "Company".into(), dataset_json: serde_json::json!({"actor_user_id": admin, "capability": "read", "expect_allowed": true}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None }, admin_ref).unwrap();
    let uncurated = test_eval_service::create(&conn, &ws, &TestCaseDefinitionInput { name: "Uncurated case".into(), description: None, test_type: "access_security".into(), target_id: "Company".into(), dataset_json: serde_json::json!({"actor_user_id": admin, "capability": "read", "expect_allowed": true}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None }, admin_ref).unwrap();

    let solution = solution_service::create(&conn, &ws, &SolutionInput { name: "Wave 1".into(), description: None, version: None, publisher_id: None }, admin_ref).unwrap();
    solution_service::add_component(&conn, &ws, &solution.id, &SolutionMemberInput { artifact_type: "test_case_definition".into(), metadata_id: curated.id.clone() }, admin_ref).unwrap();

    let run = test_eval_service::run_for_solution(&conn, &ws, &master_key(), &solution.id, admin_ref).await.unwrap();
    assert_eq!(run.results.len(), 1, "only the curated case should run");
    assert_eq!(run.results[0].test_case_id.as_deref(), Some(curated.id.as_str()));
    assert_eq!(run.solution_id.as_deref(), Some(solution.id.as_str()));
    assert_eq!(run.triggered_by, "deployment_validation");
    let _ = uncurated;
}

#[test]
fn an_agent_team_eval_case_traces_in_the_system_graph_and_deactivating_keeps_its_node() {
    let (conn, ws, admin) = setup_workspace("Graph Trace Co");
    let admin_ref = Some(admin.as_str());

    let agent = ai_agent_service::create(&conn, &ws, &AiAgentInput { name: "Teammate".into(), description: None, icon: "🤖".into(), system_prompt: "You are Teammate.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] }, admin_ref).unwrap();
    let graph = execution_graph_service::create(
        &conn, &ws,
        &ExecutionGraphInput {
            name: "Simple Team".into(), description: None,
            nodes: vec![
                GraphNodeInput { node_key: "trigger".into(), node_type: "trigger".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order: 0 },
                GraphNodeInput { node_key: "agent1".into(), node_type: "agent".into(), config_json: serde_json::json!({"agent_id": agent.id, "input_template": "{{trigger_input}}"}).to_string(), position_x: None, position_y: None, sort_order: 1 },
                GraphNodeInput { node_key: "end".into(), node_type: "end".into(), config_json: "{}".into(), position_x: None, position_y: None, sort_order: 2 },
            ],
            edges: vec![
                GraphEdgeInput { from_node_key: "trigger".into(), to_node_key: "agent1".into(), branch_label: None, sort_order: 0 },
                GraphEdgeInput { from_node_key: "agent1".into(), to_node_key: "end".into(), branch_label: None, sort_order: 1 },
            ],
        },
        admin_ref,
    )
    .unwrap();
    execution_graph_service::publish(&conn, &graph.id, &ws, admin_ref).unwrap();

    let case = test_eval_service::create(
        &conn, &ws,
        &TestCaseDefinitionInput { name: "Team greets correctly".into(), description: None, test_type: "agent_team_eval".into(), target_id: graph.id.clone(), dataset_json: serde_json::json!({"input_text": "hi", "success_criteria": "anything"}).to_string(), cost_threshold_usd: None, latency_threshold_ms: None },
        admin_ref,
    )
    .unwrap();

    // depends_on -> the execution_graph node, same resolution an
    // agent_eval case uses for its own ai_agent node.
    let lineage = system_graph_service::get_lineage(&conn, &ws, "test_case_definition", &case.id).unwrap();
    assert!(lineage.iter().any(|h| h.node.node_type == "execution_graph" && h.node.component_id == graph.id));

    test_eval_service::deactivate(&conn, &case.id, &ws, admin_ref).unwrap();
    assert!(system_graph_service::get_node(&conn, &ws, "test_case_definition", &case.id).unwrap().is_some(), "a soft-deactivated test case's node must remain in the graph");

    test_eval_service::delete(&conn, &case.id, &ws, admin_ref).unwrap();
    assert!(system_graph_service::get_node(&conn, &ws, "test_case_definition", &case.id).unwrap().is_none(), "a hard delete must remove the node");
}
