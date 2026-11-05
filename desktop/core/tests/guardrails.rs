//! WP-0133: output guardrails as versioned policy objects.
//!
//! AC-1: each built-in guardrail has a positive and a negative test (checked
//! through the exact config the built-in ships with).
//! AC-2: editing a guardrail writes a new version and a run records the
//! version it evaluated against.
//!
//! Covers: REQ-GOV-007 (WP-0133 AC-1, AC-2)

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::ai_guardrail::{GuardrailEdit, GuardrailInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::ai_guardrail_repo;
use lanesra_core::services::{guardrail_service as gs, workspace_service};
use serde_json::{json, Value};

fn builtin(key: &str) -> (&'static str, Value) {
    let b = gs::BUILTINS.iter().find(|b| b.0 == key).expect("built-in");
    (
        b.3,
        serde_json::from_str(b.4).expect("built-in config parses"),
    )
}

fn ok(checker: &str, config: &Value, output: &str, reply: Option<&str>) -> bool {
    gs::evaluate(checker, config, output, reply).unwrap().passed
}

fn setup() -> WorkspaceSetup {
    WorkspaceSetup {
        business_name: "Guard Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    }
}

#[test]
fn builtin_pii_leak() {
    // Covers: REQ-GOV-007 (WP-0133 AC-1)
    let (checker, config) = builtin("pii_leak");
    assert_eq!(checker, "pii");
    assert!(ok(
        checker,
        &config,
        "Thanks for reaching out, we will follow up soon.",
        None
    ));
    assert!(!ok(
        checker,
        &config,
        "Email me at john.doe@example.com about it.",
        None
    ));
}

#[test]
fn builtin_required_disclaimer() {
    // Covers: REQ-GOV-007 (WP-0133 AC-1)
    let (checker, config) = builtin("required_disclaimer");
    let text = config["text"].as_str().unwrap();
    assert!(ok(
        checker,
        &config,
        &format!("Here is my take. {text}"),
        None
    ));
    assert!(!ok(checker, &config, "Here is my take.", None));
}

#[test]
fn builtin_domain_allowlist() {
    // Covers: REQ-GOV-007 (WP-0133 AC-1)
    let (checker, config) = builtin("domain_allowlist");
    assert!(ok(
        checker,
        &config,
        "See https://example.com/docs for more.",
        None
    ));
    assert!(ok(checker, &config, "No links here at all.", None));
    assert!(!ok(
        checker,
        &config,
        "Go to https://evil.example.net/phish now.",
        None
    ));
}

#[test]
fn builtin_schema_conformance() {
    // Covers: REQ-GOV-007 (WP-0133 AC-1)
    let (checker, config) = builtin("json_schema");
    assert!(ok(checker, &config, r#"{"answer": "42"}"#, None));
    assert!(!ok(checker, &config, r#"{"question": "what?"}"#, None));
    assert!(!ok(checker, &config, "not json at all", None));
}

#[test]
fn builtin_tone_judge() {
    // Covers: REQ-GOV-007 (WP-0133 AC-1)
    let (checker, config) = builtin("tone");
    assert_eq!(checker, "judge");
    assert!(ok(
        checker,
        &config,
        "the output",
        Some("PASS\nIt reads as courteous.")
    ));
    assert!(!ok(
        checker,
        &config,
        "the output",
        Some("FAIL\nIt is rude.")
    ));
    // A judge guardrail without a model verdict is an error, not a silent pass.
    assert!(gs::evaluate(checker, &config, "the output", None).is_err());
}

#[test]
fn editing_writes_a_new_version_and_runs_record_it() {
    // Covers: REQ-GOV-007 (WP-0133 AC-2)
    let conn = open_in_memory_db().unwrap();
    let (ws, admin) = workspace_service::first_run_setup(&conn, &setup()).unwrap();

    // list() seeds the built-in library and returns it.
    let all = gs::list(&conn, &ws.id, Some(&admin.id)).unwrap();
    assert_eq!(all.len(), gs::BUILTINS.len());
    let disc = all
        .iter()
        .find(|g| g.builtin_key.as_deref() == Some("required_disclaimer"))
        .unwrap()
        .clone();
    assert_eq!(disc.current_version, 1);

    // A run evaluates v1 and records it (the v1 text is absent -> fails).
    let out1 = gs::evaluate_for_run(
        &conn,
        &ws.id,
        "pipeline",
        "run-1",
        &disc.id,
        "plain answer",
        None,
    )
    .unwrap();
    assert!(!out1.passed);

    // Editing the config writes version 2.
    let updated = gs::edit(
        &conn,
        &ws.id,
        &disc.id,
        &GuardrailEdit {
            name: None,
            description: None,
            enabled: None,
            config_json: Some(r#"{"mode":"must_contain","text":"DISCLAIMER"}"#.into()),
            hook_point: None,
        },
        Some(&admin.id),
    )
    .unwrap();
    assert_eq!(updated.current_version, 2);

    // A later run records v2 (and the new text now passes).
    let out2 = gs::evaluate_for_run(
        &conn,
        &ws.id,
        "pipeline",
        "run-1",
        &disc.id,
        "answer, DISCLAIMER applies",
        None,
    )
    .unwrap();
    assert!(out2.passed);

    let evals = ai_guardrail_repo::list_evaluations(&conn, &ws.id, "pipeline", "run-1").unwrap();
    assert_eq!(evals.len(), 2);
    assert_eq!(evals[0].guardrail_version, 1);
    assert!(!evals[0].passed);
    assert_eq!(evals[1].guardrail_version, 2);
    assert!(evals[1].passed);

    // Editing with the same config does not add a version.
    let same = gs::edit(
        &conn,
        &ws.id,
        &disc.id,
        &GuardrailEdit {
            name: None,
            description: None,
            enabled: None,
            config_json: Some(r#"{"mode":"must_contain","text":"DISCLAIMER"}"#.into()),
            hook_point: None,
        },
        Some(&admin.id),
    )
    .unwrap();
    assert_eq!(same.current_version, 2);
}

#[test]
fn creates_a_custom_guardrail() {
    // Covers: REQ-GOV-007 (WP-0133 AC-1)
    let conn = open_in_memory_db().unwrap();
    let (ws, admin) = workspace_service::first_run_setup(&conn, &setup()).unwrap();

    let g = gs::create(
        &conn,
        &ws.id,
        &GuardrailInput {
            name: "No profanity".into(),
            description: None,
            checker_type: "rule".into(),
            config_json: json!({"mode":"forbid_substrings","substrings":["darn"]}).to_string(),
            hook_point: "off".into(),
        },
        Some(&admin.id),
    )
    .unwrap();
    assert!(!g.is_builtin);
    assert_eq!(g.current_version, 1);

    // An invalid checker type is refused.
    assert!(gs::create(
        &conn,
        &ws.id,
        &GuardrailInput {
            name: "bad".into(),
            description: None,
            checker_type: "nonsense".into(),
            config_json: "{}".into(),
            hook_point: "off".into(),
        },
        Some(&admin.id),
    )
    .is_err());
}
