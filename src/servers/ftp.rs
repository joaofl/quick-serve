use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use log::{debug, info, error};
use unftp_sbe_fs::ServerExt;
use std::time::Duration;
use super::Server;
use crate::servers::Protocol;
use crate::utils::validation;
use std::sync::Arc;


pub trait FTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Result<Self, crate::QuickServeError> where Self: Sized;
    fn runner(&self);
}

impl FTPRunner for Server {
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

        s.protocol = Protocol::Ftp;
        FTPRunner::runner(&s);
        Ok(s)
    }

    fn runner(&self) {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address;
        let port = self.port;
        let path = self.path.to_string_lossy().to_string();

        tokio::spawn(async move {
            loop {
                debug!("FTP runner started... Waiting command to connect...");
                
                let m = match receiver.recv().await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to receive message in FTP runner: {}", e);
                        break;
                    }
                };
                debug!("Message received");

                if m.connect {
                    info!("Starting FTP server on {}:{}", bind_address, port);
                    
                    // Define new server with proper error handling
                    let server_result = libunftp::Server::with_fs(path.clone())
                        .passive_ports(50000..=65535)
                        .metrics()
                        .shutdown_indicator(async move {
                            loop {
                                info!("FTP server connected. Waiting command to disconnect...");
                                match receiver.recv().await {
                                    Ok(_) => break,
                                    Err(e) => {
                                        error!("Failed to receive stop command: {}", e);
                                        break;
                                    }
                                }
                            }
                            debug!("Gracefully terminating the FTP server");
                            // Give a few seconds to potential ongoing connections to finish, 
                            // otherwise finish immediately
                            libunftp::options::Shutdown::new().grace_period(Duration::from_secs(5))
                        })
                        .build();

                    match server_result {
                        Ok(server) => {
                            let listen_addr = format!("{}:{}", bind_address, port);
                            info!("FTP server listening on {}", listen_addr);
                            
                            if let Err(e) = server.listen(&listen_addr).await {
                                error!("Error starting the FTP server on {}: {}", listen_addr, e);
                            } else {
                                info!("FTP server stopped gracefully");
                            }
                        }
                        Err(e) => {
                            error!("Failed to build FTP server: {}", e);
                        }
                    }
                    break;
                }
            }
        });
    }
}


/////////////////////////////////////////////////////////////////////////////////////
//                                        TESTS                                    //
/////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use crate::tests::common::tests::*;
    use crate::servers::Protocol;

    #[test]
    fn test_ftp_file_download_success() {
        let proto = Protocol::Ftp;
        let port = 2223u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-ftp.bin";
        let dl_cmd = format!("curl --retry 2 --retry-delay 1 {}://127.0.0.1:{}/{} -o {}", 
            proto.to_string(), port, file_in, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    #[test]
    fn test_file_not_found() {
        let proto = Protocol::Ftp;
        let port = 2224u16;
        let file_in = "data.bin";
        let nonexistent_file = "nonexistent.bin";
        let file_out = "/tmp/data-out-ftp-404.bin";
        let dl_cmd = format!("curl --retry 1 --retry-delay 1 {}://127.0.0.1:{}/{} -o {} 2>&1 || true", 
            proto.to_string(), port, nonexistent_file, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_err(), "Expected failure for non-existent file");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("does not exist") || err_msg.contains("empty"), 
            "Expected file not found error, got: {}", err_msg);
    }

    #[test]
    fn test_path_is_directory() {
        // FTP protocol allows listing directories, unlike HTTP which returns errors
        // This test verifies that requesting a non-existent directory/file fails appropriately
        let proto = Protocol::Ftp;
        let port = 2225u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-ftp-dir.bin";
        
        // Request a non-existent subdirectory - FTP should fail to serve it
        let dl_cmd = format!("curl --retry 1 --retry-delay 1 {}://127.0.0.1:{}/nonexistent_subdir/file.txt -o {} 2>&1 || true", 
            proto.to_string(), port, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        // Accept any error - FTP may create a file with error content, or fail to create file
        assert!(result.is_err(), "Expected failure for non-existent directory path");
    }

    #[test]
    fn test_path_traversal_blocked() {
        let proto = Protocol::Ftp;
        let port = 2226u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-ftp-traversal.bin";
        let dl_cmd = format!("curl --retry 1 --retry-delay 1 {}://127.0.0.1:{}/../../etc/passwd -o {} 2>&1 || true", 
            proto.to_string(), port, file_out);

        let result = test_server_e2e(proto, port, dl_cmd, file_in, file_out);
        assert!(result.is_err(), "Expected failure for path traversal attempt");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("does not exist") || err_msg.contains("empty"), 
            "Expected file not found or empty file error, got: {}", err_msg);
    }
}
