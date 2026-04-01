use std::path::PathBuf;
use super::Server;
use crate::utils::validation;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use crate::servers::Protocol;

use std::str::FromStr;
use log::{debug, info, error};

use std::net::UdpSocket;
use dhcp4r::server as dhcp_server;
use crate::servers::dhcp_server::DhcpServer;

pub trait DHCPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Result<Self, crate::QuickServeError> where Self: Sized;
    fn runner(&self);
}

impl DHCPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Result<Self, crate::QuickServeError> {
        let mut s = Server::default();

        // Validate inputs with proper error handling
        validation::validate_ip_port(&bind_ip, port)?;

        let path = validation::ensure_trailing_slash(&path);
        s.path = Arc::new(path);
        s.bind_address = IpAddr::from_str(&bind_ip)
            .map_err(|e| crate::QuickServeError::validation(format!("Invalid IP address '{}': {}", bind_ip, e)))?;
        s.port = port;

        s.protocol = Protocol::Dhcp;
        DHCPRunner::runner(&s);
        Ok(s)
    }

    fn runner(&self) {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address;
        let port = self.port;
        let ip_port = format!("{}:{}", bind_address, port);
        let socket_bind = format!("0.0.0.0:{}", port);

        tokio::spawn(async move {
            loop {
                debug!("DHCP runner started... Waiting command to connect...");
                
                let m = match receiver.recv().await {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to receive message in DHCP runner: {}", e);
                        break;
                    }
                };
                debug!("Message received");

                if m.connect {
                    info!("Starting DHCP server on {}", ip_port);

                    let server = DhcpServer::default();

                    // Bind socket with proper error handling
                    let socket = match UdpSocket::bind(&socket_bind) {
                        Ok(socket) => {
                            info!("DHCP server bound to {}", socket_bind);
                            socket
                        }
                        Err(e) => {
                            error!("Failed to bind DHCP server to {}: {}", socket_bind, e);
                            break;
                        }
                    };

                    // Set broadcast with error handling
                    if let Err(e) = socket.set_broadcast(true) {
                        error!("Failed to set broadcast on DHCP socket: {}", e);
                        break;
                    }

                    // Parse IPv4 address with error handling
                    let ipv4 = match bind_address.to_string().parse::<Ipv4Addr>() {
                        Ok(addr) => addr,
                        Err(e) => {
                            error!("Failed to parse IPv4 address '{}': {}", bind_address, e);
                            break;
                        }
                    };

                    info!("DHCP server serving on {} with IP {}", socket_bind, ipv4);
                    dhcp_server::Server::serve(socket, ipv4, server);

                    info!("DHCP server stopped");
                    break;
                }
            }
        });
    }
}

