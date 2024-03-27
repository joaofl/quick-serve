use std::path::PathBuf;
use log::{debug};
use unftp_sbe_fs::ServerExt;
use std::time::Duration;
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;
use crate::utils::validation;
use tokio::task::JoinHandle;
use std::sync::Arc;


#[async_trait]
pub trait FTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    async fn runner(&self) -> JoinHandle<()>;
}

#[async_trait]
impl FTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");
        s.path = Arc::new(path);
        s.bind_address = bind_ip;
        s.port = port;

        s.protocol = Protocol::Ftp;
        s
    }

    async fn runner(&self) -> JoinHandle<()> {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address.clone();
        let port = self.port;

        tokio::spawn(async move {
            // Get notified about the server's spawned task
            loop {
                let m = receiver.recv().await.unwrap();
                if m.connect {
                    // Define new server
                    let _ = libunftp::Server::with_fs("/tmp/")
                        .passive_ports(50000..65535)
                        .metrics()
                        .shutdown_indicator(async move {
                            loop {
                                let _ = receiver.recv().await.unwrap();
                                break;
                            }
                            debug!("Gracefully terminating the FTP server");
                            // Give a few seconds to potential ongoing connections to finish, 
                            // otherwise finish immediately
                            libunftp::options::Shutdown::new().grace_period(Duration::from_secs(5))
                        })
                        .listen(format!("{}:{}", bind_address, port))
                        .await;

                    break;
                }
            }
        })
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
    fn e2e() {
        let proto = Protocol::Ftp;
        let port = 2223u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-ftp.bin";
        let dl_cmd = format!("wget -t2 -T1 {}://127.0.0.1:{}/{} -O {}", proto.to_string(), port, file_in, file_out);

        test_server_e2e(proto, port, dl_cmd, file_in, file_out);

    }
}
