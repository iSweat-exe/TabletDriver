use crate::app::state::TabletMapperApp;
use crate::settings::save_last_session;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub fn render_menu_bar(app: &mut TabletMapperApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::none()
                .fill(ctx.style().visuals.panel_fill)
                .inner_margin(5.0),
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
                            let shared_clone = Arc::clone(&app.shared);
                            let path_clone = path.clone();
                            std::thread::spawn(move || {
                                if let Ok(cfg) =
                                    crate::settings::load_settings_from_file(path_clone)
                                {
                                    let mut shared_config = shared_clone.config.write().unwrap();
                                    *shared_config = cfg.clone();
                                    shared_clone.config_version.fetch_add(1, Ordering::SeqCst);
                                    let _ = crate::settings::save_last_session(&cfg);
                                }
                            });
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                app.profile_name = name.to_string();
                            }
                        }
                    }
                    if ui.button("Save Settings").clicked() {
                        ui.close_menu();
                        let config = app.shared.config.read().unwrap().clone();
                        let profile = if !app.profile_name.is_empty() {
                            app.profile_name.clone()
                        } else {
                            "Default".to_string()
                        };
                        std::thread::spawn(move || {
                            let _ = crate::settings::save_settings(&profile, &config);
                        });
                        if app.profile_name.is_empty() {
                            app.profile_name = "Default".to_string();
                        }
                    }
                    if ui.button("Save Settings As...").clicked() {
                        ui.close_menu();
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(crate::settings::get_settings_dir())
                            .add_filter("JSON", &["json"])
                            .save_file()
                        {
                            let config = app.shared.config.read().unwrap().clone();
                            let path_clone = path.clone();
                            std::thread::spawn(move || {
                                if let Ok(json) = serde_json::to_string_pretty(&config) {
                                    let _ = std::fs::write(&path_clone, json);
                                }
                            });
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                app.profile_name = name.to_string();
                            }
                        }
                    }

                    if ui.button("Reset to default").clicked() {
                        ui.close_menu();
                        let default_target = crate::core::config::models::TargetArea {
                            x: 0.0,
                            y: 0.0,
                            w: 1920.0,
                            h: 1080.0,
                        };
                        let default_active = crate::core::config::models::ActiveArea {
                            x: 80.0,
                            y: 50.0,
                            w: 160.0,
                            h: 100.0,
                            rotation: 0.0,
                        };
                        let mut shared_config = app.shared.config.write().unwrap();
                        shared_config.target_area = default_target;
                        shared_config.active_area = default_active;
                    }

                    ui.separator();

                    if ui.button("Apply Settings").clicked() {
                        ui.close_menu();
                        let config = app.shared.config.read().unwrap().clone();
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
                            let config = app.shared.config.read().unwrap().clone();
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
                                let mut shared_config = app.shared.config.write().unwrap();
                                *shared_config = cfg;
                            }
                        }
                    }
                });
                ui.menu_button("Tablet", |ui| {
                    if ui.button("Open Debugger").clicked() {
                        ui.close_menu();
                        app.show_debugger = true;
                    }
                    if ui.button("Input Lag Analysis").clicked() {
                        ui.close_menu();
                        app.show_latency_stats = true;
                    }
                });
                ui.menu_button("Help", |_| {});
            });
        });
}
