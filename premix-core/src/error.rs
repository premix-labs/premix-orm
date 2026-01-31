use crate::model::ValidationError;

/// Premix-specific error type with actionable variants.
#[derive(Debug)]
pub enum PremixError {
    /// Underlying sqlx error.
    Sqlx(sqlx::Error),
    /// Optimistic locking conflict.
    VersionConflict,
    /// Validation failed.
    Validation(Vec<ValidationError>),
    /// Generic message error.
    Message(String),
}

impl std::fmt::Display for PremixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlx(err) => write!(f, "sqlx error: {}", err),
            Self::VersionConflict => write!(f, "version conflict"),
            Self::Validation(errors) => write!(f, "validation failed ({} errors)", errors.len()),
            Self::Message(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for PremixError {}

impl From<sqlx::Error> for PremixError {
    fn from(err: sqlx::Error) -> Self {
        map_sqlx_error(err)
    }
}

impl From<Vec<ValidationError>> for PremixError {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self::Validation(errors)
    }
}

/// Result alias for Premix operations.
pub type PremixResult<T> = Result<T, PremixError>;

/// Convert sqlx errors to actionable Premix errors when possible.
pub fn map_sqlx_error(err: sqlx::Error) -> PremixError {
    if let sqlx::Error::Protocol(message) = &err {
        let message = message.to_ascii_lowercase();
        if message.contains("premix save failed: version conflict") {
            return PremixError::VersionConflict;
        }
    }
    PremixError::Sqlx(err)
}
