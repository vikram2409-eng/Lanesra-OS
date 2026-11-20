//! A note added to a lane run (WP-0711, REQ-LANE-018), and a file attached
//! to one. Notes live in their own table (migration 0098); attachments reuse
//! the 0001 `attachments` table with `entity_type = "lane_run"` and the run
//! id as `entity_id`, so an attachment is visible only through its run
//! (AC-3) and the file bytes live under the data dir's `lane-attachments/`
//! folder, never in the database.

use serde::{Deserialize, Serialize};

/// One note on a lane run: which run, which step was current when it was
/// added (null when the run waited on nothing a person submits), who wrote
/// it, the body, when.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneRunNote {
    pub id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub step_id: Option<String>,
    pub author_user_id: Option<String>,
    /// The author's display name, resolved by the service when the note is
    /// read back - the runner shows a name, never a user id. Not a column.
    #[serde(default)]
    pub author_display_name: Option<String>,
    pub body: String,
    pub created_at: String,
}

/// One file attached to a lane run: the row in `attachments` plus the
/// relative path of the bytes on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneRunAttachment {
    pub id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub file_name: String,
    pub relative_path: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}
