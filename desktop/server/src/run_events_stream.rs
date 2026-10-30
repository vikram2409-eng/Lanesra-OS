//! WP-0112: live run events over Server-Sent Events (ADR-0008 §3, §5).
//!
//! - `GET /api/v1/runs/:id/events` follows one run. Each SSE message is one
//!   `lanesra.run-event` envelope, its SSE id is the event's `seq`, and a
//!   client that reconnects with `Last-Event-ID` (or `?after_seq=`) gets
//!   every event after it. The stream ends after `run.completed` or
//!   `run.failed`.
//! - `GET /api/v1/events/stream?run_id=a&run_id=b` follows several runs
//!   (`events_stream` hands those requests here). Its SSE id is a cursor
//!   across runs, so `Last-Event-ID` resumes it too. It stays open.
//!
//! `?level=summary` sends only `run.*` and `step.*` events; `steps`, the
//! default, sends every type. `tokens` and `debug` arrive with the
//! Playground (Phase 2) and are refused until then.
//!
//! Replay reads the stored events, never memory. The handler subscribes to
//! the bus before it reads them and drops anything already sent, so
//! nothing falls in between; a gap in what the bus delivers (a lagging
//! subscriber) is filled from the table.
//!
//! **Who receives events.** The UI uses its session cookie and must be an
//! Administrator (`run_event_service::require_viewer`); an API client uses
//! its Bearer key and needs the `events.read` scope. Either way only runs
//! of the caller's own workspace exist.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Path, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;
use futures_core::Stream;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;

use crate::dispatch::require_workspace_id;
use lanesra_core::models::run_event::{in_summary, RunEvent};
use lanesra_core::services::run_event_service;

use crate::api_v1::{app_err, authorize, err_json};
use crate::caller::Caller;
use crate::state::SharedState;

type Refusal = (StatusCode, Json<Value>);

pub fn router() -> Router<SharedState> {
    Router::new().route("/api/v1/runs/:id/events", get(run_events))
}

/// The query string as name/value pairs, keeping repeated names.
pub(crate) fn query_pairs(raw: Option<&str>) -> Vec<(String, String)> {
    raw.unwrap_or_default()
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decode(k), decode(v))
        })
        .collect()
}

