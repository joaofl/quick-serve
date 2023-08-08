use libunftp::{Server, ServerError};
use libunftp::auth::DefaultUser;
use unftp_sbe_fs::{Filesystem, ServerExt};
use log::{debug, info};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use std::thread;
use std::time::Duration;

pub struct FTPServer {
    server: Server<Filesystem, DefaultUser>,
    // server: Arc<Server<Filesystem, DefaultUser>>,
    address: String,
    // path: PathBuf,
    shutdown_sender: tokio::sync::broadcast::Sender<()>,
    // shutdown_receiver: tokio::sync::broadcast::Receiver<()>,
}

impl FTPServer {

    pub fn new(path: PathBuf, port: u32) -> Self {
        debug!("Created");

        let (sender, mut receiver) = broadcast::channel(1);

        let server = libunftp::Server::with_fs(path.clone())
        .passive_ports(50000..65535)
        .metrics()
        .shutdown_indicator(async move {
            receiver.recv().await;
            info!("Shutting down FTP server");
            libunftp::options::Shutdown::new().grace_period(Duration::from_secs(11))
        });

        FTPServer {
            // server: Arc::new(server),
            server: server,
            address: format!("127.0.0.1:{}", port),
            shutdown_sender: sender,
        }
    }

    pub async fn start(&self) {
        info!("Starting the FTP server at {}", self.address);

        let address_c = self.address.clone();

        // let server_arc = self.server;
        // let loc = self.clone();

        // tokio::spawn(async move {
        self.server.listen(address_c).await;
        debug!("Stopped listening...");
        // });

        debug!("Done starting...");
    }

    pub fn stop(&self) {
        info!("Stopping the FTP server at {}", self.address);
        self.shutdown_sender.send(());
    }

}



// // starts the FTP server as a Tokio task.
// fn start_ftp(
//     log: &Logger,
//     root_log: &Logger,
//     m: &clap::ArgMatches,
//     shutdown: tokio::sync::broadcast::Receiver<()>,
//     done: tokio::sync::mpsc::Sender<()>,
// ) -> Result<(), String> {
//     let event_dispatcher =
//         notify::create_event_dispatcher(Arc::new(log.new(o!("module" => "storage"))), m)?;

//     match m.value_of(args::STORAGE_BACKEND_TYPE) {
//         None | Some("filesystem") => start_ftp_with_storage(
//             log,
//             root_log,
//             m,
//             fs_storage_backend(root_log, m),
//             event_dispatcher,
//             shutdown,
//             done,
//         ),
//         Some(x) => Err(format!("unknown storage back-end type {}", x)),
//     }
// }
