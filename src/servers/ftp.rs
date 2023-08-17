use libunftp::{Server, options};

use unftp_sbe_fs::ServerExt;
use log::info;

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::broadcast;

// std::mem::drop;

pub struct FTPServer {
    sender: tokio::sync::broadcast::Sender<bool>,
}

impl FTPServer {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);

        FTPServer {
            sender,
        }
    }

    pub fn start(&self, path: PathBuf, bind_address: String){
        let mut receiver_stop = self.sender.subscribe();

        let server = 
            Server::with_fs(path.clone())
                .passive_ports(50000..65535)
                .metrics()
                .shutdown_indicator(async move {
                    let _ = receiver_stop.recv().await;
                    info!("Shutting down FTP server");
                    //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                    options::Shutdown::new().grace_period(Duration::from_secs(10))
        });

        let bind_address = bind_address.clone();
        tokio::spawn(async move {
            let _ = server.listen(bind_address).await;
        });
    }

    pub fn stop(&self){
        let _ = self.sender.send(false);
    }

}
