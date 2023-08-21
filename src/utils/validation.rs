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