use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Contact {
    pub id: String,
    pub workspace_id: String,
    pub contact_number: String,
    pub company_id: String,
    pub first_name: String,
    pub last_name: String,
    pub job_title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub is_primary: bool,
    pub status: String,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub department: Option<String>,
    pub preferred_contact_method: Option<String>,
    pub linkedin_url: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub archived_at: Option<String>,
}

// See CompanyInput's identical derive for why: lets the three fields added
// this round (department/preferred_contact_method/linkedin_url) default to
// None for every existing caller instead of requiring a mechanical touch
// of every struct literal/JSON payload across the codebase.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ContactInput {
    pub company_id: String,
    pub first_name: String,
    pub last_name: String,
    pub job_title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub is_primary: bool,
    pub status: String,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub department: Option<String>,
    pub preferred_contact_method: Option<String>,
    pub linkedin_url: Option<String>,
}

pub const CONTACT_STATUSES: &[&str] = &["Active", "Inactive", "Archived"];
