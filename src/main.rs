// Only mind errors for now
// #![allow(warnings)]

use std::{path::PathBuf};
use log::{debug};

// use chrono::NaiveDate;
// use slint::SharedString;

slint::slint!(import { AnyServeUI } from "src/ui.slint";);

mod servers {
    pub mod ftp;
}


#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Starting my shapp");

    let ui = AnyServeUI::new().unwrap();

    ui.on_start_ftp_server(move || {
        tokio::spawn(async move {
            let _ = servers::ftp::start_ftp_server(PathBuf::from("/tmp/"), 2121).await;
            // let result = fut.await.unwrap();
        });
    });

    ui.run().unwrap();
}