use log::{warn};
use tokio::sync::broadcast;
use std::{path::PathBuf};

use super::super::utils::validation;

#[derive(Default, Clone, Debug)]
pub struct Message {
    pub connect: bool,
    pub terminate: bool,
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
        // let _default_callback = || {
        //     warn!("Runner callback not set. Not doing anything...");
        // };

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

        validation::validate_ip_port(&format!("{}:{}", bind_address, port)).expect("Invalid IP");
        validation::validate_path(&path).expect("Invalid path");

        let s = Message{connect: true, terminate: false, path, bind_address, port};
        self.sender.send(s);
    }

    pub fn stop(&self){
        let mut m = Message::default();
        m.connect = false;
        m.terminate = false;
        self.sender.send(m);
    }

    pub fn terminate(&self){
        let mut m = Message::default();
        m.connect = false;
        m.terminate = true;
        self.sender.send(m);
    }
}

