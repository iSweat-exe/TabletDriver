use crate::engine::state::SharedState;
use crate::VERSION;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelemetryPayload {
    pub header: TelemetryHeader,
    pub user: TelemetryUser,
    pub device: TelemetryDevice,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelemetryHeader {
    pub event_id: String,
    pub schema_version: String,
    pub timestamp_utc: String,
    pub app_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelemetryUser {
    pub id_hash: String,
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelemetryDevice {
    pub os_family: String,
    pub os_version: String,
    pub arch: String,
    pub locale: String,
    pub tablet: Option<TabletInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TabletInfo {
    pub vendor_id: String,
    pub product_id: String,
    pub model_name: String,
}

fn get_machine_hash() -> String {
    let mut hasher = DefaultHasher::new();

    // Get stable machine ID (registry MachineGuid on Windows)
    let machine_id = machine_uid::get().unwrap_or_else(|_| "unknown_machine".to_string());
    machine_id.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}

fn get_os_info() -> (String, String) {
    let info = os_info::get();
    (
        info.os_type().to_string().to_lowercase(),
        info.version().to_string(),
    )
}

pub fn init_telemetry(shared: Arc<SharedState>) {
    let session_id = uuid::Uuid::new_v4().to_string();
    let machine_hash = get_machine_hash();

    thread::spawn(move || {
        log::info!(target: "Telemetry", "Telemetry background thread started.");

        loop {
            // Check if telemetry is enabled in config
            let enabled = shared.config.read().unwrap().enable_telemetry;
            if enabled {
                send_telemetry(&machine_hash, &session_id, &shared);
            }

            thread::sleep(Duration::from_secs(900)); // 15 minutes
        }
    });
}

fn send_telemetry(machine_hash: &str, session_id: &str, shared: &SharedState) {
    let (os_family, os_version) = get_os_info();

    // Header
    let header = TelemetryHeader {
        event_id: uuid::Uuid::new_v4().to_string(),
        schema_version: "1.0.0".to_string(),
        timestamp_utc: Utc::now().to_rfc3339(),
        app_version: VERSION.to_string(),
    };

    // User
    let user = TelemetryUser {
        id_hash: machine_hash.to_string(),
        session_id: session_id.to_string(),
    };

    // Device & Tablet Info
    let tablet_name = shared.tablet_name.read().unwrap().clone();
    let tablet_info = if tablet_name != "No Tablet Detected" {
        let vid = *shared.tablet_vid.read().unwrap();
        let pid = *shared.tablet_pid.read().unwrap();
        Some(TabletInfo {
            vendor_id: format!("{:04x}", vid),
            product_id: format!("{:04x}", pid),
            model_name: tablet_name,
        })
    } else {
        None
    };

    let device = TelemetryDevice {
        os_family,
        os_version,
        arch: std::env::consts::ARCH.to_string(),
        locale: sys_locale::get_locale().unwrap_or_else(|| "unknown".to_string()),
        tablet: tablet_info,
    };

    let payload = TelemetryPayload {
        header,
        user,
        device,
    };

    let client = reqwest::blocking::Client::new();
    match client
        // TODO: Dev Telemetry Dashboard in React
        .post("http://localhost:3000/api/telemetry")
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
    {
        Ok(resp) => {
            if resp.status().is_success() {
                log::info!(target: "Telemetry", "Telemetry sent successfully.");
            } else {
                log::warn!(target: "Telemetry", "Telemetry failed with status: {}", resp.status());
            }
        }
        Err(e) => {
            log::error!(target: "Telemetry", "Failed to send telemetry: {}", e);
        }
    }
}
