use serde::{Deserialize, Serialize};

/// Spec-matching "Organization" shape - a view over `workspaces` plus the
/// org-specific columns 0051_organization_and_org_units.sql added
/// (org_code/org_status/root_org_unit_id), not a separate table. See that
/// migration's own doc comment for why: `workspaces` already carries
/// name/legal_name/currency/locale/timezone, and a workspace already is
/// the one-tenant-per-workspace security boundary the spec calls
/// "Organization" - a parallel table would only duplicate those columns.
#[derive(Debug, Clone, Serialize)]
pub struct Organization {
    pub workspace_id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub code: String,
    pub status: String,
    pub default_currency: String,
    pub locale: String,
    pub timezone: String,
    pub root_org_unit_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Only the fields this migration added are editable here - name/legal
/// name/currency/locale/timezone already have their own editor (the
/// Business Profile admin screen, `workspace_service::update`) and this
/// pass deliberately doesn't duplicate that edit surface.
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationUpdate {
    pub code: String,
    pub status: String,
}
