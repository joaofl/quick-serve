use log::{Level, Log, Metadata, Record};
use egui::mutex::Mutex;
use std::sync::Arc;


pub struct MyLogger {
    pub logs: Arc<Mutex<String>>,
}

// impl Default for MyLogger {
//     fn default() -> Self {
//         MyLogger {
//             logs: Arc::new(Mutex::new(String::new())),
//         }
//     }
// }

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

            println!("-> {}", log_line);
            self.logs.lock().push_str(&format!("{}\n", log_line));
        }
    }

    fn flush(&self) {}
}
