//! AI & Agentic Layer, Phase 6b: the inbound webhook Trigger - "call an
//! Agent or Pipeline from anywhere and get its answer back", the same
//! Bearer-authenticated `/api/v1` surface the generic REST API already
//! exposes (`api_v1.rs`), reusing its exact `authorize`/`logged` helpers
//! rather than duplicating the auth/rate-limit/logging plumbing. Unlike
//! a schedule Trigger or the `run_ai_agent` Workflow Automation action
//! (both enqueue-and-drain - see `ai_orchestration_service`'s own doc
//! comment), this runs **inline**: the caller explicitly wants an answer
//! back in the response, and there's no record-save-transaction pressure
//! forcing a queue here the way the synchronous workflow engine has.
//!
//! Desktop has no inbound listening socket at all, so this route only
//! ever exists when Team Workspace mode is the one actually running -
//! same platform boundary `api_v1.rs`'s own doc comment already states.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use lanesra_core::services::ai_orchestration_service;

use crate::admin_actions::run_with_own_connection;
use crate::api_v1::{app_err, authorize, logged};
use crate::state::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/v1/agents/:agent_id/trigger", post(trigger_agent))
        .route("/api/v1/agent-pipelines/:pipeline_id/trigger", post(trigger_pipeline))
}

#[derive(Debug, Deserialize)]
struct TriggerBody {
    #[serde(default)]
    input: String,
}

async fn trigger_agent(State(state): State<SharedState>, headers: HeaderMap, Path(agent_id): Path<String>, Json(body): Json<TriggerBody>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    trigger(state, headers, "agent", agent_id, body).await
}

async fn trigger_pipeline(State(state): State<SharedState>, headers: HeaderMap, Path(pipeline_id): Path<String>, Json(body): Json<TriggerBody>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    trigger(state, headers, "pipeline", pipeline_id, body).await
}

async fn trigger(state: SharedState, headers: HeaderMap, target_type: &'static str, target_id: String, body: TriggerBody) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (client, workspace_id) = authorize(&state, &headers, "agents.trigger")?;
    let db_path = state.db_path.clone();
    let master_key = crate::dispatch::resolve_master_key(&state.db_path).map_err(app_err)?;
    // An API client "acts as" its owner for authorization purposes -
    // the same reasoning `ai_orchestration_service::run_triggered`'s own
    // doc comment names for why an admin-scoped agent/pipeline can run
    // unattended at all: only when `actor` genuinely resolves to a real
    // Administrator.
    let actor = client.owner_user_id.clone();
    let workspace_id_for_log = workspace_id.clone();
    let data = run_with_own_connection(db_path, move |conn| async move {
        logged(&conn, &workspace_id_for_log, ai_orchestration_service::run_triggered(&conn, &workspace_id, &master_key, target_type, &target_id, actor.as_deref(), &body.input, Some("webhook"), None, None).await)
    })
    .await?;
    Ok(Json(json!({"ok": true, "data": data})))
}
