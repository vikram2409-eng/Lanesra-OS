//! UX/UI Modernization, Phase A (issue #191): Theme Studio. Owns the 4
//! curated presets (Orbit/Slate/Ember/Aurora, exact values from the UX/UI
//! spec), contrast validation, and the publish gate - `workspace_theme_repo`
//! is raw CRUD only, same "service owns the rules, repo owns the rows"
//! split every other versioned surface in this codebase already uses.
//!
//! A workspace with no Published theme keeps the exact pre-Theme-Studio
//! look (the static defaults baked into styles.css) - this feature is
//! purely additive until an admin explicitly publishes something, the
//! same opt-in posture Phase 2's Policy Engine established.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::workspace_theme::{
    ThemeColorTokens, ThemeContrastIssue, ThemeShapeTokens, ThemeTokens, ThemeTypographyTokens, WorkspaceTheme, WorkspaceThemeInput,
    DENSITY_SCALES, RADIUS_SCALES, THEME_PRESET_KEYS,
};
use crate::repositories::{audit_repo, workspace_theme_repo};

const MIN_CONTRAST_RATIO: f64 = 4.5;

fn system_font_stack() -> String {
    "-apple-system, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif".to_string()
}

/// (preset_key, display_name, character_blurb, tokens) for all 4 curated
/// presets - exact hex values from the UX/UI Modernization spec section 14.
pub fn built_in_presets() -> Vec<(&'static str, &'static str, &'static str, ThemeTokens)> {
    vec![
        (
            "orbit",
            "Orbit",
            "Signature Lanesra: modern indigo, premium SaaS/enterprise.",
            ThemeTokens {
                color: ThemeColorTokens {
                    brand_primary: "#635BFF".into(),
                    brand_secondary: "#8B5CF6".into(),
                    surface_app: "#F7F8FC".into(),
                    surface_card: "#FFFFFF".into(),
                    surface_sidebar: "#111827".into(),
                    border_default: "#E2E5EE".into(),
                    text_primary: "#111827".into(),
                    text_secondary: "#5B6572".into(),
                    status_success: "#0F9D76".into(),
                    status_warning: "#F59E0B".into(),
                    status_danger: "#EF4444".into(),
                    status_info: "#3B82F6".into(),
                },
                typography: ThemeTypographyTokens { font_family: system_font_stack(), base_size_px: 16 },
                shape: ThemeShapeTokens { radius_scale: "rounded".into() },
                density: "comfortable".into(),
            },
        ),
        (
            "slate",
            "Slate",
            "Consulting/enterprise: restrained blue + teal.",
            ThemeTokens {
                color: ThemeColorTokens {
                    brand_primary: "#2563EB".into(),
                    brand_secondary: "#14B8A6".into(),
                    surface_app: "#F8FAFC".into(),
                    surface_card: "#FFFFFF".into(),
                    surface_sidebar: "#0F172A".into(),
                    border_default: "#E1E7F0".into(),
                    text_primary: "#0F172A".into(),
                    text_secondary: "#5B6572".into(),
                    status_success: "#16A34A".into(),
                    status_warning: "#F59E0B".into(),
                    status_danger: "#EF4444".into(),
                    status_info: "#3B82F6".into(),
                },
                typography: ThemeTypographyTokens { font_family: system_font_stack(), base_size_px: 16 },
                shape: ThemeShapeTokens { radius_scale: "soft".into() },
                density: "comfortable".into(),
            },
        ),
        (
            "ember",
            "Ember",
            "Warm executive: amber/orange with neutral graphite.",
            ThemeTokens {
                color: ThemeColorTokens {
                    brand_primary: "#C2410C".into(),
                    brand_secondary: "#EAB308".into(),
                    surface_app: "#FAFAF9".into(),
                    surface_card: "#FFFFFF".into(),
                    surface_sidebar: "#1C1917".into(),
                    border_default: "#E7E3DF".into(),
                    text_primary: "#1C1917".into(),
                    text_secondary: "#5B6572".into(),
                    status_success: "#15803D".into(),
                    status_warning: "#F59E0B".into(),
                    status_danger: "#EF4444".into(),
                    status_info: "#3B82F6".into(),
                },
                typography: ThemeTypographyTokens { font_family: system_font_stack(), base_size_px: 16 },
                shape: ThemeShapeTokens { radius_scale: "soft".into() },
                density: "comfortable".into(),
            },
        ),
        (
            "aurora",
            "Aurora",
            "Contemporary teal with violet accents, modern technical feel.",
            ThemeTokens {
                color: ThemeColorTokens {
                    brand_primary: "#0F766E".into(),
                    brand_secondary: "#7C3AED".into(),
                    surface_app: "#F4FAF9".into(),
                    surface_card: "#FFFFFF".into(),
                    surface_sidebar: "#102A2E".into(),
                    border_default: "#DCEAE8".into(),
                    text_primary: "#102A2E".into(),
                    text_secondary: "#5B6572".into(),
                    status_success: "#15803D".into(),
                    status_warning: "#F59E0B".into(),
                    status_danger: "#EF4444".into(),
                    status_info: "#3B82F6".into(),
                },
                typography: ThemeTypographyTokens { font_family: system_font_stack(), base_size_px: 16 },
                shape: ThemeShapeTokens { radius_scale: "rounded".into() },
                density: "comfortable".into(),
            },
        ),
    ]
}

