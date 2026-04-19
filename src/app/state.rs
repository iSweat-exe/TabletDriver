//! # Application State
//!
//! This module defines the primary state structures for the GUI application.
//! It contains the main `TabletMapperApp` struct which holds the UI state,
//! shared engine state, and communication channels.

use display_info::DisplayInfo;
use std::sync::Arc;
use std::time::Instant;

use crate::app::autoupdate::UpdateStatus;
use crate::drivers::TabletData;
use crate::engine::state::SharedState;
use crossbeam_channel::Receiver;

/// Represents the currently active tab in the main application window.
///
/// This enum dictates which UI panel is rendered to the user.
#[derive(PartialEq, Clone, Copy)]
pub enum AppTab {
    /// The main output/mapping tab.
    Output,
    /// Settings for anti-chatter and smoothing filters.
    Filters,
    /// Pen button bindings and pressure curve settings.
    PenSettings,
    /// Real-time tablet events and debugging output.
    Console,
    /// General application settings (startup, updater, etc).
    Settings,
    /// Changelog and update installation dialog.
    Release,
    #[cfg(debug_assertions)]
    /// Developer diagnostics and pipeline inspection.
    Developer,
}

/// The core application state structure used by the `eframe` (egui) integration.
///
/// This struct implements the `eframe::App` trait (in `update.rs`) and holds
/// all necessary state for rendering the UI and interacting with the backend engine.
pub struct TabletMapperApp {
    // Shared State
    /// Thread-safe state shared with the background engine thread.
    pub shared: Arc<SharedState>,

    // UI Local State
    pub displays: Vec<DisplayInfo>,
    pub last_update: Instant,
    pub profile_name: String,
    pub active_tab: AppTab,

    // Event Receivers
    pub tablet_receiver: Receiver<TabletData>,
    pub update_receiver: Receiver<UpdateStatus>,
    pub update_status: UpdateStatus,

    // Filters UI State
    pub selected_filter: String,

    // Debugger UI State
    pub show_debugger: bool,
    pub show_latency_stats: bool,
    pub displayed_hz: f32,
    pub last_hz_update: Instant,
    pub last_packet_count: u32,
    /// Exponential moving average of engine→UI latency (ms).
    pub ui_latency_ms: f32,
    pub min_ui_latency_ms: f32,
    pub max_ui_latency_ms: f32,
    pub avg_ui_latency_ms: f32,

    pub was_minimized: bool,

    // Console State
    pub console_search: String,
    pub console_show_info: bool,
    pub console_show_warn: bool,
    pub console_show_error: bool,
    pub console_show_debug: bool,
    pub console_autoscroll: bool,

    // System Tray
    /// Kept alive to prevent the tray icon from being dropped.
    pub tray_icon: Option<tray_icon::TrayIcon>,

    // Developer UI State (Debug only)
    #[cfg(debug_assertions)]
    pub dev_pause_pipeline: bool,
    #[cfg(debug_assertions)]
    pub dev_raw_hid_history: std::collections::VecDeque<String>,
    #[cfg(debug_assertions)]
    pub dev_pipeline_log: std::collections::VecDeque<String>,
    #[cfg(debug_assertions)]
    pub dev_show_full_config: bool,
    #[cfg(debug_assertions)]
    pub dev_filter_details_open: bool,
}
