use std::fmt::{Display, Formatter, Result};

// Other error types.
use std::error::Error as StandardError;
use std::sync::PoisonError;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorCode {
    StandardError,
    ConcurrencyError,
}

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}

impl Error {
    pub fn new(code: &ErrorCode, message: &str) -> Self {
        Self { code: *code, message: message.to_string() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let code = &self.code;
        let message = &self.message;
        write!(f, "{code:?}: {message}")
    }
}

// Implement other interoperability to other error types.

impl StandardError for Error {}

impl From<Box<dyn StandardError>> for Error {
    fn from(err: Box<dyn StandardError>) -> Self {
        let code = ErrorCode::StandardError;
        Error::new(&code, &err.to_string())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        let code = ErrorCode::ConcurrencyError;
        Error::new(&code, &err.to_string())
    }
}
