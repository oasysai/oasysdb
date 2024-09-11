use std::fmt::{Display, Formatter, Result as FormatResult};

// Import external error types below.
use std::error::Error as StandardError;

#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidParameter,
}

/// OasysDB native error type.
///
/// Fields:
/// - code: High-level error status code.
/// - message: Brief description of the error.
/// - action: Suggested action to resolve the error.
#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    message: Box<str>,
    action: Option<Box<str>>,
}

impl Error {
    /// Create an error with the given code and message.
    pub fn new(code: ErrorCode, message: impl Into<Box<str>>) -> Self {
        Self { code, message: message.into(), action: None }
    }

    /// Add a suggested action to an error instance.
    /// - action: Brief description to resolve the error.
    pub fn with_action(mut self, action: impl Into<Box<str>>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Return the error code.
    pub fn code(&self) -> &ErrorCode {
        &self.code
    }

    /// Return the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Return the suggested action if available.
    pub fn action(&self) -> Option<&str> {
        self.action.as_deref()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        write!(f, "{:?}: {}", self.code, self.message)?;
        if let Some(action) = &self.action {
            write!(f, "\nSuggestion: {action}")?;
        }

        Ok(())
    }
}

// Implement interoperability with external error types.

impl StandardError for Error {}
