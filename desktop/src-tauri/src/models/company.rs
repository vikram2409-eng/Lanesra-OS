use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Company {
    pub id: String,
    pub workspace_id: String,
    pub customer_number: String,
    pub name: String,
    pub status: String,
    pub owner_user_id: Option<String>,
    pub tax_number: Option<String>,
    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompanyInput {
    pub name: String,
    pub status: String,
    pub owner_user_id: Option<String>,
    pub tax_number: Option<String>,
    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
}

pub const COMPANY_STATUSES: &[&str] = &["Prospect", "Active Customer", "Inactive", "Archived"];
