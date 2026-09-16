use serde::{Deserialize, Serialize};

/// One of the two Record Owner principal kinds (spec §2.2's
/// record_owner_type). Kept as a thin string wrapper (not a Rust enum
/// derive-mapped straight to/from SQL) since every ownership column is
/// just TEXT with a CHECK constraint - the validation lives at the SQL
/// boundary, this just gives call sites named constants instead of magic
/// strings.
pub const OWNER_TYPE_USER: &str = "USER";
pub const OWNER_TYPE_TEAM: &str = "TEAM";

/// The four Ownership Modes an object (built-in or custom) can declare
/// (spec §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipMode {
    UserTeamOwned,
    OrgOwned,
    ParentControlled,
    SystemOwned,
}

impl OwnershipMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            OwnershipMode::UserTeamOwned => "USER_TEAM_OWNED",
            OwnershipMode::OrgOwned => "ORG_OWNED",
            OwnershipMode::ParentControlled => "PARENT_CONTROLLED",
            OwnershipMode::SystemOwned => "SYSTEM_OWNED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "USER_TEAM_OWNED" => Some(OwnershipMode::UserTeamOwned),
            "ORG_OWNED" => Some(OwnershipMode::OrgOwned),
            "PARENT_CONTROLLED" => Some(OwnershipMode::ParentControlled),
            "SYSTEM_OWNED" => Some(OwnershipMode::SystemOwned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerRef {
    pub owner_type: String,
    pub owner_id: String,
}

impl OwnerRef {
    pub fn user(user_id: impl Into<String>) -> Self {
        Self { owner_type: OWNER_TYPE_USER.to_string(), owner_id: user_id.into() }
    }
    pub fn team(team_id: impl Into<String>) -> Self {
        Self { owner_type: OWNER_TYPE_TEAM.to_string(), owner_id: team_id.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordOwnership {
    pub owner: Option<OwnerRef>,
    pub owning_org_unit_id: Option<String>,
    pub assigned_at: Option<String>,
    pub ownership_version: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OwnershipUpdate {
    pub owner_type: String,
    pub owner_id: String,
    pub owning_org_unit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnershipIneligible {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnershipTransferDryRun {
    pub eligible_ids: Vec<String>,
    pub ineligible: Vec<OwnershipIneligible>,
}
