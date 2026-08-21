use std::net::SocketAddr;
use std::path::PathBuf;

use lanesra_server::{build_router, SecurityConfig, ServerState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let data_dir = std::env::var("LANESRA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"));
    std::fs::create_dir_all(&data_dir).expect("could not create data directory");

    let frontend_dir = std::env::var("LANESRA_FRONTEND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../dist"));

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let db_path = data_dir.join("lanesra.sqlite3");
    let conn = lanesra_core::db::open_workspace_db(&db_path).expect("could not open the workspace database");
    tracing::info!(path = %db_path.display(), "opened workspace database");

    let security = SecurityConfig::from_env();
    if security.trust_proxy_https {
        tracing::info!("trusting a reverse proxy for TLS - session cookie will be marked Secure");
    }
    if !security.allowed_origins.is_empty() {
        tracing::info!(origins = ?security.allowed_origins, "CORS enabled for the listed origins");
    }

    // Integration Hub (spec §15): recurring Integration Jobs need a real
    // background scheduler, and this server is the one long-running
    // process that exists to host one - see `job_scheduler`'s own doc
    // comment for why it opens its own DB connection rather than
    // sharing `state.conn`.
    let key_file_path = data_dir.join("secret.key");
    lanesra_server::job_scheduler::spawn(db_path.clone(), key_file_path, std::time::Duration::from_secs(60));

    let state = ServerState::new(conn, db_path, security);
    let app = build_router(state, frontend_dir);

    let addr: SocketAddr = format!("{host}:{port}").parse().expect("invalid HOST/PORT");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("could not bind to address");
    tracing::info!(%addr, "Lanesra OS Team Workspace server listening");

    axum::serve(listener, app).await.expect("server error");
}
