//! # Application Lifecycle
//!
//! This module handles the initialization, startup routines, and background thread
//! management for the `TabletMapperApp`. It is responsible for loading configurations
//! and bootstrapping the various concurrent systems (engine, websocket, updater).

use display_info::DisplayInfo;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicU32;
use std::thread;
use std::time::Instant;

use crate::app::autoupdate::{self, UpdateStatus};
use crate::app::state::{AppTab, TabletMapperApp};
use crate::core::config::models::{ActiveArea, MappingConfig, TargetArea, WebSocketConfig};
use crate::drivers::TabletData;
use crate::engine::state::SharedState;
use crate::engine::tablet_manager::run_manager;
use crate::settings::load_last_session;

impl TabletMapperApp {
    /// Creates a new instance of the application and initializes all background services.
    ///
    /// # Arguments
    /// * `_ctx` - The `eframe::egui::Context` provided by the window manager.
    ///
    /// # Initialization Flow
    /// 1. **Displays**: Gathers information about connected monitors.
    /// 2. **Configuration**: Attempts to load the last session's settings from disk.
    ///    If no configuration is found, applies fallback defaults.
    /// 3. **Shared State**: Allocates the `SharedState` structure wrapped in an `Arc`.
    /// 4. **Input Engine Thread**: Spawns a background thread to run the `tablet_manager`,
    ///    which continuously polls USB/HID devices.
    /// 5. **WebSocket Thread**: Spawns a background thread to manage WebSocket connections
    ///    for external integrations (e.g., streaming overlays).
    /// 6. **Auto-Updater Thread**: Spawns a background thread to check GitHub releases
    ///    for newer versions of the software.
    pub fn new(_ctx: eframe::egui::Context) -> Self {
        let displays = DisplayInfo::all().unwrap_or_default();

        let loaded_config = load_last_session();
        let is_first_run = loaded_config.is_none();

        let config = if let Some(cfg) = loaded_config {
            log::info!(target: "App", "Using loaded configuration from last session");
            cfg
        } else {
            MappingConfig {
                mode: crate::core::config::models::DriverMode::Absolute,
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
                relative_config: crate::core::config::models::RelativeConfig::default(),
                tip_threshold: 10,
                eraser_threshold: 10,
                disable_pressure: false,
                disable_tilt: false,
                tip_binding: "Mouse Button Binding: (Button: Left)".to_string(),
                eraser_binding: "None".to_string(),
                pen_button_bindings: vec!["None".to_string(), "None".to_string()],
                run_at_startup: crate::startup::is_run_at_startup_registered(),

                websocket: WebSocketConfig::default(),
                antichatter: crate::core::config::models::AntichatterConfig::default(),
                speed_stats: crate::core::config::models::SpeedStatsConfig::default(),
                theme: crate::core::config::models::ThemePreference::System,
                lock_aspect_ratio: false,
                show_osu_playfield: false,
            }
        };

        // Apply theme before shared state move
        crate::ui::theme::apply_theme(&_ctx, config.theme);

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
            stats: RwLock::new(crate::drivers::DriverStats::default()),
        });

        let (tablet_sender, tablet_receiver) = crossbeam_channel::unbounded();

        // Spawn Input Thread
        let thread_shared = Arc::clone(&shared);
        let thread_ctx = _ctx.clone();
        log::info!(target: "App", "Spawning Input Engine thread");
        thread::spawn(move || {
            run_manager(thread_shared, thread_ctx, tablet_sender);
        });

        // Spawn WebSocket Setup Thread
        let ws_shared = Arc::clone(&shared);
        log::info!(target: "App", "Spawning WebSocket thread");
        thread::spawn(move || {
            crate::app::websocket::websocket_loop(ws_shared);
        });

        let (update_sender, update_receiver) = crossbeam_channel::bounded(1);
        log::info!(target: "App", "Spawning Auto-Updater thread");
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
            show_debugger: false,
            show_latency_stats: false,
            displayed_hz: 0.0,
            last_hz_update: Instant::now(),
            last_packet_count: 0,
            ui_latency_ms: 0.0,
            min_ui_latency_ms: f32::MAX,
            max_ui_latency_ms: 0.0,
            avg_ui_latency_ms: 0.0,
        }
    }
}
