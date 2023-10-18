use log::debug;

use tokio::time::{self, Duration};
use tokio::task;

use std::{path::PathBuf};
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;

// Create the TFTP server.
use async_tftp::server::TftpServerBuilder;
use async_tftp::Result;

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

            if msg.terminate { return };
            if msg.connect {

                let tsk = tokio::spawn(async move {
                    let addr = format!("{}:{}", msg.bind_address, msg.port);
                    let tftpd = 
                        TftpServerBuilder::with_dir_ro(msg.path).unwrap()
                        .bind(addr.parse().unwrap())
                        .build().await.unwrap();

                    tftpd.serve().await;
                });

                let msg = receiver.recv().await.unwrap();
                if !msg.connect {
                    tsk.abort();
                    debug!("TFTP server terminated");
                }
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