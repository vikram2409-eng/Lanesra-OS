//! WP-0111: records a span tree for every run (ADR-0008 §1).
//!
//! A run opens its trace with `begin_run`, which writes the root `run`
//! span. While the trace is open, `start` writes a child span under
//! whichever span is innermost on this thread - a pipeline or graph step,
//! a model call, a tool call, a firewall verdict - and `Span::end` closes
//! it. A delegation opens a `step` span inside the delegating tool call,
//! so the delegate's model and tool calls nest beneath it.
//!
//! The open spans live on a thread-local stack, the same reason
//! `principals::RunIdentityGuard` is thread-local: a run, and everything it
//! delegates to, executes on one thread because its database connection
//! cannot leave it, and nothing in a run is awaited concurrently. With no
//! trace open (the Assistant chat, a direct agent chat) `start` does
//! nothing.
//!
//! Tracing never fails a run: a span that cannot be written is skipped.
//!
//! **Bodies.** Prompt and output text is stored only when the workspace's
//! `trace_bodies` setting is `redacted` (after DLP redaction) or `full`, in
//! `ai_run_span_bodies`; with `off`, the default, none is written. Agent
//! credentials are removed in every mode. The root span records the
//! setting in force (`lanesra.trace_bodies`).
//!
//! **Do-not-persist hook (WP-0111 AC-3).** Every span's attributes and
//! bodies pass through the hooks registered with `register_persist_hook`
//! before anything is written. WP-0127 registers the do-not-persist state
//! fields here; until then the list is empty.
//!
//! **Cost of tracing (WP-0111 AC-2).** The time spent writing spans on this
//! thread is counted (`write_time_on_this_thread`); the budget is 5 ms per
//! round (WP-0015, agreed by the owner on 2026-10-23).
//!
//! **Run events (WP-0112).** Opening and closing spans also writes the run's
//! live events (ADR-0008 §2): a new root `run.started`, a step
//! `step.started` and `step.completed`, a model call `round.completed`, a
//! tool call `tool.called`, a firewall verdict that refuses a call
//! `tool.denied` (or `approval.requested` when it holds the call for a
//! person), and a closed root `run.completed` or `run.failed`. Each gets
//! the run's next gapless `seq` and goes through the same persist hooks as
//! the span. Writing and publishing them counts towards the same budget.

use std::cell::{Cell, RefCell};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use rusqlite::Connection;
use serde_json::{json, Map, Value};

use crate::domain::ids::now_iso;
use crate::domain::{AppError, AppResult};
use crate::models::run_event::{RunEvent, RUN_EVENT_SCHEMA, RUN_EVENT_VERSION};
use crate::models::run_trace::{RunSpan, SpanBody, TraceBodiesInput, TRACE_BODY_MODES};
use crate::repositories::{ai_settings_repo, audit_repo, run_event_repo, run_span_repo};

/// The longest body kept per span part; the rest is cut.
pub const MAX_BODY_CHARS: usize = 64 * 1024;

/// What a span reached when it closed (ADR-0008 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    Ok,
    Error,
    Cancelled,
}

impl SpanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SpanStatus::Ok => "ok",
            SpanStatus::Error => "error",
            SpanStatus::Cancelled => "cancelled",
        }
    }
}

/// A span about to be written: hooks may remove or change attributes and
/// bodies.
pub struct PersistDraft<'a> {
    pub kind: &'a str,
    pub attributes: &'a mut Map<String, Value>,
    pub bodies: &'a mut Vec<(&'static str, String)>,
}

pub type PersistHook = fn(&mut PersistDraft);

static PERSIST_HOOKS: RwLock<Vec<PersistHook>> = RwLock::new(Vec::new());

/// Adds a hook every span passes through before it is written (AC-3).
pub fn register_persist_hook(hook: PersistHook) {
    let mut hooks = PERSIST_HOOKS.write().unwrap_or_else(|e| e.into_inner());
    if !hooks.iter().any(|h| *h as usize == hook as usize) {
        hooks.push(hook);
    }
}

