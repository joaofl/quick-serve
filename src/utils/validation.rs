use std::net::SocketAddr;
use std::path::PathBuf;
use crate::common::QuickServeError;

pub fn validate_ip_port(ip: &str, port: u16) -> Result<(), QuickServeError> {
    // Check for empty or invalid IP
    if ip.trim().is_empty() {
        return Err(QuickServeError::validation("IP address cannot be empty"));
    }

    // Check for reserved/private IP ranges (optional security check)
    if ip == "0.0.0.0" {
        return Err(QuickServeError::validation("Binding to 0.0.0.0 is not allowed for security reasons"));
    }

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

/// Validate file path for security (prevent path traversal)
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
}
