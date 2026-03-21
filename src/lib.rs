//! # NextTabletDriver Core Library
//!
//! This library exposes the core modules of the NextTabletDriver application:
//! - **`app`**: GUI application state, lifecycle, and auto-update flow.
//! - **`core`**: Configuration models, math (transforms/matrices).
//! - **`drivers`**: Tablet data parsing, configuration, and vendor-specific protocol handling.
//! - **`engine`**: Event pipeline, shared state, input injection, and tablet management.
//! - **`filters`**: Smoothing, antichatter, and statistics filters.
//! - **`settings`**: Configuration loading/saving and session management.
//! - **`startup`**: Windows startup registration (registry/shortcuts).
//! - **`ui`**: egui panels, components, and theming.

pub mod app;
pub mod core;
pub mod drivers;
pub mod engine;
pub mod filters;
pub mod logger;
pub mod settings;
pub mod startup;
pub mod ui;

/// The current version of the application.
///
/// # Versioning Scheme
/// Format: `V.YY.DDMM.SV`
/// - `V`: Major Version
/// - `YY`: Year
/// - `DDMM`: Day and Month
/// - `SV`: Sub-Version (e.g. build increment for the day)
pub const VERSION: &str = "1.26.2103.02";
