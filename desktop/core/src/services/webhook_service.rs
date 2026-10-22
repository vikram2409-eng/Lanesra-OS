//! Integration Hub (spec §10): outbound Webhooks & Event Subscriptions.
//! Every delivery is HMAC-SHA256 signed with the subscription's own secret
//! (`X-Lanesra-Signature: sha256=<hex>`), matching §10.3 exactly, and
//! carries `event_id`/`event_type`/`occurred_at`/`workspace_id`/
//! `object_key`/`record_id` so a receiver can dedupe/trace - "at-least-
//! once delivery; consumers must tolerate duplicates."
//!
//! Retry here is immediate and bounded (a handful of attempts with a
//! short backoff, all within the same call) rather than a long-horizon
//! background queue - this codebase has no job-scheduling infrastructure
//! beyond the client-poll pattern `workflow_service::run_scheduled`
//! already documents, and `integration_job_service`'s new scheduler
//! (server-only) is the natural place a longer-horizon "retry again in an
//! hour" mechanism would eventually live, not duplicated here. A
//! permanent 4xx is never retried (spec: "Do not retry permanent 4xx
//! responses except configurable 408/409/429 cases").

use std::time::{Duration, Instant};

use hmac::{Hmac, Mac};
use rusqlite::Connection;
use serde_json::json;
use sha2::Sha256;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{
    IntegrationEvent, Webhook, WebhookDelivery, WebhookDeliveryPage, WebhookInput,
    WebhookWithSecret,
};
use crate::repositories::{
    integration_pending_event_repo, integration_secret_repo, integration_webhook_repo,
};

/// Who is managing webhooks (WP-0188): a signed-in person, who must hold
/// Administrator, or an API client, whose `webhooks.manage` scope the
/// server has already checked. An API client never borrows an
/// Administrator's rights; its changes are audited under its own id.
#[derive(Debug, Clone, Copy)]
pub enum WebhookManager<'a> {
    User(Option<&'a str>),
    ApiClient { id: &'a str, name: &'a str },
}

impl WebhookManager<'_> {
    fn authorize(&self, conn: &Connection) -> AppResult<()> {
        match self {
            Self::User(actor) => super::user_service::require_admin(conn, *actor),
            Self::ApiClient { .. } => Ok(()),
        }
    }

    /// The user recorded as creator, updater and audit actor, if any.
    fn user_id(&self) -> Option<&str> {
        match self {
            Self::User(actor) => *actor,
            Self::ApiClient { .. } => None,
        }
    }

    /// Audit details naming the API client; never a secret.
    fn audit_details(&self) -> Option<String> {
        match self {
            Self::User(_) => None,
            Self::ApiClient { id, name } => {
                Some(json!({"api_client_id": id, "api_client_name": name}).to_string())
            }
        }
    }

    fn audit(
        &self,
        conn: &Connection,
        workspace_id: &str,
        event_type: &str,
        webhook_id: &str,
        summary: &str,
    ) -> AppResult<()> {
        let summary = match self {
            Self::User(_) => summary.to_string(),
            Self::ApiClient { name, .. } => format!("{summary} (API client '{name}')"),
        };
        crate::repositories::audit_repo::record(
            conn,
            workspace_id,
            self.user_id(),
            event_type,
            Some("Webhook"),
            Some(webhook_id),
            &summary,
            self.audit_details().as_deref(),
        )?;
        Ok(())
    }
}

/// Checks what a webhook may be pointed at: a `webhook` connection in this
/// workspace whose URL the outbound guard allows (WP-0188 AC-3).
fn validate_input(conn: &Connection, workspace_id: &str, input: &WebhookInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Webhook name is required".into()));
    }
    for event_type in &input.event_types {
        if !EVENT_TYPES.contains(&event_type.as_str()) {
            return Err(AppError::Validation(format!(
                "Unknown event type '{event_type}'"
            )));
        }
    }
    let connection = super::connection_service::get(conn, workspace_id, &input.connection_id)?;
    if connection.connection_type != "webhook" {
        return Err(AppError::Validation(
            "A webhook subscription must use a 'webhook' connection (its endpoint URL)".into(),
        ));
    }
    let url = connection
        .base_url
        .as_deref()
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| AppError::Validation("This webhook connection has no URL".into()))?;
    super::ssrf_guard::check_url(url)
}

