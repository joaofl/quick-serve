use log::{Log, Metadata, Record, LevelFilter};
use std::sync::{Arc, Mutex};

pub trait MyLoggerFn {
    fn new (log_level: LevelFilter) -> Self;
}

pub struct MyLogger {
    pub log_level: LevelFilter,
    pub logs: Arc<Mutex<Vec<String>>>,
    max_logs: usize,
}

impl MyLoggerFn for MyLogger {
    fn new (log_level: LevelFilter) -> Self {
        MyLogger {
            logs: Arc::new(Mutex::new(Vec::new())),
            log_level,
            max_logs: 10000, // Maximum number of logs to keep in memory
        }
    }
}

impl Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Determine whether to enable logging for the given metadata
        // For example, you can enable all levels or only certain levels
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_line = format!(
                "[{}] {}: {}",
                record.level(),
                record.target(),
                record.args()
            );

            println!("{}", log_line);
            
            // Add log rotation to prevent memory leaks
            if let Ok(mut logs) = self.logs.lock() {
                logs.push(log_line);
                
                // If we exceed the maximum number of logs, remove the oldest ones
                if logs.len() > self.max_logs {
                    let excess = logs.len() - self.max_logs;
                    logs.drain(0..excess);
                }
            }
        }
    }

    fn flush(&self) {}
}
