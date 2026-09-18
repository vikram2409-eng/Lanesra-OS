//! Voice-First Mode, PR 1: risk classification and the confirmation/
//! approval gate (spec §6 "Risk-Based Confirmation" + §5.1 "Maximum Voice
//! Action Level"). Purely a decision function over an already-built plan -
//! it never touches the database or executes anything; `voice_execution_service`
//! is the only thing that acts on its `RiskDecision`.

use crate::models::voice::{MaxActionLevel, VoiceActionPlanBody, VoiceRisk};
use crate::services::voice_policy_service::EffectivePolicy;

/// One step's action name -> its risk class, per spec §6's table. `delete`
/// is intentionally `Blocked` - "often disabled for Voice Mode" per spec,
/// and this PR ships no delete action at all (spec §22/§33 - destructive
/// voice actions are explicitly out of MVP scope), so this arm exists to
/// fail closed if one is ever added without a matching risk review, not
/// because the planner currently emits it.
fn step_risk(action: &str) -> VoiceRisk {
    match action {
        "navigate" | "query" => VoiceRisk::None,
        "log_activity" | "create_task" => VoiceRisk::Low,
        "update_status" | "update_custom_fields" => VoiceRisk::Medium,
        "run_agent" | "run_pipeline" => VoiceRisk::Medium,
        "bulk_update" => VoiceRisk::High,
        "delete" => VoiceRisk::Blocked,
        _ => VoiceRisk::Medium,
    }
}

/// A plan's overall risk is the highest risk among its steps (spec §13:
/// a multi-action command is only as safe as its riskiest step).
pub fn classify(plan: &VoiceActionPlanBody) -> VoiceRisk {
    plan.steps.iter().map(|s| step_risk(&s.action)).max().unwrap_or(VoiceRisk::None)
}

/// Which of the 8 Voice capability toggles (Voice Governance) a given step
/// action needs - the composing, narrower gate spec §5 describes ("Voice
/// permissions can only narrow ... cannot grant access the user does not
/// already possess"). `navigate`/`query` need only Voice Search, since
/// they're read-only; every other step type currently emitted by
/// `voice_planner_service` creates or changes a record, so it needs Voice
/// Create/Update. `run_agent`/`run_pipeline` (PR 2's RUN_AGENT intent) need
/// Voice AI Agents specifically, on top of whatever the target agent's own
/// admin-only/action-name restrictions already require - this is the
/// composing, narrower gate, never a replacement for the agent's own
/// checks. `bulk_update` isn't emitted by any planner yet (PR 3), but is
/// mapped now so a future planner change can't accidentally skip this gate
/// by omission.
fn required_capability(action: &str) -> Option<&'static str> {
    match action {
        "navigate" | "query" => Some("search"),
        "create_task" | "log_activity" => Some("create"),
        "update_status" | "update_custom_fields" => Some("update"),
        "run_agent" | "run_pipeline" => Some("use_agents"),
        "bulk_update" => Some("bulk_act"),
        _ => None,
    }
}

fn capability_allowed(policy: &EffectivePolicy, capability: &str) -> bool {
    match capability {
        "search" => policy.can_search,
        "create" => policy.can_create,
        "update" => policy.can_update,
        "act" => policy.can_act,
        "bulk_act" => policy.can_bulk_act,
        "external_act" => policy.can_external_act,
        "use_agents" => policy.can_use_agents,
        _ => true,
    }
}

fn capability_label(capability: &str) -> &str {
    match capability {
        "search" => "Voice Search",
        "create" => "Voice Create",
        "update" => "Voice Update",
        "act" => "Voice Actions",
        "bulk_act" => "Voice Bulk Actions",
        "external_act" => "Voice External Actions",
        "use_agents" => "Voice AI Agents",
        _ => capability,
    }
}

#[derive(Debug, Clone)]
pub struct RiskDecision {
    pub risk: VoiceRisk,
    pub confirmation_required: bool,
    pub requires_approval: bool,
    pub blocked_reason: Option<String>,
}

/// Decides whether a plan may proceed at all under the user's Max Voice
/// Action Level (spec §5.1's four ceilings), and if so whether it needs
/// explicit confirmation or a manager-approval hold. `Act` is the only
/// level that ever skips confirmation on non-trivial risk, and even then
/// only up to High - Critical (external/financial) and Blocked (delete)
/// are never silently auto-approved by a Voice policy alone, matching
/// spec §6's own "Financial/regulated action: strong confirmation and
/// existing approval rules" / "Delete: often disabled" rows.
pub fn decide(plan: &VoiceActionPlanBody, policy: &EffectivePolicy) -> RiskDecision {
    let risk = classify(plan);

    if risk == VoiceRisk::Blocked {
        return RiskDecision { risk, confirmation_required: false, requires_approval: false, blocked_reason: Some("This action is disabled for Voice Mode.".into()) };
    }

    for step in &plan.steps {
        if let Some(cap) = required_capability(&step.action) {
            if !capability_allowed(policy, cap) {
                return RiskDecision {
                    risk,
                    confirmation_required: false,
                    requires_approval: false,
                    blocked_reason: Some(format!(
                        "Your Access Role doesn't have {} enabled for Voice Mode - ask an Administrator to grant it under Voice Governance.",
                        capability_label(cap)
                    )),
                };
            }
        }
    }

    let allowed_by_level = match policy.max_action_level {
        MaxActionLevel::AskOnly => risk == VoiceRisk::None,
        MaxActionLevel::Capture => risk <= VoiceRisk::Low,
        MaxActionLevel::ActWithConfirmation => risk <= VoiceRisk::High,
        MaxActionLevel::Act => risk <= VoiceRisk::High,
    };
    if !allowed_by_level {
        return RiskDecision {
            risk,
            confirmation_required: false,
            requires_approval: false,
            blocked_reason: Some(format!("Your Voice Action Level ({}) doesn't permit this action - ask an Administrator to raise it under Voice Governance.", policy.max_action_level.as_str())),
        };
    }

    let requires_approval = risk == VoiceRisk::Critical;
    let confirmation_required = match risk {
        VoiceRisk::None => false,
        // "Act" may skip confirmation on Low risk (spec: "may perform
        // specifically permitted low-risk actions without confirmation");
        // every other level always confirms once risk is above None.
        VoiceRisk::Low => policy.max_action_level != MaxActionLevel::Act,
        _ => true,
    };
    RiskDecision { risk, confirmation_required, requires_approval, blocked_reason: None }
}
