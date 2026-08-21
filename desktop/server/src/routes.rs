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
use lanesra_core::services::{auth_service, backup_service, workspace_service};

use crate::dispatch::{arg, dispatch, require_workspace_id, to_value};
use crate::security::{cors_layer, security_headers};
use crate::session::{clear_session_cookie, current_actor, set_session_cookie, SESSION_COOKIE};
use crate::state::SharedState;

pub fn build_router(state: SharedState, frontend_dir: PathBuf) -> Router {
    let index = frontend_dir.join("index.html");
    let static_service = ServeDir::new(frontend_dir).not_found_service(ServeFile::new(index));
    let cors = cors_layer(&state.security.allowed_origins);

    let mut router = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/invoke/:command", post(invoke))
        .merge(crate::api_v1::router())
        .merge(crate::events_stream::router())
        .merge(crate::admin_actions::router())
        .fallback_service(static_service)
        .layer(axum::middleware::from_fn_with_state(state.security.clone(), security_headers));

    if let Some(cors) = cors {
        router = router.layer(cors);
    }

    router.with_state(state)
}

async fn invoke(
    State(state): State<SharedState>,
    Path(command): Path<String>,
    jar: CookieJar,
    Json(args): Json<Value>,
) -> (CookieJar, Json<Value>) {
    // restore_backup replaces the live connection itself (see
    // backup_service::restore_from_package), so it needs mutable access to
    // the mutex slot rather than the shared `&Connection` every other
    // command gets - handled separately for that reason alone, the same
    // way login/logout are separated out for mutating the session cookie.
    if command == "restore_backup" {
        return match handle_restore(&state, &args, jar.clone()) {
            Ok((jar, value)) => (jar, Json(json!({"ok": true, "data": value}))),
            Err(err) => (jar, Json(json!({"ok": false, "error": err}))),
        };
    }

    let master_key = match crate::dispatch::resolve_master_key(&state.db_path) {
        Ok(key) => key,
        Err(err) => return (jar, Json(json!({"ok": false, "error": err.to_string()}))),
    };
    let conn = state.conn.lock().unwrap();
    match handle(&command, &args, &conn, jar.clone(), state.security.trust_proxy_https, &master_key) {
        Ok((jar, value)) => (jar, Json(json!({"ok": true, "data": value}))),
        Err(err) => (jar, Json(json!({"ok": false, "error": err}))),
    }
}

fn handle_restore(state: &SharedState, args: &Value, jar: CookieJar) -> AppResult<(CookieJar, Value)> {
    let mut conn = state.conn.lock().unwrap();
    let actor = current_actor(&conn, &jar)
        .ok_or_else(|| AppError::Validation("Not authenticated - please log in".into()))?;
    let package_base64: String = arg(args, "packageBase64")?;
    let manifest = backup_service::restore_from_package(&mut conn, &state.db_path, &package_base64, Some(&actor))?;
    Ok((jar, to_value(manifest)?))
}

/// Handles the commands that mutate the session cookie itself; everything
/// else is delegated to `dispatch`, which only needs read access to the
/// already-resolved actor. `secure_cookies` is `SecurityConfig::trust_proxy_https`.
fn handle(command: &str, args: &Value, conn: &Connection, jar: CookieJar, secure_cookies: bool, master_key: &[u8; 32]) -> AppResult<(CookieJar, Value)> {
    match command {
        "workspace_status" => {
            let workspace = lanesra_core::repositories::workspace_repo::get_current(conn)?;
            Ok((jar, to_value(workspace)?))
        }
        "first_run_setup" => {
            let setup: WorkspaceSetup = arg(args, "setup")?;
            let (workspace, user) = workspace_service::first_run_setup(conn, &setup)?;
            let token = session_repo::create(conn, &workspace.id, &user.id)?;
            let jar = set_session_cookie(jar, token, secure_cookies);
            Ok((jar, to_value((workspace, user))?))
        }
        "login" => {
            let credentials: Credentials = arg(args, "credentials")?;
            let workspace_id = require_workspace_id(conn)?;
            let user = auth_service::login(conn, &workspace_id, &credentials)?;
            let token = session_repo::create(conn, &workspace_id, &user.id)?;
            let jar = set_session_cookie(jar, token, secure_cookies);
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
            let value = dispatch(command, args, conn, Some(&actor), master_key)?;
            Ok((jar, value))
        }
    }
}
