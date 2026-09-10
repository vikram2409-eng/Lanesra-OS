//! AI & Agentic Layer, Phase 4: Agent Actions. Natural-language
//! reporting is the only action built this pass (see
//! `services::agent_service`'s own doc comment for why, and what's
//! deliberately deferred) - room is left here for the other three
//! (meeting-prep briefing, commitment capture, record-hygiene
//! suggestions) to add their own request/result types alongside these
//! once each is its own scoped phase.

use serde::{Deserialize, Serialize};

use crate::models::custom_report::{CustomReportInput, CustomReportRow};

#[derive(Debug, Clone, Deserialize)]
pub struct NlReportQuery {
    pub question: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NlReportResult {
    /// The report shape the model resolved the question into - shown to
    /// the person asking so they can see what it understood, and reused
    /// as-is if they choose to save it as a real `CustomReport`.
    pub report: CustomReportInput,
    pub rows: Vec<CustomReportRow>,
}
