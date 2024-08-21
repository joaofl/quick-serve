// #![allow(warnings)]

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

#[tokio::main]
async fn main() {
    let cli_args = Cli::parse();

    let mut log_level = LevelFilter::Info;
    if cli_args.verbose > 0 {
        log_level = LevelFilter::Debug;
    }

    let logger = Box::new(MyLogger::new(log_level));

    // Define the channel used to control the servers
    let channel: DefaultChannel<CommandMsg> = Default::default();

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace); // Set the maximum log level


    ////////////////////////////////////////////////////////////////////////
    server_starter_receiver(&channel);

    ////////////////////////////////////////////////////////////////////////
    setup_ctrlc_handler();

    ////////////////////////////////////////////////////////////////////////
    server_starter_sender(&cli_args, &channel);

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
