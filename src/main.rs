#![allow(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::debug;
use log::{error, info, warn, LevelFilter};
// use tokio::sync::Mutex;
use std::{path::PathBuf, process::exit};
// use lazy_static::lazy_static;
// use std::ops::Deref;
// use std::sync::Arc;

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

// #[cfg(feature = "ui")] use tokio::sync::broadcast::{channel, Receiver, Sender};
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

    let mut log_level = LevelFilter::Info;
    if cli_args.verbose > 0 {
        log_level = LevelFilter::Debug;
    }

    let logger = Box::new(MyLogger::new(log_level));
    // Clone the producer, so that we can pass it to the consumer inside the UI
    let logs = logger.logs.clone();


    #[cfg(feature = "ui")]
    // Define the channel used to exchange with the UI
    let channel: DefaultChannel<CommandMsg> = Default::default();


    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////
    // debug!("\n{:#?}\n", cli_args);

    // lazy_static! {
    //     static ref SPAWNED_SERVERS: Arc<Mutex<Vec<Arc<Server>>>> = Arc::new(Mutex::new(vec![]));
    // }

    #[cfg(not(feature = "ui"))]
    let headless = true;

    ////////////////////////////////////////////////////////////////////////
    /// Spawn one thread per protocol and start waiting for command
    /// to start or stop each server
    ////////////////////////////////////////////////////////////////////////
    for protocol in &[Protocol::Http, Protocol::Tftp, Protocol::Ftp] {
        let mut rcv = channel.sender.subscribe();
        tokio::spawn(async move {
            loop {
                let msg = rcv.recv().await.unwrap();

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
                    let msg = rcv.recv().await.unwrap();
                    
                    let _ = server.stop();
                    info!("Server stopped");
                }
            }
        });
    }

    ////////////////////////////////////////////////////////////////////////
    // Ctrl+c handler from here on
    ////////////////////////////////////////////////////////////////////////

    // if headless && spawned_runners.iter().count() == 0 {
    //     error!("No server(s) specified. Run with -h for more info...");
    //     exit(1);
    // }

    // Set up a handler for Ctrl+C signal
    // let spawned_servers_c = &SPAWNED_SERVERS;

    // if cli_args.headless == false {
        
    // }

    // TODO: only add handle if any server has been invoked
    // Add handle for Ctrl+C
    tokio::spawn(async move {
        ctrlc::set_handler(move || {
            warn!("Ctrl+C received. Closing connections and exiting.");
            // Try to stop all servers gracefully
            // let spawned_servers_locked = spawned_servers_c.lock().await;
            // for server in spawned_servers_locked.iter() {
            //     server.stop();
            // }

            exit(1);
        }).expect("Error setting Ctrl+C handler");
        info!("Press Ctrl+C to exit.");
    });

    ////////////////////////////////////////////////////////////////////////
    // HEADLESS related code from here on
    ////////////////////////////////////////////////////////////////////////
    if cli_args.headless {
        // Read and validate the bind address
        let bind_ip = cli_args.bind_ip;
        let path = cli_args.serve_dir;

        let mut count = 0u8;

        // CHeck for each server invoked from the command line, and send 
        // messages accordingly to start each
        if cli_args.http.is_some() {
            // let port = cli_args.http.unwrap() as u16;
            // let mut cmd = CommandMsg::new(&Protocol::Http);
            // cmd.toggle = true;
            // cmd.port = cli_args.http.unwrap() as u16;;

            let cmd = CommandMsg {
                start: true,
                port: cli_args.http.unwrap() as u16,
                bind_ip,
                path: path,
                protocol: Protocol::Http,
                ..Default::default()
            };

            let _ = channel.sender.send(cmd);
            count += 1;
        }

        if count == 0 {
            println!("No server specified. Use -h to get help");
            exit(2);
        }
        else {
            // Run for 10 seconds...
            // TODO: make this a feature
            sleep(Duration::from_secs(10)).await;
        }
    }
    ////////////////////////////////////////////////////////////////////////
    // UI related code from here on
    ////////////////////////////////////////////////////////////////////////
    // #[cfg(feature = "ui")]{
    else {
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
