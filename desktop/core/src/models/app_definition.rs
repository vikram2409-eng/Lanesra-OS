use serde::{Deserialize, Serialize};

/// App Builder Phase 1: a named, publishable grouping of already-existing
/// objects, screens and a dashboard - see the migration's own doc comment
/// for the full rationale. `object_keys` is opaque here (a frontend
/// registry resolves each key to a nav section/screen the same way
/// screen_layouts' field keys already are); `dashboard_id`, if set, points
/// at an existing `DashboardLayout` this app uses as its own Dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct AppDefinition {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub icon: String,
    pub description: Option<String>,
    pub object_keys: Vec<String>,
    pub dashboard_id: Option<String>,
    pub is_published: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppDefinitionInput {
    pub name: String,
    pub icon: String,
    pub description: Option<String>,
}

/// Covers rename/re-icon/re-describe and the object-list/dashboard edit in
/// one save, the same "no separate draft" reasoning migration 0025's own
/// doc comment gives - unlike a screen or dashboard layout, there's no
/// preview-before-publish step for what an app contains, only whether it's
/// visible yet at all (`is_published`, changed via its own publish/
/// unpublish calls below, not this update).
#[derive(Debug, Clone, Deserialize)]
pub struct AppDefinitionUpdate {
    pub name: String,
    pub icon: String,
    pub description: Option<String>,
    pub object_keys: Vec<String>,
    pub dashboard_id: Option<String>,
}

pub const APP_PERMISSION_PRINCIPAL_TYPES: &[&str] = &["role", "user"];
pub const APP_PERMISSION_LEVELS: &[&str] = &["viewer", "editor"];

/// One access grant on an app - either to a role or to one specific user
/// (the actual new capability App Builder's permission model adds - see
/// migration 0025's doc comment). `level` is `"viewer"` or `"editor"`.
#[derive(Debug, Clone, Serialize)]
pub struct AppPermission {
    pub id: String,
    pub app_id: String,
    pub principal_type: String,
    pub principal_id: String,
    pub level: String,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppPermissionInput {
    pub principal_type: String,
    pub principal_id: String,
    pub level: String,
}

/// What `app_service::effective_access` resolves an actor's access on one
/// app down to, alongside the app itself - the shape the frontend's app
/// switcher and (eventually) any editor-vs-viewer UI gating consume.
#[derive(Debug, Clone, Serialize)]
pub struct AccessibleApp {
    pub app: AppDefinition,
    /// `"viewer"` or `"editor"` - never `None` here, since this only ever
    /// appears in a list already filtered to apps the actor can access.
    pub level: String,
}