fn decode(s: &str) -> String {
    let bytes = s.replace('+', " ").into_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(b) = u8::from_str_radix(hex, 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The caller's workspace, if it may receive run events: an API client
/// with `events.read`, or a signed-in Administrator.
pub(crate) fn authorize_run_events(
    state: &SharedState,
    headers: &HeaderMap,
    jar: &CookieJar,
    caller: Caller,
) -> Result<String, Refusal> {
    if headers.contains_key(axum::http::header::AUTHORIZATION) {
        let (_client, workspace_id) = authorize(state, headers, caller, "events.read")?;
        return Ok(workspace_id);
    }
    let conn = state.conn.lock().unwrap();
    let actor = crate::session::current_actor(&conn, jar)
        .ok_or_else(|| err_json(StatusCode::UNAUTHORIZED, "Sign in, or send an API key"))?;
    run_event_service::require_viewer(&conn, Some(&actor)).map_err(app_err)?;
    require_workspace_id(&conn).map_err(app_err)
}

#[derive(Clone, Copy)]
enum Level {
    Summary,
    Steps,
}

impl Level {
    fn parse(value: Option<&str>) -> Result<Self, Refusal> {
        match value.unwrap_or("steps") {
            "summary" => Ok(Level::Summary),
            "steps" => Ok(Level::Steps),
            "tokens" | "debug" => Err(err_json(
                StatusCode::BAD_REQUEST,
                "The tokens and debug levels are not available yet; use summary or steps",
            )),
            other => Err(err_json(
                StatusCode::BAD_REQUEST,
                &format!("Unknown level '{other}'; use summary or steps"),
            )),
        }
    }

    fn admits(self, event: &RunEvent) -> bool {
        match self {
            Level::Summary => in_summary(&event.event_type),
            Level::Steps => true,
        }
    }
}

fn is_terminal(event: &RunEvent) -> bool {
    matches!(event.event_type.as_str(), "run.completed" | "run.failed")
}

fn sse(event: &RunEvent, id: i64) -> Event {
    Event::default()
        .id(id.to_string())
        .json_data(event)
        .unwrap_or_else(|_| {
            Event::default()
                .event("error")
                .data("could not serialize an event")
        })
}

fn last_event_id(headers: &HeaderMap) -> Option<i64> {
    headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse().ok())
}

async fn run_events(
    State(state): State<SharedState>,
    Path(run_id): Path<String>,
    RawQuery(raw): RawQuery,
    headers: HeaderMap,
    jar: CookieJar,
    caller: Caller,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Refusal> {
    let workspace_id = authorize_run_events(&state, &headers, &jar, caller)?;
    let pairs: HashMap<String, String> = query_pairs(raw.as_deref()).into_iter().collect();
    let level = Level::parse(pairs.get("level").map(String::as_str))?;
    let after_seq = match pairs.get("after_seq") {
        Some(v) => v
            .parse::<i64>()
            .map_err(|_| err_json(StatusCode::BAD_REQUEST, "after_seq must be a number"))?,
        None => last_event_id(&headers).unwrap_or(0),
    };
    // Subscribe before reading, so nothing falls between replay and live.
    let mut rx = run_event_service::subscribe();
    let replay = {
        let conn = state.conn.lock().unwrap();
        run_event_service::events_for_run(&conn, &workspace_id, &run_id, after_seq)
            .map_err(app_err)?
    };

    let stream = async_stream::stream! {
        let mut last = after_seq;
        let mut pending = replay;
        loop {
            for event in std::mem::take(&mut pending) {
                if event.seq <= last {
                    continue;
                }
                last = event.seq;
                if level.admits(&event) {
                    yield Ok(sse(&event, event.seq));
                }
                if is_terminal(&event) {
                    return;
                }
            }
            match rx.recv().await {
                Ok(event) => {
                    if event.workspace_id != workspace_id || event.run_id != run_id || event.seq <= last {
                        continue;
                    }
                    if event.seq == last + 1 {
                        pending.push(Arc::unwrap_or_clone(event));
                    } else {
                        pending = refill(&state, &workspace_id, &run_id, last);
                    }
                }
                Err(RecvError::Lagged(_)) => {
                    pending = refill(&state, &workspace_id, &run_id, last);
                }
                Err(RecvError::Closed) => return,
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Stored events after `last`, to fill a gap in what the bus delivered.
fn refill(state: &SharedState, workspace_id: &str, run_id: &str, last: i64) -> Vec<RunEvent> {
    let conn = state.conn.lock().unwrap();
    run_event_service::events_for_run(&conn, workspace_id, run_id, last).unwrap_or_default()
}

/// `GET /api/v1/events/stream?run_id=...` (several runs).
pub(crate) async fn several_runs(
    state: SharedState,
    pairs: Vec<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    caller: Caller,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Refusal> {
    let workspace_id = authorize_run_events(&state, &headers, &jar, caller)?;
    let mut run_ids: Vec<String> = pairs
        .iter()
        .filter(|(k, _)| k == "run_id")
        .map(|(_, v)| v.clone())
        .collect();
    run_ids.sort();
    run_ids.dedup();
    if run_ids.len() > 100 {
        return Err(err_json(
            StatusCode::BAD_REQUEST,
            "Follow at most 100 runs at once",
        ));
    }
    let get = |name: &str| {
        pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };
    let level = Level::parse(get("level"))?;
    let after = match get("after") {
        Some(v) => v
            .parse::<i64>()
            .map_err(|_| err_json(StatusCode::BAD_REQUEST, "after must be a number"))?,
        None => last_event_id(&headers).unwrap_or(0),
    };
    let mut rx = run_event_service::subscribe();
    let replay = {
        let conn = state.conn.lock().unwrap();
        run_event_service::events_after_cursor(&conn, &workspace_id, &run_ids, after)
            .map_err(app_err)?
    };

    let stream = async_stream::stream! {
        // The last seq sent per run; an event at or below it is a duplicate.
        let mut sent: HashMap<String, i64> = HashMap::new();
        let mut pending = replay;
        loop {
            for event in std::mem::take(&mut pending) {
                let last = sent.entry(event.run_id.clone()).or_insert(0);
                if event.seq <= *last {
                    continue;
                }
                *last = event.seq;
                if level.admits(&event) {
                    yield Ok(sse(&event, event.cursor));
                }
            }
            match rx.recv().await {
                Ok(event) => {
                    if event.workspace_id == workspace_id && run_ids.contains(&event.run_id) {
                        pending.push(Arc::unwrap_or_clone(event));
                    }
                }
                Err(RecvError::Lagged(_)) => {
                    let conn = state.conn.lock().unwrap();
                    let mut refilled = Vec::new();
                    for run_id in &run_ids {
                        let last = sent.get(run_id).copied().unwrap_or(0);
                        refilled.extend(
                            run_event_service::events_for_run(&conn, &workspace_id, run_id, last)
                                .unwrap_or_default(),
                        );
                    }
                    drop(conn);
                    refilled.sort_by_key(|e| e.cursor);
                    pending = refilled;
                }
                Err(RecvError::Closed) => return,
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
