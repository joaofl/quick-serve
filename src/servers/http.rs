use log::{debug, info};

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};
use crate::servers::Protocol;
use async_trait::async_trait;

use super::Server;

#[async_trait]
pub trait HTTPServerRunner {
    async fn runner(&self);
}

#[async_trait]
impl HTTPServerRunner for Server {
    async fn runner(&self) {
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();

        loop {
            let m = receiver.recv().await.unwrap();
            debug!("{:?}", m);

            if m.terminate { return };
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
                            if m.terminate { return };
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


// #[cfg(test)]
// mod tests {
//     use crate::servers::Protocol;
//     // Import necessary items for testing
//     use super::*;
//     use crate::tests::common;
//
//     #[tokio::test]
//     async fn test_http_server_e2e() {
//         let r = common::test_server::e2e(Server::new(Protocol::http)).await;
//
//         // let r = task_command.await.unwrap();
//         assert_eq!(r.0, 0, "Server did not start");
//         assert_ne!(r.1, 0, "Server did not stop");
//         assert_eq!(r.2, 0, "Server did not start");
//         assert_ne!(r.1, 0, "Server did not terminate");
//     }
// }