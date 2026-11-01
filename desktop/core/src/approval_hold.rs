//! WP-0150: a tool call the policy holds for approval pauses the run it is
//! part of, instead of being dropped.
//!
//! A pipeline step or an execution-graph agent node opens a `pausable()`
//! scope while its agent runs. When the Tool-Call Firewall records a held
//! call inside that scope it calls `hold`, and the agent loop stops with
//! `pending_error` once the call's result (the "requires approval" message)
//! is recorded. The step then reads `paused_on` and pauses the run on that
//! approval; approving it lets exactly that call run once
//! (`policy_engine_service::take_tool_grant`) and re-runs the step.
//!
//! Outside a pausable scope - the Assistant chat, a direct agent chat, a
//! consensus or peer-review step - a held call keeps its old behaviour: the
//! agent is told it needs approval and carries on, and an approval granted
//! later lets the same call run the next time it is made.
//!
//! Thread-local for the reason `principals` and `trace_service` are: a run
//! executes on one thread.

use std::cell::{Cell, RefCell};

use crate::domain::{AppError, ErrorClass};

/// The ADR-0009 code a paused agent loop stops with.
pub const PENDING_CODE: &str = "policy.approval_required";

thread_local! {
    static PAUSABLE: Cell<u32> = const { Cell::new(0) };
    static HELD: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// While held, a held tool call pauses the run instead of being dropped.
pub struct Pausable;

pub fn pausable() -> Pausable {
    PAUSABLE.with(|c| c.set(c.get() + 1));
    HELD.with(|h| h.borrow_mut().take());
    Pausable
}

impl Drop for Pausable {
    fn drop(&mut self) {
        PAUSABLE.with(|c| c.set(c.get().saturating_sub(1)));
    }
}

pub fn is_pausable() -> bool {
    PAUSABLE.with(|c| c.get() > 0)
}

/// Records that the current step must pause on `approval_id`.
pub fn hold(approval_id: &str) {
    if is_pausable() {
        HELD.with(|h| *h.borrow_mut() = Some(approval_id.to_string()));
    }
}

/// The approval the agent loop must stop for, if one was just held.
pub fn held() -> Option<String> {
    HELD.with(|h| h.borrow().clone())
}

/// The error a paused agent loop returns.
pub fn pending_error(tool: &str, approval_id: &str) -> AppError {
    AppError::classified(
        ErrorClass::Policy,
        PENDING_CODE,
        format!("'{tool}' is waiting for an administrator's approval ({approval_id})"),
    )
}

/// The approval a step paused on: `Some` when `result` is the pending
/// error and a call was held. Clears the hold.
pub fn paused_on<T>(result: &crate::domain::AppResult<T>) -> Option<String> {
    let pending = matches!(
        result,
        Err(AppError::Classified { code, .. }) if *code == PENDING_CODE
    );
    let held = HELD.with(|h| h.borrow_mut().take());
    if pending {
        held
    } else {
        None
    }
}
