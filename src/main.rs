// #![allow(warnings)]


use log::{error, info, warn};

use std::path::PathBuf;
use std::ops::Deref;
use std::sync::Arc;





mod tests;
mod utils;
mod servers;
use crate::servers::{*};

use clap::{Parser};
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
        help = "Path to serve",
        short = 'p', long, required = false,
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
    let cli_args = Cli::parse();

    let mut log_level = "info";
    if cli_args.verbose > 0 {
        log_level = "debug";
    }

    ::std::env::set_var("RUST_LOG", log_level);
    env_logger::builder()
        .format_timestamp_secs()
        .init();

    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////
    // debug!("\n{:#?}\n", cli_args);

    let mut spawned_runners = vec![];
    let mut spawned_servers = vec![];

    // Read and validate the bind address
    let bind_ip = cli_args.bind_ip;
    let path = cli_args.serve_dir;


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // TFTP from here on
    // 
    // Spin the runners to wait for any potential server start
    if cli_args.tftp.is_some() {
        let tftp_server = Arc::new(<Server as TFTPServerRunner>::new());
        let tftp_server_c = tftp_server.clone();

        spawned_servers.push(tftp_server.clone());
        spawned_runners.push(
            tokio::spawn(async move {
                TFTPServerRunner::runner(tftp_server).await
            })
        );

        let _port = cli_args.tftp.unwrap() as u16;
        let _ = tftp_server_c.start();
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


    ///////////////////////////////////////////////////////////////////////////////
    // 
    // DHCP from here on
    // 
    // Spin the runners to wait for any potential server start
    if cli_args.dhcp {
        let dhcp_server = Arc::new(<Server as DHCPServerRunner>::new());
        let dhcp_server_c = dhcp_server.clone();

        spawned_servers.push(dhcp_server.clone());
        spawned_runners.push(
            tokio::spawn(async move {
            DHCPServerRunner::runner(dhcp_server.deref()).await
            })
        );

        let _ = dhcp_server_c.start();
    }
    //////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////

    if spawned_runners.iter().count() == 0 {
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

    futures::future::join_all(spawned_runners).await;
    return;
}
