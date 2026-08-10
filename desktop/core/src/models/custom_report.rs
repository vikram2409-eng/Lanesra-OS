use serde::{Deserialize, Serialize};

pub const REPORT_AGGREGATES: &[&str] = &["count", "sum"];
pub const REPORT_GROUP_BY_SOURCES: &[&str] = &["builtin", "custom"];

#[derive(Debug, Clone, Serialize)]
pub struct CustomReport {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub entity_type: String,
    pub group_by_source: String,
    pub group_by_field: String,
    pub aggregate: String,
    pub sum_field_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomReportInput {
    pub name: String,
    pub entity_type: String,
    pub group_by_source: String,
    pub group_by_field: String,
    pub aggregate: String,
    pub sum_field_key: Option<String>,
}

pub type CustomReportUpdate = CustomReportInput;

#[derive(Debug, Clone, Serialize)]
pub struct CustomReportRow {
    pub group: String,
    pub value: f64,
}
