use crate::app::state::TabletMapperApp;
use crate::domain::MappingConfig;
use crate::ui::theme::ui_setting_row;
use eframe::egui;

pub fn render_filters_panel(
    app: &mut TabletMapperApp,
    ui: &mut egui::Ui,
    config: &mut MappingConfig,
) {
    ui.horizontal(|ui| {
        // SIDEBAR
        let sidebar_width = 160.0;
        let sidebar_height = ui.available_height();

        ui.allocate_ui_with_layout(
            egui::vec2(sidebar_width, sidebar_height),
            egui::Layout::top_down_justified(egui::Align::LEFT),
            |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_gray(245))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(220)))
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        ui.set_min_height(sidebar_height);

                        let filters = ["Devocub Antichatter"];
                        for filter_name in filters {
                            let is_selected = app.selected_filter == filter_name;
                            let res = ui.selectable_label(is_selected, filter_name);
                            if res.clicked() {
                                app.selected_filter = filter_name.to_string();
                            }
                        }
                    });
            },
        );

        // CONTENT
        ui.add_space(8.0);

        ui.vertical(|ui| {
            ui.add_space(10.0);
            if app.selected_filter == "Devocub Antichatter" {
                render_antichatter_settings(ui, config);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a filter to configure");
                });
            }
            ui.add_space(20.0);
        });
    });
}

fn render_antichatter_settings(ui: &mut egui::Ui, config: &mut MappingConfig) {
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut config.antichatter.enabled,
            "Enable Devocub Antichatter",
        );
    });
    ui.add_space(12.0);

    ui.vertical(|ui| {
        ui.set_width(ui.available_width());

        ui.spacing_mut().item_spacing.y = 8.0;

        ui_setting_row(ui, "Latency", &mut config.antichatter.latency, "ms");
        ui_setting_row(
            ui,
            "Antichatter Strength",
            &mut config.antichatter.antichatter_strength,
            "",
        );
        ui_setting_row(
            ui,
            "Antichatter Multiplier",
            &mut config.antichatter.antichatter_multiplier,
            "",
        );
        ui_setting_row(
            ui,
            "Antichatter Offset X",
            &mut config.antichatter.antichatter_offset_x,
            "",
        );
        ui_setting_row(
            ui,
            "Antichatter Offset Y",
            &mut config.antichatter.antichatter_offset_y,
            "",
        );

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        ui.checkbox(&mut config.antichatter.prediction_enabled, "Prediction");
        ui.add_space(8.0);

        ui_setting_row(
            ui,
            "Prediction Strength",
            &mut config.antichatter.prediction_strength,
            "",
        );
        ui_setting_row(
            ui,
            "Prediction Sharpness",
            &mut config.antichatter.prediction_sharpness,
            "",
        );
        ui_setting_row(
            ui,
            "Prediction Offset X",
            &mut config.antichatter.prediction_offset_x,
            "",
        );
        ui_setting_row(
            ui,
            "Prediction Offset Y",
            &mut config.antichatter.prediction_offset_y,
            "",
        );

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        ui_setting_row(ui, "Frequency", &mut config.antichatter.frequency, "hz");
    });
}
