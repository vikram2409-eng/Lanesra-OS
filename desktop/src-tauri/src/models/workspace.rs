use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub business_name: String,
    pub legal_name: Option<String>,
    pub currency_code: String,
    pub locale: String,
    pub timezone: String,
    pub default_tax_rate_bp: i64,
    pub operating_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

/// First-run setup input: business profile plus the local administrator
/// account to create alongside the workspace (5.1 First-run experience).
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceSetup {
    pub business_name: String,
    pub legal_name: Option<String>,
    pub currency_code: String,
    pub locale: String,
    pub timezone: String,
    pub default_tax_rate_bp: i64,
    pub admin_username: String,
    pub admin_display_name: String,
    pub admin_password: String,
    pub load_sample_data: bool,
}
