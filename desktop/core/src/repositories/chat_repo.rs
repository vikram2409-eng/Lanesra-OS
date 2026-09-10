use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::chat::{ChatConversation, ChatMessage};

fn map_conversation(row: &rusqlite::Row) -> rusqlite::Result<ChatConversation> {
    Ok(ChatConversation {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        user_id: row.get("user_id")?,
        mode: row.get("mode")?,
        agent_id: row.get("agent_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn map_message(row: &rusqlite::Row) -> rusqlite::Result<ChatMessage> {
    let tool_calls_json: Option<String> = row.get("tool_calls_json")?;
    Ok(ChatMessage {
        id: row.get("id")?,
        conversation_id: row.get("conversation_id")?,
        role: row.get("role")?,
        content: row.get("content")?,
        tool_calls: tool_calls_json.and_then(|s| serde_json::from_str(&s).ok()),
        tool_call_id: row.get("tool_call_id")?,
        created_at: row.get("created_at")?,
    })
}

/// One conversation per (user, mode, agent_id) - reused across visits
/// rather than recreated each time (see `chat_service`'s own doc
/// comment). `agent_id` is `""` for the fixed `"records"`/`"admin"`
/// modes, and a real `ai_agents.id` for `"agent"` mode.
pub fn get_or_create_conversation(conn: &Connection, workspace_id: &str, user_id: &str, mode: &str, agent_id: &str) -> rusqlite::Result<ChatConversation> {
    let existing = conn
        .query_row(
            "SELECT * FROM chat_conversations WHERE user_id = ?1 AND mode = ?2 AND agent_id = ?3",
            (user_id, mode, agent_id),
            map_conversation,
        )
        .optional()?;
    if let Some(conversation) = existing {
        return Ok(conversation);
    }
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO chat_conversations (id, workspace_id, user_id, mode, agent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        (&id, workspace_id, user_id, mode, agent_id, &now),
    )?;
    Ok(ChatConversation {
        id,
        workspace_id: workspace_id.to_string(),
        user_id: user_id.to_string(),
        mode: mode.to_string(),
        agent_id: agent_id.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_messages(conn: &Connection, conversation_id: &str) -> rusqlite::Result<Vec<ChatMessage>> {
    let mut stmt = conn.prepare("SELECT * FROM chat_messages WHERE conversation_id = ?1 ORDER BY created_at ASC")?;
    let rows = stmt.query_map([conversation_id], map_message)?;
    rows.collect()
}

pub fn append_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: Option<&str>,
    tool_calls: Option<&serde_json::Value>,
    tool_call_id: Option<&str>,
) -> rusqlite::Result<ChatMessage> {
    let id = new_uuid();
    let now = now_iso();
    let tool_calls_json = tool_calls.map(|v| v.to_string());
    conn.execute(
        "INSERT INTO chat_messages (id, conversation_id, role, content, tool_calls_json, tool_call_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&id, conversation_id, role, content, &tool_calls_json, tool_call_id, &now),
    )?;
    conn.execute("UPDATE chat_conversations SET updated_at = ?1 WHERE id = ?2", (&now, conversation_id))?;
    Ok(ChatMessage {
        id,
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.map(String::from),
        tool_calls: tool_calls.cloned(),
        tool_call_id: tool_call_id.map(String::from),
        created_at: now,
    })
}
