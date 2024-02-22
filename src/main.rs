// #![allow(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::{error, info, warn, LevelFilter};
use std::{path::PathBuf, process::exit};
// use std::ops::Deref;
use std::sync::Arc;

mod utils;
use utils::logger::*;

mod servers;
use crate::servers::{*};

use clap::Parser;
extern crate ctrlc;
extern crate core;

#[cfg(feature = "ui")] use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
#[cfg(feature = "ui")] mod ui;
#[cfg(feature = "ui")] use crate::ui::window::*;
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
        help = "Directory to serve",
        short = 'd', long, required = false,
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


    #[cfg(feature = "ui")]
    // Define the channel used to exchange with the UI
    let channel: DefaultChannel<UIEvent> = Default::default();

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



    ////////////////////////////////////////////////////////////////////////
    // HTTP from here on
    ////////////////////////////////////////////////////////////////////////
    // if cli_args.http.is_some() {
    // let port = cli_args.http.unwrap() as u16;
    let port = 8080;
    let server = Arc::new(<Server as HTTPRunner>::new(path.clone(), bind_ip.clone(), port));

    spawned_servers.push(server.clone());
    // spawned_runners.push(HTTPRunner::runner(http_server.clone()).await);

    let server_c = server.clone();
    spawned_runners.push(
        tokio::spawn(async move {
            HTTPRunner::runner(server_c).await
        })
    );

    // Receive from UI
    let server_c = server.clone();
    let mut receiver_clone = channel.sender.subscribe();
    let _receiver_task = tokio::spawn(async move {
        loop {
            let (proto, bind_ip, path) = receiver_clone.recv().await.unwrap();

            if proto.toggle == true {
                // let s = Arc::new(<Server as HTTPRunner>::new(path.into(), bind_ip, proto.port));
                let _ = server_c.start();
                info!("Started server");
            }
            else {
                let _ = server_c.terminate();
                info!("Server terminated");
            }
        }
    });

    ////////////////////////////////////////////////////////////////////////
    // TFTP from here on
    ////////////////////////////////////////////////////////////////////////
    if cli_args.tftp.is_some() {
        // TODO:in case of the gui version, these values should come from the UI
        let port = cli_args.tftp.unwrap() as u16;
        let tftp_server = Arc::new(<Server as TFTPRunner>::new(path.clone(), bind_ip.clone(), port));

        spawned_servers.push(tftp_server.clone());
        spawned_runners.push(TFTPRunner::runner(tftp_server.clone()).await);

        let _ = tftp_server.start();
    }

    ////////////////////////////////////////////////////////////////////////
    // FTP from here on
    ////////////////////////////////////////////////////////////////////////
    if cli_args.ftp.is_some() {
        let port = cli_args.ftp.unwrap() as u16;
        let ftp_server = Arc::new(<Server as FTPRunner>::new(path.clone(), bind_ip.clone(), port));

        spawned_servers.push(ftp_server.clone());
        spawned_runners.push(FTPRunner::runner(ftp_server.clone()).await);

        let _ = ftp_server.start();
    }

    ////////////////////////////////////////////////////////////////////////
    // Ctrl+c handler from here on
    ////////////////////////////////////////////////////////////////////////

    if headless && spawned_runners.iter().count() == 0 {
        error!("No server(s) specified. Run with -h for more info...");
        exit(1);
    }

    // Set up a handler for Ctrl+C signal
    ctrlc::set_handler(move || {
        // Handle Ctrl+C signal here
        warn!("Ctrl+C received. Closing connections and exiting.");
        // Perform cleanup operations here before exiting
        for server in &mut spawned_servers {
            server.terminate();
        }

        exit(2);

    }).expect("Error setting Ctrl+C handler");
    info!("Press Ctrl+C to exit.");

    ////////////////////////////////////////////////////////////////////////
    // UI related code from here on
    ////////////////////////////////////////////////////////////////////////
    #[cfg(feature = "ui")]{
        if cli_args.headless == false {

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([900.0, 800.0]),
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

                    ui.channel.sender = channel.sender;
                    Box::new(ui)
                }),
            );
        }
    }

    futures::future::join_all(spawned_runners).await;

    exit(0);
}



////////////////////////////////////////////////////////////////////////
// TESTS
////////////////////////////////////////////////////////////////////////
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
