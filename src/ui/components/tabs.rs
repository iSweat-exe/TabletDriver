use crate::app::state::{AppTab, TabletMapperApp};
use eframe::egui;

pub fn render_tabs(app: &mut TabletMapperApp, ctx: &egui::Context) {
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
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.active_tab, AppTab::Output, "Output");
                ui.selectable_value(&mut app.active_tab, AppTab::Filters, "Filters");
                ui.selectable_value(&mut app.active_tab, AppTab::PenSettings, "Pen Settings");
                ui.selectable_value(&mut app.active_tab, AppTab::Console, "Console");
                ui.selectable_value(&mut app.active_tab, AppTab::Settings, "Settings");
                ui.selectable_value(&mut app.active_tab, AppTab::Support, "Support");
                ui.selectable_value(&mut app.active_tab, AppTab::Release, "Release");
            });
        });
}
