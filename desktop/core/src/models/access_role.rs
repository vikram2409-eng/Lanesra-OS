//! Access Control v1 (spec: "Access Control v1" - Access Roles, capabilities,
//! Record Scopes, Access Inspector). See `access_role.rs`'s sibling
//! migration (0056_access_roles.sql) and `services::access_service` for the
//! evaluator this data model exists to serve.

use serde::{Deserialize, Serialize};

pub const CAPABILITIES: &[&str] = &["create", "read", "update", "delete", "assign"];
pub const RECORD_SCOPES: &[&str] = &["OWNER", "TEAM", "ORG_UNIT_AND_BELOW", "ORGANIZATION"];

/// The 5 actions an Access Role grants per object_key (spec: "Access Roles
/// ... per-object capability set"). Kept as a Rust enum (not a bare string)
/// since it's matched exhaustively in `access_service` and stored as 5
/// separate boolean columns, not a single TEXT column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Create,
    Read,
    Update,
    Delete,
    Assign,
}

impl Capability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::Create => "create",
            Capability::Read => "read",
            Capability::Update => "update",
            Capability::Delete => "delete",
            Capability::Assign => "assign",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "create" => Some(Capability::Create),
            "read" => Some(Capability::Read),
            "update" => Some(Capability::Update),
            "delete" => Some(Capability::Delete),
            "assign" => Some(Capability::Assign),
            _ => None,
        }
    }
}

/// The four Record Scope levels (spec: "Owner-only, Owner's Team, Owner's
/// Org Unit and below, or Organization-wide"), declared in broadest-last
/// order so the derived `Ord` directly implements "which scope wins" -
/// `access_service::capability_grant_for` folds multiple roles' scopes with
/// a plain `.max()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordScope {
    Owner,
    Team,
    OrgUnitAndBelow,
    Organization,
}

impl RecordScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordScope::Owner => "OWNER",
            RecordScope::Team => "TEAM",
            RecordScope::OrgUnitAndBelow => "ORG_UNIT_AND_BELOW",
            RecordScope::Organization => "ORGANIZATION",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "OWNER" => Some(RecordScope::Owner),
            "TEAM" => Some(RecordScope::Team),
            "ORG_UNIT_AND_BELOW" => Some(RecordScope::OrgUnitAndBelow),
            "ORGANIZATION" => Some(RecordScope::Organization),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessRole {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRoleInput {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRoleUpdate {
    pub name: String,
    pub description: String,
}

/// One row of a role's permission matrix - `object_key` is either a real
/// object_key (built-in or custom) or the wildcard `"*"`, the default for
/// any object_key without its own row (see the migration's own doc comment
/// for why this avoids re-seeding roles when a custom object is added).
#[derive(Debug, Clone, Serialize)]
pub struct AccessRoleGrant {
    pub id: String,
    pub access_role_id: String,
    pub object_key: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_assign: bool,
    pub record_scope: RecordScope,
    pub created_at: String,
    pub updated_at: String,
}

impl AccessRoleGrant {
    pub fn allows(&self, capability: Capability) -> bool {
        match capability {
            Capability::Create => self.can_create,
            Capability::Read => self.can_read,
            Capability::Update => self.can_update,
            Capability::Delete => self.can_delete,
            Capability::Assign => self.can_assign,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRoleGrantInput {
    pub object_key: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_assign: bool,
    pub record_scope: String,
}

/// The result of one capability check - what `access_service::require_capability`
/// enforces and what the Access Inspector narrates, always from the same
/// evaluation (see `access_service::explain_access`'s own doc comment).
#[derive(Debug, Clone, Serialize)]
pub struct AccessDecision {
    pub allowed: bool,
    pub scope: Option<RecordScope>,
    pub matched_role: Option<String>,
    pub reason: String,
}

/// Per-role breakdown shown by the Access Inspector - which of the actor's
/// roles had a grant that applied at all, and what it granted, even for
/// roles that didn't end up being the winning one.
#[derive(Debug, Clone, Serialize)]
pub struct AccessRoleCheck {
    pub role_name: String,
    pub matched_object_key: Option<String>,
    pub capability_granted: bool,
    pub scope: Option<RecordScope>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessInspectorResult {
    pub actor_user_id: String,
    pub object_key: String,
    pub capability: String,
    pub record_id: Option<String>,
    pub record_summary: Option<String>,
    pub roles_checked: Vec<AccessRoleCheck>,
    pub decision: AccessDecision,
}
