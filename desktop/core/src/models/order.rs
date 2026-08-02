use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Order {
    pub id: String,
    pub workspace_id: String,
    pub order_number: String,
    pub company_id: String,
    pub contact_id: Option<String>,
    pub source_quote_id: Option<String>,
    pub status: String,
    pub currency_code: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub order_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderLine {
    pub id: String,
    pub order_id: String,
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
pub struct OrderLineInput {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_cents: i64,
    pub discount_bp: i64,
    pub tax_rate_bp: i64,
}

/// Direct order creation (FR-ORD-03). Converting an existing quote uses the
/// dedicated conversion command instead, which copies its own lines.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderInput {
    pub company_id: String,
    pub contact_id: Option<String>,
    pub currency_code: String,
    pub order_date: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<OrderLineInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderWithLines {
    pub order: Order,
    pub lines: Vec<OrderLine>,
}

pub const ORDER_STATUSES: &[&str] = &[
    "Draft",
    "Confirmed",
    "Processing",
    "Partially Fulfilled",
    "Fulfilled",
    "Cancelled",
];
