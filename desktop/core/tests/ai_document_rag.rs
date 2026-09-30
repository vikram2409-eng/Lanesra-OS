//! AI Agent Platform v2, Phase 4 (GitHub issue #169): Document RAG -
//! `ai_knowledge_service`'s ingestion (chunk + embed, synchronously, on
//! Knowledge Source create/update) and retrieval (`search_knowledge`,
//! cosine similarity over the same `ai_service::embed_texts` provider call
//! `ai_vector_search.rs` already exercises for record embeddings).
//!
//! Reuses that file's own content-dependent embeddings-stub convention:
//! the stub returns a distinct vector per keyword found in the request
//! body, so cosine ranking has something real to discriminate on rather
//! than an exact-match shortcut. Service-level only, per this phase's own
//! scope note - no full chat-integration test for the `search_knowledge`
//! tool here (that round-trip is proven for `remember`/`get_memory` in
//! `ai_memory_architecture.rs`; wiring the same tool-call plumbing again
//! for `search_knowledge` would be redundant, not additive coverage).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_knowledge::{KnowledgeCollectionInput, KnowledgeSourceInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_knowledge_service, ai_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Document RAG Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [83u8; 32]
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

fn configure_openai_compatible_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "openai_compatible".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "gpt-4".into(), api_key: Some("sk-test".into()) },
        Some(admin),
    )
    .unwrap();
}

/// A fake embeddings endpoint (OpenAI-compatible `POST /embeddings` shape)
/// returning a **content-dependent** vector rather than one canned body per
/// call in order - chunking can split a source into more than one request
/// item, so the stub matches per-keyword on the whole request body, the
/// same convention `ai_vector_search.rs::spawn_embedding_stub` already
/// established.
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
            // `input` is a JSON array of every chunk in the one batched
            // request; embed_texts sends all chunks for one call together.
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let inputs: Vec<String> = parsed.get("input").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
            let embed_one = |text: &str| -> [f32; 3] {
                if text.contains("refund window is thirty days") {
                    [1.0, 0.0, 0.0]
                } else if text.contains("warranty covers manufacturing defects") {
                    [0.0, 1.0, 0.0]
                } else if text.contains("standard shipping takes five") {
                    [0.9, 0.05, 0.0]
                } else if text.contains("extended warranty may be purchased") {
                    [0.0, 0.95, 0.05]
                } else {
                    [0.0, 0.0, 1.0]
                }
            };
            let data: Vec<serde_json::Value> = if inputs.is_empty() {
                vec![serde_json::json!({"embedding": embed_one(&body)})]
            } else {
                inputs.iter().map(|t| serde_json::json!({"embedding": embed_one(t)})).collect()
            };
            let response_body = serde_json::json!({"data": data}).to_string();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response_body.len(), response_body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn refund_policy_content() -> String {
    "Our refund window is thirty days from the date of purchase. Items must be returned in original condition.".into()
}

fn warranty_content() -> String {
    "The standard warranty covers manufacturing defects for one year. An extended warranty may be purchased separately at checkout.".into()
}

#[tokio::test]
async fn create_source_chunks_and_embeds_and_is_findable_with_a_citation() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);

    let source = ai_knowledge_service::create_source(
        &conn, &ws, &master_key(),
        &KnowledgeSourceInput { name: "Refund Policy".into(), content: refund_policy_content(), collection_id: None },
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(source.chunk_count, 1, "{source:?}");
    assert_eq!(source.status, "indexed");

    let hits = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "refund window is thirty days", None, 5).await.unwrap();
    assert!(!hits.is_empty(), "expected at least one hit");
    let top = &hits[0];
    assert_eq!(top.source_id, source.id);
    assert_eq!(top.source_name, "Refund Policy");
    assert!(top.similarity > 0.9, "{top:?}");
}

