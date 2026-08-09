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
    pub business_address: Option<String>,
    pub logo_base64: Option<String>,
    pub logo_mime: Option<String>,
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

/// Editing the workspace profile after first-run (FR-BRD-01) - text/number
/// fields only. The logo is handled by a separate command
/// (`set_workspace_logo` / `clear_workspace_logo`) so a routine profile
/// edit never has to re-transmit the image payload.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceUpdate {
    pub business_name: String,
    pub legal_name: Option<String>,
    pub business_address: Option<String>,
    pub currency_code: String,
    pub locale: String,
    pub timezone: String,
    pub default_tax_rate_bp: i64,
}

/// FR-BRD-02: only PNG/JPEG are accepted, and the payload is capped - both
/// enforced in `workspace_service::set_logo`, not just the client.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceLogo {
    pub logo_base64: String,
    pub logo_mime: String,
}
