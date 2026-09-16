use serde::{Deserialize, Serialize};

pub const ORG_UNIT_TYPES: &[&str] = &["Division", "Region", "Department", "Branch", "BusinessLine", "Custom"];

#[derive(Debug, Clone, Serialize)]
pub struct OrgUnit {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub unit_type: String,
    pub parent_org_unit_id: Option<String>,
    pub manager_user_id: Option<String>,
    pub status: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub default_team_id: Option<String>,
    pub path: String,
    pub depth: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrgUnitInput {
    pub name: String,
    pub unit_type: String,
    pub parent_org_unit_id: Option<String>,
    pub manager_user_id: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrgUnitUpdate {
    pub name: String,
    pub unit_type: String,
    pub manager_user_id: Option<String>,
    pub status: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

/// What moving an Organization Unit under a different parent would affect -
/// shown to the admin before the move actually commits (spec 1.2: "Moving
/// an Organization Unit must show an access-impact analysis before
/// commit").
#[derive(Debug, Clone, Serialize)]
pub struct OrgUnitMoveImpact {
    pub descendant_unit_count: i64,
    /// (object_key, owned-record-count) for every object type that has at
    /// least one record whose owning_org_unit_id falls under the unit's
    /// current subtree.
    pub owned_record_counts: Vec<(String, i64)>,
}
