use std::net::SocketAddr;
use std::path::PathBuf;

use lanesra_server::{build_router, ServerState};

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

    let state = ServerState::new(conn, db_path);
    let app = build_router(state, frontend_dir);

    let addr: SocketAddr = format!("{host}:{port}").parse().expect("invalid HOST/PORT");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("could not bind to address");
    tracing::info!(%addr, "Lanesra OS Team Workspace server listening");

    axum::serve(listener, app).await.expect("server error");
}
