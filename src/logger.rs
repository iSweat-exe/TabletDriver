use chrono::Local;
use lazy_static::lazy_static;
use log::{LevelFilter, Log, Metadata, Record};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: String,
    pub level: String,
    pub group: String,
    pub message: String,
}

pub struct GlobalLogger {
    pub entries: Arc<RwLock<Vec<LogEntry>>>,
}

lazy_static! {
    pub static ref LOG_BUFFER: Arc<RwLock<Vec<LogEntry>>> = Arc::new(RwLock::new(Vec::new()));
}

impl Log for GlobalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let target = record.target();

            // Define relevant targets for the in-app console
            let allowed_targets = [
                "App",
                "Detect",
                "Input",
                "Config",
                "Driver",
                "NextTabletDriver",
                "WS",
                "Update",
                "Telemetry",
            ];
            let is_allowed =
                allowed_targets.contains(&target) || target.starts_with("NextTabletDriver");

            let entry = LogEntry {
                time: Local::now().format("%H:%M:%S").to_string(),
                level: format!("{:?}", record.level()),
                group: target.to_string(),
                message: format!("{}", record.args()),
            };

            // Print to stdout for debugging (will show if console window is visible)
            if cfg!(debug_assertions) {
                let log_line = format!(
                    "[{}] {} [{}] {}",
                    entry.time, entry.level, entry.group, entry.message
                );
                println!("{}", log_line);

                // Write to debug.log file (always write all logs to file)
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("debug.log")
                {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", log_line);
                }
            }

            // Only buffer allowed targets for the in-app UI to keep it clean and fast
            if is_allowed {
                if let Ok(mut entries) = self.entries.write() {
                    entries.push(entry);

                    // Cap the buffer at 500 entries to prevent memory issues
                    if entries.len() > 500 {
                        entries.remove(0);
                    }
                }
            }
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    let logger = GlobalLogger {
        entries: LOG_BUFFER.clone(),
    };
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .expect("Failed to initialize logger");
}
