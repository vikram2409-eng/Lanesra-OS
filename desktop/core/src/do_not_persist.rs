//! WP-0127 (REQ-LOOP-021): values a run may use but must never store.
//!
//! An execution-graph node marked `"do_not_persist": true` (an agent or a
//! transform) produces a value the rest of the run can use - an access
//! token, a one-time code, a personal detail fetched for one step - but
//! that is never written to checkpoints, step records, traces or audit.
//! The runtime keeps it in memory only, and on resume (after an approval,
//! a delay or a restart) runs the node again to fetch it back.
//!
//! While a run executes, every such value is registered here for the
//! thread the run executes on (the same reason `principals` is
//! thread-local: a run never leaves its thread). Each persistence point
//! scrubs registered values out of what it writes, replacing them with
//! `[not persisted]`: the graph run repository, the audit repository and,
//! through the trace persist hook (WP-0111 AC-3), every span. Spans opened
//! while a marked node executes keep no bodies at all.
//!
//! Values shorter than `MIN_SCRUB_CHARS` are still kept out of
//! checkpoints, but cannot be scrubbed out of other text without mangling
//! it, so a node should not mark values that short.

use std::cell::{Cell, RefCell};

/// Replaces a value wherever it would have been stored.
pub const PLACEHOLDER: &str = "[not persisted]";

/// The shortest value scrubbed from other text.
pub const MIN_SCRUB_CHARS: usize = 4;

thread_local! {
    static VALUES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static SUPPRESS_BODIES: Cell<u32> = const { Cell::new(0) };
}

/// Values registered since this guard was created are forgotten when it
/// drops; a run opens one for its whole execution.
pub struct Scope {
    len: usize,
}

pub fn scope() -> Scope {
    super::services::trace_service::register_persist_hook(trace_hook);
    Scope {
        len: VALUES.with(|v| v.borrow().len()),
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        VALUES.with(|v| v.borrow_mut().truncate(self.len));
    }
}

/// While held, spans keep no bodies (a marked node is executing).
pub struct BodySuppression;

pub fn suppress_bodies() -> BodySuppression {
    SUPPRESS_BODIES.with(|c| c.set(c.get() + 1));
    BodySuppression
}

impl Drop for BodySuppression {
    fn drop(&mut self) {
        SUPPRESS_BODIES.with(|c| c.set(c.get().saturating_sub(1)));
    }
}

pub fn bodies_suppressed() -> bool {
    SUPPRESS_BODIES.with(|c| c.get() > 0)
}

fn register(text: &str) {
    if text.chars().count() < MIN_SCRUB_CHARS {
        return;
    }
    VALUES.with(|v| {
        let mut values = v.borrow_mut();
        // The JSON-escaped form too, so a value inside stored JSON is found.
        let escaped = serde_json::to_string(text).unwrap_or_default();
        let escaped = escaped.trim_matches('"').to_string();
        for form in [text.to_string(), escaped] {
            if !values.contains(&form) {
                values.push(form);
            }
        }
        // Longest first, so a value containing another is replaced whole.
        values.sort_by_key(|s| std::cmp::Reverse(s.len()));
    });
}

/// Registers every string inside `value` (a node's output).
pub fn mark(value: &serde_json::Value) {
    match value {
        serde_json::Value::String(s) => register(s),
        serde_json::Value::Array(items) => items.iter().for_each(mark),
        serde_json::Value::Object(map) => map.values().for_each(mark),
        _ => {}
    }
}

pub fn is_active() -> bool {
    VALUES.with(|v| !v.borrow().is_empty())
}

/// `text` with every registered value replaced by the placeholder.
pub fn scrub(text: &str) -> String {
    VALUES.with(|v| {
        let values = v.borrow();
        let mut out = text.to_string();
        for value in values.iter() {
            if out.contains(value.as_str()) {
                out = out.replace(value.as_str(), PLACEHOLDER);
            }
        }
        out
    })
}

pub fn scrub_opt(text: Option<&str>) -> Option<String> {
    text.map(scrub)
}

fn scrub_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            let scrubbed = scrub(s);
            *s = scrubbed;
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(scrub_value),
        serde_json::Value::Object(map) => map.values_mut().for_each(scrub_value),
        _ => {}
    }
}

/// The trace persist hook (WP-0111 AC-3): scrubs every attribute and body.
fn trace_hook(draft: &mut super::services::trace_service::PersistDraft) {
    if !is_active() {
        return;
    }
    for value in draft.attributes.values_mut() {
        scrub_value(value);
    }
    for (_, body) in draft.bodies.iter_mut() {
        *body = scrub(body);
    }
}
