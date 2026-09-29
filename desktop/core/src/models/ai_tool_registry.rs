//! AI Agent Platform v2, Phase 2: the risk taxonomy the Policy Engine's
//! Tool-Call Firewall evaluates every tool call against, and the
//! per-workspace overrides an admin may layer on top of the built-in
//! defaults (`services::tool_registry_service::default_risk_for`). See
//! migration `0061_agent_policy_engine.sql`'s own doc comment for why a
//! row here is an override, not a full catalog.

use serde::{Deserialize, Serialize};

/// Broadest-last so derived `Ord` gives "at or above this risk" for free
/// (`RiskLevel::Write >= RiskLevel::LowWrite`), the same ordering trick
/// `VoiceRisk`/`MaxActionLevel` already use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// A read-only lookup - lists, gets, searches. Never writes anything.
    Read,
    /// An ordinary business-record create/update - the ongoing, ordinary
    /// work every ordinary agent does.
    LowWrite,
    /// A workspace-configuration change (a Business Rule, a Custom
    /// Object, a Webhook, ...) - affects how the workspace itself
    /// behaves, not just one record.
    Write,
    /// Calls out to a system this workspace doesn't own (a connector
    /// write action).
    ExternalAction,
    /// Removes a record from normal use (archive/soft-delete).
    Destructive,
    /// Touches identity, credentials, or triggers another agent/pipeline
    /// run - the most sensitive tier.
    Privileged,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Read => "read",
            RiskLevel::LowWrite => "low_write",
            RiskLevel::Write => "write",
            RiskLevel::ExternalAction => "external_action",
            RiskLevel::Destructive => "destructive",
            RiskLevel::Privileged => "privileged",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "read" => Some(RiskLevel::Read),
            "low_write" => Some(RiskLevel::LowWrite),
            "write" => Some(RiskLevel::Write),
            "external_action" => Some(RiskLevel::ExternalAction),
            "destructive" => Some(RiskLevel::Destructive),
            "privileged" => Some(RiskLevel::Privileged),
            _ => None,
        }
    }

    pub fn all() -> &'static [RiskLevel] {
        &[
            RiskLevel::Read,
            RiskLevel::LowWrite,
            RiskLevel::Write,
            RiskLevel::ExternalAction,
            RiskLevel::Destructive,
            RiskLevel::Privileged,
        ]
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AiToolRegistryOverride {
    pub id: String,
    pub workspace_id: String,
    pub tool_name: String,
    pub risk_level: RiskLevel,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiToolRegistryOverrideInput {
    pub tool_name: String,
    pub risk_level: RiskLevel,
}
