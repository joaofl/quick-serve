use libunftp::{Server, options};
use libunftp::auth::DefaultUser;
use unftp_sbe_fs::{Filesystem, ServerExt};
use log::{info, debug};
use std::path::PathBuf;
use tokio::sync::{broadcast};
use std::time::Duration;


pub struct FTPServer {
    server: Server<Filesystem, DefaultUser>,
    sender: tokio::sync::broadcast::Sender<bool>,
    address: String,
}

impl FTPServer {

    pub fn new(path: PathBuf, bind_address: String, port: u32) -> Self {
        let (sender, mut receiver_stop) = broadcast::channel(1);

        let address = format!("{bind_address}:{port}");

        let server = 
            Server::with_fs(path.clone())
                .passive_ports(50000..65535)
                .metrics()
                .shutdown_indicator(async move {
                    let _ = receiver_stop.recv().await;
                    info!("Shutting down FTP server");
                    //Give 10 seconds to potential ongoing connections to finish
                    // if none, finish immediately
                    options::Shutdown::new().grace_period(Duration::from_secs(10))
                });

        FTPServer {
            server,
            sender,
            address,
        }
    }

    pub fn start(&self){
        debug!("")
        // self.server.listen(self.address.clone()).await;
    }

    pub fn stop(&self){
        let _ = self.sender.send(false);
    }

}
