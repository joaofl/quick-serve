use std::path::PathBuf;
use log::{debug};
use unftp_sbe_fs::ServerExt;
use std::time::Duration;
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;
use crate::utils::validation;


#[async_trait]
pub trait FTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    async fn runner(&self);
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
                libunftp::Server::with_fs(self.path.clone())
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
                        debug!("Gracefully terminating the FTP server");
                        //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                        libunftp::options::Shutdown::new().grace_period(Duration::from_secs(10))
                    });

                // Spin and await the actual server here
                let _ = server.listen(format!("{}:{}", self.bind_address, self.port)).await;
            }
        }
    }
}


/////////////////////////////////////////////////////////////////////////////////////
//                                        TESTS                                    //
/////////////////////////////////////////////////////////////////////////////////////
// #[cfg(test)]
// mod tests {
    // #[tokio::test]
    // async fn test_e2e() {
    //     let bind_ip = String::from("127.0.0.1");
    //     let port: u16 = 2121;
    //     let (temp_dir_path, file_name) =
    //         crate::tests::common::test_server::mkfile().await.expect("Failed to create temp file...");
    //
    //     let s = Arc::new(<Server as FTPRunner>::new(temp_dir_path.clone(), bind_ip.clone(), port));
    //     let cmd = format!("wget -t2 -T1 {}://{}:{}/{} -O /tmp/out.txt",
    //                       s.protocol.to_string(), bind_ip.clone(), port, file_name);
    //
    //     crate::tests::common::test_server::test_server_e2e(s, cmd).await;
    // }
// }


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use assert_cmd::Command;
    use std::thread;

    #[test]
    fn test_e2e() {
        let server = thread::spawn(|| {
            let mut cmd = Command::cargo_bin("any-serve").unwrap();
            cmd.timeout(Duration::from_secs(2));
            cmd.args(&["--ftp", "-v"]);
            cmd.unwrap()
        });

        let client = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000));
            let mut cmd = Command::new("wget");
            cmd.env("PATH", "/bin");
            cmd.args(&["-t2", "-T1", "ftp://127.0.0.1:2121/in.txt", "-O", "/tmp/out.txt"]);
            cmd.unwrap()
        });

        let _ = server.join();
        client.join().unwrap();
    }
}