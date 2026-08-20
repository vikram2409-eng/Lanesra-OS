//! Solution Packages & Admin IA design spec, Phase 2: a real Publisher
//! registry - see migration 0029's own comment for the seeding story
//! (`lanesra`/`local` auto-created per workspace) and
//! `services::publisher_service` for key validation and the
//! reserved-keyword rules.

use serde::{Deserialize, Serialize};

/// A registered namespace owner in this workspace - every
/// `app_packages.package_id` is expected to be `"<publisher.key>.<name>"`,
/// enforced at import time (`industry_package_service::import_package`)
/// rather than left as an unvalidated string convention.
#[derive(Debug, Clone, Serialize)]
pub struct Publisher {
    pub id: String,
    pub workspace_id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    /// Auto-seeded, owns every bundled reference package - not something
    /// an admin creates by hand (the key `"lanesra"` is reserved, see
    /// `publisher_service::RESERVED_KEYS`).
    pub is_official: bool,
    /// Auto-seeded, the implicit home for hand-built customizations
    /// (key `"local"`, also reserved) - see migration 0029's own comment
    /// for what still isn't wired to it yet in this phase.
    pub is_local: bool,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublisherInput {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}
