//! # Visual Theme and Custom Widgets
//!
//! This module configures the egui global styling to create a clean, modern
//! Light theme that visually aligns with the aesthetics of OpenTabletDriver (OTD).
//! It also provides reusable helper functions for consistent layout paradigms
//! (like section headers and standardized input boxes) across panels.

use eframe::egui;

/// Injects custom spacing, colors, and strokes into the `egui::Context`.
/// Called once at application startup.
pub fn apply_theme(ctx: &egui::Context) {
    // Custom style tweaks to match OTD closer
    let mut style = (*ctx.style()).clone();

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);

    // Accent colors
    let accent_color = egui::Color32::from_rgb(0, 120, 215);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent_color);
    style.visuals.selection.bg_fill = accent_color;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    ctx.set_style(style);
}

/// Returns a color for panel backgrounds that adapts to dark/light mode.
pub fn panel_bg(visuals: &egui::Visuals) -> egui::Color32 {
    if visuals.dark_mode {
        egui::Color32::from_gray(45)
    } else {
        egui::Color32::from_gray(250)
    }
}

/// Returns a color for panel borders that adapts to dark/light mode.
pub fn panel_border(visuals: &egui::Visuals) -> egui::Color32 {
    if visuals.dark_mode {
        egui::Color32::from_gray(60)
    } else {
        egui::Color32::from_gray(235)
    }
}

/// Returns a subtle text color for labels.
pub fn label_color(visuals: &egui::Visuals) -> egui::Color32 {
    visuals.text_color().gamma_multiply(0.7)
}

/// Returns the accent background color (blue area) that adapts to theme.
pub fn accent_bg(visuals: &egui::Visuals) -> egui::Color32 {
    if visuals.dark_mode {
        egui::Color32::from_rgba_unmultiplied(60, 120, 180, 255) // Darker blue
    } else {
        egui::Color32::from_rgba_unmultiplied(137, 196, 244, 255) // Original light blue
    }
}

/// Renders a standardized section header with a title and a horizontal separator line.
pub fn ui_section_header(ui: &mut egui::Ui, title: &str) {
    let text_color = ui.visuals().strong_text_color();
    ui.horizontal(|ui| {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(title)
                .size(16.0)
                .color(text_color),
        );
    });
    ui.add_space(2.0);
    ui.add(egui::Separator::default().spacing(8.0).grow(2.0));
    ui.add_space(4.0);
}

/// Renders a styled container holding a label and an `f32` DragValue input.
///
/// This creates the "pill" style input boxes heavily used in the Output tab.
pub fn ui_input_box(ui: &mut egui::Ui, label: &str, value: &mut f32, unit: &str) {
    let visuals = ui.visuals();
    let bg_fill = panel_bg(visuals);
    let border_color = panel_border(visuals);
    let label_clr = label_color(visuals);

    egui::Frame::none()
        .fill(bg_fill)
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, border_color))
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
                    label_clr,
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

/// Renders a styled container holding a label and a `u32` DragValue input.
pub fn ui_input_box_u32(ui: &mut egui::Ui, label: &str, value: &mut u32, unit: &str) {
    let visuals = ui.visuals();
    let bg_fill = panel_bg(visuals);
    let border_color = panel_border(visuals);
    let label_clr = label_color(visuals);

    egui::Frame::none()
        .fill(bg_fill)
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, border_color))
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
                    label_clr,
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

/// Renders a wide, right-aligned setting row typically used in the Filters tab.
///
/// Features a left-aligned label and a right-aligned input box to keep long parameter
/// lists visually neat.
pub fn ui_setting_row(ui: &mut egui::Ui, label: &str, value: &mut f32, unit: &str) {
    let visuals = ui.visuals();
    let bg_fill = panel_bg(visuals);
    let border_color = panel_border(visuals);
    let label_clr = label_color(visuals);

    egui::Frame::none()
        .fill(bg_fill)
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, border_color))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.set_min_width(350.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(11.0)
                        .color(label_clr),
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
