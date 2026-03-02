use eframe::egui;
use crate::app::state::TabletMapperApp;
use crate::domain::MappingConfig;

pub fn render_filters_panel(_app: &TabletMapperApp, ui: &mut egui::Ui, _config: &mut MappingConfig) {
    ui.centered_and_justified(|ui| {
        ui.label("Coming Soon...");
    });
}
