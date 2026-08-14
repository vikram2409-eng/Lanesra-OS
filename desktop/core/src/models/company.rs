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
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub annual_revenue_cents: Option<i64>,
    pub employee_count: Option<i64>,
    pub preferred_contact_method: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

// Default + #[serde(default)] so the six fields added this round (phone
// through preferred_contact_method) don't force every existing caller -
// Rust struct-literal test fixtures and any older JSON payload alike - to
// list them explicitly; they fall back to None like every other optional
// field already does.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CompanyInput {
    pub name: String,
    pub status: String,
    pub owner_user_id: Option<String>,
    pub tax_number: Option<String>,
    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub annual_revenue_cents: Option<i64>,
    pub employee_count: Option<i64>,
    pub preferred_contact_method: Option<String>,
}

pub const COMPANY_STATUSES: &[&str] = &["Prospect", "Active Customer", "Inactive", "Archived"];
pub const PREFERRED_CONTACT_METHODS: &[&str] = &["Email", "Phone", "Text"];
