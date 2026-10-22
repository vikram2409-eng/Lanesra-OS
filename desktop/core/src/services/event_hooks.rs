//! Integration Hub (spec table 12): the one call every entity's create/
//! update/archive path makes, unconditionally, the same pattern
//! `solution_component_service::tag_local` already established for its
//! own cross-cutting concern. `emit` is plain sync and cheap in the
//! common case (no webhook subscribed to this event at all - a single
//! indexed SQL check, nothing written) - see migration 0032's own comment
//! on `integration_pending_events` for why actual delivery is deferred to
//! a separate async drain rather than attempted inline here.

use std::collections::HashMap;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::ids::new_uuid;
use crate::repositories::{integration_pending_event_repo, integration_webhook_repo};

/// Called after a record's create/update/archive has already committed.
/// `record_id`/`display_name`/`status` are whatever that entity's own
/// service already had in hand - no extra lookup needed.
pub fn emit(
    conn: &Connection,
    workspace_id: &str,
    event_type: &str,
    object_key: &str,
    record_id: &str,
    display_name: &str,
    status: &str,
) {
    // Never lets a webhook-fan-out problem break the record write it's
    // piggybacking on - same "log and move on" stance
    // `solution_component_service`'s own callers take for a tagging
    // failure that isn't the point of the call.
    let _ = try_emit(
        conn,
        workspace_id,
        event_type,
        object_key,
        record_id,
        display_name,
        status,
    );
}

fn try_emit(
    conn: &Connection,
    workspace_id: &str,
    event_type: &str,
    object_key: &str,
    record_id: &str,
    display_name: &str,
    status: &str,
) -> rusqlite::Result<()> {
    if !subscribed(conn, workspace_id, event_type)? {
        return Ok(());
    }
    queue(
        conn,
        workspace_id,
        event_type,
        object_key,
        record_id,
        &json!({"display_name": display_name, "status": status}),
    )
}

/// The cheap check every emitter makes first: is any active webhook
/// subscribed to this event type? If not, nothing is written.
fn subscribed(conn: &Connection, workspace_id: &str, event_type: &str) -> rusqlite::Result<bool> {
    Ok(
        !integration_webhook_repo::list_active_for_event(conn, workspace_id, event_type)?
            .is_empty(),
    )
}

fn queue(
    conn: &Connection,
    workspace_id: &str,
    event_type: &str,
    object_key: &str,
    record_id: &str,
    payload: &Value,
) -> rusqlite::Result<()> {
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

/// The longest value a `field.changed` event carries, in characters
/// (WP-0185). Longer values are cut and marked with `truncated`.
pub const FIELD_VALUE_LIMIT: usize = 2048;

fn cut(value: &str) -> (String, bool) {
    if value.chars().count() <= FIELD_VALUE_LIMIT {
        (value.to_string(), false)
    } else {
        (value.chars().take(FIELD_VALUE_LIMIT).collect(), true)
    }
}

/// `field.changed` (WP-0185): one event per field whose value differs
/// between the two snapshots taken around a save, with `data` =
/// `{field, source, previous, current}`. `source` is `builtin` or `custom`.
/// Never fails the save it follows.
pub fn field_changes(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    record_id: &str,
    source: &str,
    before: &HashMap<String, String>,
    after: &HashMap<String, String>,
) {
    let _ = try_field_changes(
        conn,
        workspace_id,
        object_key,
        record_id,
        source,
        before,
        after,
    );
}

fn try_field_changes(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    record_id: &str,
    source: &str,
    before: &HashMap<String, String>,
    after: &HashMap<String, String>,
) -> rusqlite::Result<()> {
    let mut changed: Vec<&String> = after
        .iter()
        .filter(|(k, v)| before.get(k.as_str()).map(String::as_str).unwrap_or("") != v.as_str())
        .map(|(k, _)| k)
        .collect();
    if changed.is_empty() || !subscribed(conn, workspace_id, "field.changed")? {
        return Ok(());
    }
    changed.sort();
    for field in changed {
        let (previous, previous_cut) = cut(before.get(field).map(String::as_str).unwrap_or(""));
        let (current, current_cut) = cut(after.get(field).map(String::as_str).unwrap_or(""));
        queue(
            conn,
            workspace_id,
            "field.changed",
            object_key,
            record_id,
            &json!({
                "field": field,
                "source": source,
                "previous": previous,
                "current": current,
                "truncated": previous_cut || current_cut,
            }),
        )?;
    }
    Ok(())
}

/// The longest error message a `workflow.failed` event carries.
const WORKFLOW_ERROR_LIMIT: usize = 500;

/// `workflow.completed` or `workflow.failed` (WP-0185), when a workflow run
/// ends. `data` = `{workflow_id, workflow_name, run_status, trigger_record}`,
/// plus a short `error` on failure. Never fails the run it follows.
#[allow(clippy::too_many_arguments)]
pub fn workflow_finished(
    conn: &Connection,
    workspace_id: &str,
    workflow_id: &str,
    workflow_name: &str,
    object_key: &str,
    record_id: &str,
    succeeded: bool,
    error: Option<&str>,
) {
    let event_type = if succeeded {
        "workflow.completed"
    } else {
        "workflow.failed"
    };
    let _ = (|| -> rusqlite::Result<()> {
        if !subscribed(conn, workspace_id, event_type)? {
            return Ok(());
        }
        let mut data = json!({
            "workflow_id": workflow_id,
            "workflow_name": workflow_name,
            "run_status": if succeeded { "success" } else { "error" },
            "trigger_record": {"object_key": object_key, "id": record_id},
        });
        if let Some(error) = error.filter(|_| !succeeded) {
            let error: String = error.chars().take(WORKFLOW_ERROR_LIMIT).collect();
            data["error"] = json!(error);
        }
        queue(conn, workspace_id, event_type, object_key, record_id, &data)
    })();
}

pub fn record_created(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    record_id: &str,
    display_name: &str,
    status: &str,
) {
    emit(
        conn,
        workspace_id,
        "record.created",
        object_key,
        record_id,
        display_name,
        status,
    );
}

pub fn record_updated(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    record_id: &str,
    display_name: &str,
    status: &str,
) {
    emit(
        conn,
        workspace_id,
        "record.updated",
        object_key,
        record_id,
        display_name,
        status,
    );
}

pub fn record_archived(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    record_id: &str,
    display_name: &str,
) {
    emit(
        conn,
        workspace_id,
        "record.archived",
        object_key,
        record_id,
        display_name,
        "archived",
    );
}
