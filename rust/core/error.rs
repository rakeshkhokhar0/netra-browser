use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Represents typed failures emitted by the Rust browser core.
#[derive(Clone, Debug)]
pub enum NetraError {
    /// Input validation failed before the requested operation could begin.
    InvalidInput(String),
    /// A requested resource or tab could not be found.
    NotFound(String),
    /// A generic operation failure occurred.
    OperationFailed(String),
}

impl Display for NetraError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::NotFound(message) => write!(f, "not found: {message}"),
            Self::OperationFailed(message) => write!(f, "operation failed: {message}"),
        }
    }
}

impl Error for NetraError {}
