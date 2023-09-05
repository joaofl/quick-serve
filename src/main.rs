// #![allow(warnings)]

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

mod servers;
use servers::FTPServer;
use servers::HTTPServer;

mod utils;

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
                    
                    // TODO: it seems that there is a massive overhead because of the two lines below
                    // let t = format!("{}\n{}", ui_weak.unwrap().get_te_logs(), &log_line);
                    // ui_weak.unwrap().set_te_logs(t.into());

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
            let port = ui_weak.unwrap().get_sb_ftp_port();
            let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());
    
            info!("Starting FTP server");
            let _ = ftp_server.start(path, bind_address, port);
            // All the above calls are non-blocking code, whereas the status
            // is received as messages asynchronously.
        }
        else {
            info!("Stopping FTP server");
            ftp_server_c.stop();
            // ui_weak.unwrap().invoke_is_connected(false);
        }
    });

    // HTTP server starts here
    let http_server = Arc::new(HTTPServer::new());
    let http_server_c = http_server.clone();

    tokio::spawn(async move {
        http_server_c.runner().await;
    });

    let ui_weak = ui.as_weak();
    ui.on_startstop_http_server(move | connect | {
        match connect {
            true => {
                info!("Starting HTTP server");
                let bind_address = ui_weak.unwrap().get_le_bind_address().to_string();
                let port = ui_weak.unwrap().get_sb_http_port() as u16;
                let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());
    
                http_server.start(path, bind_address, port);
            }
            false => {
                info!("Stopping HTTP server");
                http_server.stop();
            }
        }
    });

    //Start UI
    ui.run().unwrap();
}
