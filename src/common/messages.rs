// Disable warning since `receiver` is not used in this file, 
// while its a placeholder for the future with too much impact to remove
#![allow(dead_code)]

use crate::servers::server::Protocol;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Clone, Debug, Default)]
pub struct CommandMsg {
    pub start: bool, 
    pub port: u16,
    // pub protocol: String,
    pub protocol: Protocol,
    pub bind_ip: String,
    pub path: String,
}

impl CommandMsg {
    pub fn new(prot: &Protocol) -> Self {
        Self {
            start: false, 
            port: prot.get_default_port(),
            protocol: prot.clone(),
            ..Default::default()
        }
    }
}

// Define a struct to hold both the sender and receiver
pub struct DefaultChannel<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl<T: Clone> Default for DefaultChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = channel (100);
        DefaultChannel { sender, receiver }
    }
}
