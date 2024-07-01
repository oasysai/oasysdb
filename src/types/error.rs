use std::fmt::{Display, Formatter, Result};

// Other error types.
use arrow::error::ArrowError;
use bincode::ErrorKind as BincodeError;
use std::error::Error as StandardError;
use std::io::Error as IOError;
use std::sync::PoisonError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ArrowError,
    ConcurrencyError,
    FileError,
    SerializationError,
    StandardError,

    // Tonic-related error codes.
    ClientError,
    NotFoundError,
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

// Implement interoperability FROM other external error types.

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

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        let code = ErrorCode::FileError;
        Error::new(&code, &err.to_string())
    }
}

impl From<ArrowError> for Error {
    fn from(err: ArrowError) -> Self {
        let code = ErrorCode::ArrowError;
        Error::new(&code, &err.to_string())
    }
}

impl From<Box<BincodeError>> for Error {
    fn from(err: Box<BincodeError>) -> Self {
        let code = ErrorCode::SerializationError;
        Error::new(&code, &err.to_string())
    }
}

// Implement interoperability INTO other external error types.

impl From<Error> for tonic::Status {
    fn from(err: Error) -> Self {
        let code = match err.code {
            ErrorCode::ClientError => tonic::Code::InvalidArgument,
            ErrorCode::NotFoundError => tonic::Code::NotFound,
            _ => tonic::Code::Internal,
        };

        tonic::Status::new(code, err.message)
    }
}
