use crate::app::state::TabletMapperApp;
use crate::core::config::models::MappingConfig;
use eframe::egui;

pub fn render_pen_settings_panel(
    _app: &TabletMapperApp,
    ui: &mut egui::Ui,
    config: &mut MappingConfig,
) {
    ui.add_space(10.0);

    let frame = egui::Frame::group(ui.style())
        .fill(crate::ui::theme::panel_bg(ui.visuals()))
        .stroke(egui::Stroke::new(
            1.0,
            crate::ui::theme::panel_border(ui.visuals()),
        ))
        .inner_margin(10.0);

    ui.horizontal(|ui| {
        // Left Column: Tip Settings
        ui.vertical(|ui| {
            ui.strong("Tip Settings");
            frame.show(ui, |ui| {
                ui.set_width(320.0);
                egui::Grid::new("tip_settings_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Tip Binding");
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(&config.tip_binding)
                                    .background_color(crate::ui::theme::panel_border(ui.visuals())),
                            ));
                            if ui.button("...").clicked() {}
                        });
                        ui.end_row();

                        ui.label("Tip Threshold");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(&mut config.tip_threshold, 1..=100)
                                    .show_value(false),
                            );
                            ui.add(egui::DragValue::new(&mut config.tip_threshold).range(1..=100));
                        });
                        ui.end_row();
                    });
            });
        });

        ui.add_space(10.0);

        // Right Column: Eraser Settings
        ui.vertical(|ui| {
            ui.strong("Eraser Settings");
            frame.show(ui, |ui| {
                ui.set_width(320.0);
                egui::Grid::new("eraser_settings_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Eraser Binding");
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(&config.eraser_binding)
                                    .background_color(crate::ui::theme::panel_border(ui.visuals())),
                            ));
                            if ui.button("...").clicked() {}
                        });
                        ui.end_row();

                        ui.label("Eraser Threshold");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(&mut config.eraser_threshold, 1..=100)
                                    .show_value(false),
                            );
                            ui.add(
                                egui::DragValue::new(&mut config.eraser_threshold).range(1..=100),
                            );
                        });
                        ui.end_row();
                    });
            });
        });
    });

    ui.add_space(20.0);

    // Pen Buttons
    ui.strong("Pen Buttons");
    frame.show(ui, |ui| {
        ui.set_width(660.0);
        egui::Grid::new("pen_buttons_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                for i in 0..2 {
                    ui.label(format!("Pen Binding {}", i + 1));
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(&config.pen_button_bindings[i])
                                .background_color(crate::ui::theme::panel_border(ui.visuals())),
                        ));
                        if ui.button("...").clicked() {}
                    });
                    ui.end_row();
                }
            });
    });

    ui.add_space(20.0);

    // Miscellaneous
    ui.strong("Miscellaneous");
    frame.show(ui, |ui| {
        ui.set_width(660.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut config.disable_pressure, "Disable Pressure");
            ui.add_space(20.0);
            ui.checkbox(&mut config.disable_tilt, "Disable Tilt");
        });
    });
}
