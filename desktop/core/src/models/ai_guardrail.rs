//! WP-0133: output guardrails as versioned policy objects (REQ-GOV-007).

use serde::{Deserialize, Serialize};

/// A guardrail with its current version's checker configuration joined in.
#[derive(Debug, Clone, Serialize)]
pub struct Guardrail {
    pub id: String,
    pub workspace_id: String,
    pub builtin_key: Option<String>,
    pub name: String,
    pub description: Option<String>,
    /// 'rule' | 'pii' | 'schema' | 'judge'.
    pub checker_type: String,
    pub is_builtin: bool,
    pub current_version: i64,
    pub enabled: bool,
    /// WP-0128: where in the agent run loop this guardrail runs - `off`
    /// (not wired in), `step`, `model` or `tool`. See
    /// `policy_engine_service::HookPoint`.
    pub hook_point: String,
    /// The current version's checker configuration (raw JSON string).
    pub config_json: String,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

/// One stored version of a guardrail's configuration.
#[derive(Debug, Clone, Serialize)]
pub struct GuardrailVersion {
    pub id: String,
    pub guardrail_id: String,
    pub version: i64,
    pub config_json: String,
    pub created_at: String,
    pub created_by: Option<String>,
}

/// Create a custom guardrail, or (with only `config_json`) edit one.
#[derive(Debug, Clone, Deserialize)]
pub struct GuardrailInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub checker_type: String,
    /// Checker-specific configuration, as a JSON string.
    pub config_json: String,
    /// WP-0128: where in the run loop it runs (`off`/`step`/`model`/`tool`).
    /// Defaults `off` so a created guardrail is not wired in until chosen.
    #[serde(default = "default_hook_point")]
    pub hook_point: String,
}

pub fn default_hook_point() -> String {
    "off".to_string()
}

/// The new configuration for an edit; a new version is written from it.
#[derive(Debug, Clone, Deserialize)]
pub struct GuardrailEdit {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    /// When present and different, writes a new version and bumps current_version.
    #[serde(default)]
    pub config_json: Option<String>,
    /// WP-0128: change where in the loop it runs (`off`/`step`/`model`/`tool`).
    #[serde(default)]
    pub hook_point: Option<String>,
}

/// The result of evaluating a guardrail against some output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GuardrailOutcome {
    pub passed: bool,
    pub detail: String,
}

/// A recorded evaluation tying a run to the guardrail version it checked.
#[derive(Debug, Clone, Serialize)]
pub struct GuardrailEvaluation {
    pub id: String,
    pub workspace_id: String,
    pub run_kind: String,
    pub run_id: String,
    pub guardrail_id: String,
    pub guardrail_version: i64,
    pub passed: bool,
    pub detail: Option<String>,
    pub created_at: String,
}
