use std::process::exit;

use log::info;
use log::warn;

pub fn setup_ctrlc_handler() {
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
}