#[tokio::test]
async fn update_source_reindexes_replacing_old_chunks() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);

    let source = ai_knowledge_service::create_source(
        &conn, &ws, &master_key(),
        &KnowledgeSourceInput { name: "Policy Doc".into(), content: refund_policy_content(), collection_id: None },
        Some(&admin),
    )
    .await
    .unwrap();

    let before = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "refund window is thirty days", None, 5).await.unwrap();
    assert!(before.iter().any(|h| h.source_id == source.id && h.similarity > 0.9), "{before:?}");

    ai_knowledge_service::update_source(
        &conn, &ws, &master_key(), &source.id,
        &KnowledgeSourceInput { name: "Policy Doc".into(), content: warranty_content(), collection_id: None },
        Some(&admin),
    )
    .await
    .unwrap();

    // The old keyword's chunk is gone - the new content no longer contains it.
    let after_old = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "refund window is thirty days", None, 5).await.unwrap();
    assert!(!after_old.iter().any(|h| h.source_id == source.id && h.similarity > 0.9), "{after_old:?}");

    // The new keyword is now findable in the same source.
    let after_new = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "standard warranty covers manufacturing defects", None, 5).await.unwrap();
    assert!(after_new.iter().any(|h| h.source_id == source.id && h.similarity > 0.9), "{after_new:?}");
}

#[tokio::test]
async fn collection_scoped_search_only_returns_sources_in_that_collection() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);

    let support_collection = ai_knowledge_service::create_collection(&conn, &ws, &KnowledgeCollectionInput { name: "Support Docs".into(), description: None }, Some(&admin)).unwrap();
    let other_collection = ai_knowledge_service::create_collection(&conn, &ws, &KnowledgeCollectionInput { name: "Internal Docs".into(), description: None }, Some(&admin)).unwrap();

    let in_scope = ai_knowledge_service::create_source(
        &conn, &ws, &master_key(),
        &KnowledgeSourceInput { name: "Refund Policy".into(), content: refund_policy_content(), collection_id: Some(support_collection.id.clone()) },
        Some(&admin),
    )
    .await
    .unwrap();
    ai_knowledge_service::create_source(
        &conn, &ws, &master_key(),
        &KnowledgeSourceInput { name: "Warranty Policy".into(), content: warranty_content(), collection_id: Some(other_collection.id.clone()) },
        Some(&admin),
    )
    .await
    .unwrap();

    let scoped = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "refund window is thirty days", Some(&support_collection.id), 5).await.unwrap();
    assert!(scoped.iter().all(|h| h.source_id == in_scope.id), "{scoped:?}");
    assert!(!scoped.is_empty());

    // The warranty source's own content is not in the Support Docs collection at all.
    let cross_scope = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "standard warranty covers manufacturing defects", Some(&support_collection.id), 5).await.unwrap();
    assert!(cross_scope.iter().all(|h| h.similarity < 0.9), "{cross_scope:?}");
}

#[tokio::test]
async fn delete_source_removes_it_from_search_results() {
    let (conn, ws, admin) = setup_workspace();
    let port = spawn_embedding_stub();
    configure_openai_compatible_key(&conn, &ws, &admin, port);

    let source = ai_knowledge_service::create_source(
        &conn, &ws, &master_key(),
        &KnowledgeSourceInput { name: "Refund Policy".into(), content: refund_policy_content(), collection_id: None },
        Some(&admin),
    )
    .await
    .unwrap();

    let before = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "refund window is thirty days", None, 5).await.unwrap();
    assert!(before.iter().any(|h| h.source_id == source.id));

    ai_knowledge_service::delete_source(&conn, &ws, &source.id, Some(&admin)).unwrap();

    let after = ai_knowledge_service::search_knowledge(&conn, &ws, &master_key(), "refund window is thirty days", None, 5).await.unwrap();
    assert!(after.iter().all(|h| h.source_id != source.id), "{after:?}");
}

#[test]
fn knowledge_source_management_requires_an_administrator() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);

    let collection = ai_knowledge_service::create_collection(&conn, &ws, &KnowledgeCollectionInput { name: "Docs".into(), description: None }, Some(&admin)).unwrap();

    let err = ai_knowledge_service::create_collection(&conn, &ws, &KnowledgeCollectionInput { name: "Nope".into(), description: None }, Some(&rep)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("administrator"), "{err}");

    let err = ai_knowledge_service::list_sources(&conn, &ws, None, Some(&rep)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("administrator"), "{err}");

    let err = ai_knowledge_service::delete_collection(&conn, &ws, &collection.id, Some(&rep)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("administrator"), "{err}");

    // No actor at all - rejected before any role check even runs.
    assert!(ai_knowledge_service::list_collections(&conn, &ws, None).is_err());
}
