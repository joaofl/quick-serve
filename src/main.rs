// Dev-only
// #![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);


mod servers { pub mod ftp; }
use servers::ftp::FTPServer;

mod utils;

use log::{info, debug, error, LevelFilter};
mod logger;
use logger::MyLogger;


#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "debug");

    let (sender, mut receiver) = broadcast::channel(10);
    let logger = Box::new(MyLogger{sender});

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level

    let ui = AnyServeUI::new().unwrap();
    let ui_weak_textedit = ui.as_weak();
    let ui_weak_ftps = ui.as_weak();

    let ftp_server = Arc::new(FTPServer::new());
    let ftp_server_clone = ftp_server.clone();

    // Get logs printed at the text-box upon a log line print
    slint::spawn_local(async move {
        loop {
            match receiver.recv().await {
                Ok(text) => {

                    let log_line = format!("{}\n", &text);
                    let t = ui_weak_textedit.unwrap().get_te_logs() + &log_line;
                    
                    // TODO: make sure this "frame-margin" = 38 is true for all the OSs. Maybe calculate it in the begining
                    let old_pos = (ui_weak_textedit.unwrap().get_te_vh() + 38.0 - ui_weak_textedit.unwrap().get_te_h()) * -1.0;
                    ui_weak_textedit.unwrap().set_te_logs(t);
                    let new_pos = (ui_weak_textedit.unwrap().get_te_vh() + 38.0 - ui_weak_textedit.unwrap().get_te_h()) * -1.0;
                    //Only auto-scroll if already "glued" to the botton 
                    if ui_weak_textedit.unwrap().get_te_vy() == old_pos {
                        ui_weak_textedit.unwrap().set_te_vy(new_pos);
                    }

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
        let bind_address = ui_weak_ftps.unwrap().get_le_bind_address().to_string();
        match utils::validate_ip_port(&bind_address) {
            Ok(()) => debug!("Valid IP:PORT: {:?}", bind_address),
            Err(error) => {
                error!("Validation error: {}", error);
                return;
            }
        }

        // Read and validate the path to be served
        let path = PathBuf::from(ui_weak_ftps.unwrap().get_le_path().to_string());
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
