// Only mind errors for now
#![allow(warnings)]

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

    let shared_ftp_server_c0 = Arc::new(FTPServer::new(PathBuf::from("/tmp/"), 2121));
    let shared_ftp_server_c1 = shared_ftp_server_c0.clone();

    ui.on_start_ftp_server(move || {
        shared_ftp_server_c0.start();
    });
    
    ui.on_stop_ftp_server(move || {
        shared_ftp_server_c1.stop();
    });

    ui.run().unwrap();
}