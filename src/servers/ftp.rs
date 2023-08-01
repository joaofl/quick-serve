use libunftp::ServerError;
use log::info;
use std::path::{PathBuf};
use unftp_sbe_fs::ServerExt;

pub async fn start_ftp_server(path: PathBuf, port: u32) -> Result<(), ServerError> {
    info!("Starting the FTP server");

    // let ftp_home = std::env::temp_dir();
    let server = libunftp::Server::with_fs(path).passive_ports(50000..65535);

    let address = format!("127.0.0.1:{}", port);

    // server.listen(address).await
    server.listen(address).await
}