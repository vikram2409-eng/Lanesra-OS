//! AI & Agentic Layer, Phase 3: the Unified Activity Timeline's generic
//! log - one row per email/call/message tied to a Company, Contact or
//! Opportunity (see `services::activity_service`'s own doc comment for
//! why exactly these three, and what's deliberately not built yet).
//! Distinct from `models::audit::AuditEvent`: that records what this
//! workspace's own users changed *on* a record; this records what
//! happened *around* a record from outside the CRM.

use serde::{Deserialize, Serialize};

pub const ACTIVITY_ENTITY_TYPES: &[&str] = &["Company", "Contact", "Opportunity"];
pub const ACTIVITY_CHANNELS: &[&str] = &["email", "call", "message"];
pub const ACTIVITY_DIRECTIONS: &[&str] = &["inbound", "outbound"];

#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub channel: String,
    pub direction: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub participants: Option<String>,
    pub occurred_at: String,
    /// `"manual"` for everything logged this phase - a future
    /// connection-based channel (an IMAP email Connection, a Slack
    /// connector) sets this to that connection's id instead, so this
    /// column already exists when that lands rather than needing a new
    /// migration then too.
    pub source: String,
    pub created_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityInput {
    pub entity_type: String,
    pub entity_id: String,
    pub channel: String,
    pub direction: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub participants: Option<String>,
    pub occurred_at: String,
}
