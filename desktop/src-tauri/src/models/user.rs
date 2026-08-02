use serde::{Deserialize, Serialize};

/// Safe user representation returned to the frontend - never includes the
/// password hash.
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: String,
    pub workspace_id: String,
    pub username: String,
    pub display_name: String,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Internal row shape including the password hash, used only inside the
/// repository/auth layer.
pub struct UserRecord {
    pub id: String,
    pub workspace_id: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub roles: Vec<String>,
}
