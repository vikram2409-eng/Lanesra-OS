//! Saved Views (product backlog "Saved Views & Bulk Actions"): a named,
//! persisted filter/sort/column/grouping combination for one object_key -
//! see migration 0034's own comment for why `filters`/`columns` are plain
//! JSON blobs rather than a modeled query, and `services::saved_view_service`
//! for visibility rules and the one-default-per-object_key invariant.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SavedView {
    pub id: String,
    pub workspace_id: String,
    pub object_key: String,
    pub name: String,
    pub owner_user_id: String,
    /// Denormalized at read time (not stored) so the UI can show "by
    /// Priya Shah" without a second round trip - see
    /// `saved_view_service::list_for_object`.
    pub owner_name: Option<String>,
    pub visibility: String,
    /// `{custom_field_key: value}`, the exact shape
    /// `useCustomFieldFilters` already produces client-side.
    pub filters: HashMap<String, String>,
    pub sort_field: Option<String>,
    pub sort_direction: String,
    /// Field keys in display order; `None` means "this list screen's own
    /// default columns."
    pub columns: Option<Vec<String>>,
    pub group_by_field: Option<String>,
    pub is_object_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SavedViewInput {
    pub object_key: String,
    pub name: String,
    pub visibility: String,
    #[serde(default)]
    pub filters: HashMap<String, String>,
    pub sort_field: Option<String>,
    #[serde(default = "default_sort_direction")]
    pub sort_direction: String,
    pub columns: Option<Vec<String>>,
    pub group_by_field: Option<String>,
}

fn default_sort_direction() -> String {
    "asc".to_string()
}
