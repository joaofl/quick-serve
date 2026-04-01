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
            println!("[{}] {}: {}", record.level(), record.target(), record.args());

            // Only push logs from internal components to the UI
            if record.target().starts_with("quick_serve") {
                let ui_icon = match record.level() {
                    log::Level::Error => "❌",
                    log::Level::Warn  => "⚠",
                    log::Level::Info  => "ℹ",
                    log::Level::Debug => "?",
                    log::Level::Trace => "~",
                };
                let ui_line = format!("{} {}", ui_icon, record.args());

                if let Ok(mut logs) = self.logs.lock() {
                    logs.push(ui_line);

                    if logs.len() > self.max_logs {
                        let excess = logs.len() - self.max_logs;
                        logs.drain(0..excess);
                    }
                }
            }
        }
    }

    fn flush(&self) {}
}
