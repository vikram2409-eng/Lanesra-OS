//! Integration Hub (spec table 12): the one call every entity's create/
//! update/archive path makes, unconditionally, the same pattern
//! `solution_component_service::tag_local` already established for its
//! own cross-cutting concern. `emit` is plain sync and cheap in the
//! common case (no webhook subscribed to this event at all - a single
//! indexed SQL check, nothing written) - see migration 0032's own comment
//! on `integration_pending_events` for why actual delivery is deferred to
//! a separate async drain rather than attempted inline here.

use rusqlite::Connection;
use serde_json::json;

use crate::domain::ids::new_uuid;
use crate::repositories::{integration_pending_event_repo, integration_webhook_repo};

/// Called after a record's create/update/archive has already committed.
/// `record_id`/`display_name`/`status` are whatever that entity's own
/// service already had in hand - no extra lookup needed.
pub fn emit(conn: &Connection, workspace_id: &str, event_type: &str, object_key: &str, record_id: &str, display_name: &str, status: &str) {
    // Never lets a webhook-fan-out problem break the record write it's
    // piggybacking on - same "log and move on" stance
    // `solution_component_service`'s own callers take for a tagging
    // failure that isn't the point of the call.
    let _ = try_emit(conn, workspace_id, event_type, object_key, record_id, display_name, status);
}

fn try_emit(conn: &Connection, workspace_id: &str, event_type: &str, object_key: &str, record_id: &str, display_name: &str, status: &str) -> rusqlite::Result<()> {
    let subscribed = integration_webhook_repo::list_active_for_event(conn, workspace_id, event_type)?;
    if subscribed.is_empty() {
        return Ok(());
    }
    let payload = json!({"display_name": display_name, "status": status});
    integration_pending_event_repo::insert(
        conn,
        &new_uuid(),
        workspace_id,
        event_type,
        object_key,
        record_id,
        &payload.to_string(),
        None,
    )
}

pub fn record_created(conn: &Connection, workspace_id: &str, object_key: &str, record_id: &str, display_name: &str, status: &str) {
    emit(conn, workspace_id, "record.created", object_key, record_id, display_name, status);
}

pub fn record_updated(conn: &Connection, workspace_id: &str, object_key: &str, record_id: &str, display_name: &str, status: &str) {
    emit(conn, workspace_id, "record.updated", object_key, record_id, display_name, status);
}

pub fn record_archived(conn: &Connection, workspace_id: &str, object_key: &str, record_id: &str, display_name: &str) {
    emit(conn, workspace_id, "record.archived", object_key, record_id, display_name, "archived");
}
