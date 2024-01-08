use log::{info};
use tokio::sync::broadcast;
use std::{path::PathBuf};



#[derive(Default, PartialEq)]
pub enum Protocol {
    Http,
    Tftp,
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
            _ => "none",
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Message {
    pub connect: bool,
    pub terminate: bool,
}

pub struct Server {
    pub sender: broadcast::Sender<Message>,
    pub protocol: Protocol,
    pub path: PathBuf,
    pub bind_address: String,
    pub port: u16
}

impl Default for Server {
    fn default() -> Self {
        Server {
            sender: broadcast::channel(10).0,
            protocol: Protocol::default(),
            path: PathBuf::default(),
            bind_address: String::default(),
            port: 0,
        }
    }
}

impl Server {
    pub fn start(&self) -> Result<(), String> {
        info!("Starting {} server bind to {}:{}", self.protocol.to_string(), self.bind_address, self.port);
        info!("Serving {}", self.path.to_string_lossy());

        let s = Message{connect: true, terminate: false};
        let _ = self.sender.send(s).map_err(|err| format!("Error sending message: {:?}", err))?;
        Ok(())
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

