//! Workspace feature flags (WP-0010, ADR-0012).
//!
//! Every flag lives in [`REGISTRY`]; a flag that is not there does not
//! exist, and code refers to flags by their constant so a typo cannot
//! compile. Resolution order for one workspace and edition:
//!
//! 1. A `building` flag is off unless the build is in developer mode (a
//!    debug build with `LANESRA_DEVELOPER_MODE=1`); only then may an
//!    override turn it on. Release builds never enable a `building` flag.
//! 2. Otherwise an Administrator's workspace override wins.
//! 3. Otherwise the edition default from the registry.
//!
//! What flags may not do (ADR-0012 §4): they never gate a migration, a
//! security or correctness fix, or audit and policy enforcement.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::feature_flag::{
    Defaults, Edition, FeatureFlagList, FeatureFlagOverrideInput, Flag, ResolvedFlag, Stage,
};
use crate::repositories::{audit_repo, feature_flag_repo};
use crate::services::access_service;

/// The Playground shell (three-pane Build / Conversation / Trace screen).
/// Built under WP-0203; removed once the Playground reaches general
/// availability (WP-0223 ships it in all three editions together).
pub const PLAYGROUND_SHELL: Flag = Flag {
    key: "playground.shell",
    description: "Agent Playground: build, chat and trace panes",
    work_package: "WP-0203",
    remove_by: "WP-0223",
    default: Defaults {
        desktop: false,
        workspace: false,
    },
    stage: Stage::Building,
};

/// Every flag. Add new flags here, with a constant above.
pub const REGISTRY: &[&Flag] = &[&PLAYGROUND_SHELL];

/// The error a disabled feature's command or route returns. Its kind is
/// `not_found` (the REST API answers 404) and its message starts with
/// `feature_disabled:` so clients can tell it apart from a missing record.
pub fn disabled_error(flag: &Flag) -> AppError {
    AppError::NotFound(format!("feature_disabled: {}", flag.key))
}

/// Developer mode is only possible in a debug build. In a release build
/// this is a compile-time `false`, so no setting or environment variable
/// can turn a `building` flag on in a shipped app (ADR-0012 §5).
pub fn developer_mode() -> bool {
    cfg!(debug_assertions)
        && std::env::var("LANESRA_DEVELOPER_MODE")
            .map(|v| v == "1")
            .unwrap_or(false)
}

pub fn find(key: &str) -> Option<&'static Flag> {
    REGISTRY.iter().copied().find(|f| f.key == key)
}

fn resolve(
    conn: &Connection,
    workspace_id: &str,
    edition: Edition,
    flag: &Flag,
    developer_mode: bool,
) -> AppResult<(bool, bool)> {
    if flag.stage == Stage::Building && !developer_mode {
        return Ok((false, false));
    }
    match feature_flag_repo::get(conn, workspace_id, flag.key)? {
        Some(o) => Ok((o.enabled, true)),
        None => Ok((flag.default_for(edition), false)),
    }
}

/// Whether `flag` is on for this workspace and edition.
pub fn is_enabled(
    conn: &Connection,
    workspace_id: &str,
    edition: Edition,
    flag: &Flag,
    developer_mode: bool,
) -> AppResult<bool> {
    resolve(conn, workspace_id, edition, flag, developer_mode).map(|(enabled, _)| enabled)
}

/// Guard for a command or route behind `flag`: `Ok(())` when on, else
/// [`disabled_error`].
pub fn require_enabled(
    conn: &Connection,
    workspace_id: &str,
    edition: Edition,
    flag: &Flag,
    developer_mode: bool,
) -> AppResult<()> {
    if is_enabled(conn, workspace_id, edition, flag, developer_mode)? {
        Ok(())
    } else {
        Err(disabled_error(flag))
    }
}

fn can_toggle(flag: &Flag, developer_mode: bool) -> bool {
    flag.stage != Stage::Building || developer_mode
}

/// The resolved flags for this workspace, for the UI's `useFeature` hook
/// and the admin screen. Every signed-in user may read it. `building`
/// flags are listed only in developer mode.
pub fn list(
    conn: &Connection,
    workspace_id: &str,
    edition: Edition,
    developer_mode: bool,
) -> AppResult<FeatureFlagList> {
    let mut flags = Vec::with_capacity(REGISTRY.len());
    for flag in REGISTRY {
        if flag.stage == Stage::Building && !developer_mode {
            continue;
        }
        let (enabled, overridden) = resolve(conn, workspace_id, edition, flag, developer_mode)?;
        flags.push(ResolvedFlag {
            key: flag.key.to_string(),
            description: flag.description.to_string(),
            work_package: flag.work_package.to_string(),
            stage: flag.stage.as_str().to_string(),
            enabled,
            overridden,
            can_toggle: can_toggle(flag, developer_mode),
        });
    }
    Ok(FeatureFlagList {
        developer_mode,
        flags,
    })
}

/// Set or clear an Administrator's override for this workspace. Audited.
pub fn set_override(
    conn: &Connection,
    workspace_id: &str,
    edition: Edition,
    input: &FeatureFlagOverrideInput,
    actor: Option<&str>,
    developer_mode: bool,
) -> AppResult<FeatureFlagList> {
    access_service::require_admin_or_explicit_update(
        conn,
        actor,
        "FeatureFlag",
        "Only an Administrator can change feature flags",
    )?;
    let flag = find(&input.key)
        .ok_or_else(|| AppError::Validation(format!("Unknown feature flag '{}'", input.key)))?;
    if !can_toggle(flag, developer_mode) {
        return Err(AppError::Validation(format!(
            "'{}' is still being built and can only be changed in a developer build",
            flag.key
        )));
    }
    let summary = match input.enabled {
        Some(enabled) => {
            feature_flag_repo::upsert(conn, workspace_id, flag.key, enabled, actor)?;
            format!(
                "Feature flag '{}' turned {}",
                flag.key,
                if enabled { "on" } else { "off" }
            )
        }
        None => {
            feature_flag_repo::delete(conn, workspace_id, flag.key)?;
            format!("Feature flag '{}' reset to the edition default", flag.key)
        }
    };
    let details = serde_json::json!({ "key": flag.key, "enabled": input.enabled }).to_string();
    audit_repo::record(
        conn,
        workspace_id,
        actor,
        "feature_flag_changed",
        Some("FeatureFlag"),
        Some(flag.key),
        &summary,
        Some(&details),
    )?;
    list(conn, workspace_id, edition, developer_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_keys_are_unique_and_well_formed() {
        // Covers: REQ-PRE-006 (WP-0010)
        let mut keys: Vec<&str> = REGISTRY.iter().map(|f| f.key).collect();
        let before = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), before, "duplicate feature flag key");
        for f in REGISTRY {
            let (area, feature) = f.key.split_once('.').expect("key is area.feature");
            for part in [area, feature] {
                assert!(
                    !part.is_empty()
                        && part
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                    "key '{}' must be lower snake case area.feature",
                    f.key
                );
            }
            assert!(f.work_package.starts_with("WP-"), "{} names its WP", f.key);
            assert!(!f.remove_by.is_empty(), "{} names remove_by", f.key);
        }
    }

    #[test]
    fn building_flags_default_off_in_every_edition() {
        // Covers: REQ-PRE-006 (WP-0010 AC-2)
        for f in REGISTRY.iter().filter(|f| f.stage == Stage::Building) {
            assert!(
                !f.default.desktop && !f.default.workspace,
                "{} is building, so must default off",
                f.key
            );
        }
    }
}
