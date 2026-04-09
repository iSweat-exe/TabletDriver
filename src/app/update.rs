//! # Application Update Loop
//!
//! This module contains the implementation of the `eframe::App` trait for
//! `TabletMapperApp`. The `update` function defined here is called by egui
//! every frame to process events, update state, and render the user interface.

use crate::app::state::{AppTab, TabletMapperApp};
use crate::settings::save_last_session;
use crate::ui::panels::console::render_console_panel;
use crate::ui::panels::filters::render_filters_panel;
use crate::ui::panels::output::render_output_panel;
use crate::ui::panels::pen_settings::render_pen_settings_panel;
use crate::ui::panels::release::render_release_panel;
use crate::ui::panels::settings::render_settings_panel;
use crate::ui::panels::support::render_support_panel;
use eframe::egui;
use std::sync::atomic::Ordering;

impl eframe::App for TabletMapperApp {
    /// The main application loop called by egui.
    ///
    /// # Responsibilities
    /// 1. **Data Synchronization**: Flushes the backend receiver queues to get the latest
    ///    tablet state (`TabletData`) and update statuses.
    /// 2. **Dialogs & Modals**: Renders global overlay elements (e.g., update dialog).
    /// 3. **UI Layout**: Organizes the screen into Menu Bar, Tabs, Main Panel, and Footer.
    /// 4. **State Persistence**: Detects if user interaction modified the configuration
    ///    and writes changes to both the shared engine state and disk.
    /// 5. **Debugging & Overlays**: Handles the optional advanced debugger viewport.
    ///
    /// # Performance
    /// The GUI runs asynchronously from the input capture thread. `update` requests
    /// repaints automatically when events are fired, but also enforces a minimum 1Hz
    /// refresh rate for passive status updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Event Driven Data Sync ---
        // Drain all pending tablet events to get the latest state
        let mut last_data = None;
        while let Ok(data) = self.tablet_receiver.try_recv() {
            last_data = Some(data);
        }
        if let Some(data) = last_data {
            if let Some(receive_time) = data.receive_time {
                let latency = receive_time.elapsed().as_secs_f32() * 1000.0;
                self.ui_latency_ms = latency;
                self.min_ui_latency_ms = self.min_ui_latency_ms.min(latency);
                self.max_ui_latency_ms = self.max_ui_latency_ms.max(latency);
                self.avg_ui_latency_ms += (latency - self.avg_ui_latency_ms) * 0.1;
            }
            let mut shared_data = self.shared.tablet_data.write().unwrap();
            *shared_data = data;
        }

        // Check for updates
        if let Ok(status) = self.update_receiver.try_recv() {
            if let crate::app::autoupdate::UpdateStatus::Available(release) = &status {
                log::info!(target: "Update", "Update available: {}", release.tag_name);
            }
            self.update_status = status;
        }

        crate::ui::components::update_dialog::render_update_dialog(self, ctx);

        // Get snapshot of data for UX
        // We modify local copies of config then push to shared if changed
        let mut config = self.shared.config.read().unwrap().clone();
        let initial_config = config.clone();

        // --- System Tray Handle ---
        if config.system_tray_on_minimize {
            let is_minimized = ctx.input(|i| i.viewport().minimized).unwrap_or(false);

            // Only hide the window once when it transitions to minimized
            if is_minimized && !self.was_minimized {
                log::info!(target: "Tray", "Window minimized, hiding to system tray...");
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }

            self.was_minimized = is_minimized;
        }

        // Calc Screen Bounds - Required for both Viz and Inputs
        let mut min_x = 0.0;
        let mut min_y = 0.0;
        let mut max_x = 1920.0;
        let mut max_y = 1080.0;
        if !self.displays.is_empty() {
            let mut mx = i32::MAX;
            let mut my = i32::MAX;
            let mut ax = i32::MIN;
            let mut ay = i32::MIN;
            for d in &self.displays {
                mx = mx.min(d.x);
                my = my.min(d.y);
                ax = ax.max(d.x + d.width as i32);
                ay = ay.max(d.y + d.height as i32);
            }
            min_x = mx as f32;
            min_y = my as f32;
            max_x = ax as f32;
            max_y = ay as f32;
        }

        // === UI PANELS (ORDER MATTERS) ===

        // 1. Top Menu Bar
        crate::ui::components::menu_bar::render_menu_bar(self, ctx);

        // 2. Tabs
        crate::ui::components::tabs::render_tabs(self, ctx);

        // 3. Bottom Footer
        crate::ui::components::footer::render_footer(self, ctx, &mut config);

