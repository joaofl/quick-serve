#![allow(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::{error, info, warn, LevelFilter};

use std::path::PathBuf;
use std::ops::Deref;
use std::sync::Arc;

mod utils;
use utils::logger::*;

mod servers;
use crate::servers::{*};

use clap::Parser;
extern crate ctrlc;
extern crate core;

#[cfg(feature = "ui")] mod ui;
#[cfg(feature = "ui")] use crate::ui::window::UI;
#[cfg(feature = "ui")] use egui::{Style, Visuals};


#[derive(Parser, Debug)]
#[command(author, version, about = "Quick-Serve", long_about = "Instant file serving made easy")]
struct Cli {
    
    // If the UI gets compiled, give the option to run headless
    #[cfg(feature = "ui")]
    #[arg(
        help = "Headless",
        short = 'H', long, required = false,
    )] headless: bool,

    #[arg(
        help = "Bind IP",
        short, long, required = false,
        default_value = "127.0.0.1",
        value_name = "IP",
        require_equals = true,
    )] bind_ip: String,
    
    #[arg(
        help = "Path to serve",
        short = 'p', long, required = false,
        default_value = "/tmp/",
        value_name = "PATH",
        require_equals = true,
    )] serve_dir: PathBuf,

    #[arg(
        help = "Verbose logging",
        short, long, required = false,
        action = clap::ArgAction::Count,
    )] verbose: u8,

    #[arg(
        default_missing_value = "8080",
        help = "Start the HTTP server [default port: 8080]",
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] http: Option<u32>,

    #[arg(
        default_missing_value = "2121",
        help = "Start the FTP server [default port: 2121]",
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] ftp: Option<u32>,

    #[arg(
        default_missing_value = "6969",
        help = "Start the TFTP server [default port: 6969]",
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] tftp: Option<u32>,
}


#[tokio::main]
async fn main() {
    let cli_args = Cli::parse();

    // let mut log_level = "info";
    // if cli_args.verbose > 0 {
    //     log_level = "debug";
    // }

    let logger = Box::new(MyLogger::new());
    // Clone the producer, so that we can pass it to the consumer
    // in the UI
    let logs = logger.logs.clone();

    // ::std::env::set_var("RUST_LOG", log_level);
    // env_logger::builder()
    //     .format_timestamp_secs();

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////
    // debug!("\n{:#?}\n", cli_args);

    let mut spawned_runners = vec![];
    let mut spawned_servers = vec![];

    // Read and validate the bind address
    let bind_ip = cli_args.bind_ip;
    let path = cli_args.serve_dir;

    #[cfg(not(feature = "ui"))]
    let headless = true;
    #[cfg(feature = "ui")]
    let headless = cli_args.headless;


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // TFTP from here on
    // 
    // Spin the runners to wait for any potential server start
    if cli_args.tftp.is_some() {
        // || headless == false
        // loop {
        //     // Enter loop which wait for messages to either start or stop the servers

        //     if headless == false {
        //         // when using the GUI, wait for the turn-on event
        //         // message here
        //     }

            // TODO:in case of the gui version, these values should come from the UI
            let port = cli_args.tftp.unwrap() as u16;
            let tftp_server = Arc::new(<Server as TFTPServerRunner>::new(path.clone(), bind_ip.clone(), port));
            let tftp_server_c = tftp_server.clone();

            spawned_servers.push(tftp_server.clone());
            spawned_runners.push(
                tokio::spawn(async move {
                    TFTPServerRunner::runner(tftp_server).await
                })
            );

            let _port = cli_args.tftp.unwrap() as u16;
            let _ = tftp_server_c.start();

            // if headless {
            //     // exit the loop after running once
            //     break;
            // }

            // tftp_server_c.terminate();
            // del(tftp_server);
        // }

    }


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // FTP from here on
    // 
    // Spin the runners to wait for any potential server start
    if cli_args.ftp.is_some() {
        let port = cli_args.ftp.unwrap() as u16;
        let ftp_server = Arc::new(<Server as FTPRunner>::new(path.clone(), bind_ip.clone(), port));
        let ftp_server_c = ftp_server.clone();

        spawned_servers.push(ftp_server.clone());
        spawned_runners.push(
            tokio::spawn(async move {
                FTPRunner::runner(ftp_server.deref()).await
            })
        );

        spawned_runners.push(
            tokio::spawn(async move {
                let _ = ftp_server_c.start();
            })
        );
    }


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // HTTP from here on
    // 
    // Spin the runners to wait for any potential server start
    if cli_args.http.is_some() {
        let port = cli_args.http.unwrap() as u16;
        let server = Arc::new(<Server as HTTPRunner>::new(path.clone(), bind_ip.clone(), port));
        let server_c = server.clone();

        spawned_servers.push(server.clone());
        spawned_runners.push(
            tokio::spawn(async move {
                HTTPRunner::runner(server).await
            })
        );
        spawned_runners.push(
            tokio::spawn(async move {
                let _ = server_c.start();
            })
        );
    }

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////

    if headless && spawned_runners.iter().count() == 0 {
        error!("No server(s) specified. Run with -h for more info...");
        return;
    }

    // Set up a handler for Ctrl+C signal
    ctrlc::set_handler(move || {
        // Handle Ctrl+C signal here
        warn!("Ctrl+C received. Closing connections and exiting.");
        // Perform cleanup operations here before exiting
        for server in &mut spawned_servers {
            server.terminate();
        }

    }).expect("Error setting Ctrl+C handler");
    info!("Press Ctrl+C to exit.");

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////
    // let headless = true;
    #[cfg(feature = "ui")]{
        if cli_args.headless == false {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([700.0, 800.0]),
                    ..Default::default()
            };

            let _ = eframe::run_native(
                "Quick-Serve",
                options,
                Box::new(|cc| {
                    let style = Style {
                        visuals: Visuals::light(),
                        ..Style::default()
                    };
                    cc.egui_ctx.set_style(style);
                    let mut ui = UI::new(cc);
                    ui.logs = logs;
                    Box::new(ui)
                }),
            );
        }
    }

    futures::future::join_all(spawned_runners).await;
    return;
}

#[cfg(test)]
mod tests {
    use predicates::prelude::*;
    use assert_cmd::Command;
    pub mod common;

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("quick-serve").unwrap();
        cmd.arg("--help");
        cmd.assert().success().stdout(predicate::str::contains("Usage: quick-serve"));
    }
}
