use super::*;

// Other error types.
use bincode::ErrorKind as BincodeError;
use pyo3::exceptions::PyValueError;
use sled::Error as SledError;
use std::error::Error as StandardError;
use std::io::Error as IOError;

/// A custom error type containing the error message.
#[derive(Debug)]
pub struct Error(String);

impl Error {
    /// Create a new error with the given message.
    pub fn new(message: &str) -> Self {
        message.into()
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.0
    }

    // Common collection errors.

    /// Creates error: The collection is not found.
    pub fn collection_not_found() -> Self {
        let message = "The collection is not found.";
        message.into()
    }

    /// Creates error when the collection record limit is reached.
    pub fn collection_limit() -> Self {
        let max = u32::MAX;
        let brief = "The collection limit is reached.";
        let detail = format!("The max number of records is {max}.");
        let message = format!("{brief} {detail}");
        message.into()
    }

    // Common record errors.

    /// Creates error when vector record is not found.
    pub fn record_not_found() -> Self {
        let message = "The vector record is not found.";
        message.into()
    }

    /// Creates error when getting vector with invalid dimension.
    pub fn invalid_dimension(found: usize, expected: usize) -> Self {
        let brief = "Invalid vector dimension.";
        let detail = format!("Expected {expected}, found {found}.");
        let message = format!("{brief} {detail}");
        message.into()
    }
}

// Quality of life conversions to Error type.

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error(err.to_string())
    }
}

// Interoperability with other error types.

impl From<Box<dyn StandardError>> for Error {
    fn from(err: Box<dyn StandardError>) -> Self {
        Error(err.to_string())
    }
}

impl From<SledError> for Error {
    fn from(err: SledError) -> Self {
        Error(err.to_string())
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error(err.to_string())
    }
}

impl From<Box<BincodeError>> for Error {
    fn from(err: Box<BincodeError>) -> Self {
        Error(err.to_string())
    }
}

impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        PyErr::new::<PyValueError, String>(err.0)
    }
}
