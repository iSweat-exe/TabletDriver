use eframe::egui;

pub fn apply_theme(ctx: &egui::Context) {
    // LIGHT THEME
    ctx.set_visuals(egui::Visuals::light());

    // Custom style tweaks to match OTD closer
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.visuals.widgets.active.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 120, 215));
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 120, 215);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    ctx.set_style(style);
}

// Helper for OTD-style section headers
pub fn ui_section_header(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(title)
                .size(16.0)
                .color(egui::Color32::from_gray(60)),
        );
    });
    ui.add_space(2.0);
    ui.add(egui::Separator::default().spacing(8.0).grow(2.0));
    ui.add_space(4.0);
}

// Helper for OTD-style input boxes
pub fn ui_input_box(ui: &mut egui::Ui, label: &str, value: &mut f32, unit: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(250))
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(235)))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.set_min_width(115.0); // Enforce consistent box width
            ui.horizontal(|ui| {
                // Fixed width label for alignment
                let label_w = 45.0;
                let (rect, _) =
                    ui.allocate_at_least(egui::vec2(label_w, 14.0), egui::Sense::hover());
                ui.painter().text(
                    rect.left_center(),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_gray(120),
                );

                ui.add_space(4.0);

                // DragValue
                ui.spacing_mut().item_spacing.x = 4.0;
                let response = ui.add(egui::DragValue::new(value).speed(1.0).fixed_decimals(1));
                if response.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                }

                if !unit.is_empty() {
                    ui.label(
                        egui::RichText::new(unit)
                            .size(10.0)
                            .color(egui::Color32::from_gray(180)),
                    );
                }
            });
        });
}

pub fn ui_input_box_u32(ui: &mut egui::Ui, label: &str, value: &mut u32, unit: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(250))
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(235)))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.set_min_width(115.0);
            ui.horizontal(|ui| {
                let label_w = 45.0;
                let (rect, _) =
                    ui.allocate_at_least(egui::vec2(label_w, 14.0), egui::Sense::hover());
                ui.painter().text(
                    rect.left_center(),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_gray(120),
                );

                ui.add_space(4.0);
                ui.spacing_mut().item_spacing.x = 4.0;
                let response = ui.add(egui::DragValue::new(value).speed(1.0));
                if response.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                }

                if !unit.is_empty() {
                    ui.label(
                        egui::RichText::new(unit)
                            .size(10.0)
                            .color(egui::Color32::from_gray(180)),
                    );
                }
            });
        });
}

// Wide helper for settings with long labels (like filters)
pub fn ui_setting_row(ui: &mut egui::Ui, label: &str, value: &mut f32, unit: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(250))
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(235)))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.set_min_width(350.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(11.0)
                        .color(egui::Color32::from_gray(120)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !unit.is_empty() {
                        ui.label(
                            egui::RichText::new(unit)
                                .size(10.0)
                                .color(egui::Color32::from_gray(180)),
                        );
                        ui.add_space(4.0);
                    }

                    ui.spacing_mut().item_spacing.x = 4.0;
                    let response = ui.add(
                        egui::DragValue::new(value)
                            .speed(0.1)
                            .fixed_decimals(1)
                            .clamp_existing_to_range(false),
                    );
                    if response.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                    }
                });
            });
        });
}
