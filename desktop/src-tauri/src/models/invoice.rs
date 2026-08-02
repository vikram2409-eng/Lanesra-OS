use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Invoice {
    pub id: String,
    pub workspace_id: String,
    pub invoice_number: String,
    pub company_id: String,
    pub contact_id: Option<String>,
    pub source_order_id: Option<String>,
    pub status: String,
    pub currency_code: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub amount_paid_cents: i64,
    pub balance_cents: i64,
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceLine {
    pub id: String,
    pub invoice_id: String,
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
pub struct InvoiceLineInput {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity_milli: i64,
    pub unit_price_cents: i64,
    pub discount_bp: i64,
    pub tax_rate_bp: i64,
}

/// Direct invoice creation (FR-INV-03). Converting an existing order uses
/// the dedicated conversion command instead, which copies its own lines.
#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceInput {
    pub company_id: String,
    pub contact_id: Option<String>,
    pub currency_code: String,
    pub issue_date: Option<String>,
    pub due_date: Option<String>,
    pub payment_terms: Option<String>,
    pub notes: Option<String>,
    pub lines: Vec<InvoiceLineInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceWithLines {
    pub invoice: Invoice,
    pub lines: Vec<InvoiceLine>,
    pub payments: Vec<Payment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Payment {
    pub id: String,
    pub invoice_id: String,
    pub amount_cents: i64,
    pub paid_at: String,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentInput {
    pub amount_cents: i64,
    pub paid_at: String,
    pub method: Option<String>,
    pub reference: Option<String>,
}

pub const INVOICE_STATUSES: &[&str] = &[
    "Draft",
    "Issued",
    "Partially Paid",
    "Paid",
    "Overdue",
    "Void",
    "Cancelled",
];
