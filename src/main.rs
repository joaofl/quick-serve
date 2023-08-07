// Only mind errors for now
// #![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;
use log::{info, debug};

slint::slint!(import { AnyServeUI } from "src/ui.slint";);

mod servers { pub mod ftp; }
use crate::servers::ftp::FTPServer;


fn main() {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Starting my shapp");

    let ui = AnyServeUI::new().unwrap();

    let shared_ftp_server = Arc::new(FTPServer::new(PathBuf::from("/tmp/"), 2121));
    let ftp_server_c1 = Arc::clone(&shared_ftp_server);
    let ftp_server_c2 = Arc::clone(&shared_ftp_server);

    ui.on_start_ftp_server(move || {
        ftp_server_c1.start();
    });
    
    ui.on_stop_ftp_server(move || {
        ftp_server_c2.stop();
    });

    ui.run().unwrap();
}