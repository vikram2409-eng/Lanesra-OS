use serde::{Deserialize, Serialize};

/// Dashboard customization Phase 1: a workspace can have several named
/// dashboard layouts, each an ordered list of widgets, assigned to roles
/// with a required default fallback - structurally identical to
/// `ScreenLayout` (see that model's and its migration's doc comments for
/// the full rationale), just at the workspace level instead of per
/// entity_type, since a dashboard isn't scoped to any one object.
#[derive(Debug, Clone, Serialize)]
pub struct DashboardLayout {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub is_default: bool,
    /// Roles this layout is shown to - see
    /// `dashboard_layout_service::resolve_effective_dashboard`. Ignored on
    /// the default layout, which is the fallback for any role no other
    /// layout claims.
    pub roles: Vec<String>,
    pub draft: DashboardWidgets,
    /// `None` until first published - the live Dashboard only ever
    /// renders this, never the draft.
    pub published: Option<DashboardWidgets>,
    /// Per-app scoped automation - see `BusinessRule::app_id`'s doc
    /// comment for the full rationale; identical mechanism. Unlike
    /// business rules/workflows, a dashboard layout's `app_id` has no
    /// bearing on `resolve_effective_dashboard`'s own role resolution -
    /// it's purely which app's Admin screen shows the layout by default.
    pub app_id: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DashboardWidgets {
    pub widgets: Vec<DashboardWidget>,
}

/// One tile on the dashboard. `kind` selects how `config` is shaped -
/// Phase 1 ships `"kpi"`, whose config is `{"kpi_key": "..."}` (the same
/// opaque KPI-key string `Workspace.dashboard_kpi_prefs` already stored
/// workspace-wide - see that field's own doc comment - now scoped per
/// dashboard layout instead). Phase 2 adds `"chart"`, whose config is
/// `{"report_id": "..."}` - an existing saved Custom Report (see
/// `custom_report_service`), run fresh each time the dashboard renders;
/// a `report_id` whose report was since deleted is simply skipped by the
/// frontend, the same "stale key, no server-side cleanup needed" choice
/// a KPI widget's `kpi_key` already makes if a KPI definition is ever
/// retired. This layer never inspects `config`'s shape beyond routing it
/// in and out as JSON, the same "opaque to this layer, a frontend
/// registry resolves it" choice `screen_layout`'s field keys and
/// relationship keys already make.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub kind: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardLayoutInput {
    pub name: String,
    /// KPI keys to pre-populate as the new layout's initial widgets (in
    /// order) - mirrors `ScreenLayoutInput::initial_fields`. May be empty.
    pub initial_kpi_keys: Vec<String>,
    /// `None` (the default) means workspace-wide - see
    /// `BusinessRuleInput::app_id`'s doc comment.
    #[serde(default)]
    pub app_id: Option<String>,
}

/// Covers rename, role reassignment, and any draft edit (widget add/
/// remove/reorder) in one save - same reasoning as `ScreenLayoutUpdate`.
#[derive(Debug, Clone, Deserialize)]
pub struct DashboardLayoutUpdate {
    pub name: String,
    pub roles: Vec<String>,
    pub draft: DashboardWidgets,
    #[serde(default)]
    pub app_id: Option<String>,
}
