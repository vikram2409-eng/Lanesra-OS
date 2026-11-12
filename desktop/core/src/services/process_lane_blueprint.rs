//! A read-only blueprint of a Process Lane (WP-0712, REQ-LANE-020).
//!
//! The blueprint renders a lane definition - purpose, stage chevrons with
//! their steps (kind, role, writes), run-data fields and mapped record
//! fields - so anyone who may run a lane can read what it will do before
//! it runs. It carries the definition only: no run, no anchor record, no
//! entered values - the service reads `process_lanes` /
//! `process_lane_versions` and nothing else, so a Viewer may read it
//! (AC-3). Any authenticated workspace member may call these reads; only
//! Administrators may change a lane, which stays in `process_lane_service`.
//!
//! Note on AC-3: the shared Receive Shipment example's own authored text
//! names PO-2026-001 as its worked example (step descriptions, the close
//! notification) - that text is part of the definition and the blueprint
//! renders the definition faithfully. What the blueprint never carries is
//! live record data read from a record table. Revising the example's
//! worked-example wording is a WP-0701 example change, not this WP's.
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::{AppError, AppResult};
use crate::models::process_lane::{parse_definition, LaneDefinition};
use crate::repositories::process_lane_repo;

/// The blueprint of one lane: its latest version's definition, rendered
/// read-only. The `version` and `status` are the version shown, so the
/// page can say whether what is on screen is a Draft or Published.
#[derive(Debug, Clone, Serialize)]
pub struct LaneBlueprint {
    pub lane_key: String,
    pub name: String,
    pub object_key: String,
    pub version: i64,
    pub status: String,
    pub definition: LaneDefinition,
}

/// One row of the lane list: enough to pick a lane and say which version
/// the blueprint would show. Carries no definition and no record data.
#[derive(Debug, Clone, Serialize)]
pub struct LaneBlueprintSummary {
    pub lane_key: String,
    pub name: String,
    pub object_key: String,
    pub latest_version: i64,
    pub latest_status: String,
}

fn require_authenticated(actor_user_id: Option<&str>) -> AppResult<()> {
    match actor_user_id {
        Some(_) => Ok(()),
        None => Err(AppError::Validation("Not authenticated".into())),
    }
}

/// The blueprint of the lane's latest version (Draft or Published).
/// A lane with no versions has no blueprint and reads as not found.
pub fn get_lane_blueprint(
    conn: &Connection,
    workspace_id: &str,
    lane_key: &str,
    actor_user_id: Option<&str>,
) -> AppResult<LaneBlueprint> {
    require_authenticated(actor_user_id)?;
    let lane = process_lane_repo::get_lane_by_key(conn, workspace_id, lane_key)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let version = process_lane_repo::latest_version(conn, &lane.id)?
        .ok_or_else(|| AppError::NotFound("Process lane".into()))?;
    let file = parse_definition(&version.definition_json)?;
    Ok(LaneBlueprint {
        lane_key: lane.lane_key,
        name: lane.name,
        object_key: lane.object_key,
        version: version.version,
        status: version.status,
        definition: file.lane,
    })
}

/// Every lane in the workspace, one summary row each, ordered by name.
/// Lanes with no versions are left out: they have no blueprint to show.
pub fn list_lane_blueprints(
    conn: &Connection,
    workspace_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<LaneBlueprintSummary>> {
    require_authenticated(actor_user_id)?;
    let lanes = process_lane_repo::list_lanes(conn, workspace_id)?;
    let mut summaries = Vec::new();
    for lane in lanes {
        if let Some(version) = process_lane_repo::latest_version(conn, &lane.id)? {
            summaries.push(LaneBlueprintSummary {
                lane_key: lane.lane_key,
                name: lane.name,
                object_key: lane.object_key,
                latest_version: version.version,
                latest_status: version.status,
            });
        }
    }
    Ok(summaries)
}
