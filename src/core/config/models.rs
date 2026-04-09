//! # Configuration Models
//!
//! This module defines the data structures used to serialize and deserialize
//! the application's configuration state (typically saved to `settings.json`).
//! It includes models for tablet mapping arrays, UI preferences, and filter settings.

use serde::{Deserialize, Serialize};

/// Represents the absolute physical mapping area on the tablet surface.
///
/// Units are in **millimeters** representing the distance from the top-left corner
/// of the tablet's active zone.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveArea {
    pub x: f32,        // Millimeters
    pub y: f32,        // Millimeters
    pub w: f32,        // Millimeters
    pub h: f32,        // Millimeters
    pub rotation: f32, // Degrees
}

/// Represents the target mapping area on the user's connected monitors.
///
/// Units are in absolute virtual **pixels** spanning across all displays.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetArea {
    pub x: f32, // Pixels
    pub y: f32, // Pixels
    pub w: f32, // Pixels
    pub h: f32, // Pixels
}

/// Determines how pen movement translates to cursor movement.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum DriverMode {
    #[default]
    Absolute,
    Relative,
}

/// User preference for application theme.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
}

/// Settings specific to `Relative` (mouse-like) driver mode.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelativeConfig {
    pub x_sensitivity: f32,
    pub y_sensitivity: f32,
    pub rotation: f32,
    pub reset_time_ms: u32,
}

impl Default for RelativeConfig {
    fn default() -> Self {
        Self {
            x_sensitivity: 10.0,
            y_sensitivity: 10.0,
            rotation: 0.0,
            reset_time_ms: 100,
        }
    }
}

fn default_threshold() -> u16 {
    10
}
fn default_false() -> bool {
    false
}
fn default_true() -> bool {
    true
}
fn default_tip_binding() -> String {
    "Mouse Button Binding: (Button: Left)".to_string()
}
fn default_eraser_binding() -> String {
    "None".to_string()
}
fn default_button_bindings() -> Vec<String> {
    vec!["None".to_string(), "None".to_string()]
}
fn default_ws_port() -> u16 {
    8080
}
fn default_ws_hz() -> u32 {
    60
}

/// Configuration for the embedded WebSocket server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebSocketConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_ws_port")]
    pub port: u16,
    #[serde(default = "default_ws_hz")]
    pub polling_rate_hz: u32,
    #[serde(default = "default_true")]
    pub send_coordinates: bool,
    #[serde(default = "default_true")]
    pub send_pressure: bool,
    #[serde(default = "default_true")]
    pub send_tilt: bool,
    #[serde(default = "default_true")]
    pub send_status: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            port: default_ws_port(),
            polling_rate_hz: default_ws_hz(),
            send_coordinates: default_true(),
            send_pressure: default_true(),
            send_tilt: default_true(),
            send_status: default_true(),
        }
    }
}

/// Configuration for the Devocub Antichatter implementation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AntichatterConfig {
    pub enabled: bool,
    pub latency: f32,
    pub antichatter_strength: f32,
    pub antichatter_multiplier: f32,
    pub antichatter_offset_x: f32,
    pub antichatter_offset_y: f32,
    pub prediction_enabled: bool,
    pub prediction_strength: f32,
    pub prediction_sharpness: f32,
    pub prediction_offset_x: f32,
    pub prediction_offset_y: f32,
    pub frequency: f32,
}

impl Default for AntichatterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            latency: 2.0,
            antichatter_strength: 3.0,
            antichatter_multiplier: 1.0,
            antichatter_offset_x: 0.0,
            antichatter_offset_y: 1.0,
            prediction_enabled: false,
            prediction_strength: 1.1,
            prediction_sharpness: 1.0,
            prediction_offset_x: 3.0,
            prediction_offset_y: 0.3,
            frequency: 1000.0,
        }
    }
}

/// Units used for reporting pen speed telemetry.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum SpeedUnit {
    #[default]
    MillimetersPerSecond,
    MetersPerSecond,
    KilometersPerHour,
    MilesPerHour,
}

/// Configuration for the Speed Statistics UDP telemetry sender.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpeedStatsConfig {
    pub enabled: bool,
    pub ip: String,
    pub port: u16,
    pub unit: SpeedUnit,
}

impl Default for SpeedStatsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ip: "127.0.0.1".to_string(),
            port: 9001,
            unit: SpeedUnit::MillimetersPerSecond,
        }
    }
}

/// The root configuration struct for the application.
///
/// This structure holds all user-adjustable parameters and is the
/// primary object serialized to disk. Default struct fields are provided by individual functions
/// to facilitate serde compatibility for adding new fields to older config files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingConfig {
    #[serde(default)]
    pub mode: DriverMode,
    pub active_area: ActiveArea,
    pub target_area: TargetArea,
    #[serde(default)]
    pub relative_config: RelativeConfig,
    #[serde(default)]
    pub antichatter: AntichatterConfig,
    #[serde(default)]
    pub speed_stats: SpeedStatsConfig,
    #[serde(default = "default_threshold")]
    pub tip_threshold: u16,
    #[serde(default = "default_threshold")]
    pub eraser_threshold: u16,
    #[serde(default = "default_false")]
    pub disable_pressure: bool,
    #[serde(default = "default_false")]
    pub disable_tilt: bool,
    #[serde(default = "default_tip_binding")]
    pub tip_binding: String,
    #[serde(default = "default_eraser_binding")]
    pub eraser_binding: String,
    #[serde(default = "default_button_bindings")]
    pub pen_button_bindings: Vec<String>,
    #[serde(default = "default_false")]
    pub run_at_startup: bool,
    #[serde(default = "default_false")]
    pub system_tray_on_minimize: bool,
    #[serde(default)]
    pub websocket: WebSocketConfig,
    #[serde(default)]
    pub theme: ThemePreference,
    #[serde(default)]
    pub lock_aspect_ratio: bool,
    #[serde(default)]
    pub show_osu_playfield: bool,
}
