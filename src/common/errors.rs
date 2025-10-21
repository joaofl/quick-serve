use std::fmt;
use std::io;

/// Custom error types for the quick-serve application
///
/// Provides specific error categories for better error handling and debugging.
#[derive(Debug)]
pub enum QuickServeError {
    /// Network-related errors (binding, connections, etc.)
    Network(String),
    /// Validation errors (invalid paths, IPs, ports, etc.)
    Validation(String),
    /// Server lifecycle errors (start/stop failures)
    ServerLifecycle(String),
    /// IO errors (file operations, etc.)
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
    /// Creates a validation error
    ///
    /// # Arguments
    /// * `msg` - The error message
    pub fn validation(msg: impl Into<String>) -> Self {
        QuickServeError::Validation(msg.into())
    }

    /// Creates a server lifecycle error
    ///
    /// # Arguments
    /// * `msg` - The error message
    pub fn server_lifecycle(msg: impl Into<String>) -> Self {
        QuickServeError::ServerLifecycle(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_formatting() {
        let err = QuickServeError::validation("Test validation error");
        assert_eq!(err.to_string(), "Validation error: Test validation error");

        let err = QuickServeError::server_lifecycle("Test lifecycle error");
        assert_eq!(err.to_string(), "Server lifecycle error: Test lifecycle error");

        let err = QuickServeError::Network("Test network error".to_string());
        assert_eq!(err.to_string(), "Network error: Test network error");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let quick_err: QuickServeError = io_err.into();
        
        match quick_err {
            QuickServeError::Io(_) => (),
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_addr_parse_error_conversion() {
        let parse_err = "invalid_ip:8080".parse::<std::net::SocketAddr>().unwrap_err();
        let quick_err: QuickServeError = parse_err.into();
        
        match quick_err {
            QuickServeError::Network(_) => (),
            _ => panic!("Expected Network variant"),
        }
    }

    #[test]
    fn test_error_helper_functions() {
        let err = QuickServeError::validation("test");
        assert!(matches!(err, QuickServeError::Validation(_)));

        let err = QuickServeError::server_lifecycle("test");
        assert!(matches!(err, QuickServeError::ServerLifecycle(_)));
    }
}
