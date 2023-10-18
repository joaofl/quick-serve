#![allow(warnings)]

slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);

use log::{info, error, debug, LevelFilter};

use std::path::PathBuf;
use std::ops::Deref;
use std::sync::Arc;
use std::{env, process};

use tokio::sync::broadcast;

mod tests;
mod utils;
use utils::logger::MyLogger;
mod servers;
use crate::servers::{*};

#[tokio::main]
async fn main() {
    // ::std::env::set_var("RUST_LOG", "debug");

    let (sender, mut receiver) = broadcast::channel(10);
    let logger = Box::new(MyLogger{sender});

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level

    let ui = AnyServeUI::new().unwrap();

    //
    // Get every new log line printed into the text-box
    // TODO: this likely goes better into logger.rs
    let ui_weak = ui.as_weak();

    slint::spawn_local(async move {
        loop {
            match receiver.recv().await {
                Ok(log_line) => {
                    // Make this a filter into the UI. Maybe from a list?
                    // Fact is that this text edit from slint cannot deal with too
                    // many lines. The whole application freezes.
                    if log_line.contains("any_serve") {
                        ui_weak.unwrap().invoke_add_log_line((&log_line).into());
                    }

                    if (ui_weak.unwrap().get_cb_auto_scroll()) {
                        ui_weak.unwrap().invoke_textedit_scroll_to_end();
                    }
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

    ui.on_show_open_dialog(move || {
        let d = utils::file_dialog::show_open_dialog(PathBuf::from("/tmp/"));
        debug!("Selected {}", d.display());

        ui_weak.unwrap().set_le_path(d.to_str().unwrap().into());
    });


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // TFTP from here on
    // 
    // Spawn task along with its local clones
    let tftp_server = Arc::new(<Server as TFTPServerRunner>::new());
    let tftp_server_c = tftp_server.clone();
    let ui_weak = ui.as_weak();

    tokio::spawn(async move {
        TFTPServerRunner::runner(tftp_server_c.deref()).await
    });

    // // Unable to make async calls inside the closure below
    ui.on_startstop_tftp_server(move |connect| {
        if connect {
            // Read and validate the bind address
            let bind_address = ui_weak.unwrap().get_le_bind_address().to_string();
            let port = ui_weak.unwrap().get_sb_tftp_port() as u16;
            let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());

            match tftp_server.start(path, bind_address, port) {
                Ok(result) => {
                    info!("tftp server started successfully");
                }
                Err(error) => {
                    error!("Issue while starting tftp server: {}", error);
                    //Uncheck button
                    ui_weak.unwrap().set_bt_start_tftp(false);
                }
            }
        }
        else {
            tftp_server.stop();
            // Unblock UI elements if no other connection exists
        }
    });

    ///////////////////////////////////////////////////////////////////////////////
    // 
    // DHCP from here on
    // 
    // Spawn task along with its local clones
    let dhcp_server = Arc::new(<Server as DHCPServerRunner>::new());
    let dhcp_server_c = dhcp_server.clone();
    let ui_weak = ui.as_weak();

    tokio::spawn(async move {
        DHCPServerRunner::runner(dhcp_server_c.deref()).await
    });

    // // Unable to make async calls inside the closure below
    ui.on_startstop_dhcp_server(move |connect| {
        if connect {
            // Read and validate the bind address
            let bind_address = ui_weak.unwrap().get_le_bind_address().to_string();
            let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());

            match dhcp_server.start(path, bind_address, 0) {
                Ok(result) => {
                    info!("dhcp server started successfully");
                }
                Err(error) => {
                    error!("Issue while starting dhcp server: {}", error);
                    //Uncheck button
                    ui_weak.unwrap().set_bt_start_dhcp(false);
                }
            }
        }
        else {
            dhcp_server.stop();
        }
    });


    ///////////////////////////////////////////////////////////////////////////////
    //
    // FTP Server hereon
    // 
    // Spawn task along with its local clones
    let ftp_server = Arc::new(<Server as FTPServerRunner>::new());
    let ftp_server_c = ftp_server.clone();
    let ui_weak = ui.as_weak();

    tokio::spawn(async move {
        FTPServerRunner::runner(ftp_server_c.deref()).await
    });

    // Unable to make async calls inside the closure below
    ui.on_startstop_ftp_server(move |connect| {
        if connect {
            // Read and validate the bind address
            let bind_address = ui_weak.unwrap().get_le_bind_address().to_string();
            let port = ui_weak.unwrap().get_sb_ftp_port() as u16;
            let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());

            match ftp_server.start(path, bind_address, port) {
                Ok(result) => {
                    info!("ftp server started successfully");
                }
                Err(error) => {
                    error!("Issue while starting ftp server: {}", error);
                    //Uncheck button
                    ui_weak.unwrap().set_bt_start_ftp(false);
                }
            }
        }
        else {
            ftp_server.stop();
        }
    });

    ///////////////////////////////////////////////////////////////////////////////
    //
    // HTTPServer hereon
    //
    let http_server = Arc::new(<Server as HTTPServerRunner>::new());
    let http_server_c = http_server.clone();
    let ui_weak = ui.as_weak();

    tokio::spawn(async move {
        HTTPServerRunner::runner(http_server_c.deref()).await
    });

    ui.on_startstop_http_server(move | connect | {
        if connect {
            let path = PathBuf::from(ui_weak.unwrap().get_le_path().to_string());
            let bind_address = ui_weak.unwrap().get_le_bind_address().to_string();
            let port = ui_weak.unwrap().get_sb_http_port() as u16;

            match http_server.start(path, bind_address, port) {
                Ok(result) => {
                    info!("http server started successfully");
                }
                Err(error) => {
                    error!("Issue while starting http server: {}", error);
                    //Uncheck button
                    ui_weak.unwrap().set_bt_start_http(false);
                }
            }
        }
        else {
            http_server.stop();
        }
    });


    // TODO: Start UI if no command line is set, 
    // otherwise, interpret the command, and run the desired stuff
    ui.run().unwrap();
}
