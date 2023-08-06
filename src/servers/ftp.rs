use libunftp::{ServerError};
use unftp_sbe_fs::{ServerExt};
use log::info;
use std::path::PathBuf;

// pub struct FTPServer {
    // server: Server<Filesystem, DefaultUser>,
    // address: String,
// }

// impl FTPServer {

// pub async fn start_ftp_server(path: PathBuf, port: u32) -> impl Future<Output = Result<(), ServerError>>  {
pub async fn start_ftp_server(path: PathBuf, port: u32) -> Result<(), ServerError>  {
    let server = libunftp::Server::with_fs(path.clone()).passive_ports(50000..65535);
    let address = format!("127.0.0.1:{}", port);

    info!("Starting the FTP server at {}", address);
    server.listen(address.clone()).await
}

    // pub async fn start(&self) {
    //     info!("Starting the FTP server at {}", self.address);
    //     // self.server.listen(self.address.clone()).await.unwrap();
    // }
// }
