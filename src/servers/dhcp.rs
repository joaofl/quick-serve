use async_trait::async_trait;
use std::path::PathBuf;
use super::Server;
use crate::utils::validation;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use crate::servers::Protocol;

use std::str::FromStr;
use log::debug;

use std::net::UdpSocket;
use dhcp4r::server as dhcp_server;
use crate::servers::dhcp_server::DhcpServer;

#[async_trait]
pub trait DHCPRunner {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self;
    fn runner(&self);
}

#[async_trait]
impl DHCPRunner for Server {
    fn new(path: PathBuf, bind_ip: String, port: u16) -> Self {
        let mut s = Server::default();

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
        let ip_port = format!("{}:{}", bind_address, port);
        let socket_bind = format!("0.0.0.0:{}", port);

        tokio::spawn(async move {
            loop {
                debug!("DHCP runner started... Waiting command to connect...");
                let m = receiver.recv().await.unwrap();
                debug!("Message received");

                if m.connect {
                    debug!("DHCP server started on {}", ip_port);

                    let server = DhcpServer::default();

                    let socket = UdpSocket::bind(socket_bind.clone()).unwrap();
                    socket.set_broadcast(true).unwrap();

                    let ipv4: Ipv4Addr = bind_address.clone().to_string().parse().unwrap();
                    dhcp_server::Server::serve(socket, ipv4, server);

                    debug!("DHCP server stopped");
                    break;
                }
            }
        });
    }
}


#[cfg(test)]
mod tests {
    use testcontainers::core::{CmdWaitFor, ExecCommand};
    use testcontainers::{runners::SyncRunner, GenericImage};
    use std::env;
    use std::process::Command;

    // For some reason, this INIT.call_once does not work on the CI
    // so I have to call it for every test, since I could not yet find
    // a way to have it as a fixture

    // static INIT: Once = Once::new();
    fn build_images() {
        // INIT.call_once(|| {
            // Create the docker images here
            let cwd = env::var("CARGO_MANIFEST_DIR").unwrap();

            let _out = Command::new("docker")
                .arg("compose")
                .arg("build")
                .current_dir(format!("{cwd}/docker/"))
                .output()
                .expect(&format!("Failed to execute command. Check directory {}", cwd));
        // });
    }


    fn run_command(args: &str, wait_for: &str) -> (String, String) {
        let custom_image = GenericImage::new("test_image", "latest");
        let container = custom_image.start().unwrap();

        let args_array: Vec<&str> = args.split_whitespace().collect();

        // exit code, it waits for result
        let mut res = container
            .exec(
                ExecCommand::new(args_array)
                .with_cmd_ready_condition(CmdWaitFor::message_on_stderr(wait_for))
                .with_cmd_ready_condition(CmdWaitFor::seconds(10))
            )
            .unwrap_or_else(|e| {
                panic!("Failed to run cmd {}\nError:\n{:?}", args, e.to_string());
            });

        let out = String::from_utf8(res.stdout_to_vec().unwrap()).unwrap();
        let err = String::from_utf8(res.stderr_to_vec().unwrap()).unwrap();

        (out, err)
    }


    #[test]
    fn ip_assigning() {
        build_images();

        let client_thread = std::thread::spawn(move || {
            let (_out, err) = run_command("dhclient -4 -d -v -p 6768", "bound to");

            let expected_lines = [
                "binding to user-specified port",
                "DHCPDISCOVER on",
                "bound to",
            ];

            for expected in &expected_lines {
                assert!(err.contains(expected), 
                    "Expected line not found: {}\nCheck on the complete logs:\n{}", expected, err);
            }
        });


        let server_thread = std::thread::spawn(move || {
            // Run the DHCP server on another thread
            let (out, _err) = run_command("quick-serve --dhcp=6767 -v --bind-ip=172.12.1.4", "dhcp_server: offered");

            let expected_lines = [
                "DHCP server started",
                "dhcp_server: Request received",
                "dhcp_server: offered",
            ];
    
            for expected in &expected_lines {
                assert!(out.contains(expected), 
                    "Expected line not found: {}\nCheck on the complete logs:\n{}", expected, out);
            }
        });

        client_thread.join().unwrap();
        server_thread.join().unwrap();
    }
}
