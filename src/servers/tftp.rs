use log::{info, debug};

use super::{Protocol, Server};
use async_trait::async_trait;

// Create the TFTP server.
use async_tftp::server::TftpServerBuilder;
use std::path::PathBuf;
use crate::utils::validation;
use std::{sync::Arc};

#[async_trait]
pub trait TFTPServerRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    async fn runner(self: Arc<Self>);
}

#[async_trait]
impl TFTPServerRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        let path = validation::ensure_trailing_slash(&path);
        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");
        s.path = path;
        s.bind_address = bind_ip;
        s.port = port;

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