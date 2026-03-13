//! # Input Processing Pipeline
//!
//! This module defines the `Pipeline` struct, which is responsible for taking raw
//! decoded hardware packets (`TabletData`) from a specific vendor driver and
//! pushing them through the mathematical and filtering transformations required
//! to produce OS-ready cursor coordinates.

use crate::core::config::models::{DriverMode, MappingConfig};
use crate::drivers::TabletData;
use std::time::Duration;
use std::time::Instant;

/// The core processing pipeline for tablet input events.
///
/// It maintains internal state across frames (such as previous coordinates for
/// relative mode or filter history) and orchestrates the flow from raw data ->
/// filters -> transformation -> OS injection.
pub struct Pipeline {
    /// The last known absolute screen X coordinate, used for calculating relative deltas.
    last_screen_x: f32,
    /// The last known absolute screen Y coordinate, used for calculating relative deltas.
    last_screen_y: f32,
    /// The timestamp of the previous packet, used to reset relative tracking after inactivity.
    last_packet_time: Instant,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            last_screen_x: -1.0,
            last_screen_y: -1.0,
            last_packet_time: Instant::now(),
        }
    }

    /// Resets the internal tracking for relative mode.
    /// This prevents massive cursor jumps when the pen is lifted and placed back
    /// down on a different part of the tablet.
    pub fn reset_relative(&mut self) {
        self.last_screen_x = -1.0;
        self.last_screen_y = -1.0;
    }

    /// Processes a single hardware packet through the entire stack.
    ///
    /// # Processing Steps
    /// 1. **Connection Check**: If the tablet is disconnected or out of range,
    ///    releases the primary button and resets filters/relative tracking.
    /// 2. **Physical Mapping**: Converts raw hardware units to millimeters (`x_mm`, `y_mm`).
    /// 3. **Normalization**: Maps the physical `mm` coordinates onto a `[0.0, 1.0]` UV
    ///    space based on the user's Active Area and Rotation settings.
    /// 4. **Filtering**: Passes the UV coordinates through the active `FilterPipeline`
    ///    (e.g., Antichatter, Smoothing).
    /// 5. **Projection & Injection**:
    ///    - *Absolute Mode*: Projects UV to Screen Space pixels and injects absolute changes.
    ///    - *Relative Mode*: Converts physical `mm` deltas to pixel deltas based on sensitivity.
    /// 6. **Pressure**: Evaluates current pressure against the tip threshold to trigger clicks.
    pub fn process(
        &mut self,
        data: &TabletData,
        driver: &dyn crate::drivers::NextTabletDriver,
        config: &MappingConfig,
        injector: &mut crate::engine::injector::Injector,
        filters: &mut crate::filters::FilterPipeline,
    ) {
        if !data.is_connected {
            injector.set_left_button(false);
            self.reset_relative();
            filters.reset();
            return;
        }

        let now = Instant::now();
        let (max_w, max_h, max_p) = driver.get_specs();
        let (phys_w, phys_h) = driver.get_physical_specs();

        let x_mm = (data.x as f32 / max_w) * phys_w;
        let y_mm = (data.y as f32 / max_h) * phys_h;

        let (mut u, mut v) = crate::core::math::transform::physical_to_normalized(
            x_mm,
            y_mm,
            config.active_area.x,
            config.active_area.y,
            config.active_area.w,
            config.active_area.h,
            config.active_area.rotation,
        );

        let (nx, ny) = filters.process(u, v, config);
        u = nx;
        v = ny;

        match config.mode {
            DriverMode::Absolute => {
                let (screen_x, screen_y) = crate::core::math::transform::normalized_to_screen(
                    u,
                    v,
                    config.target_area.x,
                    config.target_area.y,
                    config.target_area.w,
                    config.target_area.h,
                );

                if (screen_x - self.last_screen_x).abs() > 0.1
                    || (screen_y - self.last_screen_y).abs() > 0.1
                {
                    injector.move_absolute(screen_x, screen_y);
                    self.last_screen_x = screen_x;
                    self.last_screen_y = screen_y;
                }
            }
            DriverMode::Relative => {
                if now.duration_since(self.last_packet_time)
                    > Duration::from_millis(config.relative_config.reset_time_ms as u64)
                {
                    self.reset_relative();
                }
                self.last_packet_time = now;

                if self.last_screen_x != -1.0 && self.last_screen_y != -1.0 {
                    let (dx_px, dy_px) = crate::core::math::transform::apply_relative_delta(
                        x_mm,
                        y_mm,
                        self.last_screen_x,
                        self.last_screen_y,
                        config.relative_config.rotation,
                        config.relative_config.x_sensitivity,
                        config.relative_config.y_sensitivity,
                    );
                    injector.move_relative(dx_px, dy_px);
                }

                self.last_screen_x = x_mm;
                self.last_screen_y = y_mm;
            }
        }

        let pressure = if config.disable_pressure {
            max_p as u16
        } else {
            data.pressure
        };
        let threshold_raw = (config.tip_threshold as f32 / 100.0) * max_p;
        let is_down = pressure as f32 > threshold_raw;

        injector.set_left_button(is_down);
    }
}
