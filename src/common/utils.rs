use std::process::exit;

use log::info;
use log::warn;
use tokio::sync::broadcast::Sender;

use crate::{CommandMsg, PROTOCOL_LIST};

pub fn setup_ctrlc_handler(sender: Sender<CommandMsg>) {
    ////////////////////////////////////////////////////////////////////////
    // Ctrl+c handler - gracefully stop all servers before exiting
    ////////////////////////////////////////////////////////////////////////
    tokio::spawn(async move {
        ctrlc::set_handler(move || {
            warn!("Ctrl+C received. Stopping all servers and exiting...");
            
            // Send stop messages to all servers
            for protocol in PROTOCOL_LIST {
                let stop_msg = CommandMsg {
                    start: false,
                    protocol: protocol.clone(),
                    ..Default::default()
                };
                
                // Send twice to ensure both the server task and runner exit
                let _ = sender.send(stop_msg.clone());
                let _ = sender.send(stop_msg);
            }
            
            // Give servers a moment to shut down gracefully
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            info!("All servers stopped. Exiting.");
            exit(0);
        }).expect("Error setting Ctrl+C handler");
        info!("Press Ctrl+C to exit.");
    });
}

/// Helper function for testing - sends stop messages without exiting
#[cfg(test)]
pub fn send_shutdown_signals(sender: &Sender<CommandMsg>) {
    for protocol in PROTOCOL_LIST {
        let stop_msg = CommandMsg {
            start: false,
            protocol: protocol.clone(),
            ..Default::default()
        };
        
        // Send twice to ensure both the server task and runner exit
        let _ = sender.send(stop_msg.clone());
        let _ = sender.send(stop_msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultChannel;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_shutdown_signals() {
        let channel: DefaultChannel<CommandMsg> = Default::default();
        let mut receiver = channel.sender.subscribe();
        
        send_shutdown_signals(&channel.sender);
        
        // Collect all messages
        let mut received_messages = Vec::new();
        let max_messages = PROTOCOL_LIST.len() * 2;
        
        for _ in 0..max_messages {
            match timeout(Duration::from_millis(100), receiver.recv()).await {
                Ok(Ok(msg)) => received_messages.push(msg),
                _ => break,
            }
        }
        
        // Verify count and structure
        assert_eq!(received_messages.len(), max_messages, 
            "Should receive 2 stop messages per protocol");
        
        // Count per protocol and verify message structure
        let mut protocol_counts = std::collections::HashMap::new();
        
        for msg in &received_messages {
            assert!(!msg.start, "All messages should be stop signals");
            assert!(msg.bind_ip.is_empty() && msg.path.is_empty() && msg.port == 0,
                "Stop messages should have default values");
            
            *protocol_counts.entry(&msg.protocol).or_insert(0) += 1;
        }
        
        // Each protocol should receive exactly 2 messages
        assert_eq!(protocol_counts.len(), PROTOCOL_LIST.len());
        for count in protocol_counts.values() {
            assert_eq!(*count, 2, "Each protocol should receive exactly 2 stop messages");
        }
    }
}
