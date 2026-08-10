use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CustomRecord {
    pub id: String,
    pub workspace_id: String,
    pub object_key: String,
    pub display_number: String,
    pub primary_name: String,
    pub status: String,
    pub owner_user_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomRecordInput {
    pub object_key: String,
    pub primary_name: String,
    pub status: String,
    pub owner_user_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomRecordUpdate {
    pub primary_name: String,
    pub status: String,
    pub owner_user_id: Option<String>,
    pub notes: Option<String>,
}