fn apply_hooks(
    kind: &str,
    attributes: &mut Map<String, Value>,
    bodies: &mut Vec<(&'static str, String)>,
) {
    let hooks = PERSIST_HOOKS.read().unwrap_or_else(|e| e.into_inner());
    for hook in hooks.iter() {
        hook(&mut PersistDraft {
            kind,
            attributes,
            bodies,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyMode {
    Off,
    Redacted,
    Full,
}

impl BodyMode {
    fn parse(s: &str) -> Self {
        match s {
            "redacted" => BodyMode::Redacted,
            "full" => BodyMode::Full,
            _ => BodyMode::Off,
        }
    }
}

struct ActiveTrace {
    trace_id: String,
    workspace_id: String,
    run_kind: String,
    run_id: String,
    seq: i64,
    /// The last run event `seq` written (WP-0112).
    event_seq: i64,
    bodies: BodyMode,
    /// Open spans, innermost last.
    open: Vec<OpenSpan>,
}

struct OpenSpan {
    span_id: String,
    is_step: bool,
}

thread_local! {
    static TRACES: RefCell<Vec<ActiveTrace>> = const { RefCell::new(Vec::new()) };
    static WRITE_TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
}

/// Time spent writing spans on this thread since the last reset.
pub fn write_time_on_this_thread() -> Duration {
    WRITE_TIME.with(Cell::get)
}

pub fn reset_write_time_on_this_thread() {
    WRITE_TIME.with(|t| t.set(Duration::ZERO));
}

/// Runs a write, counts its time, and ignores a failure: tracing never
/// fails a run.
fn timed_write(f: impl FnOnce() -> rusqlite::Result<()>) {
    let started = Instant::now();
    let _ = f();
    WRITE_TIME.with(|t| t.set(t.get() + started.elapsed()));
}

fn new_trace_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn new_span_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..16].to_string()
}

fn object(attrs: Value) -> Map<String, Value> {
    match attrs {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn clean_body(mode: BodyMode, text: &str) -> String {
    let text = super::agent_identity_service::redact_credential_material(text);
    let text = match mode {
        BodyMode::Redacted => super::dlp_service::redact(&text),
        _ => text,
    };
    if text.chars().count() > MAX_BODY_CHARS {
        let cut: String = text.chars().take(MAX_BODY_CHARS).collect();
        format!("{cut}…[cut at {MAX_BODY_CHARS} characters]")
    } else {
        text
    }
}

/// A span that is open until `end` (or `finish` for a run) is called. A
/// span dropped without being ended leaves its row open (no end time),
/// which is what a reader sees for work that never finished.
pub struct Span {
    inner: Option<SpanInner>,
}

struct SpanInner {
    span_id: String,
    parent: Option<String>,
    kind: String,
    name: String,
    started: Instant,
    attributes: Map<String, Value>,
    bodies: Vec<(&'static str, String)>,
}

impl Span {
    fn noop() -> Self {
        Span { inner: None }
    }

    pub fn is_recording(&self) -> bool {
        self.inner.is_some()
    }

    pub fn span_id(&self) -> Option<&str> {
        self.inner.as_ref().map(|i| i.span_id.as_str())
    }

    /// Adds or replaces an attribute, written when the span ends.
    pub fn set(&mut self, key: &str, value: Value) {
        if let Some(inner) = &mut self.inner {
            inner.attributes.insert(key.to_string(), value);
        }
    }

    /// Keeps `text` as this span's `input` or `output` body, if the
    /// workspace's `trace_bodies` setting keeps bodies at all.
    pub fn body(&mut self, part: &'static str, text: &str) {
        let Some(inner) = &mut self.inner else { return };
        // WP-0127: a do-not-persist node's own spans keep no bodies.
        if crate::do_not_persist::bodies_suppressed() {
            return;
        }
        let mode = TRACES.with(|t| t.borrow().last().map(|a| a.bodies));
        match mode {
            Some(BodyMode::Off) | None => {}
            Some(mode) => inner.bodies.push((part, clean_body(mode, text))),
        }
    }

    /// Closes the span with `status`, adding `attrs`.
    pub fn end(mut self, conn: &Connection, status: SpanStatus, attrs: Value) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };
        for (k, v) in object(attrs) {
            inner.attributes.insert(k, v);
        }
        let duration_ms = inner.started.elapsed().as_millis() as i64;
        apply_hooks(&inner.kind, &mut inner.attributes, &mut inner.bodies);
        let event = closing_event(&inner, status, duration_ms);
        let attributes_json = Value::Object(inner.attributes).to_string();
        timed_write(|| {
            run_span_repo::finish(
                conn,
                &inner.span_id,
                status.as_str(),
                &now_iso(),
                duration_ms,
                &attributes_json,
            )
        });
        for (part, body) in &inner.bodies {
            timed_write(|| run_span_repo::put_body(conn, &inner.span_id, part, body));
        }
        if let Some((event_type, data)) = event {
            emit(
                conn,
                event_type,
                Some(&inner.span_id),
                inner.parent.as_deref(),
                data,
            );
        }
        pop_span(&inner.span_id);
    }

    /// Closes the span as `ok`, or as `error` with the error's code and
    /// message.
    pub fn end_result<T>(self, conn: &Connection, result: &AppResult<T>, attrs: Value) {
        match result {
            Ok(_) => self.end(conn, SpanStatus::Ok, attrs),
            Err(e) => {
                let mut attrs = object(attrs);
                attrs.extend(error_attributes(e));
                self.end(conn, SpanStatus::Error, Value::Object(attrs));
            }
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(inner) = &self.inner {
            pop_span(&inner.span_id);
        }
    }
}

/// `lanesra.error.*` attributes for a failed span (ADR-0009 class and code).
pub fn error_attributes(e: &AppError) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert(
        "lanesra.error.class".into(),
        json!(format!("{:?}", e.class()).to_lowercase()),
    );
    if let AppError::Classified { code, .. } = e {
        m.insert("lanesra.error.code".into(), json!(code));
    }
    let message = super::agent_identity_service::redact_credential_material(&e.to_string());
    m.insert(
        "lanesra.error.message".into(),
        json!(message.chars().take(500).collect::<String>()),
    );
    m
}

fn pop_span(span_id: &str) {
    TRACES.with(|t| {
        if let Some(active) = t.borrow_mut().last_mut() {
            if let Some(pos) = active.open.iter().rposition(|s| s.span_id == span_id) {
                active.open.truncate(pos);
            }
        }
    });
}

/// A short text attribute for an event's `data`: at most 500 characters.
fn short(value: Option<&Value>) -> Value {
    match value {
        Some(Value::String(s)) if s.chars().count() > 500 => {
            json!(s.chars().take(500).collect::<String>())
        }
        Some(v) => v.clone(),
        None => Value::Null,
    }
}

/// The event a closing span produces, if any.
fn closing_event(
    inner: &SpanInner,
    status: SpanStatus,
    duration_ms: i64,
) -> Option<(&'static str, Map<String, Value>)> {
    let a = &inner.attributes;
    let mut data = Map::new();
    let event_type = match inner.kind.as_str() {
        "run" => {
            let outcome = a
                .get("lanesra.run.status")
                .and_then(Value::as_str)
                .map(str::to_string);
            data.insert("status".into(), json!(status.as_str()));
            data.insert("duration_ms".into(), json!(duration_ms));
            if let Some(outcome) = outcome {
                data.insert("outcome".into(), json!(outcome));
            }
            if let Some(Value::String(reason)) = a.get("lanesra.stop_reason") {
                data.insert("stop_reason".into(), json!(reason));
            }
            if status == SpanStatus::Ok {
                "run.completed"
            } else {
                data.insert("error_code".into(), short(a.get("lanesra.error.code")));
                data.insert("message".into(), short(a.get("lanesra.error.message")));
                "run.failed"
            }
        }
        "step" => {
            data.insert("name".into(), json!(inner.name));
            data.insert("status".into(), json!(status.as_str()));
            data.insert("duration_ms".into(), json!(duration_ms));
            if status != SpanStatus::Ok {
                data.insert("error_code".into(), short(a.get("lanesra.error.code")));
            }
            "step.completed"
        }
        "model_call" => {
            data.insert("round".into(), short(a.get("lanesra.round")));
            data.insert("status".into(), json!(status.as_str()));
            data.insert("provider".into(), short(a.get("gen_ai.system")));
            data.insert("model".into(), short(a.get("gen_ai.request.model")));
            data.insert(
                "input_tokens".into(),
                short(a.get("gen_ai.usage.input_tokens")),
            );
            data.insert(
                "output_tokens".into(),
                short(a.get("gen_ai.usage.output_tokens")),
            );
            data.insert("served_by".into(), short(a.get("lanesra.served_by")));
            data.insert("tools_off".into(), short(a.get("lanesra.tools_off")));
            data.insert("duration_ms".into(), json!(duration_ms));
            "round.completed"
        }
        "tool_call" => {
            data.insert("tool".into(), json!(inner.name));
            data.insert("status".into(), json!(status.as_str()));
            data.insert("outcome".into(), short(a.get("lanesra.tool.outcome")));
            data.insert("duration_ms".into(), json!(duration_ms));
            if status != SpanStatus::Ok {
                data.insert("error_code".into(), short(a.get("lanesra.error.code")));
            }
            "tool.called"
        }
        "policy" => {
            let verdict = a
                .get("lanesra.policy.verdict")
                .and_then(Value::as_str)
                .unwrap_or("allow");
            data.insert("tool".into(), short(a.get("lanesra.tool.name")));
            data.insert("verdict".into(), json!(verdict));
            data.insert("risk".into(), short(a.get("lanesra.policy.risk")));
            match verdict {
                "allow" => return None,
                "require_approval" => {
                    data.insert("subject".into(), json!("tool_call"));
                    "approval.requested"
                }
                _ => {
                    data.insert("reason".into(), short(a.get("lanesra.error.code")));
                    "tool.denied"
                }
            }
        }
        _ => return None,
    };
    Some((event_type, data))
}

/// Writes and publishes one run event in this thread's trace; does nothing
/// with no trace open. `step_id` is the innermost open step span.
fn emit(
    conn: &Connection,
    event_type: &'static str,
    span_id: Option<&str>,
    parent_span_id: Option<&str>,
    mut data: Map<String, Value>,
) {
    let mut no_bodies = Vec::new();
    apply_hooks("event", &mut data, &mut no_bodies);
    let event = TRACES.with(|t| {
        let traces = t.borrow();
        let active = traces.last()?;
        let step_id = active
            .open
            .iter()
            .rev()
            .find(|s| s.is_step)
            .map(|s| s.span_id.clone());
        Some(RunEvent {
            schema: RUN_EVENT_SCHEMA.into(),
            v: RUN_EVENT_VERSION,
            seq: active.event_seq + 1,
            event_type: event_type.into(),
            at: now_iso(),
            workspace_id: active.workspace_id.clone(),
            run_kind: active.run_kind.clone(),
            run_id: active.run_id.clone(),
            step_id,
            span_id: span_id.map(str::to_string),
            parent_span_id: parent_span_id.map(str::to_string),
            data: Value::Object(data),
            cursor: 0,
        })
    });
    let Some(event) = event else { return };
    let started = Instant::now();
    let stored = super::run_event_service::publish(conn, event);
    WRITE_TIME.with(|t| t.set(t.get() + started.elapsed()));
    if stored {
        TRACES.with(|t| {
            if let Some(active) = t.borrow_mut().last_mut() {
                active.event_seq += 1;
            }
        });
    }
}

/// `approval.requested` for a run that now waits for a person: a pipeline
/// step or an execution-graph Approval node. `data` names what waits.
pub fn emit_approval_requested(conn: &Connection, data: Value) {
    emit(conn, "approval.requested", None, None, object(data));
}

/// Opens a child span under the innermost open span of this thread's
/// trace. Does nothing when no trace is open.
pub fn start(conn: &Connection, kind: &str, name: &str, attrs: Value) -> Span {
    let opened = TRACES.with(|t| {
        let mut traces = t.borrow_mut();
        let active = traces.last_mut()?;
        active.seq += 1;
        let span_id = new_span_id();
        let parent = active.open.last().map(|s| s.span_id.clone());
        active.open.push(OpenSpan {
            span_id: span_id.clone(),
            is_step: kind == "step",
        });
        Some((
            span_id,
            parent,
            active.trace_id.clone(),
            active.workspace_id.clone(),
            active.run_kind.clone(),
            active.run_id.clone(),
            active.seq,
        ))
    });
    let Some((span_id, parent, trace_id, workspace_id, run_kind, run_id, seq)) = opened else {
        return Span::noop();
    };
    let mut attributes = object(attrs);
    let mut no_bodies = Vec::new();
    apply_hooks(kind, &mut attributes, &mut no_bodies);
    let attributes_json = Value::Object(attributes.clone()).to_string();
    timed_write(|| {
        run_span_repo::insert(
            conn,
            &span_id,
            &trace_id,
            parent.as_deref(),
            &workspace_id,
            &run_kind,
            &run_id,
            seq,
            kind,
            name,
            &now_iso(),
            &attributes_json,
        )
    });
    if kind == "step" {
        let mut data = Map::new();
        data.insert("name".into(), json!(name));
        data.insert("agent".into(), short(attributes.get("lanesra.agent.name")));
        emit(
            conn,
            "step.started",
            Some(&span_id),
            parent.as_deref(),
            data,
        );
    }
    Span {
        inner: Some(SpanInner {
            span_id,
            parent,
            kind: kind.to_string(),
            name: name.to_string(),
            started: Instant::now(),
            attributes,
            bodies: Vec::new(),
        }),
    }
}

/// A run's open trace. `finish` closes the root span; dropping it without
/// `finish` leaves the root open, which is right for a run that is now
/// waiting (an approval, a delay) and will continue the same trace.
pub struct RunTrace {
    root: Span,
}

impl RunTrace {
    pub fn trace_id(&self) -> Option<String> {
        self.root.inner.as_ref()?;
        TRACES.with(|t| t.borrow().last().map(|a| a.trace_id.clone()))
    }

    /// Closes the root span. `None` leaves it open for a run that waits.
    pub fn finish(mut self, conn: &Connection, status: Option<SpanStatus>, attrs: Value) {
        if let Some(status) = status {
            let root = std::mem::replace(&mut self.root, Span::noop());
            root.end(conn, status, attrs);
        }
    }
}

impl Drop for RunTrace {
    fn drop(&mut self) {
        // Leave the root row as it is; only this thread's frame goes.
        let root = std::mem::replace(&mut self.root, Span::noop());
        if root.inner.is_some() {
            std::mem::forget(root);
        }
        TRACES.with(|t| {
            t.borrow_mut().pop();
        });
    }
}

/// Opens (or, for a resumed run, reopens) the trace of `run_kind`/`run_id`
/// on this thread. A run started inside another run's step gets its own
/// trace and records the parent's trace id.
pub fn begin_run(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
    name: &str,
    attrs: Value,
) -> RunTrace {
    let mode = ai_settings_repo::ensure_default(conn, workspace_id)
        .map(|s| BodyMode::parse(&s.trace_bodies))
        .unwrap_or(BodyMode::Off);
    let parent_trace = TRACES.with(|t| t.borrow().last().map(|a| a.trace_id.clone()));
    let existing = run_span_repo::root_for_run(conn, run_kind, run_id)
        .ok()
        .flatten();
    let seq = run_span_repo::max_seq(conn, run_kind, run_id).unwrap_or(0);
    let event_seq = run_event_repo::max_seq(conn, run_kind, run_id).unwrap_or(0);
    let (trace_id, root_id, reopened) = match &existing {
        Some(root) => (root.trace_id.clone(), root.span_id.clone(), true),
        None => (new_trace_id(), new_span_id(), false),
    };
    let mut attributes = existing
        .as_ref()
        .map(|r| object(r.attributes.clone()))
        .unwrap_or_default();
    attributes.extend(object(attrs));
    attributes.insert("lanesra.run.kind".into(), json!(run_kind));
    attributes.insert("lanesra.run.id".into(), json!(run_id));
    attributes.insert(
        "lanesra.trace_bodies".into(),
        json!(match mode {
            BodyMode::Off => "off",
            BodyMode::Redacted => "redacted",
            BodyMode::Full => "full",
        }),
    );
    if let Some(parent) = parent_trace {
        attributes.insert("lanesra.parent_trace_id".into(), json!(parent));
    }
    let mut no_bodies = Vec::new();
    apply_hooks("run", &mut attributes, &mut no_bodies);
    if !reopened {
        let attributes_json = Value::Object(attributes.clone()).to_string();
        timed_write(|| {
            run_span_repo::insert(
                conn,
                &root_id,
                &trace_id,
                None,
                workspace_id,
                run_kind,
                run_id,
                seq + 1,
                "run",
                name,
                &now_iso(),
                &attributes_json,
            )
        });
    }
    TRACES.with(|t| {
        t.borrow_mut().push(ActiveTrace {
            trace_id,
            workspace_id: workspace_id.to_string(),
            run_kind: run_kind.to_string(),
            run_id: run_id.to_string(),
            seq: if reopened { seq } else { seq + 1 },
            event_seq,
            bodies: mode,
            open: vec![OpenSpan {
                span_id: root_id.clone(),
                is_step: false,
            }],
        })
    });
    if !reopened {
        let mut data = Map::new();
        data.insert("name".into(), json!(name));
        for (key, field) in [
            ("lanesra.run.target_type", "target_type"),
            ("lanesra.run.topology", "topology"),
            ("lanesra.run.triggered_by", "triggered_by"),
        ] {
            if let Some(v) = attributes.get(key) {
                data.insert(field.into(), short(Some(v)));
            }
        }
        emit(conn, "run.started", Some(&root_id), None, data);
    }
    RunTrace {
        root: Span {
            inner: Some(SpanInner {
                span_id: root_id,
                parent: None,
                kind: "run".into(),
                name: name.to_string(),
                started: Instant::now(),
                attributes,
                bodies: Vec::new(),
            }),
        },
    }
}

/// Every span of a run, in the order they started. Any signed-in user of
/// the workspace may read the tree; bodies need `span_bodies`.
pub fn spans_for_run(
    conn: &Connection,
    workspace_id: &str,
    run_kind: &str,
    run_id: &str,
) -> AppResult<Vec<RunSpan>> {
    Ok(run_span_repo::list_for_run(
        conn,
        workspace_id,
        run_kind,
        run_id,
    )?)
}

/// A span's stored bodies - prompts, outputs, tool arguments and results -
/// for an Administrator only (ADR-0008 §4: bodies are a separate,
/// permission-checked read).
pub fn span_bodies(
    conn: &Connection,
    workspace_id: &str,
    span_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<SpanBody>> {
    super::user_service::require_admin(conn, actor_user_id)?;
    Ok(run_span_repo::bodies_for_span(conn, workspace_id, span_id)?)
}

/// Sets whether traces keep prompt and output bodies (ADR-0008 §4).
/// Administrator only, and audited.
pub fn set_trace_bodies(
    conn: &Connection,
    workspace_id: &str,
    input: &TraceBodiesInput,
    actor_user_id: Option<&str>,
) -> AppResult<crate::models::ai::AiSettings> {
    super::user_service::require_admin(conn, actor_user_id)?;
    let mode = input.trace_bodies.trim();
    if !TRACE_BODY_MODES.contains(&mode) {
        return Err(AppError::Validation(
            "Trace bodies must be off, redacted or full".into(),
        ));
    }
    let before = ai_settings_repo::ensure_default(conn, workspace_id)?;
    ai_settings_repo::set_trace_bodies(conn, workspace_id, mode)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "ai_trace_bodies_changed",
        Some("ai_settings"),
        Some(workspace_id),
        &format!(
            "Trace bodies changed from '{}' to '{mode}'",
            before.trace_bodies
        ),
        Some(&json!({"from": before.trace_bodies, "to": mode}).to_string()),
    )?;
    Ok(ai_settings_repo::ensure_default(conn, workspace_id)?)
}
