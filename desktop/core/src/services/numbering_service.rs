//! Admin flexibility: lets an Administrator override the auto-generated
//! number format (prefix + zero-padded digit width) per entity type, e.g.
//! turning Company's "CUS-000001" into "ACC-000001" or "ACC-ab0001" (the
//! letters are just part of the chosen prefix text - there's no separate
//! alpha-segment syntax to learn). The actual formatting logic lives in
//! `domain::numbering::allocate_number`, which consults the same
//! `numbering_configs` table this service writes to; this module is only
//! the admin-facing CRUD + validation layer on top of it.

use rusqlite::Connection;

use crate::domain::numbering::{self, NumberingConfig};
use crate::domain::{AppError, AppResult};
use crate::models::numbering_override::{EffectiveNumbering, NumberingOverrideInput, NUMBERING_ENTITY_TYPES};
use crate::repositories::{numbering_override_repo, user_repo};

const ENTITY_CONFIGS: &[(&str, &NumberingConfig)] = &[
    ("Company", &numbering::COMPANY),
    ("Contact", &numbering::CONTACT),
    ("Opportunity", &numbering::OPPORTUNITY),
    ("Product", &numbering::PRODUCT),
    ("Quote", &numbering::QUOTE),
    ("Order", &numbering::ORDER),
    ("Invoice", &numbering::INVOICE),
    ("Contract", &numbering::CONTRACT),
    ("Task", &numbering::TASK),
];

fn config_for(entity_type: &str) -> Option<&'static NumberingConfig> {
    ENTITY_CONFIGS.iter().find(|(t, _)| *t == entity_type).map(|(_, c)| *c)
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can change number formats".into()));
    }
    Ok(())
}

fn example_number(config: &NumberingConfig, prefix: &str, digits: i64) -> String {
    let width = digits as usize;
    if config.uses_year {
        let year = chrono::Utc::now().format("%Y").to_string();
        format!("{prefix}-{year}-{:0width$}", 1)
    } else {
        format!("{prefix}-{:0width$}", 1)
    }
}

/// The current effective (prefix, digits) for every numbered entity type -
/// either an admin override or the built-in default - so the admin screen
/// has one unified list rather than overrides-plus-hardcoded-defaults.
pub fn list_effective(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<EffectiveNumbering>> {
    require_admin(conn, actor_user_id)?;
    let mut result = Vec::with_capacity(ENTITY_CONFIGS.len());
    for (label, config) in ENTITY_CONFIGS {
        let over = numbering_override_repo::get_for_entity(conn, workspace_id, config.entity_type)?;
        let (prefix, digits, is_custom) = match over {
            Some(o) => (o.prefix, o.digits, true),
            None => (config.default_prefix.to_string(), config.digits as i64, false),
        };
        result.push(EffectiveNumbering {
            entity_type: label.to_string(),
            example: example_number(config, &prefix, digits),
            prefix,
            digits,
            is_custom,
        });
    }
    Ok(result)
}

pub fn set_override(conn: &Connection, workspace_id: &str, input: &NumberingOverrideInput, actor_user_id: Option<&str>) -> AppResult<EffectiveNumbering> {
    require_admin(conn, actor_user_id)?;
    if !NUMBERING_ENTITY_TYPES.contains(&input.entity_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid entity type '{}'", input.entity_type)));
    }
    let config = config_for(&input.entity_type).expect("validated against NUMBERING_ENTITY_TYPES above");
    let prefix = input.prefix.trim();
    if prefix.is_empty() || prefix.chars().count() > 20 {
        return Err(AppError::Validation("Prefix must be 1-20 characters".into()));
    }
    if !(1..=10).contains(&input.digits) {
        return Err(AppError::Validation("Digit width must be between 1 and 10".into()));
    }

    let saved = numbering_override_repo::upsert(conn, workspace_id, config.entity_type, prefix, input.digits)?;
    Ok(EffectiveNumbering {
        entity_type: input.entity_type.clone(),
        example: example_number(config, &saved.prefix, saved.digits),
        prefix: saved.prefix,
        digits: saved.digits,
        is_custom: true,
    })
}

/// Reverts an entity type to its built-in default format.
pub fn reset_override(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<EffectiveNumbering> {
    require_admin(conn, actor_user_id)?;
    let config = config_for(entity_type).ok_or_else(|| AppError::Validation(format!("Invalid entity type '{entity_type}'")))?;
    numbering_override_repo::delete(conn, workspace_id, config.entity_type)?;
    Ok(EffectiveNumbering {
        entity_type: entity_type.to_string(),
        example: example_number(config, config.default_prefix, config.digits as i64),
        prefix: config.default_prefix.to_string(),
        digits: config.digits as i64,
        is_custom: false,
    })
}
