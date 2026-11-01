//! WP-0113: the two checks every cell of the full-trace conformance matrix
//! must pass. Shared by `core/tests/trace_conformance.rs` and
//! `server/tests/trace_conformance.rs` (`#[path]`); the including file also
//! includes `event_schema.rs` as `event_schema`.
//!
//! **A full trace** (ADR-0008, WP-0111, WP-0112): one closed root `run`
//! span; every other span closed, in the same trace, under a parent that
//! exists; at least one `step` span per agent step the run recorded; every
//! `model_call` under a step and carrying token counts; every `tool_call`
//! under a `model_call` with exactly one firewall `policy` span beneath it;
//! and run events numbered 1, 2, 3 ... from `run.started` to a terminal
//! event, each valid against its schema.
//!
//! **A complete tool audit** (REQ-GOV-003, WP-0109): every tool call that
//! ran left exactly one `ai_agent_tool_call` audit entry naming the agent,
//! the tool and the effective principal.

// Each including test uses a different part of this module.
#![allow(dead_code)]

use std::collections::HashMap;

use lanesra_core::models::run_trace::RunSpan;
use lanesra_core::services::{run_event_service, trace_service};
use rusqlite::Connection;
use serde_json::Value;

/// What a checked trace contained.
#[derive(Debug)]
pub struct TraceSummary {
    pub steps: usize,
    pub model_calls: usize,
    /// Tool calls that reached the firewall and its audit: they ran, failed,
    /// or were refused (malformed and capped calls never get that far).
    pub tools_run: Vec<String>,
}

/// Asserts a finished run has a full trace; `agent_steps` is how many agent
/// steps the run itself recorded.
pub fn assert_full_trace(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
    agent_steps: usize,
    cell: &str,
) -> TraceSummary {
    let spans: Vec<RunSpan> =
        trace_service::spans_for_run(conn, workspace_id, run_kind, run_id).unwrap();
    assert!(!spans.is_empty(), "{cell}: the run recorded no trace");
    let by_id: HashMap<&str, &RunSpan> = spans.iter().map(|s| (s.span_id.as_str(), s)).collect();

    let roots: Vec<&RunSpan> = spans.iter().filter(|s| s.kind == "run").collect();
    assert_eq!(roots.len(), 1, "{cell}: one root span");
    let root = roots[0];
    assert!(
        root.parent_span_id.is_none(),
        "{cell}: the root has no parent"
    );
    for s in &spans {
        assert_eq!(
            s.trace_id, root.trace_id,
            "{cell}: {} is in another trace",
            s.name
        );
        assert!(
            s.status.is_some() && s.ended_at.is_some(),
            "{cell}: span {} ({}) never closed",
            s.name,
            s.kind
        );
        if s.kind != "run" {
            let parent = s
                .parent_span_id
                .as_deref()
                .expect("a child span has a parent");
            assert!(
                by_id.contains_key(parent),
                "{cell}: {} has a missing parent",
                s.name
            );
        }
    }

    let parent_kind = |s: &RunSpan| by_id[s.parent_span_id.as_deref().unwrap()].kind.clone();
    let steps = spans.iter().filter(|s| s.kind == "step").count();
    assert!(
        steps >= agent_steps,
        "{cell}: {steps} step spans for {agent_steps} agent steps"
    );
    let mut model_calls = 0;
    let mut tools_run = Vec::new();
    for s in &spans {
        match s.kind.as_str() {
            "model_call" => {
                model_calls += 1;
                assert_eq!(
                    parent_kind(s),
                    "step",
                    "{cell}: a model call outside a step"
                );
                assert!(
                    s.attributes.get("gen_ai.usage.input_tokens").is_some(),
                    "{cell}: a model call without token counts"
                );
            }
            "tool_call" => {
                assert_eq!(
                    parent_kind(s),
                    "model_call",
                    "{cell}: tool call {} outside a model call",
                    s.name
                );
                let verdicts = spans
                    .iter()
                    .filter(|p| {
                        p.kind == "policy" && p.parent_span_id.as_deref() == Some(&s.span_id)
                    })
                    .count();
                assert_eq!(
                    verdicts, 1,
                    "{cell}: tool call {} has {verdicts} firewall verdicts",
                    s.name
                );
                let outcome = s
                    .attributes
                    .get("lanesra.tool.outcome")
                    .and_then(Value::as_str);
                if matches!(outcome, Some("ok") | Some("error") | Some("blocked")) {
                    tools_run.push(s.name.clone());
                }
            }
            _ => {}
        }
    }
    assert!(model_calls >= 1, "{cell}: no model call was traced");

    let events = run_event_service::events_for_run(conn, workspace_id, run_id, 0).unwrap();
    assert!(!events.is_empty(), "{cell}: no run events");
    let seqs: Vec<i64> = events.iter().map(|e| e.seq).collect();
    assert_eq!(
        seqs,
        (1..=events.len() as i64).collect::<Vec<_>>(),
        "{cell}: event numbering"
    );
    assert_eq!(events[0].event_type, "run.started", "{cell}");
    let last = &events[events.len() - 1].event_type;
    assert!(
        last == "run.completed" || last == "run.failed",
        "{cell}: the last event is {last}"
    );
    for e in &events {
        super::event_schema::assert_valid(&serde_json::to_value(e).unwrap());
    }

    TraceSummary {
        steps,
        model_calls,
        tools_run,
    }
}

/// The audit position now, to check only the entries a cell adds.
pub fn audit_mark(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COALESCE(MAX(rowid), 0) FROM audit_events",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

/// Asserts every tool call that ran (`tools_run`) left exactly one tool-call
/// audit entry after `mark`, naming an agent, the tool and the effective
/// principal; returns those entries.
pub fn assert_tool_audit(
    conn: &Connection,
    mark: i64,
    tools_run: &[String],
    cell: &str,
) -> Vec<Value> {
    let mut stmt = conn
        .prepare(
            "SELECT details_json FROM audit_events
              WHERE event_type = 'ai_agent_tool_call' AND rowid > ?1 ORDER BY rowid",
        )
        .unwrap();
    let audits: Vec<Value> = stmt
        .query_map([mark], |r| r.get::<_, Option<String>>(0))
        .unwrap()
        .map(|d| serde_json::from_str(&d.unwrap().expect("details_json")).unwrap())
        .collect();
    let mut audited: Vec<String> = audits
        .iter()
        .map(|a| a["tool"].as_str().unwrap_or_default().to_string())
        .collect();
    let mut expected = tools_run.to_vec();
    audited.sort();
    expected.sort();
    assert_eq!(
        audited, expected,
        "{cell}: tool calls and their audit entries differ"
    );
    for a in &audits {
        assert!(
            a["agent_id"].as_str().is_some_and(|s| !s.is_empty()),
            "{cell}: {a}"
        );
        assert!(
            a["effective_principal"]["type"].as_str().is_some(),
            "{cell}: no effective principal in {a}"
        );
    }
    audits
}
