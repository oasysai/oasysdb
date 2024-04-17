// Other error types.
use bincode::ErrorKind as BincodeError;
use sled::Error as SledError;
use std::error::Error as StandardError;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IOError;

#[cfg(feature = "py")]
use super::*;

#[cfg(feature = "py")]
use pyo3::exceptions::PyValueError;

/// The type of error.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    StandardError,
    IOError,
    DatabaseError,
    CollectionError,
    DistanceError,
    SerializationError,
}

/// A custom error object with error type and message.
#[derive(Debug)]
pub struct Error {
    /// Type of error.
    pub kind: ErrorKind,
    /// Why the error occurred.
    pub message: String,
}

impl Error {
    /// Create a new error with the given message.
    pub fn new(kind: &ErrorKind, message: &str) -> Self {
        Self { kind: *kind, message: message.to_string() }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    // Common errors.

    /// Creates error: The collection is not found.
    pub fn collection_not_found() -> Self {
        let message = "The collection is not found.";
        let kind = ErrorKind::DatabaseError;
        Error::new(&kind, message)
    }

    /// Creates error when the collection record limit is reached.
    pub fn collection_limit() -> Self {
        let max = u32::MAX;
        let brief = "The collection limit is reached.";
        let detail = format!("The max number of records is {max}.");

        let message = format!("{brief} {detail}");
        let kind = ErrorKind::CollectionError;
        Error::new(&kind, &message)
    }

    /// Creates error when vector record is not found.
    pub fn record_not_found() -> Self {
        let message = "The vector record is not found.";
        let kind = ErrorKind::CollectionError;
        Error::new(&kind, message)
    }

    /// Creates error when getting vector with invalid dimension.
    pub fn invalid_dimension(found: usize, expected: usize) -> Self {
        let brief = "Invalid vector dimension.";
        let detail = format!("Expected {expected}, found {found}.");

        let message = format!("{brief} {detail}");
        let kind = ErrorKind::CollectionError;
        Error::new(&kind, &message)
    }

    /// Error when the distance function is not supported.
    pub fn invalid_distance() -> Self {
        let message = "Distance function not supported.";
        let kind = ErrorKind::DistanceError;
        Error::new(&kind, message)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let kind = &self.kind;
        let message = &self.message;
        write!(f, "{kind:?}: {message}")
    }
}

// Interoperability with other error types.

impl StandardError for Error {}

impl From<Box<dyn StandardError>> for Error {
    fn from(err: Box<dyn StandardError>) -> Self {
        let kind = ErrorKind::StandardError;
        Error::new(&kind, &err.to_string())
    }
}

impl From<SledError> for Error {
    fn from(err: SledError) -> Self {
        let kind = ErrorKind::DatabaseError;
        Error::new(&kind, &err.to_string())
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        let kind = ErrorKind::IOError;
        Error::new(&kind, &err.to_string())
    }
}

impl From<Box<BincodeError>> for Error {
    fn from(err: Box<BincodeError>) -> Self {
        let kind = ErrorKind::SerializationError;
        Error::new(&kind, &err.to_string())
    }
}

#[cfg(feature = "py")]
impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        PyErr::new::<PyValueError, String>(err.message)
    }
}
