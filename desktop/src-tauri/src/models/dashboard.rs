use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StageCount {
    pub stage: String,
    pub count: i64,
    pub value_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentActivity {
    pub occurred_at: String,
    pub event_type: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummary {
    pub open_pipeline_value_cents: i64,
    pub open_pipeline_count: i64,
    pub won_revenue_cents: i64,
    pub outstanding_invoices_cents: i64,
    pub overdue_invoices_cents: i64,
    pub overdue_invoices_count: i64,
    pub quotes_awaiting_response: i64,
    pub pipeline_by_stage: Vec<StageCount>,
    pub recent_activity: Vec<RecentActivity>,
}
