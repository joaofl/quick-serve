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
    async fn runner(self: Arc<Self>) -> JoinHandle<()>;
}

#[async_trait]
impl FTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");
        s.path = path;
        s.bind_address = bind_ip;
        s.port = port;

        s.protocol = Protocol::Ftp;
        return s;
    }

    async fn runner(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            // Get notified about the server's spawned task
            let mut receiver = self.sender.subscribe();
            loop {
                let m = receiver.recv().await.unwrap();
                let mut receiver2 = self.sender.subscribe();

                if m.terminate { return };
                if m.connect {
                    // Define new server
                    let server = 
                    libunftp::Server::with_fs(self.path.clone())
                        .passive_ports(50000..65535)
                        .metrics()
                        .shutdown_indicator(async move {
                            loop {
                                let m2 = receiver2.recv().await.unwrap();
                                if m2.terminate { break }
                                if m2.connect { continue } // Not for me. Go wait another msg
                                else { break }
                            }
                            debug!("Gracefully terminating the FTP server");
                            //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                            libunftp::options::Shutdown::new().grace_period(Duration::from_secs(5))
                        });

                    // Spin and await the actual server here
                    let _ = server.listen(format!("{}:{}", self.bind_address, self.port)).await;
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
