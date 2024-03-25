use log::info;
use tokio::sync::broadcast;
use std::{path::PathBuf, sync::Arc};


#[derive(Debug, Default, PartialEq, Clone)]
pub enum Protocol {
    #[default]
    Http,
    Tftp,
    Ftp,
}

pub const PROTOCOL_LIST: [&'static Protocol; 3] = [&Protocol::Http, &Protocol::Tftp, &Protocol::Ftp];

impl Protocol {
    pub fn to_string(&self) -> &str {
        match self {
            Protocol::Http => "http",
            Protocol::Ftp  => "ftp",
            Protocol::Tftp => "tftp",
        }
    }
    pub fn get_default_port(&self) -> u16 {
        match self {
            Protocol::Http => 8080,
            Protocol::Ftp  => 2121,
            Protocol::Tftp => 6969,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Message {
    pub connect: bool,
}

pub struct Server {
    pub sender: broadcast::Sender<Message>,
    pub protocol: Protocol,
    pub path: Arc<PathBuf>,
    pub bind_address: String,
    pub port: u16
}

impl Default for Server {
    fn default() -> Self {
        Server {
            sender: broadcast::channel(10).0,
            protocol: Protocol::default(),
            path: Arc::new(PathBuf::default()),
            bind_address: String::default(),
            port: 0,
        }
    }
}

impl Server {
    pub fn start(&self) -> Result<(), String> {
        info!("Starting {} server bind to {}:{}", self.protocol.to_string(), self.bind_address, self.port);
        info!("Serving {}", self.path.to_string_lossy());

        let s = Message{connect: true};
        let _ = self.sender.send(s).map_err(|err| format!("Error sending message: {:?}", err))?;
        Ok(())
    }

    pub fn stop(&self){
        // Stop the serving loop to exit the application. 
        // Mostly required by the headless version (single sessions).

        // First stop and to then stop
        let m = Message {connect: false};

        // Send twice. Once to make sure the server is stopped (inner loop)
        // and the second to ensure runner exits.
        let _ = self.sender.send(m.clone());
        let _ = self.sender.send(m);
        info!("{} server stopped", self.protocol.to_string());
    }
}

