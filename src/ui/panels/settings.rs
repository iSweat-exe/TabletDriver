use crate::app::state::TabletMapperApp;
use crate::core::config::models::MappingConfig;
use crate::ui::theme::ui_section_header;
use eframe::egui;

pub fn render_settings_panel(
    _app: &TabletMapperApp,
    ui: &mut egui::Ui,
    config: &mut MappingConfig,
) {
    ui.add_space(10.0);
    ui_section_header(ui, "General Settings");

    let frame = egui::Frame::group(ui.style())
        .fill(crate::ui::theme::panel_bg(ui.visuals()))
        .stroke(egui::Stroke::new(
            1.0,
            crate::ui::theme::panel_border(ui.visuals()),
        ))
        .inner_margin(10.0);

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());

        let old_run_at_startup = config.run_at_startup;
        if ui
            .checkbox(&mut config.run_at_startup, "Run at startup")
            .on_hover_text("Automatically launch the application when Windows starts.")
            .changed()
            && let Err(e) = crate::startup::set_run_at_startup(config.run_at_startup)
        {
            log::error!(target: "App", "Failed to update startup setting: {}", e);
            config.run_at_startup = old_run_at_startup;
        }

        ui.add_space(4.0);
        ui.checkbox(&mut config.system_tray_on_minimize, "System Tray when Minimize")
            .on_hover_text("Hide the application to the system tray when minimized.");

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Application Theme:");
            egui::ComboBox::from_id_salt("theme_selector")
                .selected_text(format!("{:?}", config.theme))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::System,
                        "System",
                    );
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::Light,
                        "Light",
                    );
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::Dark,
                        "Dark",
                    );
                    ui.separator();
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::CatppuccinLatte,
                        "Catppuccin Latte",
                    );
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::CatppuccinFrappe,
                        "Catppuccin Frappe",
                    );
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::CatppuccinMacchiato,
                        "Catppuccin Macchiato",
                    );
                    ui.selectable_value(
                        &mut config.theme,
                        crate::core::config::models::ThemePreference::CatppuccinMocha,
                        "Catppuccin Mocha",
                    );
                });
        });
    });

    ui.add_space(10.0);
    ui_section_header(ui, "WebSocket Server");
    let ws_frame = egui::Frame::group(ui.style())
        .fill(crate::ui::theme::panel_bg(ui.visuals()))
        .stroke(egui::Stroke::new(
            1.0,
            crate::ui::theme::panel_border(ui.visuals()),
        ))
        .inner_margin(10.0);

    ws_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.horizontal(|ui| {
            ui.checkbox(&mut config.websocket.enabled, "Enable WebSocket Server");
            if config.websocket.enabled {
                ui.label(egui::RichText::new("Running").color(egui::Color32::from_rgb(0, 200, 0)));
            } else {
                ui.label(egui::RichText::new("Stopped").color(egui::Color32::RED));
            }
        });

        ui.add_enabled_ui(config.websocket.enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(egui::DragValue::new(&mut config.websocket.port).range(1024..=65535));

                ui.add_space(20.0);

                ui.label("Polling Rate (Hz):");
                ui.add(egui::DragValue::new(&mut config.websocket.polling_rate_hz).range(1..=1000));
            });

            ui.add_space(5.0);
            ui.label("Payload Data:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut config.websocket.send_coordinates, "Coordinates");
                ui.checkbox(&mut config.websocket.send_pressure, "Pressure");
                ui.checkbox(&mut config.websocket.send_tilt, "Tilt");
                ui.checkbox(&mut config.websocket.send_status, "Status & Keys");
            });
        });
    });
}