pub fn get_preset(key: &str) -> Option<ThemeTokens> {
    built_in_presets().into_iter().find(|(k, _, _, _)| *k == key).map(|(_, _, _, tokens)| tokens)
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn hex_to_linear_rgb(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let channel = |s: &str| -> Option<f64> {
        let v = u8::from_str_radix(s, 16).ok()? as f64 / 255.0;
        Some(if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) })
    };
    Some((channel(&hex[0..2])?, channel(&hex[2..4])?, channel(&hex[4..6])?))
}

fn relative_luminance(hex: &str) -> Option<f64> {
    let (r, g, b) = hex_to_linear_rgb(hex)?;
    Some(0.2126 * r + 0.7152 * g + 0.0722 * b)
}

/// WCAG 2.x contrast ratio between two hex colors - 1.0 (no contrast) to
/// 21.0 (black on white). Returns `None` if either color fails to parse,
/// which `validate_tokens` below treats as its own reported issue rather
/// than silently skipping the pair.
fn contrast_ratio(hex_a: &str, hex_b: &str) -> Option<f64> {
    let la = relative_luminance(hex_a)?;
    let lb = relative_luminance(hex_b)?;
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    Some((lighter + 0.05) / (darker + 0.05))
}

/// A representative, not exhaustive, set of text/surface pairs - full
/// per-component contrast auditing (hover/pressed states, chart palette,
/// focus ring) stays a named follow-up rather than something this first
/// pass silently claims to cover.
pub fn validate_tokens(tokens: &ThemeTokens) -> Vec<ThemeContrastIssue> {
    let pairs: [(&str, &str, &str); 3] = [
        ("Primary text on app background", &tokens.color.text_primary, &tokens.color.surface_app),
        ("Primary text on card surface", &tokens.color.text_primary, &tokens.color.surface_card),
        ("Primary button text (white) on brand primary", "#FFFFFF", &tokens.color.brand_primary),
    ];
    let mut issues = Vec::new();
    for (label, a, b) in pairs {
        let ratio = contrast_ratio(a, b).unwrap_or(0.0);
        if ratio < MIN_CONTRAST_RATIO {
            issues.push(ThemeContrastIssue { pair_label: label.to_string(), ratio, required_ratio: MIN_CONTRAST_RATIO });
        }
    }
    issues
}

fn validate_shape(input: &WorkspaceThemeInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Theme name is required".into()));
    }
    if let Some(preset_key) = &input.preset_key {
        if !THEME_PRESET_KEYS.contains(&preset_key.as_str()) {
            return Err(AppError::Validation(format!("Unknown preset '{preset_key}'")));
        }
    }
    if !RADIUS_SCALES.contains(&input.tokens.shape.radius_scale.as_str()) {
        return Err(AppError::Validation(format!("Unknown radius scale '{}'", input.tokens.shape.radius_scale)));
    }
    if !DENSITY_SCALES.contains(&input.tokens.density.as_str()) {
        return Err(AppError::Validation(format!("Unknown density '{}'", input.tokens.density)));
    }
    Ok(())
}

