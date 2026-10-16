//! Workspace feature flags (WP-0010, ADR-0012). A flag is defined once in
//! code - `services::feature_flag_service::REGISTRY` - with a default for
//! each edition; an Administrator may override that default per workspace
//! (`feature_flag_overrides`, migration 0080). The online demo has no
//! default of its own: it shows a feature only once the release that turns
//! it on for desktop and Team Workspace is published (owner decision,
//! ADR-0013 #3).

use serde::{Deserialize, Serialize};

/// Which edition is asking. The desktop app and the Team Workspace server
/// each pass their own; the same workspace override applies to both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Desktop,
    Workspace,
}

/// Where a flag is in its life (ADR-0012 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Unfinished: hidden, and only a developer build with developer mode on
    /// can turn it on. Release builds can never enable it.
    Building,
    /// Ready to try: an Administrator may turn it on for their workspace.
    Preview,
    /// Released: on by default; the flag now only exists to be removed.
    Ga,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Building => "building",
            Stage::Preview => "preview",
            Stage::Ga => "ga",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Defaults {
    pub desktop: bool,
    pub workspace: bool,
}

/// One flag in the registry. Keys are `area.feature`, lower snake case.
#[derive(Debug)]
pub struct Flag {
    pub key: &'static str,
    pub description: &'static str,
    /// The work package that introduced the flag.
    pub work_package: &'static str,
    /// The work package or release by which the flag and its old code path
    /// are deleted.
    pub remove_by: &'static str,
    pub default: Defaults,
    pub stage: Stage,
}

impl Flag {
    pub fn default_for(&self, edition: Edition) -> bool {
        match edition {
            Edition::Desktop => self.default.desktop,
            Edition::Workspace => self.default.workspace,
        }
    }
}

/// One flag as the current workspace sees it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedFlag {
    pub key: String,
    pub description: String,
    pub work_package: String,
    pub stage: String,
    pub enabled: bool,
    /// True when an Administrator's override decides `enabled`, false when
    /// it is the edition default.
    pub overridden: bool,
    /// Whether the current build lets an Administrator change it.
    pub can_toggle: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FeatureFlagList {
    /// Developer mode: only possible in a debug build with
    /// `LANESRA_DEVELOPER_MODE=1`. Shows `building` flags and the toggle
    /// screen; release builds always report false.
    pub developer_mode: bool,
    pub flags: Vec<ResolvedFlag>,
}

/// Set (`enabled: Some`) or clear (`enabled: None`, back to the edition
/// default) a workspace override.
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlagOverrideInput {
    pub key: String,
    pub enabled: Option<bool>,
}

/// A stored override row.
#[derive(Debug, Clone, Serialize)]
pub struct FeatureFlagOverride {
    pub id: String,
    pub workspace_id: String,
    pub flag_key: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub updated_by_principal_id: Option<String>,
}
