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

                    let ms = DhcpServer::default();

                    let socket = UdpSocket::bind(socket_bind.clone()).unwrap();
                    socket.set_broadcast(true).unwrap();

                    let ipv4: Ipv4Addr = bind_address.clone().to_string().parse().unwrap();
                    dhcp_server::Server::serve(socket, ipv4, ms);

                    debug!("DHCP server stopped");
                    break;
                }
            }
        });
    }
}







#[cfg(test)]
mod tests {

    // use testcontainers_modules::{postgres, testcontainers::runners::SyncRunner};

    // use crate::tests::common::tests::*;
    use std::io::{stdout, BufRead};
    use crate::servers::Protocol;
    use testcontainers::{core::{IntoContainerPort, WaitFor}, runners::SyncRunner, GenericImage, ImageExt};

    // #[test]
    // fn e2e() {
    //     let proto = Protocol::Dhcp;
    //     let port = 2223u16;
    // }


    #[test]
    fn client() {

        println!("Client test");

        // let image = ImageExt::with_cmd(self, cmd)
        // let docker = clients::Cli::default();
        let custom_image = GenericImage::new("client_image", "latest");

        // let custom_image = custom_image
        //     .with_env_var("DEBUG", "1")
        //     .with_cmd(vec!["sleep", "5"]);

        let container = custom_image.start().unwrap();


        let stderr = container.stderr(true);
        // let stdout = container.stdout(true);

        // it's possible to send logs to another thread
        let log_follower_thread = std::thread::spawn(move || {
            // let stdout_lines = stdout.lines();
            // for line in stdout_lines {
            //     println!("stdout: {}", line.unwrap());
            // }

            let mut std_lines = stderr.lines();
            let expected_messages = [
                "binding to user-specified port",
            ];
            for expected_message in expected_messages {
                let line = std_lines.next().expect("line must exist")?;
                if !line.contains(expected_message) {
                    println!("Log message ('{}') doesn't contain expected message ('{}')", line, expected_message);
                    anyhow::bail!(
                        "Log message ('{}') doesn't contain expected message ('{}')",
                        line,
                        expected_message
                    );
                }
            }
            Ok(())
        });


            // let expected_messages = [
            //     "binding to user-specified port",
            //     "dadasdasdasdasd",
            // ];

            // let mut stdout_lines = stdout.lines();
            // for expected_message in expected_messages {
            //     let line = stdout_lines.next().expect("line must exist").unwrap();
            //     if !line.contains(expected_message) {
            //         println!("Log message ('{}') doesn't contain expected message ('{}')", line, expected_message);
            //     }
            // }

        let _ = log_follower_thread
            .join().unwrap_or_else(|_| Err(anyhow::anyhow!("failed to join log follower thread")));

        // logs are accessible after container is stopped
        let _ = container.stop();


        let stdout = String::from_utf8(container.stdout_to_vec().unwrap()).unwrap();

        println!("*************stdout:\n\n{}", stdout);

    }

    // #[test]
    // fn sync_logs_are_accessible() -> anyhow::Result<()> {
    //     let image = GenericImage::new("testcontainers/helloworld", "1.1.0");
    //     let container = image.start()?;

    //     let stderr = container.stderr(true);

    //     // it's possible to send logs to another thread
    //     let log_follower_thread = std::thread::spawn(move || {
    //         let mut stderr_lines = stderr.lines();
    //         let expected_messages = [
    //             "DELAY_START_MSEC: 0",
    //             "Sleeping for 0 ms",
    //             "Starting server on port 8080",
    //             "Sleeping for 0 ms",
    //             "Starting server on port 8081",
    //             "Ready, listening on 8080 and 8081",
    //         ];
    //         for expected_message in expected_messages {
    //             let line = stderr_lines.next().expect("line must exist")?;
    //             if !line.contains(expected_message) {
    //                 anyhow::bail!(
    //                     "Log message ('{}') doesn't contain expected message ('{}')",
    //                     line,
    //                     expected_message
    //                 );
    //             }
    //         }
    //         Ok(())
    //     });
    //     log_follower_thread
    //         .join()
    //         .map_err(|_| anyhow::anyhow!("failed to join log follower thread"))??;

    //     // logs are accessible after container is stopped
    //     container.stop()?;

    //     // stdout is empty
    //     let stdout = String::from_utf8(container.stdout_to_vec()?)?;
    //     assert_eq!(stdout, "");
    //     // stderr contains 6 lines
    //     let stderr = String::from_utf8(container.stderr_to_vec()?)?;
    //     assert_eq!(
    //         stderr.lines().count(),
    //         6,
    //         "unexpected stderr size: {}",
    //         stderr
    //     );
    //     Ok(())
    // }

}
