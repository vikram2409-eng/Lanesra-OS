//! Storage for `models::activity::Activity` - mirrors `audit_repo.rs`'s
//! own two-function shape exactly (`create`/`list_for_entity`), since
//! this is architecturally the same kind of thing: a generic,
//! entity-agnostic log queryable by `(entity_type, entity_id)`.

use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::activity::{Activity, ActivityInput};

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &ActivityInput,
    source: &str,
    created_by: Option<&str>,
) -> rusqlite::Result<Activity> {
    let id = new_uuid();
    let created_at = now_iso();
    conn.execute(
        "INSERT INTO activities (id, workspace_id, entity_type, entity_id, channel, direction, subject, body, participants, occurred_at, source, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        (
            &id,
            workspace_id,
            &input.entity_type,
            &input.entity_id,
            &input.channel,
            &input.direction,
            &input.subject,
            &input.body,
            &input.participants,
            &input.occurred_at,
            source,
            created_by,
            &created_at,
        ),
    )?;
    Ok(Activity {
        id,
        workspace_id: workspace_id.to_string(),
        entity_type: input.entity_type.clone(),
        entity_id: input.entity_id.clone(),
        channel: input.channel.clone(),
        direction: input.direction.clone(),
        subject: input.subject.clone(),
        body: input.body.clone(),
        participants: input.participants.clone(),
        occurred_at: input.occurred_at.clone(),
        source: source.to_string(),
        created_by: created_by.map(String::from),
        created_at,
    })
}

pub fn list_for_entity(conn: &Connection, entity_type: &str, entity_id: &str) -> rusqlite::Result<Vec<Activity>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM activities WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY occurred_at DESC",
    )?;
    let rows = stmt.query_map((entity_type, entity_id), |row| {
        Ok(Activity {
            id: row.get("id")?,
            workspace_id: row.get("workspace_id")?,
            entity_type: row.get("entity_type")?,
            entity_id: row.get("entity_id")?,
            channel: row.get("channel")?,
            direction: row.get("direction")?,
            subject: row.get("subject")?,
            body: row.get("body")?,
            participants: row.get("participants")?,
            occurred_at: row.get("occurred_at")?,
            source: row.get("source")?,
            created_by: row.get("created_by")?,
            created_at: row.get("created_at")?,
        })
    })?;
    rows.collect()
}
