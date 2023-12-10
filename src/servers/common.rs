use log::{info, error};
use tokio::sync::broadcast;
use std::{path::PathBuf};

use crate::utils::validation;

#[derive(Default, PartialEq)]
pub enum Protocol {
    Http,
    Tftp,
    Dhcp,
    Ftp,
    #[default]
    None,
}

impl Protocol {
    pub fn to_string(&self) -> &str {
        match self {
            Protocol::Http => "http",
            Protocol::Ftp  => "ftp",
            Protocol::Tftp => "tftp",
            Protocol::Dhcp => "dhcp",
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
    pub fn start(&self, path: PathBuf, bind_address: String, port: u16) -> Result<(), String> {
        validation::validate_path(&path)?;
        validation::validate_ip_port(&format!("{}:{}", bind_address, port))?;

        info!("Starting {} server bind to {}:{}", self.protocol.to_string(), bind_address, port);
        info!("Serving {}", path.to_string_lossy());

        let s = Message{connect: true, terminate: false, path, bind_address, port};
        let _ = self.sender.send(s).map_err(|err| format!("Error sending message: {:?}", err))?;
        Ok(())
    }


    pub fn stop(&self){
        // Stop serving, but continues the loop listening
        // to messages to potentially re-start serving
        let mut m = Message::default();
        m.connect = false;
        m.terminate = false;

        info!("Stopping {} server", self.protocol.to_string());
        let _ = self.sender.send(m);
    }

    pub fn terminate(&self){
        // Stop the serving loop to exit the application. 
        // Mostly required by the headless version (single sessions).

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

