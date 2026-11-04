use std::path::{Path, PathBuf};

use axum::extract::{Path as AxumPath, State};
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
use lanesra_core::services::{
    auth_service, backup_service, web_session_service, workspace_service,
};

use crate::caller::Caller;
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
        .merge(crate::api_webhooks::router())
        .merge(crate::api_integration_admin::router())
        .merge(crate::agent_v1::router())
        .merge(crate::events_stream::router())
        .merge(crate::run_events_stream::router())
        .merge(crate::admin_actions::router())
        .merge(crate::mcp::router())
        .fallback_service(static_service)
        .layer(axum::middleware::from_fn_with_state(
            state.security.clone(),
            security_headers,
        ));

    if let Some(cors) = cors {
        router = router.layer(cors);
    }

    router.with_state(state)
}

async fn invoke(
    State(state): State<SharedState>,
    AxumPath(command): AxumPath<String>,
    caller: Caller,
    jar: CookieJar,
    Json(args): Json<Value>,
) -> (CookieJar, Json<Value>) {
    let client_addr = caller.0.map(|ip| ip.to_string());
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
    match handle(
        &command,
        &args,
        &conn,
        jar.clone(),
        state.security.trust_proxy_https,
        &master_key,
        &state.db_path,
        client_addr.as_deref(),
    ) {
        Ok((jar, value)) => (jar, Json(json!({"ok": true, "data": value}))),
        Err(err) => (jar, Json(json!({"ok": false, "error": err}))),
    }
}

fn handle_restore(
    state: &SharedState,
    args: &Value,
    jar: CookieJar,
) -> AppResult<(CookieJar, Value)> {
    let mut conn = state.conn.lock().unwrap();
    let actor = current_actor(&conn, &jar)
        .ok_or_else(|| AppError::Validation("Not authenticated - please log in".into()))?;
    let package_base64: String = arg(args, "packageBase64")?;
    let manifest = backup_service::restore_from_package(
        &mut conn,
        &state.db_path,
        &package_base64,
        Some(&actor),
    )?;
    Ok((jar, to_value(manifest)?))
}

/// Handles the commands that mutate the session cookie itself; everything
/// else is delegated to `dispatch`, which only needs read access to the
/// already-resolved actor. `secure_cookies` is `SecurityConfig::trust_proxy_https`.
#[allow(clippy::too_many_arguments)]
fn handle(
    command: &str,
    args: &Value,
    conn: &Connection,
    jar: CookieJar,
    secure_cookies: bool,
    master_key: &[u8; 32],
    db_path: &Path,
    client_addr: Option<&str>,
) -> AppResult<(CookieJar, Value)> {
    match command {
        "workspace_status" => {
            let workspace = lanesra_core::repositories::workspace_repo::get_current(conn)?;
            Ok((jar, to_value(workspace)?))
        }
        "first_run_setup" => {
            // WP-0106: a network-reachable server gates first-run behind the
            // one-time setup token printed at startup. No token file (desktop
            // or a test that never ran startup) means no gate.
            let provided = args.get("setup_token").and_then(Value::as_str);
            let enforced = crate::setup_token::verify(db_path, provided)?;
            let setup: WorkspaceSetup = arg(args, "setup")?;
            let (workspace, user) = workspace_service::first_run_setup(conn, &setup)?;
            if enforced {
                crate::setup_token::consume(db_path);
            }
            let token = session_repo::create(conn, &workspace.id, &user.id)?;
            let jar = set_session_cookie(jar, token, secure_cookies);
            Ok((jar, to_value((workspace, user))?))
        }
        "login" => {
            let credentials: Credentials = arg(args, "credentials")?;
            let workspace_id = require_workspace_id(conn)?;
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let user =
                auth_service::login_at(conn, &workspace_id, &credentials, client_addr, now_ms)?;
            let token = session_repo::create(conn, &workspace_id, &user.id)?;
            let jar = set_session_cookie(jar, token, secure_cookies);
            Ok((jar, to_value(user)?))
        }
        "list_active_sessions" => {
            let actor = current_actor(conn, &jar)
                .ok_or_else(|| AppError::Validation("Not authenticated - please log in".into()))?;
            let workspace_id = require_workspace_id(conn)?;
            let current_id = jar.get(SESSION_COOKIE).map(|c| c.value().to_string());
            let sessions = web_session_service::list_active_sessions(
                conn,
                &workspace_id,
                Some(&actor),
                current_id.as_deref(),
            )?;
            Ok((jar, to_value(sessions)?))
        }
        "sign_out_all_sessions" => {
            let actor = current_actor(conn, &jar)
                .ok_or_else(|| AppError::Validation("Not authenticated - please log in".into()))?;
            let workspace_id = require_workspace_id(conn)?;
            let ended = web_session_service::sign_out_all(conn, &workspace_id, Some(&actor))?;
            // The caller's own session was just ended; clear its cookie too.
            Ok((clear_session_cookie(jar), json!({ "ended": ended })))
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