pub fn list_versions(conn: &Connection, workspace_id: &str) -> AppResult<Vec<WorkspaceTheme>> {
    Ok(workspace_theme_repo::list_versions(conn, workspace_id)?)
}

pub fn get_published(conn: &Connection, workspace_id: &str) -> AppResult<Option<WorkspaceTheme>> {
    Ok(workspace_theme_repo::get_published(conn, workspace_id)?)
}

pub fn get(conn: &Connection, id: &str, workspace_id: &str) -> AppResult<WorkspaceTheme> {
    let theme = workspace_theme_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Theme".into()))?;
    if theme.workspace_id != workspace_id {
        return Err(AppError::NotFound("Theme".into()));
    }
    Ok(theme)
}

pub fn save_draft(
    conn: &Connection,
    workspace_id: &str,
    existing_id: Option<&str>,
    input: &WorkspaceThemeInput,
    actor_user_id: Option<&str>,
) -> AppResult<WorkspaceTheme> {
    require_admin(conn, actor_user_id)?;
    validate_shape(input)?;
    if let Some(id) = existing_id {
        let existing = get(conn, id, workspace_id)?;
        if existing.status != "draft" {
            return Err(AppError::Validation("Only a Draft theme can be edited - publish a new draft instead of editing a Published or Archived one".into()));
        }
        workspace_theme_repo::update_draft(conn, id, input)?;
        audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("theme"), Some(id), &format!("Updated theme draft '{}'", input.name), None)?;
        get(conn, id, workspace_id)
    } else {
        let created = workspace_theme_repo::create_draft(conn, workspace_id, input, actor_user_id)?;
        audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("theme"), Some(&created.id), &format!("Created theme draft '{}'", input.name), None)?;
        Ok(created)
    }
}

/// AI-AC-15-style gate (named after the UX/UI spec's own UX-AC-15):
/// blocks Publish outright on a critical contrast failure rather than a
/// privileged override, matching the spec's own "prefer no override in
/// early versions" instruction.
pub fn publish(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<WorkspaceTheme> {
    require_admin(conn, actor_user_id)?;
    let theme = get(conn, id, workspace_id)?;
    if theme.status != "draft" {
        return Err(AppError::Validation("Only a Draft theme can be published".into()));
    }
    let issues = validate_tokens(&theme.tokens);
    if !issues.is_empty() {
        let detail = issues
            .iter()
            .map(|i| format!("{} ({:.2}:1, needs {:.1}:1)", i.pair_label, i.ratio, i.required_ratio))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::Validation(format!("Can't publish: critical contrast failure - {detail}")));
    }
    let published = workspace_theme_repo::publish(conn, id, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "publish", Some("theme"), Some(id), &format!("Published theme '{}'", theme.name), None)?;
    Ok(published)
}

/// "Roll back" never mutates history - it creates a brand-new Draft
/// carrying `from_version`'s own tokens, validates it exactly like any
/// other publish, and publishes that new version. The old Published
/// version an admin is rolling back from is left exactly as it was,
/// now Archived.
pub fn rollback_to_version(conn: &Connection, workspace_id: &str, from_version: i64, actor_user_id: Option<&str>) -> AppResult<WorkspaceTheme> {
    require_admin(conn, actor_user_id)?;
    let source = workspace_theme_repo::list_versions(conn, workspace_id)?
        .into_iter()
        .find(|t| t.version == from_version)
        .ok_or_else(|| AppError::NotFound("Theme version".into()))?;
    let input = WorkspaceThemeInput {
        name: format!("{} (rolled back from v{})", source.name, source.version),
        preset_key: source.preset_key.clone(),
        tokens: source.tokens.clone(),
    };
    let draft = workspace_theme_repo::create_draft(conn, workspace_id, &input, actor_user_id)?;
    let published = publish(conn, &draft.id, workspace_id, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("theme"), Some(&draft.id), &format!("Rolled back theme to version {from_version} (now '{}')", input.name), None)?;
    Ok(published)
}

pub fn delete_draft(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let theme = get(conn, id, workspace_id)?;
    if theme.status != "draft" {
        return Err(AppError::Validation("Only a Draft theme can be deleted".into()));
    }
    workspace_theme_repo::delete_draft(conn, id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "delete", Some("theme"), Some(id), &format!("Deleted theme draft '{}'", theme.name), None)?;
    Ok(())
}
