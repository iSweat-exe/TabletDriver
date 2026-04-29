use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};
use std::sync::{Arc, LazyLock, RwLock};

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

pub static LOG_BUFFER: LazyLock<Arc<RwLock<Vec<LogEntry>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

impl Log for GlobalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let target = record.target();

            // Whitelist: only these named targets appear in the in-app console
            let allowed_targets = [
                "App",
                "UI",
                "HID",
                "TabletManager",
                "Pipeline",
                "Config",
                "Startup",
                "Update",
                "Stats",
                "Tray",
                "Timer",
                "WebSocket",
                "Telemetry",
                "Driver",
                "Detect",
            ];
            let is_allowed = allowed_targets
                .iter()
                .any(|&t| target == t || target.starts_with(&format!("{}::", t)))
                || target.starts_with("NextTabletDriver");

            let entry = LogEntry {
                time: Local::now().format("%H:%M:%S").to_string(),
                level: format!("{:?}", record.level()),
                group: target.to_string(),
                message: format!("{}", record.args()),
            };

            if cfg!(debug_assertions) {
                let log_line = format!(
                    "[{}] {} [{}] {}",
                    entry.time, entry.level, entry.group, entry.message
                );
                println!("{}", log_line);

                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("debug.log")
                {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", log_line);
                }
            }

            if is_allowed && let Ok(mut entries) = self.entries.write() {
                entries.push(entry);

                // Ring buffer: evict oldest when full
                if entries.len() > 500 {
                    entries.remove(0);
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
