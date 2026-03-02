
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveArea {
    pub x: f32, // Millimeters
    pub y: f32, // Millimeters
    pub w: f32, // Millimeters
    pub h: f32, // Millimeters
    pub rotation: f32, // Degrees
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetArea {
    pub x: f32, // Pixels
    pub y: f32, // Pixels
    pub w: f32, // Pixels
    pub h: f32, // Pixels
}


fn default_threshold() -> u16 { 10 }
fn default_false() -> bool { false }
fn default_true() -> bool { true }
fn default_tip_binding() -> String { "Mouse Button Binding: (Button: Left)".to_string() }
fn default_eraser_binding() -> String { "None".to_string() }
fn default_button_bindings() -> Vec<String> { vec!["None".to_string(), "None".to_string()] }
fn default_ws_port() -> u16 { 8080 }
fn default_ws_hz() -> u32 { 60 }

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingConfig {
    pub active_area: ActiveArea,
    pub target_area: TargetArea,
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
    #[serde(default)]
    pub websocket: WebSocketConfig,
}
