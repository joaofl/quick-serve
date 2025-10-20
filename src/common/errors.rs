use std::fmt;
use std::io;

/// Custom error types for the quick-serve application
#[derive(Debug)]
pub enum QuickServeError {
    /// Network-related errors
    Network(String),
    /// Validation errors
    Validation(String),
    /// Server lifecycle errors
    ServerLifecycle(String),
    /// IO errors
    Io(io::Error),
}

impl fmt::Display for QuickServeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuickServeError::Network(msg) => write!(f, "Network error: {}", msg),
            QuickServeError::Validation(msg) => write!(f, "Validation error: {}", msg),
            QuickServeError::ServerLifecycle(msg) => write!(f, "Server lifecycle error: {}", msg),
            QuickServeError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for QuickServeError {}

impl From<io::Error> for QuickServeError {
    fn from(err: io::Error) -> Self {
        QuickServeError::Io(err)
    }
}

impl From<std::net::AddrParseError> for QuickServeError {
    fn from(err: std::net::AddrParseError) -> Self {
        QuickServeError::Network(format!("Invalid address: {}", err))
    }
}

impl From<tokio::task::JoinError> for QuickServeError {
    fn from(err: tokio::task::JoinError) -> Self {
        QuickServeError::ServerLifecycle(format!("Task join error: {}", err))
    }
}

/// Result type alias for quick-serve operations
pub type QuickServeResult<T> = Result<T, QuickServeError>;

/// Helper functions for creating specific error types
impl QuickServeError {
    pub fn validation(msg: impl Into<String>) -> Self {
        QuickServeError::Validation(msg.into())
    }

    pub fn server_lifecycle(msg: impl Into<String>) -> Self {
        QuickServeError::ServerLifecycle(msg.into())
    }
}