pub const EVENT_TYPES: &[&str] = &[
    "record.created",
    "record.updated",
    "record.archived",
    "field.changed",
    "workflow.completed",
    "workflow.failed",
];

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Webhook> {
    let webhook = integration_webhook_repo::get(conn, id)?
        .ok_or_else(|| AppError::NotFound("Webhook".into()))?;
    if webhook.workspace_id != workspace_id {
        return Err(AppError::NotFound("Webhook".into()));
    }
    Ok(webhook)
}

/// Creates a webhook and returns it WITHOUT its signing secret. This is
/// the one the AI `create_webhook` tool uses: a tool result enters an agent
/// transcript, so it must never carry a credential. Anything that shows the
/// secret to a person calls `create_with_secret`.
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    input: &WebhookInput,
    actor_user_id: Option<&str>,
) -> AppResult<Webhook> {
    Ok(create_with_secret(conn, workspace_id, master_key, input, actor_user_id)?.webhook)
}

/// Creates a webhook and returns it with its signing secret, the only time
/// the value is ever shown (WP-0183). The receiver needs it to verify
/// `X-Lanesra-Signature`; an admin who loses it rotates it.
pub fn create_with_secret(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    input: &WebhookInput,
    actor_user_id: Option<&str>,
) -> AppResult<WebhookWithSecret> {
    create_with_secret_by(
        conn,
        workspace_id,
        master_key,
        input,
        WebhookManager::User(actor_user_id),
    )
}

/// `create_with_secret` for any manager, including an API client.
pub fn create_with_secret_by(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    input: &WebhookInput,
    manager: WebhookManager,
) -> AppResult<WebhookWithSecret> {
    manager.authorize(conn)?;
    validate_input(conn, workspace_id, input)?;
    let actor_user_id = manager.user_id();
    let secret_value = super::secret_service::generate_random_secret(32);
    let (ciphertext, nonce) = super::secret_service::encrypt(master_key, &secret_value)?;
    let secret_id = new_uuid();
    integration_secret_repo::insert(
        conn,
        &secret_id,
        workspace_id,
        &format!("{} HMAC signing secret", input.name),
        &ciphertext,
        &nonce,
        actor_user_id,
    )?;
    let event_types_json =
        serde_json::to_string(&input.event_types).unwrap_or_else(|_| "[]".into());
    let webhook = integration_webhook_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        input.name.trim(),
        &input.connection_id,
        &event_types_json,
        input.object_scope.as_deref(),
        input.filter_json.as_deref(),
        input.payload_version.as_deref().unwrap_or("1"),
        Some(&secret_id),
        input.retry_policy_json.as_deref().unwrap_or("{}"),
        actor_user_id,
    )?;
    manager.audit(
        conn,
        workspace_id,
        "webhook.created",
        &webhook.id,
        &format!("Created webhook '{}'", webhook.name),
    )?;
    Ok(WebhookWithSecret {
        webhook,
        signing_secret: secret_value,
    })
}

