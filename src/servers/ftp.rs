use log::{debug};

use unftp_sbe_fs::ServerExt;

use std::time::Duration;
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;


#[async_trait]
pub trait FTPServerRunner {
    fn new() -> Self;
    async fn runner(&self);
}

#[async_trait]
impl FTPServerRunner for Server {
    fn new() -> Self {
        let mut s = Server::default();
        s.protocol = Protocol::Ftp;
        return s;
    }
    async fn runner(&self) {
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();
        
        loop {
            let m = receiver.recv().await.unwrap();
            let mut receiver2 = self.sender.subscribe();

            if m.terminate { return };
            if m.connect {
                // Define new server
                let server = 
                libunftp::Server::with_fs(m.path.clone())
                    .passive_ports(50000..65535)
                    .metrics()
                    .shutdown_indicator(async move {
                        // let r2 = receiver_2.clone();
                        loop {
                            let m2 = receiver2.recv().await.unwrap();
                            if m2.terminate { break }
                            if m2.connect { continue } // Not for me. Go wait another msg
                            else { break }
                        }
                        debug!("Gracefully terminating the HTTP server");
                        //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                        libunftp::options::Shutdown::new().grace_period(Duration::from_secs(10))
                    });

                // Spin and await the actual server here
                let _ = server.listen(format!("{}:{}", m.bind_address, m.port)).await;
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
    async fn test_ftp_server_e2e() {
        let s = <Server as FTPServerRunner>::new();
        let r = common::test_server::e2e(s, 2121).await;

        assert_eq!(r.0, 0, "Server did not start");
        assert_ne!(r.1, 0, "Server did not stop");
        assert_eq!(r.2, 0, "Server did not start");
        assert_ne!(r.3, 0, "Server did not terminate");
    }
}