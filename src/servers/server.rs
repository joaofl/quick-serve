use std::path::PathBuf;
use log::info;
use tokio::sync::broadcast;

pub struct Server {
    command_sender: broadcast::Sender<(bool, PathBuf, String, i32)>,
}

impl Server {
    pub fn new() -> Self {
        let (command_sender, _) = broadcast::channel(1);

        Server {
            command_sender,
        }
    }

    pub fn start(&self, path: PathBuf, bind_address: String, port: i32) {
        let _ = self.command_sender.send((true, path, bind_address, port));
    }

    pub fn stop(&self){
        let _ = self.command_sender.send((false, PathBuf::new(), String::new(), 0));
    }

    pub async fn runner(&self) {
        // Get notified about the server's spawned task
        let mut command_receiver = self.command_sender.subscribe();

        loop {
            let (connect, path, bind_address, port) = 
                command_receiver.recv().await.unwrap();

            match connect {
                true => {
                    info!("Starting the HTTP server at {}:{}:{}", bind_address, port, path.to_string_lossy())
                    // Spin and await the actual server here
                    // the command_receiver to stop should be passed over here as well
                }
                false => {
                    info!("Stopping HTTP server")
                    // Send message to the spawn http server here, to
                    // gracefully shut it down
                }
            }
        }
    }
}
