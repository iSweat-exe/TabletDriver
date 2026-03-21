pub mod antichatter;
pub mod stats;

use crate::app::state::TabletMapperApp;
use crate::core::config::models::MappingConfig;
use eframe::egui;

pub fn render_filters_panel(
    app: &mut TabletMapperApp,
    ui: &mut egui::Ui,
    config: &mut MappingConfig,
) {
    ui.horizontal(|ui| {
        let sidebar_width = 175.0;
        let sidebar_height = ui.available_height();

        ui.allocate_ui_with_layout(
            egui::vec2(sidebar_width, sidebar_height),
            egui::Layout::top_down_justified(egui::Align::LEFT),
            |ui| {
                egui::Frame::new()
                    .fill(crate::ui::theme::panel_bg(ui.visuals()))
                    .stroke(egui::Stroke::new(
                        1.0,
                        crate::ui::theme::panel_border(ui.visuals()),
                    ))
                    .inner_margin(4.0)
                    .corner_radius(8.0)
                    .show(ui, |ui| {
                        ui.set_min_height(sidebar_height);

                        let filters = ["Devocub Antichatter", "HandSpeed WebSocket"];
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

        ui.add_space(8.0);

        ui.vertical(|ui| {
            ui.add_space(10.0);
            match app.selected_filter.as_str() {
                "Devocub Antichatter" => antichatter::render_antichatter_settings(ui, config),
                "HandSpeed WebSocket" => stats::render_stats_settings(app, ui, config),
                _ => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a filter to configure");
                    });
                }
            }
            ui.add_space(20.0);
        });
    });
}
