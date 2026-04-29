use crate::app::state::{TabletMapperApp, UiSnapshot};
use eframe::egui;

pub fn render_menu_bar(app: &mut TabletMapperApp, ctx: &egui::Context, snapshot: &UiSnapshot) {
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
                        app.load_settings();
                    }

                    if ui.button("Save Settings").clicked() {
                        ui.close();
                        app.save_settings(&snapshot.config);
                    }

                    if ui.button("Save Settings As...").clicked() {
                        ui.close();
                        app.save_settings_as(snapshot.config.clone());
                    }

                    if ui.button("Reset to default").clicked() {
                        ui.close();
                        app.reset_to_default();
                    }

                    ui.separator();

                    if ui.button("Export .Json").clicked() {
                        ui.close();
                        app.export_settings(&snapshot.config);
                    }
                    if ui.button("Import .Json").clicked() {
                        ui.close();
                        app.import_settings();
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
