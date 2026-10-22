//! WP-0107: who an agent run acts as (ADR-0006, ADR-0016 §1).
//!
//! Built on Access Control v1, not a parallel role model: a human's rights
//! are its user's Access Roles (`access_service`) plus the `Administrator`
//! user role (`user_service::require_admin`). An agent run never adds
//! rights to those:
//!
//! - An **interactive** run (a person started it) acts as that person,
//!   further limited by the run's Agent Identity when it has one. It never
//!   exceeds the person's access, even when the identity holds more
//!   (REQ-GOV-002).
//! - An **unattended** run (schedule, webhook, drained queue) acts as its
//!   Agent Identity only. It never borrows an Administrator's rights, not
//!   even from the owner of the API client that triggered it (REQ-GOV-003).
//!
//! Agent Identities themselves are stored by WP-0108; this module only
//! needs what one grants for a run (`AgentIdentityGrant`).
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107). Threats: T-03, T-12.

use std::cell::RefCell;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domain::{AppError, AppResult, ErrorClass};

/// The three kinds of principal (ADR-0006). Every audit row names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalType {
    /// A person who signs in (password, OIDC or SAML).
    Human,
    /// A Lanesra-issued identity an agent runs as. Never signs in.
    AgentIdentity,
    /// A programmatic client. Never lends its owner's rights to a run.
    ApiClient,
}

/// What a run's Agent Identity is allowed, as resolved for that run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentIdentityGrant {
    pub id: String,
    pub workspace_id: String,
    /// Holds the `Administrator` role in its workspace.
    pub is_administrator: bool,
}

/// The identity an agent run acts with, fixed when the run starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectiveIdentity {
    Interactive {
        user_id: String,
        agent_identity: Option<AgentIdentityGrant>,
    },
    Unattended {
        agent_identity: Option<AgentIdentityGrant>,
        /// Whose Access Roles limit record reads and writes. For a webhook
        /// this is the API client's owner, so the run can never see more
        /// than they can; it is never a source of Administrator rights.
        record_actor: Option<String>,
    },
}

impl EffectiveIdentity {
    pub fn interactive(user_id: &str) -> Self {
        Self::Interactive {
            user_id: user_id.to_string(),
            agent_identity: None,
        }
    }

    pub fn unattended(record_actor: Option<&str>) -> Self {
        Self::Unattended {
            agent_identity: None,
            record_actor: record_actor.map(str::to_string),
        }
    }

    /// For callers whose actor is already "the person who started this, or
    /// nobody": a user id means interactive, `None` means unattended.
    pub fn from_actor(actor: Option<&str>) -> Self {
        match actor {
            Some(user_id) => Self::interactive(user_id),
            None => Self::unattended(None),
        }
    }

    pub fn with_agent_identity(self, grant: AgentIdentityGrant) -> Self {
        match self {
            Self::Interactive { user_id, .. } => Self::Interactive {
                user_id,
                agent_identity: Some(grant),
            },
            Self::Unattended { record_actor, .. } => Self::Unattended {
                agent_identity: Some(grant),
                record_actor,
            },
        }
    }

    pub fn is_unattended(&self) -> bool {
        matches!(self, Self::Unattended { .. })
    }

    /// The principal the run acts as.
    pub fn principal_type(&self) -> PrincipalType {
        match self {
            Self::Interactive { .. } => PrincipalType::Human,
            Self::Unattended { .. } => PrincipalType::AgentIdentity,
        }
    }

    pub fn agent_identity(&self) -> Option<&AgentIdentityGrant> {
        match self {
            Self::Interactive { agent_identity, .. } | Self::Unattended { agent_identity, .. } => {
                agent_identity.as_ref()
            }
        }
    }

    /// The user whose Access Roles record access is checked against.
    pub fn record_actor(&self) -> Option<&str> {
        match self {
            Self::Interactive { user_id, .. } => Some(user_id),
            Self::Unattended { record_actor, .. } => record_actor.as_deref(),
        }
    }

    /// Refuses unless every principal in the run holds `Administrator`:
    /// the person (interactive) and the Agent Identity, when there is one.
    /// An unattended run needs an Agent Identity that holds it itself.
    pub fn require_admin(&self, conn: &Connection, workspace_id: &str) -> AppResult<()> {
        if let Some(grant) = self.agent_identity() {
            if grant.workspace_id != workspace_id {
                return Err(denied("This agent's identity belongs to another workspace"));
            }
        }
        match self {
            Self::Interactive {
                user_id,
                agent_identity,
            } => {
                crate::services::user_service::require_admin(conn, Some(user_id))?;
                match agent_identity {
                    Some(grant) if !grant.is_administrator => Err(denied(
                        "This agent's identity does not hold the Administrator role",
                    )),
                    _ => Ok(()),
                }
            }
            Self::Unattended { agent_identity, .. } => match agent_identity {
                Some(grant) if grant.is_administrator => Ok(()),
                _ => Err(denied(
                    "An unattended run of an admin-scoped agent needs an Agent Identity \
                     that holds the Administrator role; it never borrows an Administrator's rights",
                )),
            },
        }
    }
}

fn denied(message: &str) -> AppError {
    AppError::classified(ErrorClass::Policy, "policy.access_denied", message)
}

impl EffectiveIdentity {
    /// Who a tool call or audit entry acted as (WP-0109 AC-1).
    pub fn audit_json(&self) -> Value {
        match self {
            Self::Interactive {
                user_id,
                agent_identity,
            } => json!({
                "type": "human",
                "user_id": user_id,
                "agent_identity_id": agent_identity.as_ref().map(|g| &g.id),
            }),
            Self::Unattended {
                agent_identity,
                record_actor,
            } => json!({
                "type": "agent_identity",
                "agent_identity_id": agent_identity.as_ref().map(|g| &g.id),
                "record_actor": record_actor,
            }),
        }
    }

    /// Makes this the identity of the run executing on this thread until
    /// the guard drops. A run (and everything it delegates to) executes on
    /// one thread: the database connection cannot leave it.
    pub fn enter_run(&self) -> RunIdentityGuard {
        RUN_IDENTITY.with(|stack| stack.borrow_mut().push(self.clone()));
        RunIdentityGuard
    }
}

thread_local! {
    static RUN_IDENTITY: RefCell<Vec<EffectiveIdentity>> = const { RefCell::new(Vec::new()) };
}

pub struct RunIdentityGuard;

impl Drop for RunIdentityGuard {
    fn drop(&mut self) {
        RUN_IDENTITY.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// The identity of the run executing on this thread, if any.
pub fn current_run_identity() -> Option<EffectiveIdentity> {
    RUN_IDENTITY.with(|stack| stack.borrow().last().cloned())
}

/// The effective principal for an audit entry: the run's identity, or for
/// a call outside any run, the person making it.
pub fn audit_principal(actor: Option<&str>) -> Value {
    current_run_identity()
        .unwrap_or_else(|| EffectiveIdentity::from_actor(actor))
        .audit_json()
}
