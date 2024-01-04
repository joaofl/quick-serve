use log::{info, debug};

use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;

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
    use std::time::Duration;
    use assert_cmd::Command;
    use std::thread;

    #[test]
    fn test_e2e() {
        let server = thread::spawn(|| {
            let mut cmd = Command::cargo_bin("any-serve").unwrap();
            cmd.timeout(Duration::from_secs(1));
            cmd.args(&["--tftp", "-v"]);
            cmd.unwrap()
        });

        let client = thread::spawn(|| {
            let mut cmd = Command::new("tftp");
            cmd.env("PATH", "/bin");
            cmd.args(&["127.0.0.1", "6969", "-c", "get", "in.txt"]);
            cmd.unwrap()
        });

        let _ = server.join();
        client.join().unwrap();
    }
}
