use log::{debug, info};
use tokio::sync::broadcast;
use std::{path::PathBuf, default};

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};


#[derive(Default, Clone)]
struct Message {
    connect: bool,
    path: PathBuf,
    bind_address: String,
    port: u16,
}

struct Server {
    sender: broadcast::Sender<Message>,
}

impl Default for Server {
    fn default() -> Self {
        Server { 
            sender: broadcast::channel(1).0 
        }
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

    async fn runner(&self) { 
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();

        loop {
            let m = receiver.recv().await.unwrap();
            self.consume(m);
        }
    }

    fn consume(&self, m: Message){
        todo!()
    }
}

trait HTTPServer {
    fn consume(&self);
}

trait FTPServer {
    fn consume(&self);
}

impl HTTPServer for Server {
    fn consume(&self) {
        info!("Doing the HTTP serving here");
    }
}

impl FTPServer for Server {
    fn consume(&self) {
        info!("Doing the HTTP serving here");
    }
}


//     pub fn new() -> Self {
//         HTTPServer::default()
//     }

//     pub fn start(&self, path: PathBuf, bind_address: String, port: u16) {
//         let s = Message {connect: true, path, bind_address, port};
//         let _ = self.command.sender.send(s);
//     }

//     pub fn stop(&self){
//         let m = Message::default();
//         let _ = self.command.sender.send(m);
//     }

//     pub async fn runner(&self) {
//         // Get notified about the server's spawned task
//         let mut command_receiver = self.command.sender.subscribe();

//         loop {
//             // let (connect, path, bind_address, port) = command_receiver.recv().await.unwrap();
//             let m = command_receiver.recv().await.unwrap();

//             if m.connect {
//                 info!("Starting the server at {}:{}:{}", m.bind_address, m.port, m.path.to_string_lossy());
//                 // Spin and await the actual server here
//                 // Parse the IP address string into an IpAddr
//                 let ip: IpAddr = m.bind_address.parse().expect("Invalid IP address");

//                 // Create a SocketAddr from the IpAddr and port
//                 let socket_addr = SocketAddr::new(ip, m.port);

//                 let service = ServeDir::new(m.path);
//                 let server = hyper::server::Server::bind(&socket_addr)
//                 .serve(tower::make::Shared::new(service))
//                 .with_graceful_shutdown(async {
//                     loop {
//                         let connect = command_receiver.recv().await.unwrap().connect;
//                         if connect { continue; } // Not for me. Go wait another msg
//                         else { break; }
//                     }
//                     debug!("Gracefully terminating the HTTP server");
//                 });

//                 server.await.expect("server error");
//             }
//         }
//     }
//     // fn start_server(){
        
//     // }
// }
