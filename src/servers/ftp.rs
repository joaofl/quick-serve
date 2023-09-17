use libunftp;

use log::{debug, info};
use unftp_sbe_fs::ServerExt;

use std::time::Duration;
use super::Server;


pub struct FTPServer {
    name: String,
    pub server: Server,
}


impl FTPServer {
    pub fn new() -> Self {
        FTPServer {
            name: "HTTP".to_string(), 
            server: Server::new(),
        }
    }

    pub async fn runner(&self) { 
        // Get notified about the server's spawned task
        let mut receiver_1 = self.server.sender.subscribe();
        
        loop {
            let m = receiver_1.recv().await.unwrap();
            debug!("{:?}", m);
            let mut receiver_2 = self.server.sender.subscribe();

            if m.terminate { return };
            if m.connect {

                let server = 
                libunftp::Server::with_fs(m.path.clone())
                    .passive_ports(50000..65535)
                    .metrics()
                    .shutdown_indicator(async move {
                        // let r2 = receiver_2.clone();
                        loop {
                            let m2 = receiver_2.recv().await.unwrap();
                            if m2.terminate { break }
                            if m2.connect { continue } // Not for me. Go wait another msg
                            else { break }
                        }
                        debug!("Gracefully terminating the HTTP server");
                        //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                        libunftp::options::Shutdown::new().grace_period(Duration::from_secs(10))
                    });

                let full_address = format!("{}:{}", m.bind_address, m.port);
                server.listen(full_address).await;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    // Import necessary items for testing
    use super::*;
    use std::sync::Arc;
    use tokio::time::{self, Duration};
    use tokio::process::Command as AsyncCommand;

    extern crate ftp;
    use std::fs::File;
    use std::io::prelude::*;

    #[tokio::test]
    async fn test_ftp_server() {
        let ftp_server = Arc::new(FTPServer::new());
        let ftp_server_c = ftp_server.clone();

        let bind_address = "127.0.0.1".to_string();
        let bind_address_c = bind_address.clone();
        let port: u16 = 2121;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();
        // Create a temporary file inside the directory
        let mut temp_file = File::create(path.join("file.txt")).expect("Failed to create temp file");
        // Write some data to the temporary file
        temp_file.write_all(b"This is a temporary file!").expect("Failed to write to temp file");

        let t1 = tokio::spawn(async move {
            ftp_server.runner().await;
        });

        let t2 = tokio::spawn(async move {
            time::sleep(Duration::from_millis(100)).await;
            let _r = ftp_server_c.server.start(path, bind_address_c, port);
            time::sleep(Duration::from_millis(500)).await;
            info!("Stopping FTP server");
            ftp_server_c.server.terminate();
        });

        let t3 = tokio::spawn( async move {
            time::sleep(Duration::from_millis(200)).await;

            let output1 = AsyncCommand::new("wget")
                .arg("--timeout=1")
                .arg("--tries=1")
                .arg("--output-document=/tmp/file-recv.txt")
                .arg("ftp://127.0.0.1:2121/file.txt")
                .output()
                .await.expect("Failed to execute command");

            time::sleep(Duration::from_millis(700)).await;

            // let output2 = output_cmd.await.expect("Failed to execute command");
            let output2 = AsyncCommand::new("wget")
                .arg("--timeout=1")
                .arg("--tries=1")
                .arg("--output-document=/tmp/file-recv.txt")
                .arg("ftp://127.0.0.1:2121/file.txt")
                .output()
                .await.expect("Failed to execute command");

            ( output1.status.code().unwrap(), output2.status.code().unwrap() )
        });


        let (r1, r2) = t3.await.unwrap();
        assert_eq!(r1, 0, "Error downloading file");
        assert_ne!(r2, 0, "Server did not shutdown");

        tokio::join!(t1, t2);
    }
}