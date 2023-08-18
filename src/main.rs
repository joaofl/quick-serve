// Dev-only
#![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;

use log::{info, debug, error, logger};
use log::{Level, Metadata, Record, LevelFilter};

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);
use slint::SharedString;

mod servers { pub mod ftp; }
use servers::ftp::FTPServer;

mod utils;

mod logger;
use logger::MyLogger;

use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "debug");

    let (sender, mut receiver) = broadcast::channel(10);
    let logger = Box::new(MyLogger{sender});

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level

    let ui = AnyServeUI::new().unwrap();
    let ui_weak = ui.as_weak();

    let ftp_server = Arc::new(FTPServer::new());
    let ftp_server_clone = ftp_server.clone();

    let shared_ui = Arc::new(ui);
    let ui = shared_ui.clone();
    let ui_clone = shared_ui.clone();

    // Get logs printed at the text-box upon a log line print
    slint::spawn_local(async move {
        loop {
            match receiver.recv().await {
                Ok(text) => {
                    let t = ui_weak.unwrap().get_te_logs() + &text + "\n";
                    ui_weak.unwrap().set_te_logs(t);
                }
                Err(_) => {
                    println!("Something went wrong while receiving log message");
                    continue;
                }
            };
        };
    }).unwrap();


    ui.on_start_ftp_server(move || {
        // Read and validate the bind address
        let bind_address = ui_clone.get_le_bind_address().to_string();
        match utils::validate_ip_port(&bind_address) {
            Ok(()) => debug!("Valid IP:PORT: {:?}", bind_address),
            Err(error) => {
                error!("Validation error: {}", error);
                return;
            }
        }

        // Read and validate the path to be served
        let path = PathBuf::from(ui_clone.get_le_path().to_string());
        match utils::validate_path(&path) {
            Ok(()) => debug!("Valid path: {:?}", path),
            Err(error) => {
                error!("Validation error: {}", error);
                return;
            }
        }

        info!("Starting the server");
        ftp_server.start(path, bind_address);
    });
    
    ui.on_stop_ftp_server(move || {
        info!("Stopping the server");
        ftp_server_clone.stop();
    });

    debug!("Starting UI");
    ui.run().unwrap();
}
