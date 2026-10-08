use serde::{Deserialize, Serialize};

/// UX/UI Modernization, Phase A (issue #191): Theme Studio.
pub const THEME_STATUSES: [&str; 3] = ["draft", "published", "archived"];
pub const THEME_PRESET_KEYS: [&str; 4] = ["orbit", "slate", "ember", "aurora"];
pub const RADIUS_SCALES: [&str; 3] = ["sharp", "soft", "rounded"];
pub const DENSITY_SCALES: [&str; 3] = ["comfortable", "compact", "dense"];

/// A single stored theme version - Draft, Published, or Archived once
/// superseded. `tokens_json` is the opaque storage shape; services parse
/// it into `ThemeTokens` before handing it to a caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTheme {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub status: String,
    pub version: i64,
    pub preset_key: Option<String>,
    pub tokens: ThemeTokens,
    pub created_at: String,
    pub created_by: Option<String>,
    pub published_at: Option<String>,
    pub published_by: Option<String>,
}

/// Input for saving a draft (new or edited) - never touches a Published
/// row, mirroring agent_version_service's own immutable-once-published rule.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceThemeInput {
    pub name: String,
    pub preset_key: Option<String>,
    pub tokens: ThemeTokens,
}

/// The full token document a theme version carries. Deliberately a
/// representative v1 slice of the spec's full token catalog (semantic
/// color + a basic typography/shape/density choice) - per-component
/// hover/pressed-state derivation, elevation and motion tokens are named,
/// scoped-out follow-up, not silently dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTokens {
    pub color: ThemeColorTokens,
    pub typography: ThemeTypographyTokens,
    pub shape: ThemeShapeTokens,
    /// 'comfortable' | 'compact' | 'dense'.
    pub density: String,
    /// Runtime UX Modernization (issue #198): the categorical palette
    /// charts cycle through for per-series/per-group color (distinct from
    /// `color`'s semantic brand/status tokens, which stay single-purpose).
    /// `#[serde(default)]` so a theme row saved before this field existed
    /// deserializes with the same built-in default every preset ships -
    /// the same "absent means not yet set" convention `DashboardWidget.layout`
    /// already uses.
    #[serde(default)]
    pub chart: ThemeChartTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeChartTokens {
    /// Ordered list of hex colors a multi-series/grouped chart cycles
    /// through by index - never inspected for length by this layer beyond
    /// `theme_service::validate_shape`'s own minimum-count check.
    pub palette: Vec<String>,
}

impl Default for ThemeChartTokens {
    fn default() -> Self {
        ThemeChartTokens {
            palette: DEFAULT_CHART_PALETTE.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// The same neutral categorical palette every curated preset in
/// `theme_service::built_in_presets` starts from before customizing it -
/// distinct enough from each other under normal vision and under the most
/// common color-vision deficiencies (deuteranopia/protanopia) to stay
/// distinguishable without relying on hue alone.
pub const DEFAULT_CHART_PALETTE: [&str; 6] = ["#635BFF", "#0F9D76", "#F59E0B", "#EF4444", "#3B82F6", "#8B5CF6"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColorTokens {
    pub brand_primary: String,
    pub brand_secondary: String,
    pub surface_app: String,
    pub surface_card: String,
    pub surface_sidebar: String,
    pub border_default: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub status_success: String,
    pub status_warning: String,
    pub status_danger: String,
    pub status_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypographyTokens {
    pub font_family: String,
    pub base_size_px: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeShapeTokens {
    /// 'sharp' | 'soft' | 'rounded'.
    pub radius_scale: String,
}

/// One WCAG 2.2-style contrast check result - see
/// theme_service::validate_tokens for the pairs actually checked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeContrastIssue {
    pub pair_label: String,
    pub ratio: f64,
    pub required_ratio: f64,
}
