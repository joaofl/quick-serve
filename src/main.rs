// #![allow(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![cfg_attr(not(feature = "ui"), allow(dead_code))]


use log::debug;
use log::{info, warn, LevelFilter};
use std::process::exit;

mod utils;
use utils::logger::*;

mod servers;
use crate::servers::*;

mod common;
use crate::common::*;

use clap::Parser;
use clap::ArgAction;

extern crate ctrlc;
extern crate core;

use tokio::time::{sleep, Duration};

#[cfg(feature = "ui")] mod ui;
#[cfg(feature = "ui")] use crate::ui::window::*;
#[cfg(feature = "ui")] use egui::{Style, Visuals};


#[derive(Parser, Debug)]
#[command(author, version, about = "Quick-Serve", long_about = "Instant file serving made easy")]
struct Cli {
    // Even with the GUI compiled, the headless arg remains
    // to prevent breaking scripts if the GUI versions gets
    // replaced by the headless. It has no effect on the actual 
    // headless
    #[arg(
        help = "Headless",
        long, required = false,
        action = ArgAction::SetTrue,
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
    )] serve_dir: String,

    #[arg(
        help = "Verbose logging",
        short, long, required = false,
        action = clap::ArgAction::Count,
    )] verbose: u8,

    #[arg(
        default_missing_value = Protocol::Http.get_default_port().to_string(),
        help = format!("Start the HTTP server [default port: {}]", Protocol::Http.get_default_port().to_string()),
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] http: Option<u32>,

    #[arg(
        default_missing_value = Protocol::Ftp.get_default_port().to_string(),
        help = format!("Start the FTP server [default port: {}]", Protocol::Ftp.get_default_port().to_string()),
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] ftp: Option<u32>,

    #[arg(
        default_missing_value = Protocol::Tftp.get_default_port().to_string(),
        help = format!("Start the TFTP server [default port: {}]", Protocol::Tftp.get_default_port().to_string()),
        long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] tftp: Option<u32>,
}


#[tokio::main]
async fn main() {
    let cli_args = Cli::parse();

    let mut log_level = LevelFilter::Info;
    if cli_args.verbose > 0 {
        log_level = LevelFilter::Debug;
    }

    let logger = Box::new(MyLogger::new(log_level));
    // Clone the producer, so that we can pass it to the consumer inside the UI
    #[cfg(feature = "ui")]
    let logs = logger.logs.clone();

    // Define the channel used to control the servers
    let channel: DefaultChannel<CommandMsg> = Default::default();

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level


    ////////////////////////////////////////////////////////////////////////
    // Spawn one thread per protocol and start waiting for command
    // to start or stop each server
    ////////////////////////////////////////////////////////////////////////
    for protocol in PROTOCOL_LIST {
        let mut rcv = channel.sender.subscribe();
        tokio::spawn(async move {
            loop {
                // debug!("Start waiting for messages");
                // debug!("Done waiting for message... Checking it now");
                let msg = rcv.recv().await.expect("Failed to receive message");
                if msg.protocol != *protocol {
                    debug!("\"not my business...\" said the {} thread", protocol.to_string());
                    continue;
                }

                if msg.start == true {
                    let server;

                    match msg.protocol {
                        Protocol::Http =>{
                            server = <Server as HTTPRunner>::new(msg.path.into(), msg.bind_ip, msg.port);
                        },
                        Protocol::Ftp =>{
                            server = <Server as FTPRunner>::new(msg.path.into(), msg.bind_ip, msg.port);
                        },
                        Protocol::Tftp =>{
                            server = <Server as TFTPRunner>::new(msg.path.into(), msg.bind_ip, msg.port);
                        },
                    } 

                    // Wait the receiver to listen before the sender sends the 1rst msg
                    // TODO: use some flag instead
                    sleep(Duration::from_millis(100)).await;
                    let _ = server.start();
                    info!("Started server");
                    
                    // Once started, wait for termination
                    let _msg = rcv.recv().await.unwrap();
                    
                    let _ = server.stop();
                    info!("Server stopped");
                }
            }
        });
    }

    ////////////////////////////////////////////////////////////////////////
    // Ctrl+c handler from here on
    ////////////////////////////////////////////////////////////////////////
    // TODO: only add handle if any server has been invoked
    // Add handle for Ctrl+C
    tokio::spawn(async move {
        ctrlc::set_handler(move || {
            warn!("Ctrl+C received. Closing connections and exiting.");
            //TODO: send a message to stop all servers and wait 10

            exit(1);
        }).expect("Error setting Ctrl+C handler");
        info!("Press Ctrl+C to exit.");
    });


    /////////////////////////////////////////////////////////////////////////
    //
    /////////////////////////////////////////////////////////////////////////
    let headless;
    #[cfg(feature = "ui")]{
        headless = cli_args.headless;
    }
    #[cfg(not(feature = "ui"))]{
        headless = true;
    }

    ////////////////////////////////////////////////////////////////////////
    // HEADLESS related code from here on
    ////////////////////////////////////////////////////////////////////////
    if headless {
        // Read and validate the bind address
        let bind_ip = cli_args.bind_ip;
        let path = cli_args.serve_dir;

        let mut count = 0u8;

        let mut cmd = CommandMsg {
            start: true,
            bind_ip,
            path,
            ..Default::default()
        };

        // CHeck for each server invoked from the command line, and send 
        // messages accordingly to start each
        if cli_args.http.is_some() {
            cmd.protocol = Protocol::Http;
            cmd.port = cli_args.http.unwrap() as u16;
            let _ = channel.sender.send(cmd.clone());
            count += 1;
        }

        if cli_args.ftp.is_some() {
            cmd.protocol = Protocol::Ftp;
            cmd.port = cli_args.ftp.unwrap() as u16;
            let _ = channel.sender.send(cmd.clone());
            count += 1;
        }

        if cli_args.tftp.is_some() {
            cmd.protocol = Protocol::Tftp;
            cmd.port = cli_args.tftp.unwrap() as u16;
            let _ = channel.sender.send(cmd.clone());
            count += 1;
        }

        if count == 0 {
            println!("No server specified. Use -h for help");
            exit(2);
        }
        else {
            // TODO: make this a feature: run for N seconds and exit
            // TODO: get some periodic stats as well
            loop {
                sleep(Duration::from_secs(60)).await;
            }

        }
    }
    ////////////////////////////////////////////////////////////////////////
    // UI related code from here on
    ////////////////////////////////////////////////////////////////////////
    #[cfg(feature = "ui")]{
    if ! headless {
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

    // futures::future::join_all(spawned_runners).await;
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
