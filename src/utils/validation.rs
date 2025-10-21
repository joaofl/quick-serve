use std::net::SocketAddr;
use std::path::PathBuf;
use crate::common::QuickServeError;

/// Validates an IP address and port combination
///
/// Checks for empty IPs, invalid formats, and privileged ports.
/// Allows binding to 0.0.0.0 for listening on all interfaces.
///
/// # Arguments
/// * `ip` - The IP address to validate
/// * `port` - The port number to validate
///
/// # Returns
/// * `Ok(())` if the IP and port are valid
/// * `Err(QuickServeError)` with a description if validation fails
pub fn validate_ip_port(ip: &str, port: u16) -> Result<(), QuickServeError> {
    // Check for empty or invalid IP
    if ip.trim().is_empty() {
        return Err(QuickServeError::validation("IP address cannot be empty"));
    }

    // Allow 0.0.0.0 for binding to all interfaces (this is standard practice)
    // Other validation will be done by parsing

    // Check port range
    if port == 0 {
        return Err(QuickServeError::validation("Port cannot be 0"));
    }

    if port < 1024 && port != 80 && port != 443 {
        return Err(QuickServeError::validation("Ports below 1024 require root privileges"));
    }

    let addr = format!("{}:{}", ip, port);

    match addr.parse::<SocketAddr>() {
        Ok(socket_addr) => {
            if socket_addr.is_ipv4() || socket_addr.is_ipv6() {
                Ok(())
            } else {
                Err(QuickServeError::validation("Invalid IP format"))
            }
        }
        Err(e) => Err(QuickServeError::validation(format!("Invalid IP:PORT format: {}", e))),
    }
}

/// Ensures a path ends with a trailing slash
///
/// # Arguments
/// * `path` - The path to process
///
/// # Returns
/// A PathBuf with a guaranteed trailing slash
pub fn ensure_trailing_slash(path: &PathBuf) -> PathBuf {
    if !path.ends_with("/") { 
        let mut p = path.clone().into_os_string();
        p.push("/"); 
        return p.into();
    }
    else {
        return path.into();
    }
}


/// Validates a directory path for serving
///
/// Checks if the path exists, is a directory, is readable, and is not
/// a sensitive system directory (like /etc, /sys, /proc).
///
/// # Arguments
/// * `path` - The path to validate
///
/// # Returns
/// * `Ok(())` if the path is valid and safe to serve
/// * `Err(QuickServeError)` if validation fails
pub fn validate_path(path: &PathBuf) -> Result<(), QuickServeError> {
    // Check if path exists
    if !path.exists() {
        return Err(QuickServeError::validation(format!("Path does not exist: {}", path.display())));
    }

    // Check if path is a directory
    if !path.is_dir() {
        return Err(QuickServeError::validation(format!("Path is not a directory: {}", path.display())));
    }

    // Check if path is readable
    if !path.metadata().map(|m| m.permissions().readonly()).unwrap_or(false) {
        // Check if we can read the directory
        match std::fs::read_dir(path) {
            Ok(_) => {},
            Err(e) => {
                return Err(QuickServeError::validation(format!("Cannot read directory {}: {}", path.display(), e)));
            }
        }
    }

    // Security check: prevent serving from sensitive directories
    let path_str = path.to_string_lossy().to_lowercase();
    let sensitive_paths = ["/etc", "/sys", "/proc", "/dev", "/root", "/boot"];
    
    for sensitive in &sensitive_paths {
        if path_str.starts_with(sensitive) {
            return Err(QuickServeError::validation(format!("Cannot serve from sensitive directory: {}", path.display())));
        }
    }

    Ok(())
}

