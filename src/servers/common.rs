use log::info;
use tokio::sync::broadcast;
use std::{path::PathBuf};

use crate::utils::validation;

#[derive(Default, PartialEq)]
pub enum Protocol {
    Http,
    Ftp,
    #[default]
    None,
}

impl Protocol {
    pub fn to_string(&self) -> &str {
        match self {
            Protocol::Http => "http",
            Protocol::Ftp => "ftp",
            _ => "none",
        }
    }
}

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
    pub protocol: Protocol,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            sender: broadcast::channel(10).0,
            protocol: Protocol::default(),
        }
    }
}

impl Server {
    pub fn start(&self, path: PathBuf, bind_address: String, port: u16) {

        validation::validate_ip_port(&format!("{}:{}", bind_address, port)).unwrap_or_else(|error| {info!("Invalid IP")});
        validation::validate_path(&path).unwrap_or_else(|error| {info!("Invalid path")});

        let s = Message{connect: true, terminate: false, path, bind_address, port};
        let _ = self.sender.send(s);
    }

    pub fn stop(&self){
        let mut m = Message::default();
        m.connect = false;
        m.terminate = false;
        let _ = self.sender.send(m);
    }

    pub fn terminate(&self){
        // First stop and to then terminate
        let mut m = Message::default();
        m.connect = false;
        m.terminate = true;
        // Send twice. Once to make sure the server is terminated (inner loop)
        // and the second to ensure runner exits.
        let _ = self.sender.send(m.clone());
        let _ = self.sender.send(m);
    }
}

