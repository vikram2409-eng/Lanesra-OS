use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub id: String,
    pub workspace_id: String,
    pub occurred_at: String,
    pub user_id: Option<String>,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub summary: String,
    pub details_json: Option<String>,
}
