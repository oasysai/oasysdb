use std::fmt::{Display, Formatter, Result};

// External error types.
use bincode::Error as BincodeError;
use serde_json::Error as JSONError;
use sqlx::Error as SQLError;
use std::error::Error as StandardError;
use std::io::Error as IOError;
use std::sync::PoisonError;

#[allow(missing_docs)]
#[derive(Debug, Eq, PartialEq)]
pub enum ErrorCode {
    // Data source related.
    InvalidSource,
    MissingSource,

    // Data type related.
    InvalidID,
    InvalidMetadata,
    InvalidVector,
    InvalidParameter,

    // Other generic errors.
    InternalError,
    RequestError,
    NotFound,

    // External error types.
    ConcurrencyError,
    FileError,
    SerializationError,
    SQLError,
}

/// The native error type for OasysDB operations.
#[derive(Debug)]
pub struct Error {
    /// Represents cause or source of the error.
    pub code: ErrorCode,
    /// Details about the error and why it occurred.
    pub message: String,
}

impl Error {
    /// Creates a new error instance.
    /// - `code`: Error code.
    /// - `message`: Details why the error occurred.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

// Implement interoperability with other error types.

impl StandardError for Error {}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        let code = ErrorCode::FileError;
        Error::new(code, err.to_string())
    }
}

impl From<BincodeError> for Error {
    fn from(err: BincodeError) -> Self {
        let code = ErrorCode::SerializationError;
        Error::new(code, err.to_string())
    }
}

impl From<SQLError> for Error {
    fn from(err: SQLError) -> Self {
        let code = ErrorCode::SQLError;
        Error::new(code, err.to_string())
    }
}

impl From<JSONError> for Error {
    fn from(err: JSONError) -> Self {
        let code = ErrorCode::SerializationError;
        Error::new(code, err.to_string())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        let code = ErrorCode::ConcurrencyError;
        Error::new(code, err.to_string())
    }
}
