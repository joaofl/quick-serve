use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, UdpSocket};
use std::ops::Add;
use std::time::{Duration, Instant};




#[async_trait]
pub trait DHCPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

#[async_trait]
impl DHCPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

        validation::validate_path(&path).expect("Invalid path");
        validation::validate_ip_port(&bind_ip, port).expect("Invalid bind IP");

        let path = validation::ensure_trailing_slash(&path);
        s.path = Arc::new(path);
        s.bind_address = IpAddr::from_str(&bind_ip).expect("Invalid IP address");
        s.port = port;

        s.protocol = Protocol::Dhcp;
        DHCPRunner::runner(&s);
        s
    }

    fn runner(&self) {
        let mut receiver = self.sender.subscribe();

        let bind_address = self.bind_address;
        let port = self.port;
        let path = self.path.to_string_lossy().to_string();

        tokio::spawn(async move {

            loop {
                debug!("DHCP runner started... Waiting command to connect...");
                let m = receiver.recv().await.unwrap();
                debug!("Message received");

                if m.connect {
                    info!("Connecting...");
                    // Define new server
                    // let _ = libundhcp::Server::with_fs(path)
                    //     .passive_ports(50000..65535)
                    //     .metrics()
                    //     .shutdown_indicator(async move {
                    //         loop {
                    //             info!("Connected. Waiting command to disconnect...");
                    //             let _ = receiver.recv().await.unwrap();
                    //             break;
                    //         }
                    //         debug!("Gracefully terminating the DHCP server");
                    //         // Give a few seconds to potential ongoing connections to finish, 
                    //         // otherwise finish immediately
                    //         libundhcp::options::Shutdown::new().grace_period(Duration::from_secs(5))
                    //     })
                    //     .build()
                    //     .unwrap()
                    //     .listen(format!("{}:{}", bind_address, port))
                    //     .await.expect("Error starting the HTTP server...");
                    break;
                }
            }
        });
    }
}





