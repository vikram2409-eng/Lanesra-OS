//! Raw storage for `ai_token_usage` (migration 0041) - real per-(workspace,
//! agent, user, day) token accounting. `agent_id`/`user_id` use `""`
//! rather than `NULL` for "not an agent run"/"no actor", the same
//! sentinel-empty-string convention `chat_conversations.agent_id` already
//! established - see `services::ai_gateway_service` for how the System/
//! Agent budget checks are computed from these totals.

use chrono::Utc;
use rusqlite::Connection;

fn today_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// Adds today's real token counts for one dispatch - called once per
/// gateway (or plain `ai_service::complete_with_tools`) call that
/// actually reached a provider, from the usage the provider's own
/// response returned, never estimated.
pub fn increment(conn: &Connection, workspace_id: &str, agent_id: &str, user_id: &str, input_tokens: i64, output_tokens: i64) -> rusqlite::Result<()> {
    let today = today_date();
    conn.execute(
        "INSERT INTO ai_token_usage (workspace_id, agent_id, user_id, usage_date, input_tokens, output_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (workspace_id, agent_id, user_id, usage_date) DO UPDATE SET
            input_tokens = input_tokens + excluded.input_tokens,
            output_tokens = output_tokens + excluded.output_tokens",
        rusqlite::params![workspace_id, agent_id, user_id, today, input_tokens, output_tokens],
    )?;
    Ok(())
}

/// The "System" tier: every token spent by this workspace today, across
/// every agent and user.
pub fn today_totals_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0) FROM ai_token_usage WHERE workspace_id = ?1 AND usage_date = ?2",
        rusqlite::params![workspace_id, today_date()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

/// The "Agent" tier: every token spent by one agent today, across every
/// user who ran it.
pub fn today_totals_for_agent(conn: &Connection, workspace_id: &str, agent_id: &str) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0) FROM ai_token_usage WHERE workspace_id = ?1 AND agent_id = ?2 AND usage_date = ?3",
        rusqlite::params![workspace_id, agent_id, today_date()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

/// The "User" tier - recorded today for visibility in the Gateway health
/// view, not yet an enforced cap (no per-user budget dial exists this
/// phase - stated plainly rather than pretending one is enforced).
pub fn today_totals_for_user(conn: &Connection, workspace_id: &str, user_id: &str) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0) FROM ai_token_usage WHERE workspace_id = ?1 AND user_id = ?2 AND usage_date = ?3",
        rusqlite::params![workspace_id, user_id, today_date()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}
