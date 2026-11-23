//! Structured command errors with stable codes and safe messages.
//!
//! The frontend branches on `code`, never on the message text. Messages are
//! written to be safe to display: they never carry a stack trace, a file
//! path outside the app, or a credential.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database unavailable")]
    Database(#[source] rusqlite::Error),
    #[error("record not found")]
    NotFound(&'static str),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("state conflict: {0}")]
    Conflict(String),
    #[error("internal error")]
    Internal(&'static str),
}

impl AppError {
    /// The stable machine-readable code. This is the only part the UI may
    /// branch on; the message is for people.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "db_error",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidInput(_) => "invalid_input",
            AppError::Conflict(_) => "conflict",
            AppError::Internal(_) => "internal_error",
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        AppError::Database(value)
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Validate a short identifier supplied by the frontend.
pub fn validate_id(kind: &str, value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(format!("{kind} must not be empty")));
    }
    if trimmed.len() > 128 {
        return Err(AppError::InvalidInput(format!(
            "{kind} must be at most 128 characters"
        )));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(AppError::InvalidInput(format!(
            "{kind} may only contain letters, digits, '-', '_' and '.'"
        )));
    }
    Ok(trimmed.to_string())
}

/// Validate a free-text field with an explicit maximum length.
pub fn validate_text(kind: &str, value: &str, max: usize) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.chars().count() > max {
        return Err(AppError::InvalidInput(format!(
            "{kind} must be at most {max} characters"
        )));
    }
    Ok(trimmed.to_string())
}
