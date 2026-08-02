use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Contract {
    pub id: String,
    pub workspace_id: String,
    pub contract_number: String,
    pub company_id: String,
    pub contact_id: Option<String>,
    pub source_quote_id: Option<String>,
    pub title: String,
    pub r#type: Option<String>,
    pub value_cents: i64,
    pub currency_code: String,
    pub owner_user_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub renewal_date: Option<String>,
    pub notice_period_days: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

/// Deliberately has no opportunity_id field - FR-CTR-03 / FR-OPP-06 / BR-009
/// prohibit a contract from ever referencing an opportunity.
#[derive(Debug, Clone, Deserialize)]
pub struct ContractInput {
    pub company_id: String,
    pub contact_id: Option<String>,
    pub source_quote_id: Option<String>,
    pub title: String,
    pub r#type: Option<String>,
    pub value_cents: i64,
    pub currency_code: String,
    pub owner_user_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub renewal_date: Option<String>,
    pub notice_period_days: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
}

pub const CONTRACT_STATUSES: &[&str] = &[
    "Draft",
    "Under Review",
    "Active",
    "Expiring",
    "Renewed",
    "Expired",
    "Terminated",
];
