//! Solution Packages & Admin IA design spec, Phase 3: component-tagging -
//! see migration 0030's own comment for the full story (why this exists
//! alongside `industry_package::PackageArtifact`, and how the two paths
//! that populate it - direct admin creation vs. package install - avoid
//! stepping on each other).

use serde::Serialize;

/// One component's current owner. `artifact_type` matches
/// `PackageArtifact`'s own vocabulary, minus `numbering_override` and
/// `custom_record` (see migration 0030). `installed_app_id` is `None` for
/// anything still owned by the `local` publisher - the ordinary case for
/// whatever an admin built by hand.
#[derive(Debug, Clone, Serialize)]
pub struct SolutionComponent {
    pub id: String,
    pub workspace_id: String,
    pub artifact_type: String,
    pub metadata_id: String,
    pub publisher_id: String,
    pub installed_app_id: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

/// A `SolutionComponent` joined with the display fields the Solution
/// Management "Components" and "Local Workspace" views actually need -
/// avoids making every caller re-resolve the owning publisher itself.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceComponent {
    pub component: SolutionComponent,
    pub publisher_key: String,
    pub publisher_name: String,
    pub is_local: bool,
    /// Present only when `installed_app_id` is set - the installed app's
    /// display name, the same "which install created this" context
    /// `WorkspaceArtifact::installed_app_name` already gives the
    /// package_artifacts-backed view.
    pub installed_app_name: Option<String>,
}

/// The Packaged/Custom distinction's other half: every workspace has
/// exactly one implicit Custom "package" - whatever's tagged to the
/// `local` publisher - shown in the Solution Packages list as a synthetic
/// row alongside real installed (Packaged) packages, without ever writing
/// a fake `app_packages` row for it. `components_by_type` powers the same
/// per-type breakdown the Components tab shows, one level up.
#[derive(Debug, Clone, Serialize)]
pub struct LocalWorkspaceSummary {
    pub publisher_id: String,
    pub component_count: i64,
    pub components_by_type: Vec<(String, i64)>,
}
