use libunftp::{Server, options};

use unftp_sbe_fs::ServerExt;
use log::info;

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::broadcast;

// std::mem::drop;

pub struct FTPServer {
    sender: tokio::sync::broadcast::Sender<bool>,
    path: PathBuf,
    address: String,
}

impl FTPServer {
    pub fn new(path: PathBuf, bind_address: String, port: u32) -> Self {
        let address = format!("{bind_address}:{port}");
        let (sender, _) = broadcast::channel(1);

        FTPServer {
            sender,
            path,
            address,
        }
    }

    pub fn start(&self){
        let mut receiver_stop = self.sender.subscribe();

        let server = 
            Server::with_fs(self.path.clone())
                .passive_ports(50000..65535)
                .metrics()
                .shutdown_indicator(async move {
                    let _ = receiver_stop.recv().await;
                    info!("Shutting down FTP server");
                    //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                    options::Shutdown::new().grace_period(Duration::from_secs(10))
        });

        let address = self.address.clone();
        tokio::spawn(async move {
            let _ = server.listen(address).await;
        });
    }

    pub fn stop(&self){
        let _ = self.sender.send(false);
    }

}
