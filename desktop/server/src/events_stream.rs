//! Integration Hub (spec §11/§23): a real Server-Sent-Events endpoint
//! streaming `integration_executions` rows as they're written - the
//! same unified execution log the Logs & Monitoring screen already
//! shows (spec §23), made live instead of poll-on-refresh. A short
//! (1s) poll of this server's own SQLite file stands in for a real
//! pub/sub message bus this codebase doesn't have - genuinely proven
//! against a real spawned server with a real SSE client reading the
//! stream, not a live external broker.
//!
//! Deliberately opens its **own** SQLite connection rather than sharing
//! `ServerState.conn`, for the same reason `job_scheduler` does: a
//! stream's generator function is held across many `.await` points for
//! as long as the client stays connected, and `ServerState.conn`'s
//! `std::sync::MutexGuard` isn't `Send` - it can't be captured across an
//! await inside a value that must itself be `Send + 'static` to be
//! returned from an axum handler. Auth reuses `api_v1::authorize`
//! (`events.read` scope) exactly as every other `/api/v1` route does.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::{Json, Router};
use futures_core::Stream;
use serde_json::{json, Value};

use crate::api_v1::authorize;
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new().route("/api/v1/events/stream", get(stream_events))
}

async fn stream_events(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<Value>)> {
    let (_client, workspace_id) = authorize(&state, &headers, "events.read")?;
    let db_path = state.db_path.clone();

    let stream = async_stream::stream! {
        let conn = match lanesra_core::db::open_workspace_db(&db_path) {
            Ok(conn) => conn,
            Err(e) => {
                yield Ok(Event::default().event("error").data(format!("could not open the workspace database: {e}")));
                return;
            }
        };
        // Start from "now" - a client connecting to the stream sees new
        // activity from that point on, not the entire history (the REST
        // `list_executions`/Overview endpoints already cover history).
        let mut last_rowid: i64 = conn
            .query_row("SELECT COALESCE(MAX(rowid), 0) FROM integration_executions WHERE workspace_id = ?1", [&workspace_id], |r| r.get(0))
            .unwrap_or(0);

        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        loop {
            ticker.tick().await;
            let query_result = conn
                .prepare(
                    "SELECT rowid, id, execution_type, status, direction, started_at, ended_at, records_written, records_failed, error_message
                     FROM integration_executions WHERE workspace_id = ?1 AND rowid > ?2 ORDER BY rowid",
                )
                .and_then(|mut stmt| {
                    stmt.query_map(rusqlite::params![workspace_id, last_rowid], |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            json!({
                                "id": r.get::<_, String>(1)?,
                                "execution_type": r.get::<_, String>(2)?,
                                "status": r.get::<_, String>(3)?,
                                "direction": r.get::<_, String>(4)?,
                                "started_at": r.get::<_, String>(5)?,
                                "ended_at": r.get::<_, Option<String>>(6)?,
                                "records_written": r.get::<_, i64>(7)?,
                                "records_failed": r.get::<_, i64>(8)?,
                                "error_message": r.get::<_, Option<String>>(9)?,
                            }),
                        ))
                    })
                    .map(|rows| rows.flatten().collect::<Vec<_>>())
                });

            let Ok(rows) = query_result else { continue };
            for (rowid, payload) in rows {
                last_rowid = rowid;
                let event = Event::default().event("execution").json_data(payload).unwrap_or_else(|_| Event::default().event("error").data("could not serialize execution row"));
                yield Ok(event);
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
