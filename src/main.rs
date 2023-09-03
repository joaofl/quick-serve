// Dev-only
// #![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);


mod servers { pub mod ftp; }
use servers::ftp::FTPServer;

mod utils { pub mod file_dialog; pub mod validation;}

use log::{info, debug, error, LevelFilter};
mod logger;
use logger::MyLogger;


#[tokio::main]
async fn main() {
    // ::std::env::set_var("RUST_LOG", "debug");

    let (sender, mut receiver) = broadcast::channel(10);
    let logger = Box::new(MyLogger{sender});

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level

    let ui = AnyServeUI::new().unwrap();
    let ftp_server = Arc::new(FTPServer::new());

    //
    // Get every new log line printed into the text-box
    // TODO: this likely goes better into logger.rs
    let ui_weak = ui.as_weak();

    slint::spawn_local(async move {
        loop {
            match receiver.recv().await {
                Ok(log_line) => {
                    let t = format!("{}\n{}", ui_weak.unwrap().get_te_logs(), &log_line);
                    ui_weak.unwrap().set_te_logs(t.into());

                    //Check if currently scrolled to bottom
                    // let is_glued = ui_weak.unwrap().invoke_is_glued();

                    //Only auto-scroll if already "glued" to the bottom
                    // if (is_glued) {
                        ui_weak.unwrap().invoke_textedit_scroll_to_end();
                    // }
                }
                Err(_) => {
                    println!("Something went wrong while receiving a log message");
                    continue;
                }
            };
        };
    }).unwrap();


    //
    // Spawn task along with its local clones
    let ui_weak = ui.as_weak();
    let ftp_server_c = ftp_server.clone();

    slint::spawn_local(async move {
        loop {
            match ftp_server_c.check().await {
                Ok(msg) => {
                    info!("#{}", msg)
                }
                Err(msg) => {
                    ftp_server_c.stop();
                    ui_weak.unwrap().invoke_is_connected(false);
                    error!("#{}", msg)
                }
            }
        }
    }).expect("Failed to spawn FTP status checker");


    //
    // Spawn task along with its local clones
    let ui_weak = ui.as_weak();

    ui.on_show_open_dialog(move || {
        let d = utils::file_dialog::show_open_dialog(PathBuf::from("/tmp/"));
        debug!("Selected {}", d.display());

        ui_weak.unwrap().set_le_path(d.to_str().unwrap().into());
    });


    //
    // Spawn task along with its local clones
    let ui_weak = ui.as_weak();
    let ftp_server_c = ftp_server.clone();

    // Unable to make async calls inside the closure below
    ui.on_startstop_ftp_server(move |connect| {
        if connect {
            // Read and validate the bind address
            // Get IP and port: TODO
            let bind_address = ui_weak.unwrap().get_le_bind_address().to_string();
            match utils::validation::validate_ip_port(&bind_address) {
                Ok(()) => debug!("Valid IP:PORT: {:?}", bind_address),
                Err(error) => {
                    error!("Validation error: {}", error);
                    ui_weak.unwrap().invoke_is_connected(false);
                    return;
                }
            }
            // Read and validate the dir path to be served
            let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());
            match utils::validation::validate_path(&path) {
                Ok(()) => debug!("Valid path: {:?}", path),
                Err(error) => {
                    error!("Validation error: {}", error);
                    ui_weak.unwrap().invoke_is_connected(false);
                    return;
                }
            }
    
            info!("Starting FTP server");
            ftp_server.start(path, bind_address);
            // All the above calls are non-blocking code, whereas the status
            // is received as messages asynchronously.
        }
        else {
            info!("Stopping FTP server");
            ftp_server_c.stop();
            ui_weak.unwrap().invoke_is_connected(false);
            return;
        }

    });

    //Start UI
    ui.run().unwrap();
}
