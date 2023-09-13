use std::net::SocketAddr;
use std::path::PathBuf;


pub fn validate_ip_port(address: &str) -> Result<(), String> {
    match address.parse::<SocketAddr>() {
        Ok(socket_addr) => {
            if socket_addr.is_ipv4() || socket_addr.is_ipv6() {
                Ok(())
            } else {
                Err("Invalid IP address format".to_string())
            }
        }
        Err(_) => Err("Invalid IP:PORT format".to_string()),
    }
}


pub fn validate_path(path: &PathBuf) -> Result<(), String> {
    if path.exists() && path.is_dir() {
        return Ok(());
    }
    else {
        return Err("Path does not point to valid directory".to_string());
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ip_port() {
        let valid_address = "127.0.0.1:8080";
        let result = validate_ip_port(valid_address);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_invalid_ip_port_format() {
        let invalid_address = "not_an_ip_address";
        let result = validate_ip_port(invalid_address);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
        assert_eq!(result.err(), Some("Invalid IP:PORT format".to_string()));
    }

    #[test]
    fn test_invalid_ip_address() {
        let invalid_address = "256.0.0.1:8080"; // Invalid IP address
        let result = validate_ip_port(invalid_address);
        assert!(result.is_err(), "Expected Err, got {:?}", result);
        assert_eq!(result.err(), Some("Invalid IP:PORT format".to_string()));
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