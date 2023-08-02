use libunftp::ServerError;
use log::info;
use std::path::{PathBuf};
use unftp_sbe_fs::ServerExt;

pub struct FTPServer {
    // server: libunftp::Server,
    path: PathBuf,
    port: u32,
}

impl FTPServer {

    pub fn new(path: PathBuf, port: u32) -> Self {
        info!("Starting the FTP server");
        return FTPServer { path, port };
    }

    pub async fn start(&self) -> Result<(), ServerError> {
        // let ftp_home = std::env::temp_dir();
        let server = libunftp::Server::with_fs(self.path.clone()).passive_ports(50000..65535);
        let address = format!("127.0.0.1:{}", self.port);

        server.listen(address).await
    }
}