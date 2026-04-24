//! # Application State
//!
//! This module defines the primary state structures for the GUI application.
//! It contains the main `TabletMapperApp` struct which holds the UI state,
//! shared engine state, and communication channels.

use crate::core::config::models::MappingConfig;
use display_info::DisplayInfo;
use std::path::PathBuf;
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

/// Tracks the state of the currently active user profile.
pub struct ProfileState {
    /// Display name shown in the UI footer.
    pub name: String,
    /// Absolute path to the profile file on disk. `None` for unsaved sessions.
    pub path: Option<PathBuf>,
    /// Snapshot of the config at the time of the last save/load.
    /// Used to detect dirty (unsaved changes) state.
    pub last_saved: MappingConfig,
}

impl ProfileState {
    /// Returns `true` if the current config differs from the last saved snapshot.
    pub fn is_dirty(&self, current: &MappingConfig) -> bool {
        *current != self.last_saved
    }

    /// Returns the display name for the footer, prefixed with `*` if dirty.
    pub fn display_name(&self, current: &MappingConfig) -> String {
        let base = if self.path.is_some() {
            &self.name
        } else {
            "Unsaved Session"
        };

        if self.is_dirty(current) {
            format!("*{}", base)
        } else {
            base.to_string()
        }
    }

    /// Updates the saved snapshot after a successful save/load.
    pub fn mark_saved(&mut self, config: &MappingConfig) {
        self.last_saved = config.clone();
    }
}

/// Severity level for UI toast notifications.
#[derive(Clone, Copy, PartialEq)]
pub enum ToastLevel {
    Info,
    Warning,
    Error,
}

/// A transient notification displayed in the UI overlay.
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: Instant,
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
    pub profile: ProfileState,
    pub active_tab: AppTab,

    // Event Receivers
    pub tablet_receiver: Receiver<TabletData>,
    pub update_receiver: Receiver<UpdateStatus>,
    pub update_status: UpdateStatus,

    // Background Saver
    /// Send side of the async save channel. The background saver thread
    /// owns the receiver and writes `last_session.json` off the UI thread.
    pub save_sender: crossbeam_channel::Sender<MappingConfig>,

    // Toast Notifications
    /// Active toast queue. Max 3 simultaneous, deduplicated by message.
    pub toasts: Vec<Toast>,

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

    // Close Confirmation
    /// Whether the unsaved-changes close confirmation dialog is open.
    pub show_close_confirm: bool,
    /// If true, bypasses the close confirmation intercept.
    pub force_close: bool,

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

const MAX_TOASTS: usize = 3;

impl TabletMapperApp {
    /// Pushes a toast notification, deduplicating by message and capping at 3.
    pub fn push_toast(&mut self, message: String, level: ToastLevel) {
        // Deduplicate: don't push if an identical message is already visible
        if self.toasts.iter().any(|t| t.message == message) {
            return;
        }

        // Cap at MAX_TOASTS: remove oldest if full
        if self.toasts.len() >= MAX_TOASTS {
            self.toasts.remove(0);
        }

        self.toasts.push(Toast {
            message,
            level,
            created_at: Instant::now(),
        });
    }
}
