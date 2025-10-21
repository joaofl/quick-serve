// #![allow(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![cfg_attr(not(feature = "ui"), allow(dead_code))]


use log::LevelFilter;
use std::process::exit;

mod utils;
use utils::logger::*;

mod servers;
use crate::servers::*;

mod common;
use crate::common::*;

use clap::Parser;

extern crate ctrlc;
extern crate core;

mod ui;
use crate::ui::window::*;


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
    server_starter_receiver(&channel);

    ////////////////////////////////////////////////////////////////////////
    setup_ctrlc_handler(channel.sender.clone());

    ////////////////////////////////////////////////////////////////////////
    // HEADLESS related code from here on
    ////////////////////////////////////////////////////////////////////////
    if cli_args.headless {
        server_starter_sender(&cli_args, &channel);
    }
    ////////////////////////////////////////////////////////////////////////
    // UI related code from here on
    ////////////////////////////////////////////////////////////////////////
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
                cc.egui_ctx.set_theme(egui::Theme::Light);

                let mut ui = UI::new(cc);
                ui.logs = logs;

                ui.channel.sender = channel.sender;
                Ok(Box::new(ui))
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
        let mut cmd = Command::cargo_bin("quick-serve-gui").unwrap();
        cmd.arg("--help");
        cmd.assert().success().stdout(predicate::str::contains("Usage: quick-serve"));
    }
}
