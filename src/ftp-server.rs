use libunftp::auth::AnonymousAuthenticator;
use libunftp::ftp::FtpServer;
use libunftp::Server;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::runtime::Builder;

#[tokio::main]
async fn start_ftp_server() {
    let server_addr = SocketAddr::from_str("127.0.0.1:2121").unwrap();
    let listener = TcpListener::bind(&server_addr).await.unwrap();
    let authenticator = AnonymousAuthenticator;

    println!("FTP server listening on {:?}", server_addr);

    let server = FtpServer::new(authenticator);

    let mut runtime = Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    while let Ok((socket, _)) = listener.accept().await {
        let server = server.clone();
        runtime.spawn(async move {
            if let Err(err) = server.serve(socket).await {
                eprintln!("FTP server error: {:?}", err);
            }
        });
    }
}
