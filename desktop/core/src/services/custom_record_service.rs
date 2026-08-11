//! CRUD for records of an admin-defined custom object (spec §20.2). Number
//! allocation mirrors `domain::numbering::allocate_number` (same
//! `number_sequences` table, same atomic INSERT...ON CONFLICT pattern) but
//! reads prefix/digits straight from the object's own definition row
//! rather than a static Rust config - a custom object has no "built-in
//! default" distinct from its current definition, so there's no separate
//! override table to consult here (unlike numbering_service, which layers
//! an override on top of a hardcoded default for the nine built-in types).
//! No year component for MVP simplicity; an admin who wants one can
//! include it literally in the prefix and change it manually each year,
//! the same "prefix is free-form text" convention numbering_service uses.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::custom_object::CUSTOM_RECORD_STATUSES;
use crate::models::custom_record::{CustomRecord, CustomRecordInput, CustomRecordUpdate};
use crate::repositories::{custom_object_repo, custom_record_repo};
use crate::services::{relationship_service, workflow_service};

fn resolve_active_object(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<crate::models::custom_object::CustomObjectDefinition> {
    let def = custom_object_repo::get_by_key(conn, workspace_id, object_key)?
        .ok_or_else(|| AppError::Validation(format!("Unknown object type '{object_key}'")))?;
    if !def.is_active {
        return Err(AppError::Validation(format!("'{}' is not currently active", def.plural_label)));
    }
    Ok(def)
}

fn allocate_number(conn: &Connection, workspace_id: &str, object_key: &str, prefix: &str, digits: i64) -> AppResult<String> {
    let width = digits as usize;
    let next_value: i64 = conn.query_row(
        "INSERT INTO number_sequences (id, workspace_id, entity_type, prefix, period_key, next_value)
         VALUES (?1, ?2, ?3, ?4, '', 1)
         ON CONFLICT (workspace_id, entity_type, period_key)
         DO UPDATE SET next_value = number_sequences.next_value + 1
         RETURNING next_value",
        (new_uuid(), workspace_id, object_key, prefix),
        |row| row.get(0),
    )?;
    Ok(format!("{prefix}-{next_value:0width$}"))
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &CustomRecordInput,
    actor_user_id: Option<&str>,
) -> AppResult<CustomRecord> {
    let def = resolve_active_object(conn, workspace_id, &input.object_key)?;
    if input.primary_name.trim().is_empty() {
        return Err(AppError::Validation(format!("{} name is required", def.singular_label)));
    }
    if !CUSTOM_RECORD_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!("Invalid status '{}'", input.status)));
    }
    let display_number = allocate_number(conn, workspace_id, &def.key, &def.prefix, def.digits)?;
    let id = new_uuid();
    let record = custom_record_repo::create(conn, &id, workspace_id, &display_number, input, actor_user_id)?;
    workflow_service::fire_event(conn, workspace_id, &def.key, &record.id, None, &record.status, record.owner_user_id.as_deref(), actor_user_id)?;
    Ok(record)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<CustomRecord> {
    Ok(custom_record_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Record".into()))?)
}

pub fn list(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<Vec<CustomRecord>> {
    // Confirms the object exists so a typo'd/retired key returns a clean
    // error instead of a silently empty list.
    resolve_active_object(conn, workspace_id, object_key).or_else(|_| {
        custom_object_repo::get_by_key(conn, workspace_id, object_key)?
            .ok_or_else(|| AppError::NotFound("Object type".into()))
    })?;
    Ok(custom_record_repo::list(conn, workspace_id, object_key)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &CustomRecordUpdate,
    actor_user_id: Option<&str>,
) -> AppResult<CustomRecord> {
    let before = get(conn, id)?;
    if input.primary_name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    if !CUSTOM_RECORD_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::Validation(format!("Invalid status '{}'", input.status)));
    }
    let record = custom_record_repo::update(conn, id, input, actor_user_id)?;
    workflow_service::fire_event(conn, &record.workspace_id, &before.object_key, id, Some(&before.status), &record.status, record.owner_user_id.as_deref(), actor_user_id)?;
    Ok(record)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<CustomRecord> {
    let record = get(conn, id)?;
    // Phase B (ADM-CR-06): a `restrict` custom relationship still linking
    // to this record blocks archiving it; an `archive` one has its link
    // rows cleared instead of following the record into archive.
    relationship_service::enforce_delete_behavior(conn, &record.object_key, id)?;
    Ok(custom_record_repo::archive(conn, id, actor_user_id)?)
}
