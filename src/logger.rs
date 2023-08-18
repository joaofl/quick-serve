use log::{Level, Log, Metadata, Record};

pub struct MyLogger {
    pub sender: tokio::sync::broadcast::Sender<String>,
}

impl Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Determine whether to enable logging for the given metadata
        // For example, you can enable all levels or only certain levels
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_prefix = "[FTP]";
            let _ = self.sender.send(format!("{} {}: {}", log_prefix, record.level(), record.args()));
        }
    }

    fn flush(&self) {}
}