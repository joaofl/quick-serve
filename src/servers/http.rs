// use egui::epaint::tessellator::path;
use log::{debug, info};

use tower_http::services::ServeDir;
use std::net::{IpAddr, SocketAddr};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use crate::servers::Protocol;
use async_trait::async_trait;
use crate::utils::validation;
use super::Server;

#[async_trait]
pub trait HTTPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

#[async_trait]
impl HTTPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");

        s.path = Arc::new(path);
        s.bind_address = IpAddr::from_str(&bind_ip).expect("Invalid IP address");
        s.port = port;

        s.protocol = Protocol::Http;
        HTTPRunner::runner(&s);
        s
    }

    fn runner(&self) {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address;
        let port = self.port;
        let path = self.path.clone();
        
        tokio::spawn(async move {
            loop {
                debug!("Runner started. Waiting command to connect...");
                let m = receiver.recv().await.unwrap();
                debug!("Message received");

                if m.connect {
                    info!("Connecting...");
                    // Create a SocketAddr from the IpAddr and port
                    let socket_addr = SocketAddr::new(bind_address, port);
                    let service = ServeDir::new(path.deref());
                    let _ = hyper::server::Server::bind(&socket_addr)
                        .serve(tower::make::Shared::new(service))
                        .with_graceful_shutdown(async {
                            loop {
                                info!("Connected. Waiting command to disconnect...");
                                let _m = receiver.recv().await.unwrap();
                                break;
                            }
                            info!("Gracefully terminated the HTTP server");
                        })
                        .await.expect("Error starting the HTTP server...");
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
        let proto = Protocol::Http;
        let port = 8079u16;
        let file_in = "data.bin";
        let file_out = "/tmp/data-out-http.bin";
        let dl_cmd = format!("wget -t2 -T1 {}://127.0.0.1:{}/{} -O {}", proto.to_string(), port, file_in, file_out);

        test_server_e2e(proto, port, dl_cmd, file_in, file_out);
    }
}
