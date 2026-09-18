//! Voice-First Mode, PR 1 of 3 (spec: "Lanesra OS Voice-First Mode" v0.1
//! Voice Assist + v0.2 Voice Actions). See the sibling migration
//! (0057_voice_mode_v1.sql) and `services::voice_execution_service`'s own
//! doc comment for the non-negotiable design principle this data model
//! exists to serve: a voice command is planned/confirmed/audited here, but
//! every actual write goes through the exact same entity service functions
//! the UI/API already call - never a shadow enforcement path.

use serde::{Deserialize, Serialize};

pub const VOICE_CAPABILITIES: &[&str] =
    &["use_voice", "search", "create", "update", "act", "bulk_act", "external_act", "use_agents"];

/// The four ceilings a policy can put on what Voice may do at all (spec
/// §5.1), declared broadest-last so derived `Ord` gives "which level is more
/// permissive" for free, mirroring `RecordScope`'s own ordering trick in
/// `models::access_role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaxActionLevel {
    AskOnly,
    Capture,
    ActWithConfirmation,
    Act,
}

impl MaxActionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            MaxActionLevel::AskOnly => "ask_only",
            MaxActionLevel::Capture => "capture",
            MaxActionLevel::ActWithConfirmation => "act_with_confirmation",
            MaxActionLevel::Act => "act",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ask_only" => Some(MaxActionLevel::AskOnly),
            "capture" => Some(MaxActionLevel::Capture),
            "act_with_confirmation" => Some(MaxActionLevel::ActWithConfirmation),
            "act" => Some(MaxActionLevel::Act),
            _ => None,
        }
    }
}

/// A plan's classified risk (spec §6's action-class table), broadest-last so
/// `.max()` across a multi-step plan gives its overall risk directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoiceRisk {
    None,
    Low,
    Medium,
    High,
    Critical,
    Blocked,
}

impl VoiceRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            VoiceRisk::None => "none",
            VoiceRisk::Low => "low",
            VoiceRisk::Medium => "medium",
            VoiceRisk::High => "high",
            VoiceRisk::Critical => "critical",
            VoiceRisk::Blocked => "blocked",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(VoiceRisk::None),
            "low" => Some(VoiceRisk::Low),
            "medium" => Some(VoiceRisk::Medium),
            "high" => Some(VoiceRisk::High),
            "critical" => Some(VoiceRisk::Critical),
            "blocked" => Some(VoiceRisk::Blocked),
            _ => None,
        }
    }
}

/// Safe representation returned to the frontend - never includes `pin_hash`,
/// mirroring `models::user::User` vs `UserRecord`'s exact split.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceUserSettings {
    pub user_id: String,
    pub enabled: bool,
    pub pin_set: bool,
    pub pin_set_at: Option<String>,
    pub failed_attempts: i64,
    pub locked_until: Option<String>,
    pub response_channel: String,
    pub spoken_detail: String,
    pub auto_speak_confirmations: bool,
    pub quiet_mode: bool,
    pub unlock_duration_minutes: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Internal row including the PIN hash, used only inside
