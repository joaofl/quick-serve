use log::debug;

use tokio::time::{self, Duration};
use tokio::task;

use std::{path::PathBuf};
use super::Server;
use async_trait::async_trait;
use crate::servers::Protocol;

use std::net::{Ipv4Addr, UdpSocket};

use std::collections::HashMap;
use dhcp4r::{options, packet, server, bytes_u32};

use std::ops::Add;
use std::time::{Instant};

// Server configuration
const SERVER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 76);
const IP_START: [u8; 4] = [192, 168, 0, 180];
const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
const DNS_IPS: [Ipv4Addr; 2] = [
    // Google DNS servers
    Ipv4Addr::new(8, 8, 8, 8),
    Ipv4Addr::new(4, 4, 4, 4),
];
const ROUTER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 254);
const LEASE_DURATION_SECS: u32 = 7200;
const LEASE_NUM: u32 = 100;

// Derived constants
const IP_START_NUM: u32 = bytes_u32!(IP_START);


#[async_trait]
pub trait DHCPServerRunner {
    fn new() -> Self;
    async fn runner(&self);
}

#[async_trait]
impl DHCPServerRunner for Server {
    fn new() -> Self {
        let mut s = Server::default();
        s.protocol = Protocol::Dhcp;
        return s;
    }
    async fn runner(&self) {
        // Get notified about the server's spawned task
        let mut receiver = self.sender.subscribe();

        loop {
            let msg = receiver.recv().await.unwrap();

            if msg.terminate { return };
            if msg.connect {

                let tsk = tokio::spawn(async move {
                    
                    let socket = UdpSocket::bind("0.0.0.0:6767").unwrap();
                    socket.set_broadcast(true).unwrap();
                
                    let ms = MyServer {
                        leases: HashMap::new(),
                        last_lease: 0,
                        lease_duration: Duration::new(LEASE_DURATION_SECS as u64, 0),
                    };
                    server::Server::serve(socket, SERVER_IP, ms);
                });

                let msg = receiver.recv().await.unwrap();
                if !msg.connect {
                    // TODO: Currently the por used is left busy by aborting the task.
                    // There must be a graceful shutdown.
                    tsk.abort();
                    debug!("DHCP server terminated");
                }
            }
        }
    }
}


struct MyServer {
    leases: HashMap<Ipv4Addr, ([u8; 6], Instant)>,
    last_lease: u32,
    lease_duration: Duration,
}

impl server::Handler for MyServer {
    fn handle_request(&mut self, server: &server::Server, in_packet: packet::Packet) {
        match in_packet.message_type() {
            Ok(options::MessageType::Discover) => {
                // Prefer client's choice if available
                if let Some(options::DhcpOption::RequestedIpAddress(addr)) =
                    in_packet.option(options::REQUESTED_IP_ADDRESS)
                {
                    let addr = *addr;
                    if self.available(&in_packet.chaddr, &addr) {
                        reply(server, options::MessageType::Offer, in_packet, &addr);
                        return;
                    }
                }
                // Otherwise prefer existing (including expired if available)
                if let Some(ip) = self.current_lease(&in_packet.chaddr) {
                    reply(server, options::MessageType::Offer, in_packet, &ip);
                    return;
                }
                // Otherwise choose a free ip if available
                for _ in 0..LEASE_NUM {
                    self.last_lease = (self.last_lease + 1) % LEASE_NUM;
                    if self.available(
                        &in_packet.chaddr,
                        &((IP_START_NUM + &self.last_lease).into()),
                    ) {
                        reply(
                            server,
                            options::MessageType::Offer,
                            in_packet,
                            &((IP_START_NUM + &self.last_lease).into()),
                        );
                        break;
                    }
                }
            }

            Ok(options::MessageType::Request) => {
                // Ignore requests to alternative DHCP server
                if !server.for_this_server(&in_packet) {
                    return;
                }
                let req_ip = match in_packet.option(options::REQUESTED_IP_ADDRESS) {
                    Some(options::DhcpOption::RequestedIpAddress(x)) => *x,
                    _ => in_packet.ciaddr,
                };
                if !&self.available(&in_packet.chaddr, &req_ip) {
                    nak(server, in_packet, "Requested IP not available");
                    return;
                }
                self.leases.insert(
                    req_ip,
                    (in_packet.chaddr, Instant::now().add(self.lease_duration)),
                );
                reply(server, options::MessageType::Ack, in_packet, &req_ip);
            }

            Ok(options::MessageType::Release) | Ok(options::MessageType::Decline) => {
                // Ignore requests to alternative DHCP server
                if !server.for_this_server(&in_packet) {
                    return;
                }
                if let Some(ip) = self.current_lease(&in_packet.chaddr) {
                    self.leases.remove(&ip);
                }
            }

            // TODO - not necessary but support for dhcp4r::INFORM might be nice
            _ => {}
        }
    }
}

impl MyServer {
    fn available(&self, chaddr: &[u8; 6], addr: &Ipv4Addr) -> bool {
        let pos: u32 = (*addr).into();
        pos >= IP_START_NUM
            && pos < IP_START_NUM + LEASE_NUM
            && match self.leases.get(addr) {
                Some(x) => x.0 == *chaddr || Instant::now().gt(&x.1),
                None => true,
            }
    }

    fn current_lease(&self, chaddr: &[u8; 6]) -> Option<Ipv4Addr> {
        for (i, v) in &self.leases {
            if &v.0 == chaddr {
                return Some(*i);
            }
        }
        return None;
    }
}

fn reply(
    s: &server::Server,
    msg_type: options::MessageType,
    req_packet: packet::Packet,
    offer_ip: &Ipv4Addr,
) {
    let _ = s.reply(
        msg_type,
        vec![
            options::DhcpOption::IpAddressLeaseTime(LEASE_DURATION_SECS),
            options::DhcpOption::SubnetMask(SUBNET_MASK),
            options::DhcpOption::Router(vec![ROUTER_IP]),
            options::DhcpOption::DomainNameServer(DNS_IPS.to_vec()),
        ],
        *offer_ip,
        req_packet,
    );
}

fn nak(s: &server::Server, req_packet: packet::Packet, message: &str) {
    let _ = s.reply(
        options::MessageType::Nak,
        vec![options::DhcpOption::Message(message.to_string())],
        Ipv4Addr::new(0, 0, 0, 0),
        req_packet,
    );
}



#[cfg(test)]
mod tests {
    // Import necessary items for testing
    use super::*;
    use crate::tests::common;

    #[tokio::test]
    async fn test_dhcp_server_e2e() {
        let s = <Server as DHCPServerRunner>::new();
        // let r = common::test_server::e2e(s, 6969).await;

        // assert_eq!(r.0, 0, "Server did not start");
        // assert_ne!(r.1, 0, "Server did not stop");
        // assert_eq!(r.2, 0, "Server did not start");
        // assert_ne!(r.3, 0, "Server did not terminate");
        todo!("Tests not yet implemented");
    }
}