//! # Application State
//!
//! This module defines the primary state structures for the GUI application.
//! It contains the main `TabletMapperApp` struct which holds the UI state,
//! shared engine state, and communication channels.

// use eframe::egui; // Unused import
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
    /// Links to Discord, GitHub, and documentation.
    Support,
    /// Changelog and update installation dialog.
    Release,
}

/// The core application state structure used by the `eframe` (egui) integration.
///
/// This struct implements the `eframe::App` trait (in `update.rs`) and holds
/// all necessary state for rendering the UI and interacting with the backend engine.
pub struct TabletMapperApp {
    // Shared State
    /// Thread-safe shared state containing configuration, tablet data, and statistics.
    /// Shared between the GUI thread and the background engine thread.
    pub shared: Arc<SharedState>,

    // UI Local State
    /// Information about connected monitors/displays, used for absolute mapping bounds.
    pub displays: Vec<DisplayInfo>,
    /// Timestamp of the last successful UI frame update.
    pub last_update: Instant,
    /// Name of the currently loaded profile (e.g., "Default").
    pub profile_name: String,
    /// The currently active UI tab.
    pub active_tab: AppTab,

    // Event Receiver
    /// Channel receiver for incoming tablet data (positions, pressure, button states).
    pub tablet_receiver: Receiver<TabletData>,
    /// Channel receiver for application update status changes.
    pub update_receiver: Receiver<UpdateStatus>,
    /// The current state of the application updater (Idle, Checking, Available, etc).
    pub update_status: UpdateStatus,

    // Filters UI State
    /// The name of the currently selected filter in the Filters tab dropdown.
    pub selected_filter: String,

    // Debugger UI State
    /// Whether the advanced polling rate / jitter debugger overlay is visible.
    pub show_debugger: bool,
    /// Whether the complex input lag and performance statistics overlay is visible.
    pub show_latency_stats: bool,
    /// The calculated and displayed polling rate (Hz).
    pub displayed_hz: f32,
    /// Timestamp of the last polling rate calculation.
    pub last_hz_update: Instant,
    /// The number of packets processed at the last polling rate calculation interval.
    pub last_packet_count: u32,
    /// Real-time perceived latency between driver thread and UI thread (ms).
    pub ui_latency_ms: f32,
    pub min_ui_latency_ms: f32,
    pub max_ui_latency_ms: f32,
    pub avg_ui_latency_ms: f32,

    pub was_minimized: bool,

    // System Tray
    /// Optional tray icon instance to keep it alive.
    pub tray_icon: Option<tray_icon::TrayIcon>,
}
