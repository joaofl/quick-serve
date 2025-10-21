use log::{info, debug, error};

use super::{Protocol, Server};

// Create the TFTP server.
use async_tftp::server::TftpServerBuilder;
use std::{net::IpAddr, ops::Deref, path::PathBuf, str::FromStr};
use crate::utils::validation;
use std::sync::Arc;


pub trait TFTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Result<Self, crate::QuickServeError> where Self: Sized;
    fn runner(&self);
}

impl TFTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Result<Self, crate::QuickServeError> {
        let mut s = Server::default();
        
        // Validate inputs with proper error handling
        validation::validate_path(&path)?;
        validation::validate_ip_port(&bind_ip, port)?;
        
        let path = validation::ensure_trailing_slash(&path);
        s.path = Arc::new(path);
        s.bind_address = IpAddr::from_str(&bind_ip)
            .map_err(|e| crate::QuickServeError::validation(format!("Invalid IP address '{}': {}", bind_ip, e)))?;
        s.port = port;

        s.protocol = Protocol::Tftp;
        TFTPRunner::runner(&s);
        Ok(s)
    }
    fn runner(&self) {
        let mut receiver = self.sender.subscribe();
        
        let bind_address = self.bind_address.clone();
        let port = self.port;
        let path = self.path.clone();

        tokio::spawn(async move {
            loop {
                // Get notified about the server's spawned task
                let m = match receiver.recv().await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to receive message in TFTP runner: {}", e);
                        break;
                    }
                };

                if m.connect {
                    info!("Starting TFTP server on {}:{}", bind_address, port);
                    let tsk = tokio::spawn(async move {
                        let addr = format!("{}:{}", bind_address, port);
                        
                        // Build TFTP server with proper error handling
                        let tftpd_result = TftpServerBuilder::with_dir_ro(path.deref())
                            .map_err(|e| format!("Failed to create TFTP server: {}", e))
                            .and_then(|builder| {
                                addr.parse()
                                    .map_err(|e| format!("Invalid address '{}': {}", addr, e))
                                    .map(|parsed_addr| builder.bind(parsed_addr))
                            });

                        match tftpd_result {
                            Ok(builder) => {
                                match builder.build().await {
                                    Ok(tftpd) => {
                                        info!("TFTP server listening on {}", addr);
                                        if let Err(e) = tftpd.serve().await {
                                            error!("TFTP server error: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to build TFTP server: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to create TFTP server: {}", e);
                            }
                        }
                    });

                    // Wait for stop command
                    match receiver.recv().await {
                        Ok(_) => {
                            info!("Stop command received, shutting down TFTP server");
                            tsk.abort();
                            debug!("TFTP server stopped");
                            break;
                        }
                        Err(e) => {
                            error!("Failed to receive stop command: {}", e);
                            tsk.abort();
                            break;
                        }
                    }
                }
            }
        });
    }
}


#[cfg(test)]
mod tests {
    use crate::tests::common::tests::*;
    use crate::servers::Protocol;

    #[test]
    fn test_tftp_file_download_success() {
        let proto = Protocol::Tftp;
        let port = 6966u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-tftp.bin";
        let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get {} {}", port, file_in, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    #[test]
    fn test_file_not_found() {
        let proto = Protocol::Tftp;
        let port = 6967u16;
        let file_in = "data.bin";
        let nonexistent_file = "nonexistent.bin";
        let file_out = "/tmp/data-out-tftp-404.bin";
        let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get {} {} 2>&1 || true", 
            port, nonexistent_file, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_err(), "Expected failure for non-existent file");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("does not exist") || err_msg.contains("empty"), 
            "Expected file not found error, got: {}", err_msg);
    }

    #[test]
    fn test_path_is_directory() {
        let proto = Protocol::Tftp;
        let port = 6968u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-tftp-dir.bin";
        // TFTP will try to get an empty path which should fail
        let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get '' {} 2>&1 || true", 
            port, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_err(), "Expected failure for directory path");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("does not exist") || err_msg.contains("empty"), 
            "Expected file not found or empty file error, got: {}", err_msg);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let proto = Protocol::Tftp;
        let port = 6969u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-tftp-traversal.bin";
        let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get ../../etc/passwd {} 2>&1 || true", 
            port, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_err(), "Expected failure for path traversal attempt");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("does not exist") || err_msg.contains("empty"), 
            "Expected file not found or empty file error, got: {}", err_msg);
    }
}
