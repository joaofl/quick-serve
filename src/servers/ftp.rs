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

            if m.connect {

                let server = 
                libunftp::Server::with_fs(m.path.clone())
                    .passive_ports(50000..65535)
                    .metrics()
                    .shutdown_indicator(async move {
                        // let r2 = receiver_2.clone();
                        loop {
                            let connect = receiver_2.recv().await.unwrap().connect;
                            if connect { continue } // Not for me. Go wait another msg
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
    // Import necessary items for testing
    use super::*;
    use std::sync::Arc;
    use std::process::Command;
    use tokio::time::{self, Duration};

    use std::io::prelude::*;
    use std::fs::File;

    #[tokio::test]
    async fn test_ftp_server() {
        let ftp_server = Arc::new(FTPServer::new());
        let ftp_server_c = ftp_server.clone();

        let bind_address = "127.0.0.1".to_string();
        let port: u16 = 2121;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();

        // Create a temporary file inside the directory
        let mut temp_file = File::create(path.join("temp_file.txt")).expect("Failed to create temp file");

        // Write some data to the temporary file
        temp_file.write_all(b"Hello, this is a temporary file!").expect("Failed to write to temp file");


        tokio::spawn(async move {
            ftp_server_c.runner().await;
        });

        let _r = ftp_server.server.start(path.clone(), bind_address.clone(), port);
        // info!("{:?}", _r);

        time::sleep(Duration::from_secs(1)).await;

        // Create a Command to run wget with a timeout
        let status = Command::new("wget")
            .arg(format!("ftp://{}:{}/temp_file.txt",bind_address.clone(), port))
            .status()
            .expect("wget could not be executed");

        println!("ls: {status}");

        assert!(status.success(), "Failed to fetch temp file");

        info!("Stopping FTP server");
        ftp_server.server.stop();
    }
}