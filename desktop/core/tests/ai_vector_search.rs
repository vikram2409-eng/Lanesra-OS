//! AI & Agentic Layer, Phase 7g (part 2): vector search over Custom
//! Object records - the reindex queue (migration 0048's triggers ->
//! `vector_search_service::drain_pending_embeddings`/`reindex_workspace`),
//! real embeddings via the configured provider (`ai_service::embed_texts`),
//! and ranking by cosine similarity (`semantic_search_records`) rather
//! than FTS5's bm25() keyword match (`search_service::search_custom_records`,
//! unaffected - see `ai_context_layer.rs`'s own tests for that). Reuses
//! `ai_context_layer.rs`'s own Custom Object/Field builders.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_service, custom_object_service, custom_record_service, user_service, vector_search_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Vector Search Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [91u8; 32]
}

fn non_admin_user(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: "rep".into(), display_name: "Sales Rep".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn configure_openai_compatible_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "openai_compatible".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "gpt-4".into(), api_key: Some("sk-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn asset_object_input() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Asset".into(), plural_label: "Assets".into(), icon: "🔧".into(), prefix: "AST".into(), digits: 4 }
}

fn create_asset(conn: &rusqlite::Connection, ws: &str, admin: &str, object_key: &str, primary_name: &str) -> lanesra_core::models::custom_record::CustomRecord {
    custom_record_service::create(
        conn, ws,
        &CustomRecordInput { object_key: object_key.into(), primary_name: primary_name.into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(admin),
    )
    .unwrap()
}

/// A fake embeddings endpoint (OpenAI-compatible `POST /embeddings`
/// shape) that returns a **content-dependent** 3-dimensional vector
/// instead of one canned body per call in order - `embed_texts` makes
/// one real request per record while reindexing, but the pending queue's
/// drain order isn't something a test should have to pin down, so the
/// stub matches on which record's own name appears in the request body
/// instead. Genuinely different "meanings" get genuinely different
/// vectors (a real provider's own embeddings would too), enough for
/// cosine similarity to have something real to discriminate on.
fn spawn_embedding_stub() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut content_length: usize = 0;
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) => break,
                    Ok(_) => {
                        if l == "\r\n" || l.trim().is_empty() {
                            break;
                        }
                        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body_buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body_buf);
            let body = String::from_utf8_lossy(&body_buf).to_string();
            let embedding: [f32; 3] = if body.contains("ShipRecord") {
                [1.0, 0.0, 0.0]
            } else if body.contains("TruckRecord") {
                [0.0, 1.0, 0.0]
            } else if body.contains("OrangeRecord") {
                [0.0, 0.0, 1.0]
            } else if body.contains("vessel at sea") {
                // Close to, but not identical to, ShipRecord's own vector -
                // proving real cosine ranking, not an exact-match shortcut.
                [0.9, 0.1, 0.0]
            } else {
                [0.0, 0.0, 0.0]
            };
            let response_body = serde_json::json!({"data": [{"embedding": embedding}]}).to_string();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response_body.len(), response_body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

#[tokio::test]
async fn reindex_workspace_requires_admin() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let err = vector_search_service::reindex_workspace(&conn, &ws, &master_key(), Some(&rep)).await.unwrap_err();
    assert!(err.to_string().contains("Administrator"), "{err}");
}

#[tokio::test]
async fn anthropic_has_no_embeddings_api_and_says_so_plainly() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    create_asset(&conn, &ws, &admin, &asset_def.key, "Anything");

    configure_anthropic_key(&conn, &ws, &admin, 0);
    let err = vector_search_service::reindex_workspace(&conn, &ws, &master_key(), Some(&admin)).await.unwrap_err();
    assert!(err.to_string().contains("Anthropic has no embeddings API"), "{err}");
}

#[tokio::test]
async fn creating_a_record_enqueues_it_and_reindexing_stores_a_real_embedding_and_clears_the_queue() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    create_asset(&conn, &ws, &admin, &asset_def.key, "ShipRecord Atlas");

    let before = vector_search_service::status(&conn, &ws).unwrap();
    assert_eq!((before.embedded_count, before.pending_count), (0, 1), "a brand-new record is enqueued, not yet embedded");

    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);
    let reindexed = vector_search_service::reindex_workspace(&conn, &ws, &master_key(), Some(&admin)).await.unwrap();
    assert_eq!(reindexed, 1);

    let after = vector_search_service::status(&conn, &ws).unwrap();
    assert_eq!((after.embedded_count, after.pending_count), (1, 0));
}

#[tokio::test]
async fn archiving_a_record_deletes_its_embedding() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    let asset = create_asset(&conn, &ws, &admin, &asset_def.key, "TruckRecord Bravo");

    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);
    vector_search_service::reindex_workspace(&conn, &ws, &master_key(), Some(&admin)).await.unwrap();
    assert_eq!(vector_search_service::status(&conn, &ws).unwrap().embedded_count, 1);

    custom_record_service::archive(&conn, &asset.id, Some(&admin)).unwrap();
    let status = vector_search_service::status(&conn, &ws).unwrap();
    assert_eq!((status.embedded_count, status.pending_count), (0, 0), "archiving should drop the embedding immediately, not just enqueue a re-embed");
}

#[tokio::test]
async fn semantic_search_ranks_by_cosine_similarity_not_keyword_overlap() {
    let (conn, ws, admin) = setup_workspace();
    let asset_def = custom_object_service::create(&conn, &ws, &asset_object_input(), Some(&admin)).unwrap();
    let ship = create_asset(&conn, &ws, &admin, &asset_def.key, "ShipRecord Atlas");
    create_asset(&conn, &ws, &admin, &asset_def.key, "TruckRecord Bravo");
    create_asset(&conn, &ws, &admin, &asset_def.key, "OrangeRecord Charlie");

    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);
    let reindexed = vector_search_service::reindex_workspace(&conn, &ws, &master_key(), Some(&admin)).await.unwrap();
    assert_eq!(reindexed, 3);

    // The query shares not one literal word with any record's name - a
    // keyword search would find nothing - but the stub maps it to a
    // vector close to ShipRecord's own, proving this is genuinely
    // similarity-ranked, not a disguised substring match.
    let hits = vector_search_service::semantic_search_records(&conn, &ws, &master_key(), "vessel at sea", None, 10).await.unwrap();
    assert_eq!(hits.len(), 3, "{hits:?}");
    assert_eq!(hits[0].record_id, ship.id, "the ship should rank first: {hits:?}");
    assert!(hits[0].similarity > hits[1].similarity && hits[1].similarity > hits[2].similarity, "{hits:?}");

    // Scoped to a different object_key - no results even though the
    // embedded record exists workspace-wide.
    assert!(vector_search_service::semantic_search_records(&conn, &ws, &master_key(), "vessel at sea", Some("nonexistent_object"), 10).await.unwrap().is_empty());
}

#[tokio::test]
async fn semantic_search_with_no_embedded_records_yet_returns_no_hits_not_an_error() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);
    let hits = vector_search_service::semantic_search_records(&conn, &ws, &master_key(), "anything", None, 10).await.unwrap();
    assert!(hits.is_empty());
}
