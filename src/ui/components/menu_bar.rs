use crate::app::state::{TabletMapperApp, ToastLevel};
use eframe::egui;
use std::sync::atomic::Ordering;

pub fn render_menu_bar(app: &mut TabletMapperApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::new()
                .fill(ctx.style().visuals.panel_fill)
                .inner_margin(5.0),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load Settings...").clicked() {
                        ui.close();
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(crate::settings::get_settings_dir())
                            .add_filter("JSON", &["json"])
                            .pick_file()
                        {
                            match crate::settings::load_settings_from_file(&path) {
                                Ok((cfg, corrections)) => {
                                    {
                                        let mut shared_config = app.shared.config.write().unwrap();
                                        *shared_config = cfg.clone();
                                        app.shared.config_version.fetch_add(1, Ordering::SeqCst);
                                    }
                                    let _ = app.save_sender.try_send(cfg.clone());

                                    // Update profile state to track this file
                                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                        app.profile.name = name.to_string();
                                    }
                                    app.profile.path = Some(path);
                                    app.profile.mark_saved(&cfg);

                                    crate::settings::save_session_meta(
                                        &crate::settings::SessionMeta {
                                            profile_name: app.profile.name.clone(),
                                            profile_path: app.profile.path.clone(),
                                        },
                                    );

                                    if !corrections.is_empty() {
                                        app.push_toast(
                                            format!(
                                                "Config repaired: {} field(s) reset to defaults",
                                                corrections.len()
                                            ),
                                            ToastLevel::Warning,
                                        );
                                    }
                                    app.push_toast(
                                        "Settings loaded successfully".to_string(),
                                        ToastLevel::Info,
                                    );
                                }
                                Err(e) => {
                                    app.push_toast(
                                        format!("Failed to load settings: {}", e),
                                        ToastLevel::Error,
                                    );
                                }
                            }
                        }
                    }

                    if ui.button("Save Settings").clicked() {
                        ui.close();
                        save_current_settings(app);
                    }

                    if ui.button("Save Settings As...").clicked() {
                        ui.close();
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(crate::settings::get_settings_dir())
                            .add_filter("JSON", &["json"])
                            .save_file()
                        {
                            let config = app.shared.config.read().unwrap().clone();
                            match crate::settings::save_to_path(&path, &config) {
                                Ok(()) => {
                                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                        app.profile.name = name.to_string();
                                    }
                                    app.profile.path = Some(path);
                                    app.profile.mark_saved(&config);
                                    let _ = app.save_sender.try_send(config);

                                    crate::settings::save_session_meta(
                                        &crate::settings::SessionMeta {
                                            profile_name: app.profile.name.clone(),
                                            profile_path: app.profile.path.clone(),
                                        },
                                    );
                                    app.push_toast("Settings saved".to_string(), ToastLevel::Info);
                                }
                                Err(e) => {
                                    app.push_toast(
                                        format!("Failed to save: {}", e),
                                        ToastLevel::Error,
                                    );
                                }
                            }
                        }
                    }

                    if ui.button("Reset to default").clicked() {
                        ui.close();
                        {
                            let mut shared_config = app.shared.config.write().unwrap();
                            let theme = shared_config.theme;
                            let run_at_startup = shared_config.run_at_startup;

                            *shared_config = crate::core::config::models::MappingConfig::default();
                            shared_config.theme = theme;
                            shared_config.run_at_startup = run_at_startup;

                            app.shared.config_version.fetch_add(1, Ordering::SeqCst);
                        }
                        app.push_toast("Settings reset to default (Unsaved)".to_string(), ToastLevel::Info);
                    }

                    ui.separator();

                    if ui.button("Export .Json").clicked() {
                        ui.close();
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("settings_export.json")
                            .add_filter("JSON", &["json"])
                            .save_file()
                        {
                            let config = app.shared.config.read().unwrap().clone();
                            match crate::settings::save_to_path(&path, &config) {
                                Ok(()) => {
                                    app.push_toast(
                                        "Settings exported".to_string(),
                                        ToastLevel::Info,
                                    );
                                }
                                Err(e) => {
                                    app.push_toast(
                                        format!("Export failed: {}", e),
                                        ToastLevel::Error,
                                    );
                                }
                            }
                        }
                    }
                    if ui.button("Import .Json").clicked() {
                        ui.close();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                        {
                            match crate::settings::load_settings_from_file(&path) {
                                Ok((cfg, corrections)) => {
                                    let mut shared_config = app.shared.config.write().unwrap();
                                    *shared_config = cfg;
                                    if !corrections.is_empty() {
                                        // Drop the write lock before pushing toast
                                        drop(shared_config);
                                        app.push_toast(
                                            format!(
                                                "Imported config repaired: {} field(s) reset",
                                                corrections.len()
                                            ),
                                            ToastLevel::Warning,
                                        );
                                    }
                                }
                                Err(e) => {
                                    app.push_toast(
                                        format!("Import failed: {}", e),
                                        ToastLevel::Error,
                                    );
                                }
                            }
                        }
                    }
                });
                ui.menu_button("Tablet", |ui| {
                    if ui.button("Open Debugger").clicked() {
                        ui.close();
                        app.show_debugger = true;
                    }
                    if ui.button("Input Lag Analysis").clicked() {
                        ui.close();
                        app.show_latency_stats = true;
                    }
                });
                ui.menu_button("Help", |_| {});
            });
        });
}

pub fn save_current_settings(app: &mut TabletMapperApp) {
    let config = app.shared.config.read().unwrap().clone();

    if let Some(ref path) = app.profile.path {
        // Save back to the loaded profile path
        match crate::settings::save_to_path(path, &config) {
            Ok(()) => {
                app.profile.mark_saved(&config);
                let _ = app.save_sender.try_send(config);
                app.push_toast("Settings saved".to_string(), ToastLevel::Info);
            }
            Err(e) => {
                app.push_toast(format!("Failed to save: {}", e), ToastLevel::Error);
            }
        }
    } else {
        // No profile path — prompt "Save As"
        if let Some(path) = rfd::FileDialog::new()
            .set_directory(crate::settings::get_settings_dir())
            .add_filter("JSON", &["json"])
            .save_file()
        {
            match crate::settings::save_to_path(&path, &config) {
                Ok(()) => {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        app.profile.name = name.to_string();
                    }
                    app.profile.path = Some(path);
                    app.profile.mark_saved(&config);
                    let _ = app.save_sender.try_send(config);

                    crate::settings::save_session_meta(&crate::settings::SessionMeta {
                        profile_name: app.profile.name.clone(),
                        profile_path: app.profile.path.clone(),
                    });
                    app.push_toast("Settings saved".to_string(), ToastLevel::Info);
                }
                Err(e) => {
                    app.push_toast(format!("Failed to save: {}", e), ToastLevel::Error);
                }
            }
        }
    }
}
