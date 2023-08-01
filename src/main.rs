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


#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "quick_serve=debug");

    let ui = AnyServeUI::new().unwrap();

    let fut_ftp = servers::ftp::start_ftp_server(PathBuf::from("/tmp/"), 21);
    let result = try_join!(fut_ftp);

    match result {
        Ok(s) => info!("result is...\n{:?}", s),
        Err(e) => error!("There was an error: {:?}", e)
    };

    // let r_ftp = fut_ftp.await.unwrap();
    // println!("Result 1: {}", r_ftp);

    ui.run().unwrap();
}