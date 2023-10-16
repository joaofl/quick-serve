use log::{debug};

// use untftp_sbe_fs::ServerExt;

use std::{time::Duration, path::PathBuf};
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;

// Create the TFTP server.
use tftpd::{Config};

use std::net::Ipv4Addr;

#[async_trait]
pub trait TFTPServerRunner {
    fn new() -> Self;
    async fn runner(&self);
}

#[async_trait]
impl TFTPServerRunner for Server {
    fn new() -> Self {
        let mut s = Server::default();
        s.protocol = Protocol::Tftp;
        return s;
    }
    async fn runner(&self) {
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();
        
        loop {
            let msg = receiver.recv().await.unwrap();
            let mut receiver2 = self.sender.subscribe();

            if msg.terminate { return };
            if msg.connect {

                let mut config = Config {
                    ip_address: msg.bind_address.parse::<Ipv4Addr>().unwrap(),
                    port: msg.port,
                    directory: msg.path.clone(),
                    single_port: false,
                    read_only: false,
                    duplicate_packets: 1,
                    overwrite: false,
                };

                let mut server = tftpd::Server::new(&config).unwrap();

                println!(   
                    "Running TFTP Server on {}:{} in {}",
                    config.ip_address,
                    config.port,
                    config.directory.display()
                );

                tokio::spawn(async move {
                    server.listen();
                });

                loop {
                    let m = receiver.recv().await.unwrap();
                    if m.terminate { return };
                    if m.connect { continue } // Not for me. Go wait another msg
                    else { break }
                }
                // Kill the server spawn here
                debug!("Gracefully terminating the HTTP server");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Import necessary items for testing
    use super::*;
    use crate::tests::common;

    #[tokio::test]
    async fn test_tftp_server_e2e() {
        let s = <Server as TFTPServerRunner>::new();
        let r = common::test_server::e2e(s, 6969).await;

        assert_eq!(r.0, 0, "Server did not start");
        assert_ne!(r.1, 0, "Server did not stop");
        assert_eq!(r.2, 0, "Server did not start");
        assert_ne!(r.3, 0, "Server did not terminate");
    }
}