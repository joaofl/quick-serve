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


