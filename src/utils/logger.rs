use log::{Level, Log, Metadata, Record};

pub struct MyLogger {
    pub sender: tokio::sync::broadcast::Sender<String>,
}

impl Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Determine whether to enable logging for the given metadata
        // For example, you can enable all levels or only certain levels
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_line = format!(
                "[{}] {}: {}",
                record.level(),
                record.target(),
                record.args()
            );

            let _ = self.sender.send(log_line.clone());
            println!("-> {}", log_line);
        }
    }

    fn flush(&self) {}
}
