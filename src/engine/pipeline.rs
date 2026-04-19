//! # Input Processing Pipeline
//!
//! This module defines the `Pipeline` struct, which is responsible for taking raw
//! decoded hardware packets (`TabletData`) from a specific vendor driver and
//! pushing them through the mathematical and filtering transformations required
//! to produce OS-ready cursor coordinates.

use crate::core::config::models::{DriverMode, MappingConfig};
use crate::drivers::TabletData;
use std::sync::Arc;
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
        shared: &Arc<crate::engine::state::SharedState>,
    ) {
        #[cfg(debug_assertions)]
        let pipeline_start = Instant::now();

        if !data.is_connected {
            injector.set_left_button(false);
            self.reset_relative();
            filters.reset();
            return;
        }

        // Skip non-positional reports (aux, tool ID, out-of-range)
        let status = data.status.as_str();
        if !matches!(status, "Contact" | "Hover" | "Active") {
            return;
        }

        #[cfg(debug_assertions)]
        if let Ok(mut stage) = shared.debug_pipeline_stage.write() {
            *stage = "Normalize".to_string();
        }

        let now = Instant::now();
        let (max_w, max_h, max_p) = driver.get_specs();
        let (phys_w, phys_h) = driver.get_physical_specs();

        let x_mm = (data.x as f32 / max_w) * phys_w;
        let y_mm = (data.y as f32 / max_h) * phys_h;

        #[cfg(debug_assertions)]
        let transform_start = Instant::now();

        let (mut u, mut v) = crate::core::math::transform::physical_to_normalized(
            x_mm,
            y_mm,
            config.active_area.x,
            config.active_area.y,
            config.active_area.w,
            config.active_area.h,
            config.active_area.rotation,
        );

        #[cfg(debug_assertions)]
        {
            if let Ok(mut uv) = shared.debug_last_uv.write() {
                *uv = (u, v);
            }
            shared.debug_transform_time_ns.store(
                transform_start.elapsed().as_nanos() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
        }

        #[cfg(debug_assertions)]
        if let Ok(mut stage) = shared.debug_pipeline_stage.write() {
            *stage = "Filter".to_string();
        }

        #[cfg(debug_assertions)]
        let filter_start = Instant::now();

        let (nx, ny) = filters.process(u, v, config);
        u = nx;
        v = ny;

        #[cfg(debug_assertions)]
        {
            shared.debug_filter_time_ns.store(
                filter_start.elapsed().as_nanos() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
            if let Ok(mut fuv) = shared.debug_last_filtered_uv.write() {
                *fuv = (u, v);
            }
        }

        #[cfg(debug_assertions)]
        if let Ok(mut stage) = shared.debug_pipeline_stage.write() {
            *stage = "Project".to_string();
        }

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

                #[cfg(debug_assertions)]
                {
                    if let Ok(mut debug_sc) = shared.debug_last_screen.write() {
                        *debug_sc = (screen_x, screen_y);
                    }
                }

                injector.move_absolute(screen_x, screen_y, u, v);
                self.last_screen_x = screen_x;
                self.last_screen_y = screen_y;
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

                    // Approximate screen pos — no absolute reference in relative mode
                    #[cfg(debug_assertions)]
                    {
                        if let Ok(mut debug_sc) = shared.debug_last_screen.write() {
                            *debug_sc = (debug_sc.0 + dx_px, debug_sc.1 + dy_px);
                        }
                    }
                }

                self.last_screen_x = x_mm;
                self.last_screen_y = y_mm;
            }
        }

        #[cfg(debug_assertions)]
        if let Ok(mut stage) = shared.debug_pipeline_stage.write() {
            *stage = "Inject".to_string();
        }

        let pressure = if config.disable_pressure {
            max_p as u16
        } else {
            data.pressure
        };
        let threshold_raw = (config.tip_threshold as f32 / 100.0) * max_p;
        let is_down = pressure as f32 > threshold_raw;

        injector.set_left_button(is_down);

        #[cfg(debug_assertions)]
        {
            shared
                .debug_inject_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            shared.debug_pipeline_time_ns.store(
                pipeline_start.elapsed().as_nanos() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
    }
}
