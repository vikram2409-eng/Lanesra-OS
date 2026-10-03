//! Admin Control Center Modernization (issue #197): Recently Viewed /
//! Pinned admin items - one row per (workspace, user, admin_tab); see
//! migration 0071's own doc comment for why pinning reuses the same row
//! rather than a second table.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AdminNavItem {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub admin_tab: String,
    pub label: String,
    pub is_pinned: bool,
    pub last_viewed_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordAdminVisitInput {
    pub admin_tab: String,
    pub label: String,
}
