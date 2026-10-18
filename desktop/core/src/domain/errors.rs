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
    /// An error with an ADR-0009 class and a stable dotted code, e.g.
    /// `provider.timeout`. Retry and failover decide on `class`; callers
    /// match on `code`, never on the message text.
    #[error("{message}")]
    Classified {
        class: ErrorClass,
        code: &'static str,
        /// Safe to show a user; never contains a secret.
        message: String,
        /// From `Retry-After`, when the provider sent one.
        retry_after_ms: Option<u64>,
        /// The provider's HTTP status, when the error came from one.
        http_status: Option<u16>,
    },
}

/// The five error classes (ADR-0009). A new class needs a new ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Transient,
    Policy,
    Validation,
    Configuration,
    Internal,
}

impl ErrorClass {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorClass::Transient => "transient",
            ErrorClass::Policy => "policy",
            ErrorClass::Validation => "validation",
            ErrorClass::Configuration => "configuration",
            ErrorClass::Internal => "internal",
        }
    }
}

impl AppError {
    /// A classified error with no provider status or retry hint.
    pub fn classified(class: ErrorClass, code: &'static str, message: impl Into<String>) -> Self {
        AppError::Classified {
            class,
            code,
            message: message.into(),
            retry_after_ms: None,
            http_status: None,
        }
    }

    /// The ADR-0009 class. Existing variants map onto the classes as the
    /// ADR's table says.
    pub fn class(&self) -> ErrorClass {
        match self {
            AppError::NotFound(_) | AppError::Validation(_) | AppError::Conflict(_) => {
                ErrorClass::Validation
            }
            AppError::PolicyBlocked(_) => ErrorClass::Policy,
            AppError::Database(e) => match e.sqlite_error_code() {
                Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
                    ErrorClass::Transient
                }
                _ => ErrorClass::Internal,
            },
            AppError::Classified { class, .. } => *class,
        }
    }

    /// The stable code, for classified errors only.
    pub fn code(&self) -> Option<&'static str> {
        match self {
            AppError::Classified { code, .. } => Some(code),
            _ => None,
        }
    }
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
            AppError::Classified { class, .. } => class.as_str(),
        };
        let mut state = serializer.serialize_struct("AppError", 4)?;
        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("class", self.class().as_str())?;
        state.serialize_field("code", &self.code())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_variants_map_onto_the_adr_0009_classes() {
        // Covers: REQ-PRE-003 (WP-0005 AC-2; ADR-0009 §2 table)
        assert_eq!(
            AppError::NotFound("x".into()).class(),
            ErrorClass::Validation
        );
        assert_eq!(
            AppError::Validation("x".into()).class(),
            ErrorClass::Validation
        );
        assert_eq!(
            AppError::Conflict("x".into()).class(),
            ErrorClass::Validation
        );
        assert_eq!(
            AppError::PolicyBlocked("x".into()).class(),
            ErrorClass::Policy
        );
        let busy = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            None,
        );
        assert_eq!(AppError::Database(busy).class(), ErrorClass::Transient);
        let other = rusqlite::Error::QueryReturnedNoRows;
        assert_eq!(AppError::Database(other).class(), ErrorClass::Internal);
    }

    #[test]
    fn a_classified_error_serializes_its_class_and_code() {
        // Covers: REQ-PRE-003 (WP-0005 AC-2)
        let e = AppError::classified(ErrorClass::Transient, "provider.timeout", "took too long");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "transient");
        assert_eq!(v["class"], "transient");
        assert_eq!(v["code"], "provider.timeout");
        assert_eq!(v["message"], "took too long");
        let legacy = serde_json::to_value(AppError::Validation("bad".into())).unwrap();
        assert_eq!(legacy["kind"], "validation");
        assert_eq!(legacy["class"], "validation");
        assert!(legacy["code"].is_null());
    }
}
