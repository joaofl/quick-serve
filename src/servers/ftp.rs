use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use log::{debug, info};
use unftp_sbe_fs::ServerExt;
use std::time::Duration;
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;
use crate::utils::validation;
use std::sync::Arc;


#[async_trait]
pub trait FTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

#[async_trait]
impl FTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");

        let path = validation::ensure_trailing_slash(&path);
        s.path = Arc::new(path);
        s.bind_address = IpAddr::from_str(&bind_ip).expect("Invalid IP address");
        s.port = port;

        s.protocol = Protocol::Ftp;
        FTPRunner::runner(&s);
        s
    }

    fn runner(&self) {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address;
        let port = self.port;
        let path = self.path.to_string_lossy().to_string();

        tokio::spawn(async move {

            loop {
                debug!("FTP runner started... Waiting command to connect...");
                let m = receiver.recv().await.unwrap();
                debug!("Message received");

                if m.connect {
                    info!("Connecting...");
                    // Define new server
                    let _ = libunftp::Server::with_fs(path)
                        .passive_ports(50000..65535)
                        .metrics()
                        .shutdown_indicator(async move {
                            loop {
                                info!("Connected. Waiting command to disconnect...");
                                let _ = receiver.recv().await.unwrap();
                                break;
                            }
                            debug!("Gracefully terminating the FTP server");
                            // Give a few seconds to potential ongoing connections to finish, 
                            // otherwise finish immediately
                            libunftp::options::Shutdown::new().grace_period(Duration::from_secs(5))
                        })
                        .build()
                        .unwrap()
                        .listen(format!("{}:{}", bind_address, port))
                        .await.expect("Error starting the FTP server...");
                    break;
                }
            }
        });
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
        let dl_cmd = format!("curl  --retry 2 --retry-delay 1 {}://127.0.0.1:{}/{} -o {}", proto.to_string(), port, file_in, file_out);

        test_server_e2e(proto, port, dl_cmd, file_in, file_out);

    }
}
