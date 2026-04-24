//! # Application Update Loop
//!
//! This module contains the implementation of the `eframe::App` trait for
//! `TabletMapperApp`. The `update` function defined here is called by egui
//! every frame to process events, update state, and render the user interface.

use crate::app::state::{AppTab, TabletMapperApp, ToastLevel};
use crate::ui::panels::console::render_console_panel;
#[cfg(debug_assertions)]
use crate::ui::panels::developer::render_developer_panel;
use crate::ui::panels::filters::render_filters_panel;
use crate::ui::panels::output::render_output_panel;
use crate::ui::panels::pen_settings::render_pen_settings_panel;
use crate::ui::panels::release::render_release_panel;
use crate::ui::panels::settings::render_settings_panel;
use eframe::egui::{self, Shadow};
use std::sync::atomic::Ordering;
use std::time::Duration;

/// Duration before a toast notification auto-dismisses.
const TOAST_DURATION: Duration = Duration::from_secs(3);

impl eframe::App for TabletMapperApp {
    /// The main application loop called by egui.
    ///
    /// # Responsibilities
    /// 1. **Data Synchronization**: Flushes the backend receiver queues to get the latest
    ///    tablet state (`TabletData`) and update statuses.
    /// 2. **Dialogs & Modals**: Renders global overlay elements (e.g., update dialog).
    /// 3. **UI Layout**: Organizes the screen into Menu Bar, Tabs, Main Panel, and Footer.
    /// 4. **State Persistence**: Detects if user interaction modified the configuration
    ///    and sends changes to the background saver thread asynchronously.
    /// 5. **Debugging & Overlays**: Handles the optional advanced debugger viewport.
    /// 6. **Toast Notifications**: Renders transient notifications and auto-dismisses expired ones.
    ///
    /// # Performance
    /// The GUI runs asynchronously from the input capture thread. `update` requests
    /// repaints automatically when events are fired, but also enforces a minimum 1Hz
    /// refresh rate for passive status updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Shortcuts
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::S,
            ))
        }) {
            crate::ui::components::menu_bar::save_current_settings(self);
        }

        // Drain pending tablet events, keeping only the latest
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

            #[cfg(debug_assertions)]
            if !self.dev_pause_pipeline {
                self.dev_raw_hid_history.push_front(data.raw_data.clone());
                if self.dev_raw_hid_history.len() > 50 {
                    self.dev_raw_hid_history.pop_back();
                }
            }

            let mut shared_data = self.shared.tablet_data.write().unwrap();
            *shared_data = data;

            ctx.request_repaint();
        }

        if let Ok(status) = self.update_receiver.try_recv() {
            if let crate::app::autoupdate::UpdateStatus::Available(release) = &status {
                log::info!(target: "Update", "Update available: {}", release.tag_name);
            }
            self.update_status = status;
        }

        crate::ui::components::update_dialog::render_update_dialog(self, ctx);

        // Unsaved Changes Close Guard
        {
            let close_requested = ctx.input(|i| i.viewport().close_requested());
            let config_snapshot = self.shared.config.read().unwrap().clone();
            let is_dirty = self.profile.is_dirty(&config_snapshot);

            if close_requested && is_dirty && !self.show_close_confirm && !self.force_close {
                // Block the close and show the confirmation dialog
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_close_confirm = true;
            }
        }

        if self.show_close_confirm {
            let frame = egui::Frame::window(&ctx.style()).shadow(Shadow::NONE);
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(frame)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label("Are you sure you want to close the application?");
                        ui.label("The current profile has unsaved changes.");
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.show_close_confirm = false;
                            }
                            ui.add_space(8.0);
                            if ui
                                .button(
                                    egui::RichText::new("Close Anyway")
                                        .color(egui::Color32::from_rgb(220, 80, 80)),
                                )
                                .clicked()
                            {
                                // REVERT last_session.json to the unmodified state before closing
                                let _ =
                                    crate::settings::save_last_session(&self.profile.last_saved);
                                self.force_close = true;
                                self.show_close_confirm = false;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ui.add_space(4.0);
                    });
                });
        }

        // Clone config for diffing — UI mutates this copy, then we push back if changed
        let mut config = self.shared.config.read().unwrap().clone();
        let initial_config = config.clone();

        // System Tray Minimize-to-Tray
        if config.system_tray_on_minimize {
            let is_minimized = ctx.input(|i| i.viewport().minimized).unwrap_or(false);

            if is_minimized && !self.was_minimized {
                log::info!(target: "Tray", "Window minimized, hiding to system tray...");
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }

            self.was_minimized = is_minimized;
        }

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

        crate::ui::components::menu_bar::render_menu_bar(self, ctx);

        crate::ui::components::tabs::render_tabs(self, ctx);

        crate::ui::components::footer::render_footer(self, ctx, &mut config);

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            AppTab::Output => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_output_panel(self, ui, &mut config, min_x, min_y, max_x, max_y);
                });
            }
            AppTab::Filters => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_filters_panel(self, ui, &mut config);
                });
            }
            AppTab::PenSettings => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_pen_settings_panel(self, ui, &mut config);
                });
            }
            AppTab::Console => {
                render_console_panel(self, ui);
            }
            AppTab::Settings => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_settings_panel(self, ui, &mut config);
                });
            }
            AppTab::Release => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_release_panel(self, ui);
                });
            }
            #[cfg(debug_assertions)]
            AppTab::Developer => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_developer_panel(self, ui, &config);
                });
            }
        });

        // Push config back to shared state only if the UI actually mutated it
        if config != initial_config {
            log::info!(target: "Config", "Configuration changed via UI");
            if config.theme != initial_config.theme {
                crate::ui::theme::apply_theme(ctx, config.theme);
            }
            {
                let mut shared_config = self.shared.config.write().unwrap();
                *shared_config = config.clone();

                self.shared.config_version.fetch_add(1, Ordering::SeqCst);
            }

            // Send to background saver — non-blocking, drops if channel full
            let _ = self.save_sender.try_send(config.clone());
        }

        // Toast Notifications
        // Expire old toasts
        self.toasts
            .retain(|t| t.created_at.elapsed() < TOAST_DURATION);

        // Render active toasts anchored to bottom-right
        if !self.toasts.is_empty() {
            // Request repaint so toasts auto-dismiss smoothly
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        for (i, toast) in self.toasts.iter().enumerate() {
            let offset_y = i as f32 * 50.0 + 10.0;
            let id = egui::Id::new("toast").with(i);

            let (bg_color, text_color) = match toast.level {
                ToastLevel::Info => (egui::Color32::from_rgb(40, 120, 60), egui::Color32::WHITE),
                ToastLevel::Warning => {
                    (egui::Color32::from_rgb(180, 130, 30), egui::Color32::WHITE)
                }
                ToastLevel::Error => (egui::Color32::from_rgb(180, 50, 50), egui::Color32::WHITE),
            };

            egui::Area::new(id)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, offset_y))
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(&toast.message).color(text_color));
                        });
                });
        }

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
                        // HZ
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

        // Cap UI to ~1 FPS idle repaint to avoid burning CPU/GPU
        // when no tablet data is flowing. Real repaints are event-driven.
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
