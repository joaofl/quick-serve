use log::{debug, info, error, warn};
use tokio::sync::broadcast;
use std::{path::PathBuf, default};

use tower_http::services::ServeDir;
use std::net::{SocketAddr, IpAddr};


#[derive(Default, Clone, Debug)]
pub struct Message {
    pub connect: bool,
    pub path: PathBuf,
    pub bind_address: String,
    pub port: u16,
}

pub struct Server { 
    pub sender: broadcast::Sender<Message>,
    // pub callback: Box<dyn Fn()>, 
}

impl Default for Server {
    fn default() -> Self {
        let default_callback = || {
            warn!("Runner callback not set. Not doing anything...");
        };

        Server { 
            sender: broadcast::channel(1).0,
            // callback: Box::new(default_callback),
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

    // pub async fn runner(&self) { 
    //     // Get notified about the server's spawned task
    //     let mut receiver = self.sender.subscribe();

    //     loop {
    //         let m = receiver.recv().await.unwrap();
    //         debug!("{:?}", m);

    //         if m.connect {
    //             // (self.callback)(m, receiver);
    //             debug!("what????")
    //         }
    //     }
    // }
}

