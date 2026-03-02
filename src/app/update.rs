use crate::app::state::{AppTab, TabletMapperApp};
use crate::settings::{save_last_session, save_settings};
use crate::ui::panels::console::render_console_panel;
use crate::ui::panels::filters::render_filters_panel;
use crate::ui::panels::output::render_output_panel;
use crate::ui::panels::pen_settings::render_pen_settings_panel;
use crate::ui::panels::settings::render_settings_panel;
use crate::ui::panels::tools::render_tools_panel;
use eframe::egui;
use std::sync::atomic::Ordering;

impl eframe::App for TabletMapperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Event Driven Data Sync ---
        // Drain all pending tablet events to get the latest state
        let mut last_data = None;
        while let Ok(data) = self.tablet_receiver.try_recv() {
            last_data = Some(data);
        }
        if let Some(data) = last_data {
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

        // Show update dialog if available
        let mut update_action = None;
        if let crate::app::autoupdate::UpdateStatus::Available(release) = &self.update_status {
            let mut open = true;
            let version = release.tag_name.clone();
            egui::Window::new("Update Available")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(format!("A new version ({}) is available.", version));
                        ui.label("Would you like to install it now?");
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.button("Update Now").clicked() {
                                update_action = Some(true);
                            }
                            if ui.button("Later").clicked() {
                                update_action = Some(false);
                            }
                        });
                    });
                });
        }

        if let Some(install) = update_action {
            if install {
                if let Some(release) = self.update_status.as_release() {
                    let release_clone = release.clone();
                    std::thread::spawn(move || {
                        let _ = crate::app::autoupdate::download_and_install(release_clone);
                    });
                    self.update_status = crate::app::autoupdate::UpdateStatus::Downloading(0.0);
                }
            } else {
                self.update_status = crate::app::autoupdate::UpdateStatus::Idle;
            }
        }

        // Get snapshot of data for UX
        let tablet_name = self.shared.tablet_name.read().unwrap().clone();

        // We modify local copies of config then push to shared if changed
        let mut config = self.shared.config.read().unwrap().clone();
        let initial_config = config.clone();

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
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(2.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Load Settings...").clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new()
                                .set_directory(crate::settings::get_settings_dir())
                                .add_filter("JSON", &["json"])
                                .pick_file()
                            {
                                log::debug!(target: "App", "Loading settings from {:?}", path);
                                if let Ok(cfg) =
                                    crate::settings::load_settings_from_file(path.clone())
                                {
                                    let mut shared_config = self.shared.config.write().unwrap();
                                    *shared_config = cfg.clone();
                                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                        self.profile_name = name.to_string();
                                    }
                                    let _ = save_last_session(&cfg);
                                }
                            }
                        }
                        if ui.button("Save Settings").clicked() {
                            ui.close_menu();
                            let config = self.shared.config.read().unwrap().clone();
                            let profile = if !self.profile_name.is_empty() {
                                self.profile_name.clone()
                            } else {
                                "Default".to_string()
                            };
                            log::debug!(target: "App", "Saving settings to profile: {}", profile);
                            if !self.profile_name.is_empty() {
                                let _ = save_settings(&self.profile_name, &config);
                            } else {
                                // Default to "Default" if empty
                                let _ = save_settings("Default", &config);
                                self.profile_name = "Default".to_string();
                            }
                        }
                        if ui.button("Save Settings As...").clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new()
                                .set_directory(crate::settings::get_settings_dir())
                                .add_filter("JSON", &["json"])
                                .save_file()
                            {
                                let config = self.shared.config.read().unwrap().clone();
                                // Save to the selected path
                                if let Ok(json) = serde_json::to_string_pretty(&config) {
                                    let _ = std::fs::write(&path, json);
                                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                        self.profile_name = name.to_string();
                                    }
                                }
                            }
                        }

                        if ui.button("Reset to default").clicked() {
                            ui.close_menu();
                            let default_target = crate::domain::TargetArea {
                                x: 0.0,
                                y: 0.0,
                                w: 1920.0,
                                h: 1080.0,
                            };
                            let default_active = crate::domain::ActiveArea {
                                x: 80.0,
                                y: 50.0,
                                w: 160.0,
                                h: 100.0,
                                rotation: 0.0,
                            };
                            let mut shared_config = self.shared.config.write().unwrap();
                            shared_config.target_area = default_target;
                            shared_config.active_area = default_active;
                        }

                        ui.separator();

                        if ui.button("Apply Settings").clicked() {
                            ui.close_menu();
                            log::debug!(target: "App", "Applying settings to last session");
                            let config = self.shared.config.read().unwrap().clone();
                            let _ = save_last_session(&config);
                        }

                        ui.separator();

                        if ui.button("Export .Json").clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("settings_export.json")
                                .add_filter("JSON", &["json"])
                                .save_file()
                            {
                                let config = self.shared.config.read().unwrap().clone();
                                if let Ok(json) = serde_json::to_string_pretty(&config) {
                                    let _ = std::fs::write(path, json);
                                }
                            }
                        }
                        if ui.button("Import .Json").clicked() {
                            ui.close_menu();
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .pick_file()
                            {
                                if let Ok(cfg) = crate::settings::load_settings_from_file(path) {
                                    let mut shared_config = self.shared.config.write().unwrap();
                                    *shared_config = cfg;
                                }
                            }
                        }
                    });
                    ui.menu_button("Tablet", |ui| {
                        if ui.button("Open Debugger").clicked() {
                            ui.close_menu();
                            log::debug!(target: "App", "Launching tablet debugger");
                            let _ = std::process::Command::new("cargo")
                                .args(["run", "--bin", "debugger"])
                                .spawn();
                        }
                    });
                    ui.menu_button("Plugins", |_| {});
                    ui.menu_button("Help", |_| {});
                });
            });

        // 2. Tabs
        egui::TopBottomPanel::top("tabs")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin {
                        left: 5.0,
                        right: 5.0,
                        top: 5.0,
                        bottom: 5.0,
                    })
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(220))),
            ) // Only one clean line at bottom
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_value(&mut self.active_tab, AppTab::Output, "Output")
                        .clicked()
                    {
                        log::debug!(target: "App", "Switched to Output tab");
                    }
                    if ui
                        .selectable_value(&mut self.active_tab, AppTab::Filters, "Filters")
                        .clicked()
                    {
                        log::debug!(target: "App", "Switched to Filters tab");
                    }
                    if ui
                        .selectable_value(&mut self.active_tab, AppTab::PenSettings, "Pen Settings")
                        .clicked()
                    {
                        log::debug!(target: "App", "Switched to Pen Settings tab");
                    }
                    if ui
                        .selectable_value(&mut self.active_tab, AppTab::Tools, "Tools")
                        .clicked()
                    {
                        log::debug!(target: "App", "Switched to Tools tab");
                    }
                    if ui
                        .selectable_value(&mut self.active_tab, AppTab::Console, "Console")
                        .clicked()
                    {
                        log::debug!(target: "App", "Switched to Console tab");
                    }
                    if ui
                        .selectable_value(&mut self.active_tab, AppTab::Settings, "Settings")
                        .clicked()
                    {
                        log::debug!(target: "App", "Switched to Settings tab");
                    }
                });
            });

        // 3. Bottom Footer
        egui::TopBottomPanel::bottom("footer")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(220))),
            ) // Clean line at top
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("mode_combo")
                        .selected_text("Absolute Mode")
                        .show_ui(ui, |ui| {
                            ui.label("Absolute Mode");
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("V{}", crate::VERSION))
                                .color(egui::Color32::GRAY)
                                .strong(),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&self.profile_name).strong());
                            ui.label("Profile:");
                        });

                        egui::ComboBox::from_id_salt("device_combo")
                            .width(200.0)
                            .selected_text(tablet_name)
                            .show_ui(ui, |_| {});
                    });
                });
            });

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
                AppTab::Tools => {
                    render_tools_panel(self, ui, &mut config);
                }
            });
        });

        // Push config only if actually changed by UI inputs
        if config != initial_config {
            {
                let mut shared_config = self.shared.config.write().unwrap();
                *shared_config = config.clone();
                // Signal change to backend thread
                self.shared.config_version.fetch_add(1, Ordering::SeqCst);
            }
            // Auto-save session
            let _ = save_last_session(&config);
        }

        // Ensure we keep polling occasionally for UI status (battery, connection, etc)
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }
}
