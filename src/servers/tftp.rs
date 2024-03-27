use log::{info, debug};

use super::{Protocol, Server};
use async_trait::async_trait;

// Create the TFTP server.
use async_tftp::server::TftpServerBuilder;
use std::{ops::Deref, path::PathBuf};
use crate::utils::validation;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[async_trait]
pub trait TFTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    async fn runner(&self) -> JoinHandle<()>;
}

#[async_trait]
impl TFTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        let path = validation::ensure_trailing_slash(&path);
        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");
        s.path = Arc::new(path);
        s.bind_address = bind_ip;
        s.port = port;

        s.protocol = Protocol::Tftp;

        return s;
    }
    async fn runner(&self) -> JoinHandle<()> {

        let path = self.path.clone();

        let bind_address = self.bind_address.clone();
        let port = self.port;
        let mut receiver = self.sender.subscribe();

        tokio::spawn(async move {
            loop {
                // Get notified about the server's spawned task
                let msg = receiver.recv().await.unwrap();

                if msg.connect {
                    let tsk = tokio::spawn({
                        let path = path.clone();
                        let bind_address = bind_address.clone();
                        let port = port;
                        
                        async move {
                            let addr = format!("{}:{}", bind_address, port);
                            let tftpd =
                                TftpServerBuilder::with_dir_ro(path.deref()).unwrap()
                                    .bind(addr.parse().unwrap())
                                    .build().await.unwrap();

                            info!("Starting TFTP server...");
                            let _ = tftpd.serve().await;
                        }
                    });

                    let _ = receiver.recv().await.unwrap();
                    tsk.abort();
                    debug!("TFTP server stopped");
                }
            }
        })
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
