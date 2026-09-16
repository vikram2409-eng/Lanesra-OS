use serde::{Deserialize, Serialize};

pub const WORK_TEAM_TYPES: &[&str] = &["Operational", "Queue", "Project", "CrossFunctional", "ExternalPartner"];

#[derive(Debug, Clone, Serialize)]
pub struct WorkTeam {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub code: String,
    pub team_type: String,
    pub primary_org_unit_id: String,
    pub owner_user_id: Option<String>,
    pub can_own_records: bool,
    pub status: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkTeamInput {
    pub name: String,
    pub code: String,
    pub team_type: String,
    pub primary_org_unit_id: String,
    pub owner_user_id: Option<String>,
    pub can_own_records: bool,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkTeamUpdate {
    pub name: String,
    pub team_type: String,
    pub primary_org_unit_id: String,
    pub owner_user_id: Option<String>,
    pub can_own_records: bool,
    pub status: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamMembership {
    pub id: String,
    pub workspace_id: String,
    pub team_id: String,
    pub user_id: String,
    pub role_in_team: Option<String>,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamMembershipInput {
    pub user_id: String,
    pub role_in_team: Option<String>,
    pub effective_from: Option<String>,
}
