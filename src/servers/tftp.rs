use log::{info, debug};





use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;

// Create the TFTP server.
use async_tftp::server::TftpServerBuilder;


use std::sync::Arc;



#[async_trait]
pub trait TFTPServerRunner {
    fn new() -> Self;
    async fn runner(self: Arc<Self>);
}

#[async_trait]
impl TFTPServerRunner for Server {
    fn new() -> Self {
        let mut s = Server::default();
        s.protocol = Protocol::Tftp;
        return s;
    }
    async fn runner(self: Arc<Self>) {
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();

        loop {
            let msg = receiver.recv().await.unwrap();

            if msg.terminate { return };
            if msg.connect {
                let tsk = tokio::spawn({
                    let me = Arc::clone(&self);
                    async move {
                        let addr = format!("{}:{}", me.bind_address, me.port);
                        let tftpd =
                            TftpServerBuilder::with_dir_ro(me.path.clone()).unwrap()
                                .bind(addr.parse().unwrap())
                                .build().await.unwrap();

                        info!("Starting TFTP server...");
                        let _ = tftpd.serve().await;
                    }
                });

                let msg = receiver.recv().await.unwrap();
                if !msg.connect {
                    tsk.abort();
                    debug!("TFTP server stopped");
                    if msg.terminate { return };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_e2e() {
        todo!("Not done yet....")
    }
}