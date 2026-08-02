use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Opportunity {
    pub id: String,
    pub workspace_id: String,
    pub opportunity_number: String,
    pub company_id: String,
    pub primary_contact_id: Option<String>,
    pub name: String,
    pub stage: String,
    pub status: String,
    pub value_cents: i64,
    pub currency_code: String,
    pub probability_bp: i64,
    pub expected_close_date: Option<String>,
    pub owner_user_id: Option<String>,
    pub lost_reason: Option<String>,
    pub next_step: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpportunityInput {
    pub company_id: String,
    pub primary_contact_id: Option<String>,
    pub name: String,
    pub stage: String,
    pub status: String,
    pub value_cents: i64,
    pub currency_code: String,
    pub probability_bp: i64,
    pub expected_close_date: Option<String>,
    pub owner_user_id: Option<String>,
    pub lost_reason: Option<String>,
    pub next_step: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpportunityProduct {
    pub id: String,
    pub opportunity_id: String,
    pub product_id: String,
    pub quantity_milli: i64,
    pub unit_price_cents: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpportunityProductInput {
    pub product_id: String,
    pub quantity_milli: i64,
    pub unit_price_cents: i64,
}

pub const OPPORTUNITY_STAGES: &[&str] = &[
    "New",
    "Qualified",
    "Discovery",
    "Proposal",
    "Negotiation",
    "Won",
    "Lost",
];
pub const OPPORTUNITY_STATUSES: &[&str] = &["Open", "Won", "Lost", "Archived"];
