use log::{info, debug, error};

use super::{Protocol, Server};

// Create the TFTP server.
use async_tftp::server::TftpServerBuilder;
use std::{net::IpAddr, ops::Deref, path::PathBuf, str::FromStr};
use crate::utils::validation;
use std::sync::Arc;


pub trait TFTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

impl TFTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();
        
        // Validate inputs with proper error handling
        if let Err(e) = validation::validate_path(&path) {
            error!("Invalid path '{}': {}", path.display(), e);
            panic!("Invalid path: {}", e);
        }
        
        if let Err(e) = validation::validate_ip_port(&bind_ip, port) {
            error!("Invalid bind IP '{}:{}': {}", bind_ip, port, e);
            panic!("Invalid bind IP: {}", e);
        }
        
        let path = validation::ensure_trailing_slash(&path);
        s.path = Arc::new(path);
        s.bind_address = match IpAddr::from_str(&bind_ip) {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to parse IP address '{}': {}", bind_ip, e);
                panic!("Invalid IP address: {}", e);
            }
        };
        s.port = port;

        s.protocol = Protocol::Tftp;
        TFTPRunner::runner(&s);
        s
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
    fn e2e() {
        let proto = Protocol::Tftp;
        let port = 6966u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-tftp.bin";
        let dl_cmd = format!("tftp 127.0.0.1 {} -m binary -c get {} {}", port, file_in, file_out);

        test_server_e2e(proto, port, dl_cmd, file_in, file_out);
    }
}
