use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("{0} not found")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    /// A tool call denied or queued for approval by
    /// `policy_engine_service::evaluate` (the Tool-Call Firewall) -
    /// distinct from a plain `Validation` error so a run's own
    /// `policy_violations_count` (AI Agent Platform v2, Phase 6a) can be
    /// counted deterministically, not inferred by matching on error text.
    #[error("{0}")]
    PolicyBlocked(String),
}

// Tauri command errors must be Serialize; we expose them to the frontend as
// a plain message string plus a machine-readable kind for status handling.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let kind = match self {
            AppError::Database(_) => "database",
            AppError::NotFound(_) => "not_found",
            AppError::Validation(_) => "validation",
            AppError::Conflict(_) => "conflict",
            AppError::PolicyBlocked(_) => "policy_blocked",
        };
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
