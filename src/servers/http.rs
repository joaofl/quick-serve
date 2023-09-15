use log::{debug, info};

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};

use super::Server; 

pub struct HTTPServer {
    name: String,
    pub server: Server,
}

impl HTTPServer {
    pub fn new() -> Self {
        HTTPServer { 
            name: "HTTP".to_string(), 
            server: Server::new(),
        }
    }

    pub async fn runner(&self) { 
        // Get notified about the server's spawned task
        let mut receiver = self.server.sender.subscribe();

        loop {
            let m = receiver.recv().await.unwrap();
            debug!("{:?}", m);

            if m.terminate { break };
            if m.connect {
                info!("Starting the server at {}:{}:{}", m.bind_address, m.port, m.path.to_string_lossy());
                // Spin and await the actual server here
                // Parse the IP address string into an IpAddr
                let ip: IpAddr = m.bind_address.parse().expect("Invalid IP address");

                // Create a SocketAddr from the IpAddr and port
                let socket_addr = SocketAddr::new(ip, m.port);

                let service = ServeDir::new(m.path);
                let server = hyper::server::Server::bind(&socket_addr)
                .serve(tower::make::Shared::new(service))
                .with_graceful_shutdown(async {
                    loop {
                        let m = receiver.recv().await.unwrap();
                        if m.connect { continue } // Not for me. Go wait another msg
                        else { break }
                    }
                    debug!("Gracefully terminating the HTTP server");
                });

                server.await.expect("server error");
            }

        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::fs::File;
    use std::io::prelude::*;
    use tokio::time::{self, Duration};

    // You can also spawn a server and talk to it like any other HTTP server:
    #[tokio::test]
    async fn test_http_server() {
        // HTTPServer hereon
        //
        let http_server = Arc::new(HTTPServer::new());
        let http_server_c = http_server.clone();
        let http_server_c1 = http_server.clone();

        let t1 = tokio::spawn(async move {
            http_server_c.runner().await;
            // time::sleep(Duration::from_secs(10)).await;
        });

        let t2 = tokio::spawn(async move {
            let bind_address = "127.0.0.1".to_string();
            let port: u16 = 8080;

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let path = temp_dir.path().to_path_buf();

            // Create a temporary file inside the directory
            let mut temp_file = File::create(path.join("temp_file.txt")).expect("Failed to create temp file");

            // Write some data to the temporary file
            temp_file.write_all(b"This is a temporary file!").expect("Failed to write to temp file");

            time::sleep(Duration::from_millis(100)).await;

            let _r = http_server.server.start(path.clone(), bind_address.clone(), port);
        });

        let t3 = tokio::spawn(async move {
            time::sleep(Duration::from_secs(10)).await;
            let _r = http_server_c1.server.stop();
            let _r = http_server_c1.server.terminate();
        });

        tokio::try_join!(t1, t2, t3);
    }
}