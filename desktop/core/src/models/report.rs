use serde::{Deserialize, Serialize};

/// Every report shares a date range filter; `from`/`to` are ISO date
/// strings ("YYYY-MM-DD"). Missing bounds default to a wide-open range in
/// `report_service`, not here, so the UI can send `None` for "all time".
#[derive(Debug, Clone, Deserialize)]
pub struct ReportRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevenueByMonth {
    pub month: String, // "YYYY-MM"
    pub invoice_count: i64,
    pub total_cents: i64,
}

/// FR-RPT: "win rate by stage" from the original brainstorm doesn't
/// produce a meaningful breakdown in this schema - `stage` and `status`
/// both terminate at Won/Lost for a closed opportunity, so grouping by
/// stage just reproduces the win/loss split at 100%/0%. This groups by
/// owner instead, which is the meaningful axis, and adds a lost-reasons
/// breakdown (using the `lost_reason` field, which stage-grouping never
/// touched) as the second report in its place.
#[derive(Debug, Clone, Serialize)]
pub struct WinRateByOwner {
    pub owner_user_id: Option<String>,
    pub owner_name: String,
    pub won_count: i64,
    pub lost_count: i64,
    pub won_value_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LostReasonBreakdown {
    pub reason: String,
    pub count: i64,
    pub value_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArAgingBucket {
    pub bucket: String,
    pub invoice_count: i64,
    pub balance_cents: i64,
}

/// Invoices have no owner of their own - attributed via the invoice's
/// Company's owner_user_id, the closest thing to a deal owner this
/// schema has for a billed document.
#[derive(Debug, Clone, Serialize)]
pub struct SalesByOwner {
    pub owner_user_id: Option<String>,
    pub owner_name: String,
    pub invoice_count: i64,
    pub total_cents: i64,
}
