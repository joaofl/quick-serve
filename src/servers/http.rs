use log::debug;

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};
use std::path::PathBuf;
use std::sync::Arc;
use crate::servers::Protocol;
use async_trait::async_trait;
use crate::utils::validation;

use super::Server;

#[async_trait]
pub trait HTTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    async fn runner(self: Arc<Self>);
}

#[async_trait]
impl HTTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");
        s.path = path;
        s.bind_address = bind_ip;
        s.port = port;

        s.protocol = Protocol::Http;
        return s;
    }
    async fn runner(self: Arc<Self>) {
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();

        loop {
            let m = receiver.recv().await.unwrap();

            if m.terminate { return };
            if m.connect {
                // Spin and await the actual server here
                // Parse the IP address string into an IpAddr
                let ip: IpAddr = self.bind_address.parse().expect("Invalid IP address");

                // Create a SocketAddr from the IpAddr and port
                let socket_addr = SocketAddr::new(ip, self.port);
                let me = Arc::clone(&self);
                let service = ServeDir::new(me.path.clone());
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

#[cfg(test)]
mod tests {
    use crate::utils;
    use utils::test_utils::tests::*;
    use crate::servers::Protocol;

    #[test]
    fn e2e() {
        let proto = Protocol::Http;
        let port = 8089u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-http.bin";
        let dl_cmd = format!("wget -t2 -T1 {}://127.0.0.1:{}/{} -O {}", proto.to_string(), port, file_in, file_out);

        test_server_e2e(proto, port, dl_cmd, file_in, file_out);
    }
}