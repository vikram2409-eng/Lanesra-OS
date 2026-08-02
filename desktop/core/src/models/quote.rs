use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Quote {
    pub id: String,
    pub workspace_id: String,
    pub quote_number: String,
    pub company_id: String,
    pub contact_id: Option<String>,
    pub opportunity_id: Option<String>,
    pub status: String,
    pub currency_code: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub issue_date: Option<String>,
    pub expiry_date: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub version: i64,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteLine {
    pub id: String,
    pub quote_id: String,
    pub product_id: Option<String>,
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_cents: i64,
    pub discount_bp: i64,
    pub tax_rate_bp: i64,
    pub line_total_cents: i64,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteLineInput {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_cents: i64,
    pub discount_bp: i64,
    pub tax_rate_bp: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteInput {
    pub company_id: String,
    pub contact_id: Option<String>,
    pub opportunity_id: Option<String>,
    pub currency_code: String,
    pub issue_date: Option<String>,
    pub expiry_date: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub lines: Vec<QuoteLineInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteWithLines {
    pub quote: Quote,
    pub lines: Vec<QuoteLine>,
}

pub const QUOTE_STATUSES: &[&str] = &[
    "Draft", "Sent", "Viewed", "Accepted", "Rejected", "Expired", "Cancelled",
];
