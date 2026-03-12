use display_info::DisplayInfo;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Instant;

use crate::app::autoupdate::{self, UpdateStatus};
use crate::app::state::{AppTab, TabletMapperApp};
use crate::domain::{ActiveArea, MappingConfig, TargetArea, WebSocketConfig};
use crate::drivers::TabletData;
use crate::input::{input_loop, SharedState};
use crate::settings::load_last_session;

impl TabletMapperApp {
    pub fn new(_ctx: eframe::egui::Context) -> Self {
        let displays = DisplayInfo::all().unwrap_or_default();

        let loaded_config = load_last_session();
        let is_first_run = loaded_config.is_none();

        let config = if let Some(cfg) = loaded_config {
            log::info!(target: "App", "Using loaded configuration from last session");
            cfg
        } else {
            MappingConfig {
                mode: crate::domain::DriverMode::Absolute,
                active_area: ActiveArea {
                    x: 80.0,
                    y: 50.0,
                    w: 160.0,
                    h: 100.0,
                    rotation: 0.0,
                },
                target_area: TargetArea {
                    x: 0.0,
                    y: 0.0,
                    w: 1920.0,
                    h: 1080.0,
                },
                relative_config: crate::domain::RelativeConfig::default(),
                tip_threshold: 10,
                eraser_threshold: 10,
                disable_pressure: false,
                disable_tilt: false,
                tip_binding: "Mouse Button Binding: (Button: Left)".to_string(),
                eraser_binding: "None".to_string(),
                pen_button_bindings: vec!["None".to_string(), "None".to_string()],
                run_at_startup: crate::startup::is_run_at_startup_registered(),
                enable_telemetry: true,
                websocket: WebSocketConfig::default(),
                antichatter: crate::domain::AntichatterConfig::default(),
            }
        };

        let shared = Arc::new(SharedState {
            config: RwLock::new(config),
            config_version: AtomicU32::new(0),
            tablet_data: RwLock::new(TabletData::default()),
            tablet_name: RwLock::new("No Tablet Detected".to_string()),
            tablet_vid: RwLock::new(0),
            tablet_pid: RwLock::new(0),
            physical_size: RwLock::new((160.0, 100.0)),
            hardware_size: RwLock::new((32767.0, 32767.0)),
            is_first_run: RwLock::new(is_first_run),
            packet_count: AtomicU32::new(0),
        });

        let (tablet_sender, tablet_receiver) = crossbeam_channel::unbounded();

        // Spawn Input Thread
        let thread_shared = Arc::clone(&shared);
        let thread_ctx = _ctx.clone();
        thread::spawn(move || {
            input_loop(thread_shared, thread_ctx, tablet_sender);
        });

        // Spawn WebSocket Setup Thread
        let ws_shared = Arc::clone(&shared);
        thread::spawn(move || {
            crate::app::websocket::websocket_loop(ws_shared);
        });

        // Spawn Telemetry Thread
        let telemetry_shared = Arc::clone(&shared);
        crate::telemetry::init_telemetry(telemetry_shared);

        let (update_sender, update_receiver) = crossbeam_channel::bounded(1);
        thread::spawn(move || match autoupdate::check_for_updates() {
            Ok(Some(release)) => {
                let _ = update_sender.send(UpdateStatus::Available(release));
            }
            Ok(None) => {}
            Err(e) => {
                log::error!(target: "Update", "Failed to check for updates: {}", e);
            }
        });

        Self {
            shared,
            displays,
            last_update: Instant::now(),
            profile_name: "Default".to_string(),
            active_tab: AppTab::Output,
            tablet_receiver,
            update_receiver,
            update_status: UpdateStatus::Idle,
            selected_filter: "Devocub Antichatter".to_string(),
        }
    }
}
