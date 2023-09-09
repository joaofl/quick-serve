use libunftp;

use log::{debug};
use unftp_sbe_fs::ServerExt;

use std::time::Duration;
use super::Server;


pub struct FTPServer {
    name: String,
    pub server: Server,
}


impl FTPServer {
    pub fn new() -> Self {
        FTPServer {
            name: "HTTP".to_string(), 
            server: Server::new(),
        }
    }

    pub async fn runner(&self) { 
        // Get notified about the server's spawned task
        let mut receiver_1 = self.server.sender.subscribe();
        
        loop {
            let m = receiver_1.recv().await.unwrap();
            debug!("{:?}", m);
            let mut receiver_2 = self.server.sender.subscribe();

            if m.connect {

                let server = 
                libunftp::Server::with_fs(m.path.clone())
                    .passive_ports(50000..65535)
                    .metrics()
                    .shutdown_indicator(async move {
                        // let r2 = receiver_2.clone();
                        loop {
                            let connect = receiver_2.recv().await.unwrap().connect;
                            if connect { continue } // Not for me. Go wait another msg
                            else { break }
                        }
                        debug!("Gracefully terminating the HTTP server");
                        //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                        libunftp::options::Shutdown::new().grace_period(Duration::from_secs(10))
                    });

                let full_address = format!("{}:{}", m.bind_address, m.port);
                server.listen(full_address).await;
            }
        }
    }

}
