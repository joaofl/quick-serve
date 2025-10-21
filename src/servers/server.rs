use log::{debug, info, error};
use tokio::sync::broadcast;
use tokio::time::sleep;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;
use std::{path::PathBuf, sync::Arc};
use std::net::IpAddr;

use crate::{Cli, CommandMsg, DefaultChannel, FTPRunner, HTTPRunner, TFTPRunner, DHCPRunner, QuickServeError, QuickServeResult};


#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub enum Protocol {
    Dhcp,
    Ftp,
    #[default]
    Http,
    Tftp,
}

pub const PROTOCOL_LIST: [&'static Protocol; 4] = [&Protocol::Http, &Protocol::Tftp, &Protocol::Ftp, &Protocol::Dhcp];

impl Protocol {
    /// Returns the protocol name as a string
    pub fn to_string(&self) -> &str {
        match self {
            Protocol::Dhcp => "dhcp",
            Protocol::Ftp  => "ftp",
            Protocol::Http => "http",
            Protocol::Tftp => "tftp",
        }
    }
    
    /// Returns the default port for the protocol
    pub fn get_default_port(&self) -> u16 {
        match self {
            Protocol::Dhcp => 6767,
            Protocol::Ftp  => 2121,
            Protocol::Http => 8080,
            Protocol::Tftp => 6969,
        }
    }
}

/// Message used for internal server communication
#[derive(Default, Clone, Debug)]
pub struct Message {
    /// Whether to connect (true) or disconnect (false)
    pub connect: bool,
}

/// Represents a server instance with its configuration
pub struct Server {
    /// Broadcast sender for control messages
    pub sender: broadcast::Sender<Message>,
    /// The protocol this server handles
    pub protocol: Protocol,
    /// Path to serve files from
    pub path: Arc<PathBuf>,
    /// IP address to bind to
    pub bind_address: IpAddr,
    /// Port to listen on
    pub port: u16
}

impl Default for Server {
    fn default() -> Self {
        Server {
            sender: broadcast::channel(10).0,
            protocol: Protocol::default(),
            path: Arc::new(PathBuf::default()),
            bind_address: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 0,
        }
    }
}

impl Server {
    /// Starts the server by sending a connect message
    ///
    /// # Returns
    /// * `Ok(())` if the message was sent successfully
    /// * `Err(QuickServeError)` if sending the message failed
    pub fn start(&self) -> QuickServeResult<()> {
        info!("Starting {} server bind to {}:{}", self.protocol.to_string(), self.bind_address, self.port);
        info!("Serving {}", self.path.to_string_lossy());

        let s = Message{connect: true};
        self.sender.send(s)
            .map_err(|err| QuickServeError::server_lifecycle(format!("Error sending start message: {:?}", err)))?;
        Ok(())
    }

    /// Stops the server by sending disconnect messages
    ///
    /// Sends two disconnect messages to ensure both the inner loop and runner exit.
    ///
    /// # Returns
    /// * `Ok(())` if the messages were sent successfully
    /// * `Err(QuickServeError)` if sending messages failed
    pub fn stop(&self) -> QuickServeResult<()> {
        // Stop the serving loop to exit the application. 
        // Mostly required by the headless version (single sessions).

        info!("Stopping {} server", self.protocol.to_string());
        
        // First stop and to then stop
        let m = Message {connect: false};

        // Send twice. Once to make sure the server is stopped (inner loop)
        // and the second to ensure runner exits.
        self.sender.send(m.clone())
            .map_err(|err| QuickServeError::server_lifecycle(format!("Error sending first stop message: {:?}", err)))?;
        
        self.sender.send(m)
            .map_err(|err| QuickServeError::server_lifecycle(format!("Error sending second stop message: {:?}", err)))?;
        
        info!("{} server stopped", self.protocol.to_string());
        Ok(())
    }
}



/// Starts receiver tasks for all protocols
///
/// Spawns one async task per protocol that listens for start/stop commands
/// and manages the lifecycle of each server.
///
/// # Arguments
/// * `channel` - The broadcast channel for sending commands to servers
pub fn server_starter_receiver(channel: &DefaultChannel<CommandMsg>) {
    ////////////////////////////////////////////////////////////////////////
    // Spawn one thread per protocol and start waiting for command
    // to start or stop each server
    ////////////////////////////////////////////////////////////////////////
    for protocol in PROTOCOL_LIST {
        let mut rcv = channel.sender.subscribe();
        debug!("Spawning receiver for {}", protocol.to_string());
        tokio::spawn(async move {
            loop {
                debug!(" {} started waiting for messages", protocol.to_string());
                let msg = rcv.recv().await.expect("Failed to receive message");
                if msg.protocol != *protocol {
                    debug!("\"Not my business...\" said the {}", protocol.to_string());
                    continue;
                }

                if msg.start == true {
                    let server = match msg.protocol {
                        Protocol::Http => {
                            <Server as HTTPRunner>::new(msg.path.clone().into(), msg.bind_ip.clone(), msg.port)
                        },
                        Protocol::Ftp => {
                            <Server as FTPRunner>::new(msg.path.clone().into(), msg.bind_ip.clone(), msg.port)
                        },
                        Protocol::Tftp => {
                            <Server as TFTPRunner>::new(msg.path.clone().into(), msg.bind_ip.clone(), msg.port)
                        },
                        Protocol::Dhcp => {
                            <Server as DHCPRunner>::new(msg.path.clone().into(), msg.bind_ip.clone(), msg.port)
                        },
                    };

                    let server = match server {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to create {} server: {}", msg.protocol.to_string(), e);
                            continue;
                        }
                    };

                    // Small delay to ensure the server's internal receiver is ready
                    // This is necessary because the server spawns async tasks that need to subscribe
                    sleep(Duration::from_millis(10)).await;
                    
                    if let Err(e) = server.start() {
                        error!("Failed to start {} server: {}", msg.protocol.to_string(), e);
                        continue;
                    }
                    info!("Started {} server", msg.protocol.to_string());

                    // Once started, wait for termination
                    match rcv.recv().await {
                        Ok(_msg) => {
                            if let Err(e) = server.stop() {
                                error!("Failed to stop {} server: {}", msg.protocol.to_string(), e);
                            } else {
                                info!("{} server stopped", msg.protocol.to_string());
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive stop message for {} server: {}", msg.protocol.to_string(), e);
                            // Try to stop the server anyway
                            if let Err(stop_err) = server.stop() {
                                error!("Failed to stop {} server after receive error: {}", msg.protocol.to_string(), stop_err);
                            }
                        }
                    }
                }
            }
        });
    }
}


/// Processes CLI arguments and sends start commands for requested servers
///
/// Validates the bind address and path, then sends start messages for each
/// server protocol specified in the command-line arguments. Blocks indefinitely
/// waiting for the Ctrl+C handler to terminate the process.
///
/// # Arguments
/// * `cli_args` - Parsed command-line arguments
/// * `channel` - The broadcast channel for sending commands to servers
pub fn server_starter_sender(cli_args: &Cli, channel: &DefaultChannel<CommandMsg>) {
    // Read and validate the bind address
    let bind_ip = &cli_args.bind_ip;
    let path = &cli_args.serve_dir;

    let mut count = 0u8;

    let mut cmd = CommandMsg {
        start: true,
        bind_ip: bind_ip.to_string(),
        path: path.to_string(),
        ..Default::default()
    };

    // Check for each server invoked from the command line, and send 
    // messages accordingly to start each
    if cli_args.http.is_some() {
        cmd.protocol = Protocol::Http;
        cmd.port = cli_args.http.unwrap() as u16;
        if let Err(e) = channel.sender.send(cmd.clone()) {
            error!("Failed to send HTTP start command: {}", e);
        }
        count += 1;
    }

    if cli_args.ftp.is_some() {
        cmd.protocol = Protocol::Ftp;
        cmd.port = cli_args.ftp.unwrap() as u16;
        if let Err(e) = channel.sender.send(cmd.clone()) {
            error!("Failed to send FTP start command: {}", e);
        }
        count += 1;
    }

    if cli_args.tftp.is_some() {
        cmd.protocol = Protocol::Tftp;
        cmd.port = cli_args.tftp.unwrap() as u16;
        if let Err(e) = channel.sender.send(cmd.clone()) {
            error!("Failed to send TFTP start command: {}", e);
        }
        count += 1;
    }

    if cli_args.dhcp.is_some() {
        cmd.protocol = Protocol::Dhcp;
        cmd.port = cli_args.dhcp.unwrap() as u16;
        if let Err(e) = channel.sender.send(cmd.clone()) {
            error!("Failed to send DHCP start command: {}", e);
        }
        count += 1;
    }

    if count == 0 {
        println!("No server specified. Use -h for help");
        exit(2);
    }
    else {
        // Wait indefinitely for signals (Ctrl+C handler will terminate the process)
        // This is more efficient than busy-waiting with sleep
        info!("All servers started. Waiting for shutdown signal...");
        std::thread::park();
    }
}
