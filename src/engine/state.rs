//! # Thread-Safe Application State
//!
//! This module defines the `SharedState` structure, which provides a bridge
//! between the high-frequency background `engine` threads and the 60Hz GUI threads.

use crate::core::config::models::MappingConfig;
use crate::drivers::TabletData;
use std::sync::RwLock;
use std::sync::atomic::AtomicU32;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicU64;

/// The central thread-safe state store for the application.
///
/// Due to the disparate update rates of the background processing engine (often 100-1000Hz)
/// and the user interface (locked to vsync/60Hz), all shared data is wrapped in `RwLock`
/// or atomic types to ensure memory safety without creating massive mutex contention.
pub struct SharedState {
    /// The currently active settings (mapping area, filters, network, etc).
    pub config: RwLock<MappingConfig>,
    /// An atomic counter incremented by the UI whenever `config` is modified.
    /// The background thread checks this to avoid acquiring read-locks continuously.
    pub config_version: AtomicU32,
    /// The most recent normalized packet from the tablet (X, Y, Pressure, Pen Buttons).
    pub tablet_data: RwLock<TabletData>,
    /// The marketing/USB name of the connected tablet device.
    pub tablet_name: RwLock<String>,
    /// USB Vendor ID of the connected device.
    pub tablet_vid: RwLock<u16>,
    /// USB Product ID of the connected device.
    pub tablet_pid: RwLock<u16>,
    /// The immutable physical dimensions of the active tablet in millimeters (Width, Height).
    pub physical_size: RwLock<(f32, f32)>,
    /// The maximum raw hardware values the tablet can output (Max_X, Max_Y).
    pub hardware_size: RwLock<(f32, f32)>,
    /// Flag indicating if the user has never launched the application before (triggers welcome modal).
    pub is_first_run: RwLock<bool>,
    /// A rapidly incrementing counter of USB packets received, used by the UI to calculate real-time Hz.
    pub packet_count: AtomicU32,
    /// Tracking statistics for developer debugging (e.g., dropped packets, parse errors).
    pub stats: RwLock<crate::drivers::DriverStats>,

    // === Debug-only pipeline instrumentation (stripped from release builds) ===
    /// Current pipeline stage label (e.g., "Normalize", "Filter", "Project", "Inject").
    #[cfg(debug_assertions)]
    pub debug_pipeline_stage: RwLock<String>,
    /// Normalized UV coordinates right after physical_to_normalized (pre-filter).
    #[cfg(debug_assertions)]
    pub debug_last_uv: RwLock<(f32, f32)>,
    /// Normalized UV coordinates after the filter pipeline (post-filter).
    #[cfg(debug_assertions)]
    pub debug_last_filtered_uv: RwLock<(f32, f32)>,
    /// Final screen pixel coordinates sent to the OS injector.
    #[cfg(debug_assertions)]
    pub debug_last_screen: RwLock<(f32, f32)>,
    /// Monotonic counter of OS injection calls since startup.
    #[cfg(debug_assertions)]
    pub debug_inject_count: AtomicU32,
    /// Time spent in the filter pipeline per packet (nanoseconds).
    #[cfg(debug_assertions)]
    pub debug_filter_time_ns: AtomicU64,
    /// Time spent in coordinate transform per packet (nanoseconds).
    #[cfg(debug_assertions)]
    pub debug_transform_time_ns: AtomicU64,
    /// Time spent in the entire pipeline.process() call (nanoseconds).
    #[cfg(debug_assertions)]
    pub debug_pipeline_time_ns: AtomicU64,
}
