use log::{debug, info};
use tokio::sync::broadcast;
use std::{path::PathBuf, default};

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};


#[derive(Default, Clone, Debug)]
struct Message {
    connect: bool,
    path: PathBuf,
    bind_address: String,
    port: u16,
}

pub struct Server { sender: broadcast::Sender<Message> }

impl Default for Server {
    fn default() -> Self {
        Server { sender: broadcast::channel(1).0 }
    }
}

impl Server {
    pub fn new() -> Self {
        Server::default()
    }

    pub fn start(&self, path: PathBuf, bind_address: String, port: u16) {
        let s = Message{connect: true, path, bind_address, port};
        self.sender.send(s);
    }

    pub fn stop(&self){
        let mut m = Message::default();
        m.connect = false;
        self.sender.send(m);
    }
}


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
                        let connect = receiver.recv().await.unwrap().connect;
                        if connect { continue } // Not for me. Go wait another msg
                        else { break }
                    }
                    debug!("Gracefully terminating the HTTP server");
                });

                server.await.expect("server error");
            }

        }
    }
}