        // 4. Central Content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
                AppTab::Output => {
                    render_output_panel(self, ui, &mut config, min_x, min_y, max_x, max_y);
                }
                AppTab::Filters => {
                    render_filters_panel(self, ui, &mut config);
                }
                AppTab::PenSettings => {
                    render_pen_settings_panel(self, ui, &mut config);
                }
                AppTab::Console => {
                    render_console_panel(self, ui);
                }
                AppTab::Settings => {
                    render_settings_panel(self, ui, &mut config);
                }
                AppTab::Support => {
                    render_support_panel(self, ui);
                }
                AppTab::Release => {
                    render_release_panel(self, ui);
                }
            });
        });

        // Push config only if actually changed by UI inputs
        if config != initial_config {
            log::info!(target: "Config", "Configuration changed via UI");
            if config.theme != initial_config.theme {
                crate::ui::theme::apply_theme(ctx, config.theme);
            }
            {
                let mut shared_config = self.shared.config.write().unwrap();
                *shared_config = config.clone();
                // Signal change to backend thread
                self.shared.config_version.fetch_add(1, Ordering::SeqCst);
            }
            // Auto-save session
            let _ = save_last_session(&config);
        }

        // --- Debugger Window ---
        if self.show_debugger {
            let viewport_id = egui::ViewportId::from_hash_of("debugger_viewport");
            let mut close_requested = false;

            ctx.show_viewport_immediate(
                viewport_id,
                egui::ViewportBuilder::default()
                    .with_title("Tablet Debugger")
                    .with_inner_size([600.0, 750.0])
                    .with_resizable(true),
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        close_requested = true;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        // --- HZ CALCULATION ---
                        let current_packets = self.shared.packet_count.load(Ordering::Relaxed);
                        let elapsed_hz = self.last_hz_update.elapsed();
                        if elapsed_hz >= std::time::Duration::from_millis(200) {
                            let delta = current_packets.saturating_sub(self.last_packet_count);
                            let hz = delta as f32 / elapsed_hz.as_secs_f32();
                            self.displayed_hz += (hz - self.displayed_hz) * 0.3;
                            self.last_packet_count = current_packets;
                            self.last_hz_update = std::time::Instant::now();
                        }

                        ui.vertical_centered(|ui| {
                            let name = self.shared.tablet_name.read().unwrap().clone();
                            ui.add_space(5.0);
                            ui.heading(
                                egui::RichText::new(name).strong().extra_letter_spacing(1.5),
                            );
                        });

                        crate::ui::panels::debugger::render_debugger_panel(
                            self.shared.clone(),
                            self.displayed_hz,
                            ui,
                        );
                    });
                },
            );

            if close_requested {
                self.show_debugger = false;
            }
        }

        if self.show_latency_stats {
            let viewport_id = egui::ViewportId::from_hash_of("performance_viewport");
            let mut close_requested = false;

            ctx.show_viewport_immediate(
                viewport_id,
                egui::ViewportBuilder::default()
                    .with_title("Input Lag & Performance Analysis")
                    .with_inner_size([500.0, 600.0])
                    .with_resizable(true),
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        close_requested = true;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        let current_packets = self.shared.packet_count.load(Ordering::Relaxed);
                        let elapsed_hz = self.last_hz_update.elapsed();
                        if elapsed_hz >= std::time::Duration::from_millis(200) {
                            let delta = current_packets.saturating_sub(self.last_packet_count);
                            let hz = delta as f32 / elapsed_hz.as_secs_f32();
                            self.displayed_hz += (hz - self.displayed_hz) * 0.3;
                            self.last_packet_count = current_packets;
                            self.last_hz_update = std::time::Instant::now();
                        }

                        ui.vertical_centered(|ui| {
                            ui.add_space(5.0);
                            ui.heading(
                                egui::RichText::new("Driver Performance Monitor")
                                    .strong()
                                    .extra_letter_spacing(1.0),
                            );
                        });

                        if crate::ui::panels::performance::render_performance_panel(
                            self.shared.clone(),
                            self.displayed_hz,
                            self.ui_latency_ms,
                            self.min_ui_latency_ms,
                            self.max_ui_latency_ms,
                            self.avg_ui_latency_ms,
                            ui,
                        ) {
                            self.min_ui_latency_ms = f32::MAX;
                            self.max_ui_latency_ms = 0.0;
                            self.avg_ui_latency_ms = 0.0;
                        }
                    });
                },
            );

            if close_requested {
                self.show_latency_stats = false;
            }
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }
}
