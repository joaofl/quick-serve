#![allow(warnings)]

// slint::slint!(import { AnyServeUI } from "src/ui/ui.slint";);

use hyper::client::connect;
use log::{info, warn, error, debug, LevelFilter};

use std::path::PathBuf;
use std::ops::Deref;
use std::sync::Arc;
use std::{env, process};

use tokio::sync::broadcast;
use futures::future::join_all;

mod tests;
mod utils;
mod servers;
use crate::servers::{*};

use clap::{Args, Parser, Subcommand};
extern crate ctrlc;

#[derive(Parser, Debug)]
#[command(author, version, about = "Any-serve", long_about = "Developers swiss-knife of quick file serving")]
struct Cli {
    
    #[arg(
        help = "Bind IP",
        short, long, required = false,
        default_value = "127.0.0.1",
        value_name = "IP",
        require_equals = true,
    )] bind_ip: String,
    
    #[arg(
        help = "Directory to serve",
        short, long, required = false,
        default_value = "/tmp/",
        value_name = "DIR",
        require_equals = true,
    )] serve_dir: PathBuf,

    #[arg(
        help = "Verbose logging",
        short, long, required = false,
        action = clap::ArgAction::Count,
    )] verbose: u8,

    #[arg(
        help = "Start the DHCP server",
        short, long, required = false, 
        num_args = 0,
    )] dhcp: bool,

    #[arg(
        default_missing_value = "8080",
        help = "Start the HTTP server [default port: 8080]",
        short = 'H', long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] http: Option<u32>,

    #[arg(
        default_missing_value = "2121",
        help = "Start the FTP server [default port: 2121]",
        short, long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] ftp: Option<u32>,

    #[arg(
        default_missing_value = "6969",
        help = "Start the TFTP server [default port: 6969]",
        short, long, required = false, 
        num_args = 0..=1,
        require_equals = true,
        value_name = "PORT",
    )] tftp: Option<u32>,
}


#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::builder()
        .format_timestamp_secs()
        .init();

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////
    let cli_args = Cli::parse();
    println!("{:#?}", cli_args);

    let mut spawned_servers = vec![];
    let mut servers = vec![];

    // Read and validate the bind address
    let bind_ip = cli_args.bind_ip;
    let path = cli_args.serve_dir;


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // TFTP from here on
    // 
    // Spin the runners to wait for any potential server start
    let tftp_server = Arc::new(<Server as TFTPServerRunner>::new());
    let tftp_server_c = tftp_server.clone();

    servers.push(tftp_server.clone());

    if cli_args.tftp.is_some() {
        spawned_servers.push(
            tokio::spawn(async move {
            TFTPServerRunner::runner(tftp_server.deref()).await
            })
        );

        let port = cli_args.tftp.unwrap() as u16;

        tftp_server_c.start(path.clone(), bind_ip.clone(), port);
    }


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // FTP from here on
    // 
    // Spin the runners to wait for any potential server start
    let ftp_server = Arc::new(<Server as FTPServerRunner>::new());
    let ftp_server_c = ftp_server.clone();

    servers.push(ftp_server.clone());

    if cli_args.ftp.is_some() {
        spawned_servers.push(
            tokio::spawn(async move {
            FTPServerRunner::runner(ftp_server.deref()).await
            })
        );

        let port = cli_args.ftp.unwrap() as u16;

        ftp_server_c.start(path.clone(), bind_ip.clone(), port);
    }


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // HTTP from here on
    // 
    // Spin the runners to wait for any potential server start
    let http_server = Arc::new(<Server as HTTPServerRunner>::new());
    let http_server_c = http_server.clone();

    servers.push(http_server.clone());

    if cli_args.http.is_some() {
        spawned_servers.push(
            tokio::spawn(async move {
            HTTPServerRunner::runner(http_server.deref()).await
            })
        );

        let port = cli_args.http.unwrap() as u16;

        http_server_c.start(path.clone(), bind_ip.clone(), port);
    }


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // DHCP from here on
    // 
    // Spin the runners to wait for any potential server start
    let dhcp_server = Arc::new(<Server as DHCPServerRunner>::new());
    let dhcp_server_c = dhcp_server.clone();

    servers.push(dhcp_server.clone());

    if cli_args.dhcp {
        spawned_servers.push(
            tokio::spawn(async move {
            DHCPServerRunner::runner(dhcp_server.deref()).await
            })
        );

        dhcp_server_c.start(path.clone(), bind_ip.clone(), 0);
    }

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////

    // Set up a handler for Ctrl+C signal
    ctrlc::set_handler(move || {
        // Handle Ctrl+C signal here
        warn!("Ctrl+C received. Closing connections and exiting.");
        // You can perform cleanup operations here before exiting

        for mut server in &mut servers {
            server.terminate();
        }

    }).expect("Error setting Ctrl+C handler");
    info!("Press Ctrl+C to exit.");

    futures::future::join_all(spawned_servers).await;
    return;
}
