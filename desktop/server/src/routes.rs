use std::path::PathBuf;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;
use rusqlite::Connection;
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};

use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::user::Credentials;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::session_repo;
use lanesra_core::services::{auth_service, workspace_service};

use crate::dispatch::{arg, dispatch, require_workspace_id, to_value};
use crate::session::{clear_session_cookie, current_actor, set_session_cookie, SESSION_COOKIE};
use crate::state::SharedState;

pub fn build_router(state: SharedState, frontend_dir: PathBuf) -> Router {
    let index = frontend_dir.join("index.html");
    let static_service = ServeDir::new(frontend_dir).not_found_service(ServeFile::new(index));

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/invoke/:command", post(invoke))
        .fallback_service(static_service)
        .with_state(state)
}

async fn invoke(
    State(state): State<SharedState>,
    Path(command): Path<String>,
    jar: CookieJar,
    Json(args): Json<Value>,
) -> (CookieJar, Json<Value>) {
    let conn = state.conn.lock().unwrap();
    match handle(&command, &args, &conn, jar.clone()) {
        Ok((jar, value)) => (jar, Json(json!({"ok": true, "data": value}))),
        Err(err) => (jar, Json(json!({"ok": false, "error": err}))),
    }
}

/// Handles the commands that mutate the session cookie itself; everything
/// else is delegated to `dispatch`, which only needs read access to the
/// already-resolved actor.
fn handle(command: &str, args: &Value, conn: &Connection, jar: CookieJar) -> AppResult<(CookieJar, Value)> {
    match command {
        "workspace_status" => {
            let workspace = lanesra_core::repositories::workspace_repo::get_current(conn)?;
            Ok((jar, to_value(workspace)?))
        }
        "first_run_setup" => {
            let setup: WorkspaceSetup = arg(args, "setup")?;
            let (workspace, user) = workspace_service::first_run_setup(conn, &setup)?;
            let token = session_repo::create(conn, &workspace.id, &user.id)?;
            let jar = set_session_cookie(jar, token);
            Ok((jar, to_value((workspace, user))?))
        }
        "login" => {
            let credentials: Credentials = arg(args, "credentials")?;
            let workspace_id = require_workspace_id(conn)?;
            let user = auth_service::login(conn, &workspace_id, &credentials)?;
            let token = session_repo::create(conn, &workspace_id, &user.id)?;
            let jar = set_session_cookie(jar, token);
            Ok((jar, to_value(user)?))
        }
        "logout" => {
            if let Some(cookie) = jar.get(SESSION_COOKIE) {
                session_repo::delete(conn, cookie.value())?;
            }
            Ok((clear_session_cookie(jar), Value::Null))
        }
        "current_user" => {
            let user = match current_actor(conn, &jar) {
                Some(user_id) => auth_service::resolve_user(conn, &user_id)?,
                None => None,
            };
            Ok((jar, to_value(user)?))
        }
        _ => {
            // Every other command touches business data and requires an
            // authenticated session - unlike the desktop app, this port is
            // reachable by anyone on the LAN, so the backend itself (not
            // just the frontend's login screen) must enforce this.
            let actor = current_actor(conn, &jar)
                .ok_or_else(|| AppError::Validation("Not authenticated - please log in".into()))?;
            let value = dispatch(command, args, conn, Some(&actor))?;
            Ok((jar, value))
        }
    }
}
