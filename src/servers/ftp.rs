use libunftp::{Server, options};

use log::info;
use unftp_sbe_fs::ServerExt;

use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::broadcast;

// mod utils;
// use utils::validation;

pub struct FTPServer {
    stop_sender: tokio::sync::broadcast::Sender<bool>,
    status_sender: tokio::sync::broadcast::Sender<Result<String, String>>,
}

impl FTPServer {
    pub fn new() -> Self {
        let (stop_sender, _) = broadcast::channel(1);

        let (status_sender, _) = broadcast::channel(1);

        FTPServer {
            stop_sender,
            status_sender,
        }
    }

    pub fn start(&self, path: PathBuf, bind_address: String, port: i32) {
        let mut receiver_stop = self.stop_sender.subscribe();


        // match utils::validation::validate_ip_port(&bind_address) {
        //     Ok(()) => debug!("Valid IP:PORT: {:?}", bind_address),
        //     Err(error) => {
        //         error!("Validation error: {}", error);
        //         ui_weak.unwrap().invoke_is_connected(false);
        //         return;
        //     }
        // }

        // // Read and validate the dir path to be served
        // match utils::validation::validate_path(&path) {
        //     Ok(()) => debug!("Valid path: {:?}", path),
        //     Err(error) => {
        //         error!("Validation error: {}", error);
        //         ui_weak.unwrap().invoke_is_connected(false);
        //         return;
        //     }
        // }

        let server = 
            Server::with_fs(path.clone())
                .passive_ports(50000..65535)
                .metrics()
                .shutdown_indicator(async move {
                    let _ = receiver_stop.recv().await;
                    //Give 10 seconds to potential ongoing connections to finish, otherwise finish immediately
                    options::Shutdown::new().grace_period(Duration::from_secs(10))
        });

        let full_address = format!("{}:{}", bind_address, port);
        let status_sender_c1 = self.status_sender.clone();

        tokio::spawn(async move {
            info!("Connecting in the background to: {}", full_address);
            match server.listen(full_address).await {
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
        let _ = self.stop_sender.send(false);
    }

}
