use libunftp::{Server, options};

use unftp_sbe_fs::ServerExt;

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::broadcast;


pub struct FTPServer {
    sender: tokio::sync::broadcast::Sender<bool>,
    status_sender: tokio::sync::broadcast::Sender<Result<String, String>>,
}

impl FTPServer {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);

        let (status_sender, _) = broadcast::channel(1);

        FTPServer {
            sender,
            status_sender,
        }
    }

    pub fn start(&self, path: PathBuf, bind_address: String) {
        let mut receiver_stop = self.sender.subscribe();

        let server = 
            Server::with_fs(path.clone())
                .passive_ports(50000..65535)
                .metrics()
                .shutdown_indicator(async move {
                    let _ = receiver_stop.recv().await;
                    //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                    options::Shutdown::new().grace_period(Duration::from_secs(10))
        });

        let bind_address = bind_address.clone();
        let status_sender_c1 = self.status_sender.clone();

        tokio::spawn(async move {
            match server.listen(bind_address).await {
                Ok(()) => { 
                    let _ = status_sender_c1.send(Ok("Successfully finished".to_string())); 
                }
                Err(e) => {
                    let _ = status_sender_c1.send(Err(format!("Error starting the server {}", e.to_string())));
                }
            };
        });
        // return;
    }

    pub async fn check(&self) -> Result<String, String> {
        // Get notified about the server's spawned task
        let mut status_receiver = self.status_sender.subscribe();
        return status_receiver.recv().await.unwrap();
    }

    pub fn stop(&self){
        let _ = self.sender.send(false);
    }

}