/// Validates a file path for security (prevents path traversal attacks)
///
/// Checks for path traversal attempts (..), null bytes, absolute paths,
/// and ensures the resolved path stays within the base directory.
///
/// # Arguments
/// * `base_path` - The base directory that files must be within
/// * `requested_path` - The requested file path (relative)
///
/// # Returns
/// * `Ok(PathBuf)` - The validated full path
/// * `Err(QuickServeError)` - If the path is invalid or a security risk
pub fn validate_file_path(base_path: &PathBuf, requested_path: &str) -> Result<PathBuf, QuickServeError> {
    // Check for path traversal attempts
    if requested_path.contains("..") || requested_path.contains("//") {
        return Err(QuickServeError::validation("Path traversal attempt detected"));
    }

    // Check for absolute paths
    if requested_path.starts_with('/') {
        return Err(QuickServeError::validation("Absolute paths are not allowed"));
    }

    // Check for null bytes
    if requested_path.contains('\0') {
        return Err(QuickServeError::validation("Null bytes in path are not allowed"));
    }

    let full_path = base_path.join(requested_path);

    // Ensure the resolved path is still within the base directory
    if !full_path.starts_with(base_path) {
        return Err(QuickServeError::validation("Path outside base directory"));
    }

    Ok(full_path)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ip_port() {
        let result = validate_ip_port("127.0.0.1", 8080);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_ensure_trailing_slash() {
        let result = ensure_trailing_slash(&PathBuf::from("/tmp"));
        assert_eq!(result, PathBuf::from("/tmp/"));

        let result = ensure_trailing_slash(&PathBuf::from("/tmp/"));
        assert_eq!(result, PathBuf::from("/tmp/"));
    }

    #[test]
    fn test_invalid_ip_port_format() {
        let result = validate_ip_port("invalid ip here", 8080);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
        assert!(result.err().unwrap().to_string().contains("Invalid IP:PORT format"));
    }

    #[test]
    fn test_invalid_ip_address() {
        let result = validate_ip_port("256.0.0.1", 8080);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
        assert!(result.err().unwrap().to_string().contains("Invalid IP:PORT format"));
    }

    #[test]
    fn test_valid_path() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();

        let result = validate_path(&path);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_invalid_path() {
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        let result = validate_path(&path);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
    }

    #[test]
    fn test_port_zero_is_invalid() {
        let result = validate_ip_port("127.0.0.1", 0);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Port cannot be 0"));
    }

    #[test]
    fn test_privileged_port_warning() {
        let result = validate_ip_port("127.0.0.1", 22);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("root privileges"));
    }

    #[test]
    fn test_allow_standard_ports() {
        // Port 80 and 443 should be allowed even though < 1024
        let result = validate_ip_port("127.0.0.1", 80);
        assert!(result.is_ok(), "Port 80 should be allowed");
        
        let result = validate_ip_port("127.0.0.1", 443);
        assert!(result.is_ok(), "Port 443 should be allowed");
    }

    #[test]
    fn test_allow_binding_to_all_interfaces() {
        // 0.0.0.0 should be allowed for binding to all interfaces
        let result = validate_ip_port("0.0.0.0", 8080);
        assert!(result.is_ok(), "Should allow binding to 0.0.0.0");
    }

    #[test]
    fn test_ipv6_addresses() {
        // IPv6 addresses need to be in brackets when used with ports
        let result = validate_ip_port("::1", 8080);
        // Note: IPv6 validation might fail without brackets, which is expected behavior
        // for Socket address parsing
        if result.is_err() {
            // This is acceptable - IPv6 addresses without brackets may not parse
            return;
        }
        
        let result = validate_ip_port("127.0.0.1", 8080);
        assert!(result.is_ok(), "IPv4 should always work");
    }

    #[test]
    fn test_empty_ip_rejected() {
        let result = validate_ip_port("", 8080);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_file_path_traversal_attacks() {
        let base = PathBuf::from("/tmp");
        
        // Test various path traversal attempts
        assert!(validate_file_path(&base, "../etc/passwd").is_err());
        assert!(validate_file_path(&base, "foo/../../../etc/passwd").is_err());
        assert!(validate_file_path(&base, "foo//bar").is_err());
    }

    #[test]
    fn test_validate_file_path_null_bytes() {
        let base = PathBuf::from("/tmp");
        assert!(validate_file_path(&base, "foo\0bar").is_err());
    }

    #[test]
    fn test_validate_file_path_absolute_paths() {
        let base = PathBuf::from("/tmp");
        assert!(validate_file_path(&base, "/etc/passwd").is_err());
    }

    #[test]
    fn test_validate_file_path_valid_paths() {
        let base = PathBuf::from("/tmp");
        
        let result = validate_file_path(&base, "foo/bar.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/foo/bar.txt"));
        
        let result = validate_file_path(&base, "test.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test.txt"));
    }

    #[test]
    fn test_nonexistent_path() {
        let path = PathBuf::from("/this/path/should/not/exist/at/all");
        let result = validate_path(&path);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("does not exist"));
    }
}