/// Replaces a webhook's signing secret and returns the new one, once.
/// Deliveries after this call are signed with the new secret, so a receiver
/// must be updated at the same time (there is no grace period). Admin only;
/// the rotation is audited with who did it and never the value.
pub fn rotate_secret(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<String> {
    rotate_secret_by(
        conn,
        workspace_id,
        master_key,
        id,
        WebhookManager::User(actor_user_id),
    )
}

/// `rotate_secret` for any manager, including an API client.
pub fn rotate_secret_by(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    id: &str,
    manager: WebhookManager,
) -> AppResult<String> {
    manager.authorize(conn)?;
    let actor_user_id = manager.user_id();
    let webhook = get_owned(conn, workspace_id, id)?;
    let secret_value = super::secret_service::generate_random_secret(32);
    let (ciphertext, nonce) = super::secret_service::encrypt(master_key, &secret_value)?;
    let secret_id: Option<String> = conn.query_row(
        "SELECT secret_id FROM integration_webhooks WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    match secret_id {
        Some(existing) => integration_secret_repo::rotate(conn, &existing, &ciphertext, &nonce)?,
        None => {
            let new_id = new_uuid();
            integration_secret_repo::insert(
                conn,
                &new_id,
                workspace_id,
                &format!("{} HMAC signing secret", webhook.name),
                &ciphertext,
                &nonce,
                actor_user_id,
            )?;
            integration_webhook_repo::set_secret_id(conn, id, &new_id)?;
        }
    }
    manager.audit(
        conn,
        workspace_id,
        "webhook.secret_rotated",
        id,
        &format!("Rotated the signing secret of webhook '{}'", webhook.name),
    )?;
    Ok(secret_value)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Webhook>> {
    Ok(integration_webhook_repo::list_for_workspace(
        conn,
        workspace_id,
    )?)
}

pub fn list_deliveries(
    conn: &Connection,
    workspace_id: &str,
    webhook_id: &str,
) -> AppResult<Vec<WebhookDelivery>> {
    get_owned(conn, workspace_id, webhook_id)?;
    Ok(integration_webhook_repo::list_deliveries(
        conn, webhook_id, 50,
    )?)
}

/// One webhook, without its secret (WP-0188).
pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<Webhook> {
    get_owned(conn, workspace_id, id)
}

/// A page of delivery attempts, newest first (WP-0188 AC-5). A delivery
/// row never carries the signing secret.
pub fn list_deliveries_page(
    conn: &Connection,
    workspace_id: &str,
    webhook_id: &str,
    page: Option<i64>,
    page_size: Option<i64>,
) -> AppResult<WebhookDeliveryPage> {
    get_owned(conn, workspace_id, webhook_id)?;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).clamp(1, 500);
    let (deliveries, total) = integration_webhook_repo::list_deliveries_page(
        conn,
        webhook_id,
        page_size,
        (page - 1).saturating_mul(page_size),
    )?;
    Ok(WebhookDeliveryPage {
        deliveries,
        total,
        page,
        page_size,
    })
}

/// Replaces a webhook's settings (WP-0188). The connection is checked as on
/// create; the status and secret keep their own operations.
pub fn update_by(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    input: &WebhookInput,
    manager: WebhookManager,
) -> AppResult<Webhook> {
    manager.authorize(conn)?;
    get_owned(conn, workspace_id, id)?;
    validate_input(conn, workspace_id, input)?;
    let event_types_json =
        serde_json::to_string(&input.event_types).unwrap_or_else(|_| "[]".into());
    let webhook = integration_webhook_repo::update(
        conn,
        id,
        input.name.trim(),
        &input.connection_id,
        &event_types_json,
        input.object_scope.as_deref(),
        input.filter_json.as_deref(),
        input.payload_version.as_deref().unwrap_or("1"),
        input.retry_policy_json.as_deref().unwrap_or("{}"),
        manager.user_id(),
    )?
    .ok_or_else(|| AppError::NotFound("Webhook".into()))?;
    manager.audit(
        conn,
        workspace_id,
        "webhook.updated",
        id,
        &format!("Updated webhook '{}'", webhook.name),
    )?;
    Ok(webhook)
}

pub fn pause(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    set_status_by(
        conn,
        workspace_id,
        id,
        "paused",
        WebhookManager::User(actor_user_id),
    )
}

pub fn reactivate(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    set_status_by(
        conn,
        workspace_id,
        id,
        "active",
        WebhookManager::User(actor_user_id),
    )
}

/// Pauses (`paused`) or reactivates (`active`) a webhook.
pub fn set_status_by(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    status: &str,
    manager: WebhookManager,
) -> AppResult<()> {
    manager.authorize(conn)?;
    get_owned(conn, workspace_id, id)?;
    Ok(integration_webhook_repo::set_status(conn, id, status)?)
}

pub fn delete(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    delete_by(conn, workspace_id, id, WebhookManager::User(actor_user_id))
}

pub fn delete_by(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    manager: WebhookManager,
) -> AppResult<()> {
    manager.authorize(conn)?;
    let webhook = get_owned(conn, workspace_id, id)?;
    integration_webhook_repo::delete(conn, id)?;
    manager.audit(
        conn,
        workspace_id,
        "webhook.deleted",
        id,
        &format!("Deleted webhook '{}'", webhook.name),
    )
}

fn sign(secret: &str, body: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body.as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

/// Never retried - a config/permanent-4xx style rejection means retrying
/// with the same payload will never succeed (spec §10.3/§30's "Permanent
/// 4xx | Fail action/job item; no generic retry" row), except the three
/// the spec calls out as worth retrying anyway (408 Request Timeout, 409
/// Conflict, 429 Too Many Requests - transient by nature despite the 4xx
/// status).
fn is_retryable_status(status: u16) -> bool {
    !(400..500).contains(&status) || matches!(status, 408 | 409 | 429)
}

async fn attempt_delivery(
    endpoint_url: &str,
    secret: &str,
    payload_json: &str,
) -> (bool, Option<u16>, u64, String) {
    let started = Instant::now();
    if let Err(e) = super::ssrf_guard::check_url(endpoint_url) {
        return (false, None, 0, e.to_string());
    }
    let client = super::ssrf_guard::client_builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("HTTP client always builds");
    let signature = sign(secret, payload_json);
    let result = client
        .post(endpoint_url)
        .header("Content-Type", "application/json")
        .header("X-Lanesra-Signature", signature)
        .body(payload_json.to_string())
        .send()
        .await;
    let elapsed = started.elapsed().as_millis() as u64;
    match result {
        Ok(response) => {
            let status = response.status();
            let ok = status.is_success();
            let snippet: String = response
                .text()
                .await
                .unwrap_or_default()
                .chars()
                .take(200)
                .collect();
            (ok, Some(status.as_u16()), elapsed, snippet)
        }
        Err(e) => (
            false,
            None,
            elapsed,
            super::ssrf_guard::blocked_in(&e)
                .map_or_else(|| e.to_string(), |b| b.to_string())
                .chars()
                .take(200)
                .collect(),
        ),
    }
}

/// Delivers `event` to every active webhook subscribed to its event type
/// in `workspace_id` - what an entity's create/update/archive path calls
/// after committing (see `services::event_hooks`). Bounded, immediate
/// retry on a transient failure; a permanent 4xx or exhausted retries
/// marks the delivery failed and, after repeated failure, the
/// subscription itself Degraded (spec §10.3).
pub async fn fire_event(
    conn: &Connection,
    master_key: &[u8; 32],
    event: &IntegrationEvent,
) -> AppResult<()> {
    let webhooks = integration_webhook_repo::list_active_for_event(
        conn,
        &event.workspace_id,
        &event.event_type,
    )?;
    for webhook in webhooks {
        if let Some(scope) = &webhook.object_scope {
            if scope != &event.object_key {
                continue;
            }
        }
        let Some(endpoint_url) = &webhook.endpoint_url else {
            continue;
        };
        let secret = resolve_webhook_secret(conn, master_key, &webhook.id)?.unwrap_or_default();
        let payload = json!({
            "event_id": event.event_id,
            "event_type": event.event_type,
            "occurred_at": event.occurred_at,
            "workspace_id": event.workspace_id,
            "object_key": event.object_key,
            "record_id": event.record_id,
            "correlation_id": event.correlation_id,
            "data": event.payload,
        });
        let payload_json = payload.to_string();

        const MAX_ATTEMPTS: u32 = 3;
        let mut attempt = 1;
        loop {
            let (ok, status_code, duration_ms, snippet) =
                attempt_delivery(endpoint_url, &secret, &payload_json).await;
            integration_webhook_repo::insert_delivery(
                conn,
                &new_uuid(),
                &webhook.id,
                &event.event_id,
                &event.event_type,
                &payload_json,
                attempt as i64,
                if ok { "succeeded" } else { "failed" },
                status_code.map(|s| s as i64),
                Some(duration_ms as i64),
                Some(&snippet),
            )?;
            if ok {
                break;
            }
            let retryable = status_code.map(is_retryable_status).unwrap_or(true);
            if !retryable || attempt >= MAX_ATTEMPTS {
                let consecutive =
                    integration_webhook_repo::consecutive_failures(conn, &webhook.id)?;
                if consecutive >= 5 {
                    integration_webhook_repo::set_status(conn, &webhook.id, "degraded")?;
                }
                break;
            }
            tokio::time::sleep(Duration::from_millis(200 * 2u64.pow(attempt - 1))).await;
            attempt += 1;
        }
    }
    Ok(())
}

fn resolve_webhook_secret(
    conn: &Connection,
    master_key: &[u8; 32],
    webhook_id: &str,
) -> AppResult<Option<String>> {
    let webhook = integration_webhook_repo::get(conn, webhook_id)?
        .ok_or_else(|| AppError::NotFound("Webhook".into()))?;
    if !webhook.has_secret {
        return Ok(None);
    }
    // secret_id isn't exposed on the model (has_secret is the public
    // signal) - re-read it directly for the one place that legitimately
    // needs the real value.
    let secret_id: Option<String> = conn.query_row(
        "SELECT secret_id FROM integration_webhooks WHERE id = ?1",
        [webhook_id],
        |r| r.get(0),
    )?;
    match secret_id {
        None => Ok(None),
        Some(id) => {
            let stored = integration_secret_repo::get(conn, &id)?
                .ok_or_else(|| AppError::NotFound("Webhook secret".into()))?;
            Ok(Some(super::secret_service::decrypt(
                master_key,
                &stored.ciphertext,
                &stored.nonce,
            )?))
        }
    }
}

/// What one drain pass did. `failed` lists events whose fan-out returned an
/// error (not a failed HTTP delivery, which is recorded in the webhook's
/// delivery history and counts as handled): those stay queued and are tried
/// again on the next pass.
#[derive(Debug, Default)]
pub struct DrainReport {
    pub delivered: usize,
    pub failed: Vec<(String, String)>,
}

/// Delivers every queued `integration_pending_events` row (oldest first,
/// bounded batch) and clears each one once its webhook fan-out is done -
/// what a periodic caller (the Team Workspace scheduler tick, or desktop's
/// client poll) runs. A queued event with no payload_key mismatch always
/// resolves cleanly since `event_hooks::emit` already confirmed at least one
/// subscriber existed at enqueue time; a webhook deleted/paused since then
/// is simply skipped by `fire_event`'s own subscriber lookup.
///
/// One event that errors (for example a webhook secret that can no longer
/// be decrypted) never stops the events behind it: it is reported in
/// `DrainReport::failed`, left queued, and the pass carries on. The only
/// error returned is the queue itself being unreadable.
pub async fn drain_pending_events_report(
    conn: &Connection,
    master_key: &[u8; 32],
) -> AppResult<DrainReport> {
    let batch = integration_pending_event_repo::list_batch(conn, 100)?;
    let mut report = DrainReport::default();
    for pending in batch {
        let id = pending.id.clone();
        let payload: serde_json::Value =
            serde_json::from_str(&pending.payload_json).unwrap_or(json!({}));
        let event = IntegrationEvent {
            event_id: pending.id,
            event_type: pending.event_type,
            workspace_id: pending.workspace_id,
            object_key: pending.object_key,
            record_id: pending.record_id,
            occurred_at: crate::domain::ids::now_iso(),
            correlation_id: pending.correlation_id,
            payload,
        };
        match fire_event(conn, master_key, &event).await {
            Ok(()) => {
                integration_pending_event_repo::delete(conn, &id)?;
                report.delivered += 1;
            }
            Err(e) => report.failed.push((id, e.to_string())),
        }
    }
    Ok(report)
}

/// `drain_pending_events_report` for a caller that only needs the number of
/// events delivered and cleared.
pub async fn drain_pending_events(conn: &Connection, master_key: &[u8; 32]) -> AppResult<usize> {
    Ok(drain_pending_events_report(conn, master_key)
        .await?
        .delivered)
}

/// Spec §10.2's "Test delivery button" - a synthetic event, delivered
/// through the exact same signing/HTTP path `fire_event` uses.
pub async fn test_delivery(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    webhook_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    test_delivery_by(
        conn,
        workspace_id,
        master_key,
        webhook_id,
        WebhookManager::User(actor_user_id),
    )
    .await
}

/// `test_delivery` for any manager, including an API client.
pub async fn test_delivery_by(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    webhook_id: &str,
    manager: WebhookManager<'_>,
) -> AppResult<()> {
    manager.authorize(conn)?;
    let webhook = get_owned(conn, workspace_id, webhook_id)?;
    let event = IntegrationEvent {
        event_id: new_uuid(),
        event_type: webhook
            .event_types
            .first()
            .cloned()
            .unwrap_or_else(|| "record.created".to_string()),
        workspace_id: workspace_id.to_string(),
        object_key: "Test".to_string(),
        record_id: "test-record".to_string(),
        occurred_at: crate::domain::ids::now_iso(),
        correlation_id: None,
        payload: json!({"message": "This is a test delivery from Lanesra OS Integration Hub."}),
    };
    fire_event(conn, master_key, &event).await
}
