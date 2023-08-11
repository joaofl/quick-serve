// Dev-only
// #![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;
use log::{info, debug};

slint::slint!(import { AnyServeUI } from "src/ui.slint";);

mod servers { pub mod ftp; }
use crate::servers::ftp::FTPServer;

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Starting my shapp");

    let ui = AnyServeUI::new().unwrap();

    let path = PathBuf::from("/tmp/");
    let bind_address = format!("127.0.0.1");

    let ftp_server = Arc::new(FTPServer::new(path, bind_address, 2121));
    let ftp_server_clone = ftp_server.clone();

    ui.on_start_ftp_server(move || {
        info!("Starting the server");
        ftp_server.start();
    });
    
    ui.on_stop_ftp_server(move || {
        info!("Stopping the server");
        ftp_server_clone.stop();
    });

    debug!("Starting UI");
    ui.run().unwrap();
}
