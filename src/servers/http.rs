use log::{debug, info};
use tokio::sync::broadcast;
use std::path::PathBuf;

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};

pub struct HTTPServer {
    command_sender: broadcast::Sender<(bool, PathBuf, String, u16)>,
}

impl HTTPServer {
    pub fn new() -> Self {
        HTTPServer { command_sender: broadcast::channel(1).0 }
    }

    pub fn start(&self, path: PathBuf, bind_address: String, port: u16) {
        let _ = self.command_sender.send((true, path, bind_address, port));
    }

    pub fn stop(&self){
        let _ = self.command_sender.send((false, PathBuf::new(), String::new(), 0));
    }

    pub async fn runner(&self) {
        // Get notified about the server's spawned task
        let mut command_receiver = self.command_sender.subscribe();

        loop {
            let (connect, path, bind_address, port) = command_receiver.recv().await.unwrap();

            if connect {
                info!("Starting the HTTP server at {}:{}:{}", bind_address, port, path.to_string_lossy());
                // Spin and await the actual server here
                // Parse the IP address string into an IpAddr
                let ip: IpAddr = bind_address.parse().expect("Invalid IP address");

                // Create a SocketAddr from the IpAddr and port
                let socket_addr = SocketAddr::new(ip, port);

                let service = ServeDir::new(path);
                let server = hyper::server::Server::bind(&socket_addr)
                .serve(tower::make::Shared::new(service))
                .with_graceful_shutdown(async {
                    loop {
                        let connect = command_receiver.recv().await.unwrap().0;
                        if connect { continue; } // Not for me. Go wait another msg
                        else { break; }
                    }
                    debug!("Gracefully terminating the HTTP server");
                });

                server.await.expect("server error");
            }
        }
    }
}
