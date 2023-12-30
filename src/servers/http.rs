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
    use std::time::Duration;
    use assert_cmd::Command;
    use std::thread;

    #[test]
    fn test_e2e() {
        let server = thread::spawn(|| {
            let mut cmd = Command::cargo_bin("any-serve").unwrap();
            cmd.timeout(Duration::from_secs(1));
            cmd.args(&["--http", "-v"]);
            cmd.unwrap()
        });

        let client = thread::spawn(|| {
            let mut cmd = Command::new("wget");
            cmd.env("PATH", "/bin");
            cmd.args(&["-t2", "-T1", "http://127.0.0.1:8080/in.txt", "-O", "/tmp/out.txt"]);
            cmd.unwrap()
        });

        let _ = server.join();
        client.join().unwrap();
    }
}