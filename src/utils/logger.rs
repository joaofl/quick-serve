use log::{Log, Metadata, Record, LevelFilter};
use std::sync::{Arc, Mutex};

pub trait MyLoggerFn {
    fn new (log_level: LevelFilter) -> Self;
}

pub struct MyLogger {
    pub log_level: LevelFilter,
    pub logs: Arc<Mutex<String>>,
}

impl MyLoggerFn for MyLogger {
    fn new (log_level: LevelFilter) -> Self {
        MyLogger {
            logs: Arc::new(Mutex::new(String::new())),
            log_level,
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
            self.logs.lock().unwrap().push_str(&format!("{}\n", log_line));
        }
    }

    fn flush(&self) {}
}
