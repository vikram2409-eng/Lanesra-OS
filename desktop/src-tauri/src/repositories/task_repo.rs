use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::task::{Task, TaskInput};

const SELECT_TASK: &str = "SELECT t.*, tl.related_type AS link_related_type, tl.related_id AS link_related_id
    FROM tasks t LEFT JOIN task_links tl ON tl.task_id = t.id";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        task_number: row.get("task_number")?,
        title: row.get("title")?,
        description: row.get("description")?,
        owner_user_id: row.get("owner_user_id")?,
        priority: row.get("priority")?,
        status: row.get("status")?,
        due_date: row.get("due_date")?,
        reminder_at: row.get("reminder_at")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
        related_type: row.get("link_related_type")?,
        related_id: row.get("link_related_id")?,
    })
}

fn set_link(conn: &Connection, task_id: &str, input: &TaskInput) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM task_links WHERE task_id = ?1", [task_id])?;
    if let (Some(related_type), Some(related_id)) = (&input.related_type, &input.related_id) {
        conn.execute(
            "INSERT INTO task_links (id, task_id, related_type, related_id) VALUES (?1, ?2, ?3, ?4)",
            (new_uuid(), task_id, related_type, related_id),
        )?;
    }
    Ok(())
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    task_number: &str,
    input: &TaskInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Task> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO tasks (id, workspace_id, task_number, title, description, owner_user_id, priority, status, due_date, reminder_at, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?11, ?12)",
        rusqlite::params![
            id,
            workspace_id,
            task_number,
            &input.title,
            &input.description,
            &input.owner_user_id,
            &input.priority,
            &input.status,
            &input.due_date,
            &input.reminder_at,
            &now,
            actor_user_id,
        ],
    )?;
    set_link(conn, id, input)?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Task>> {
    conn.query_row(&format!("{SELECT_TASK} WHERE t.id = ?1"), [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Task>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT_TASK} WHERE t.workspace_id = ?1 AND t.archived_at IS NULL ORDER BY t.due_date IS NULL, t.due_date"
    ))?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn list_by_related(conn: &Connection, related_type: &str, related_id: &str) -> rusqlite::Result<Vec<Task>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT_TASK} WHERE tl.related_type = ?1 AND tl.related_id = ?2 AND t.archived_at IS NULL ORDER BY t.due_date IS NULL, t.due_date"
    ))?;
    let rows = stmt.query_map((related_type, related_id), map_row)?;
    rows.collect()
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &TaskInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Task> {
    let now = now_iso();
    conn.execute(
        "UPDATE tasks SET title = ?1, description = ?2, owner_user_id = ?3, priority = ?4, status = ?5,
            due_date = ?6, reminder_at = ?7, updated_at = ?8, updated_by = ?9
         WHERE id = ?10",
        rusqlite::params![
            &input.title,
            &input.description,
            &input.owner_user_id,
            &input.priority,
            &input.status,
            &input.due_date,
            &input.reminder_at,
            &now,
            actor_user_id,
            id,
        ],
    )?;
    set_link(conn, id, input)?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE tasks SET archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}

pub fn count_open_and_overdue(conn: &Connection, workspace_id: &str) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT
            SUM(CASE WHEN status NOT IN ('Completed', 'Cancelled') THEN 1 ELSE 0 END),
            SUM(CASE WHEN status NOT IN ('Completed', 'Cancelled') AND due_date IS NOT NULL AND date(due_date) < date('now') THEN 1 ELSE 0 END)
         FROM tasks WHERE workspace_id = ?1 AND archived_at IS NULL",
        [workspace_id],
        |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
            ))
        },
    )
}
