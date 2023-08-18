// Dev-only
// #![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, debug, error};
use tracing_subscriber::prelude::*;

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);

mod servers { pub mod ftp; }
use crate::servers::ftp::FTPServer;

mod utils;
mod subscriber;

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "debug");

    let ui = AnyServeUI::new().unwrap();

    let ftp_server = Arc::new(FTPServer::new());
    let ftp_server_clone = ftp_server.clone();

    let shared_ui = Arc::new(ui);
    let ui = shared_ui.clone();
    let ui_clone = shared_ui.clone();
    
    // let ui_clone2 = shared_ui.clone();
    // ui_clone2.set_te_logs(SharedString::from("Whatever \n"));

    let subscriber = subscriber::MySubscriber::new();
    let mut receiver = subscriber.sender.subscribe();
    subscriber.init();

    tokio::spawn(async move {
        loop {
            let txt = receiver.recv().await;
            println!("Event: {}", txt.unwrap());
            // Not sure how to change the UI from here
        }
    });


    ui.on_start_ftp_server(move || {
        let bind_address = ui_clone.get_le_bind_address().to_string();
        
        match utils::validate_ip_port(&bind_address) {
            Ok(()) => debug!("Valid IP:PORT: {:?}", bind_address),
            Err(error) => {
                error!("Validation error: {}", error);
                return;
            }
        }

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
