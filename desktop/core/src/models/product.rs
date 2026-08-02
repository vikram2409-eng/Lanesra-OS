use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Product {
    pub id: String,
    pub workspace_id: String,
    pub product_number: String,
    pub sku: Option<String>,
    pub r#type: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub unit_price_cents: i64,
    pub cost_cents: i64,
    pub tax_rate_bp: i64,
    pub default_quantity_milli: i64,
    pub is_active: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProductInput {
    pub sku: Option<String>,
    pub r#type: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub unit_price_cents: i64,
    pub cost_cents: i64,
    pub tax_rate_bp: i64,
    pub default_quantity_milli: i64,
    pub is_active: bool,
}

pub const PRODUCT_TYPES: &[&str] = &["Product", "Service"];
