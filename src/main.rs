use tokio::try_join;
use std::path::PathBuf;
use log::{error, info};

// use chrono::NaiveDate;
// use slint::SharedString;

slint::slint!(import { AnyServeUI } from "src/ui.slint";);

mod servers {
    pub mod ftp;
}

// Below the code for command line parser
// #[derive(Parser)]
// #[command(author, version, about, long_about = None)]
// #[command(next_line_help = true)]
// struct Cli {
//     #[arg(long)]
//     path: String,
//     port: u32,
// }


fn main() {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Starting my shapp");


    let ui = AnyServeUI::new().unwrap();

    let fut_ftp = servers::ftp::start_ftp_server(PathBuf::from("/tmp/"), 21);
    let result = try_join!(fut_ftp);

    match result {
        Ok(s) => info!("result is...\n{:?}", s),
        Err(e) => error!("There was an error: {:?}", e)
    };

    ui.on_start_ftp_server(move || {
        let ftp_server = servers::ftp::FTPServer::new(PathBuf::from("/tmp/"), 2121);
        // ftp_fut = ftp_server.start();
        // let ftp_join = tokio::spawn(ftp_fut);
        // let result = try_join!(ftp_server.start());
        debug!("Starting.....")
    });

    ui.run().unwrap();
}