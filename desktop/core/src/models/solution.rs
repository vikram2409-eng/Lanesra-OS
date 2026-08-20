//! Solution Packages & Admin IA design spec, Phase 4: named, scoped
//! Solutions - see migration 0031's own comment for the full "why" (the
//! Dynamics-365-style build-in-test / export / import-in-prod workflow,
//! and why "environment" needed no new modeling at all - a workspace
//! already is one).

use serde::{Deserialize, Serialize};

/// A named, versioned, admin-curated subset of this workspace's
/// components - the missing "pick exactly what goes in the box" layer
/// `export_local_workspace` (Phase 3) doesn't have, since that exports
/// everything the `local` publisher owns with no way to narrow it.
#[derive(Debug, Clone, Serialize)]
pub struct Solution {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub publisher_id: Option<String>,
    /// Present only when `publisher_id` resolves to a still-existing
    /// publisher - `ON DELETE SET NULL` means this can legitimately be
    /// `None` even when `publisher_id` itself was set at creation time.
    pub publisher_name: Option<String>,
    pub member_count: i64,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolutionInput {
    pub name: String,
    pub description: Option<String>,
    /// Defaults to `"1.0.0.0"` (see migration 0031) when omitted/blank.
    pub version: Option<String>,
    pub publisher_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolutionUpdate {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub publisher_id: Option<String>,
}

/// One curated membership row - identity is the same
/// `(artifact_type, metadata_id)` pair `SolutionComponent` uses, so a
/// component's presence in a Solution can always be cross-checked against
/// `solution_components` (migration 0030) without a second vocabulary.
#[derive(Debug, Clone, Serialize)]
pub struct SolutionMember {
    pub id: String,
    pub solution_id: String,
    pub artifact_type: String,
    pub metadata_id: String,
    pub added_at: String,
    pub added_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolutionMemberInput {
    pub artifact_type: String,
    pub metadata_id: String,
}

/// A `Solution` plus its curated members, each resolved for display the
/// same way the Components tab resolves a `WorkspaceComponent` - what the
/// Solutions tab's detail view reads to render its component picker/list.
#[derive(Debug, Clone, Serialize)]
pub struct SolutionDetail {
    pub solution: Solution,
    pub members: Vec<crate::models::solution_component::WorkspaceComponent>,
}