/// `voice_session_service`/`voice_repo`.
pub struct VoiceUserSettingsRecord {
    pub user_id: String,
    pub enabled: bool,
    pub pin_hash: Option<String>,
    pub pin_set_at: Option<String>,
    pub failed_attempts: i64,
    pub locked_until: Option<String>,
    pub response_channel: String,
    pub spoken_detail: String,
    pub auto_speak_confirmations: bool,
    pub quiet_mode: bool,
    pub unlock_duration_minutes: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetVoicePinInput {
    pub pin: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoicePreferencesInput {
    pub response_channel: String,
    pub spoken_detail: String,
    pub auto_speak_confirmations: bool,
    pub quiet_mode: bool,
    pub unlock_duration_minutes: i64,
}

/// Voice's own capability+risk policy for one Access Role (or the
/// workspace default when `access_role_id` is `None`) - a second, narrower
/// gate that composes with `access_service::require_capability`, never
/// replaces or widens it (spec §5).
#[derive(Debug, Clone, Serialize)]
pub struct VoicePolicyBinding {
    pub id: String,
    pub workspace_id: String,
    pub access_role_id: Option<String>,
    pub can_use_voice: bool,
    pub can_search: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_act: bool,
    pub can_bulk_act: bool,
    pub can_external_act: bool,
    pub can_use_agents: bool,
    pub max_action_level: MaxActionLevel,
    pub processing_boundary: String,
    pub confidence_thresholds_json: String,
    pub max_unlock_minutes: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoicePolicyBindingInput {
    pub access_role_id: Option<String>,
    pub can_use_voice: bool,
    pub can_search: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_act: bool,
    pub can_bulk_act: bool,
    pub can_external_act: bool,
    pub can_use_agents: bool,
    pub max_action_level: String,
    pub processing_boundary: String,
    pub max_unlock_minutes: i64,
}

/// A unlocked Voice Session (spec §3's state machine) - `state` is the
/// user-visible mode; `context_object_key`/`context_record_id` are the
/// screen the user was on when they last spoke (spec §7).
#[derive(Debug, Clone, Serialize)]
pub struct VoiceSession {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub state: String,
    pub unlocked_at: String,
    pub expires_at: String,
    pub context_object_key: Option<String>,
    pub context_record_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceCommand {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub transcript: String,
    pub language: String,
    pub speech_confidence: Option<f64>,
    pub status: String,
    pub correlation_id: String,
    pub created_at: String,
}

/// One candidate the resolver considered but didn't (yet) pick - shown in a
/// clarification prompt (spec §8.1's "never silently choose a low-confidence
/// record... present concise choices").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionCandidate {
    pub record_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceResolution {
    pub id: String,
    pub command_id: String,
    pub intent: String,
    pub object_key: Option<String>,
    pub record_reference_text: Option<String>,
    pub resolved_record_id: Option<String>,
    pub intent_confidence: f64,
    pub entity_confidence: Option<f64>,
    pub candidates: Vec<ResolutionCandidate>,
    pub created_at: String,
}

/// One typed step of a plan (spec §10/§13) - `field` changes on
/// `object_key`/`record_id` (an existing record) or, when `record_id` is
/// `None`, a CREATE. `action` names the underlying operation
/// (`"update_fields"`, `"create_record"`, `"log_activity"`, `"create_task"`,
/// `"run_agent"`, ...) that `voice_execution_service` dispatches on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePlanStep {
    pub action: String,
    pub object_key: String,
    pub record_id: Option<String>,
    pub fields: std::collections::HashMap<String, String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceActionPlanBody {
    pub steps: Vec<VoicePlanStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceActionPlan {
    pub id: String,
    pub command_id: String,
    pub plan: VoiceActionPlanBody,
    pub risk: VoiceRisk,
    pub confirmation_required: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmVoicePlanInput {
    pub plan_id: String,
    pub method: String,
    pub edited_plan: Option<VoiceActionPlanBody>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceConfirmation {
    pub id: String,
    pub plan_id: String,
    pub method: String,
    pub outcome: String,
    pub user_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceExecution {
    pub id: String,
    pub plan_id: String,
    pub step_index: i64,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub action: String,
    pub result: String,
    pub error_message: Option<String>,
    pub undo_token: Option<String>,
    pub undone_at: Option<String>,
    pub correlation_id: String,
    pub executed_at: String,
}

/// The full result of executing a confirmed plan - what the desktop
/// confirmation panel's "Run result" view and the demo mirror both render.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceExecutionResult {
    pub plan_id: String,
    pub status: String,
    pub executions: Vec<VoiceExecution>,
    /// Voice-First Mode, PR 2: human-readable notes generated *at execution
    /// time*, not persisted as their own column - today the only producer
    /// is a RUN_AGENT/RUN_PIPELINE step, whose actual reply text has no
    /// natural home in `voice_executions`' fixed columns (no schema change
    /// in PR 2, per this feature's own scope guardrail). The full
    /// conversation/run itself is never lost - it's still real chat/run
    /// history, reachable via the agent's own `entity_id`/`entity_type` on
    /// that execution row - this is only a same-response convenience so
    /// the voice UI can speak/show the answer immediately.
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceProviderProfile {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub kind: String,
    pub privacy_class: String,
    pub is_default: bool,
    pub last_health_check_at: Option<String>,
    pub last_health_status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// One row of "My Voice Activity" / the admin Voice Activity search - a
/// denormalized join of command+resolution+plan for display, not a stored
/// table of its own.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceActivityEntry {
    pub command_id: String,
    pub transcript: String,
    pub intent: Option<String>,
    pub object_key: Option<String>,
    pub resolved_record_id: Option<String>,
    pub plan_status: Option<String>,
    pub risk: Option<String>,
    pub user_id: String,
    pub created_at: String,
}
