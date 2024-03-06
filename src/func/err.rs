use super::*;

// Other error types.
use bincode::ErrorKind as BincodeError;
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
        let m = "The collection is not found.";
        m.into()
    }

    /// Creates error: The collection limit is reached.
    pub fn collection_limit() -> Self {
        let m = "The collection limit is reached.";
        m.into()
    }

    // Common record errors.

    /// Creates error: The record is not found.
    pub fn record_not_found() -> Self {
        let m = "The record is not found.";
        m.into()
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
        PyErr::new::<PyAny, String>(err.0)
    }
}
