// Dev-only
#![allow(warnings)]

use std::path::PathBuf;
use std::sync::Arc;
use slint::SharedString;
use tokio::sync::broadcast;

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);


mod servers { pub mod ftp; }
use servers::ftp::FTPServer;

mod utils { pub mod file_dialog; pub mod validation;}
use utils::{file_dialog, validation};

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
    let ui_weak_opendir = ui.as_weak();

    let ftp_server = Arc::new(FTPServer::new());
    let ftp_server_clone = ftp_server.clone();

    // Get logs printed at the text-box upon a log line print
    slint::spawn_local(async move {
        loop {
            match receiver.recv().await {
                Ok(text) => {

                    let log_line = format!("{}\n", &text);
                    let t = ui_weak_textedit.unwrap().get_te_logs() + &log_line;
                    
                    // TODO: move this logic to the ui itself. From here, on set `botton`
                    // TODO: make sure this "frame-margin" = 38 is true for all the OSs. Maybe calculate it in the begining
                    let old_pos = (ui_weak_textedit.unwrap().get_textedit_viewport_height() + 38.0 - ui_weak_textedit.unwrap().get_textedit_height()) * -1.0;
                    ui_weak_textedit.unwrap().set_te_logs(t);
                    let new_pos = (ui_weak_textedit.unwrap().get_textedit_viewport_height() + 38.0 - ui_weak_textedit.unwrap().get_textedit_height()) * -1.0;
                    //Only auto-scroll if already "glued" to the botton 
                    if ui_weak_textedit.unwrap().get_textedit_viewport_y() == old_pos {
                        ui_weak_textedit.unwrap().set_textedit_viewport_y(new_pos);
                    }

                }
                Err(_) => {
                    println!("Something went wrong while receiving log message");
                    continue;
                }
            };
        };
    }).unwrap();

    ui.on_show_open_dialog(move || {
        let d = utils::file_dialog::show_open_dialog(PathBuf::from("/tmp/"));
        debug!("Selected {}", d.display());

        ui_weak_opendir.unwrap().set_le_path(d.to_str().unwrap().into());
    });

    ui.on_startstop_ftp_server(move |connect| {
        // Read and validate the bind address

        if connect {
            let bind_address = ui_weak_ftps.unwrap().get_le_bind_address().to_string();
            match utils::validation::validate_ip_port(&bind_address) {
                Ok(()) => debug!("Valid IP:PORT: {:?}", bind_address),
                Err(error) => {
                    error!("Validation error: {}", error);
                    ui_weak_ftps.unwrap().invoke_is_connected(false);
                    return;
                }
            }
    
            // Read and validate the path to be served
            let path = PathBuf::from(ui_weak_ftps.unwrap().get_le_path().to_string());
            match utils::validation::validate_path(&path) {
                Ok(()) => debug!("Valid path: {:?}", path),
                Err(error) => {
                    error!("Validation error: {}", error);
                    ui_weak_ftps.unwrap().invoke_is_connected(false);
                    return;
                }
            }
    
            info!("FTP: starting server");
            match ftp_server.start(path, bind_address) {
                Ok(()) => {
                    info!("FTP: server started");
                    ui_weak_ftps.unwrap().invoke_is_connected(true);
                    return;
                }
                Err(_error) => {
                    error!("FTP: failed to start");
                    ui_weak_ftps.unwrap().invoke_is_connected(false);
                    return;
                }
            }
        }
        else {
            info!("FTP: stopping server");
            ftp_server_clone.stop();
            ui_weak_ftps.unwrap().invoke_is_connected(false);
            return;
        }

    });

    //Start UI
    ui.run().unwrap();
}
