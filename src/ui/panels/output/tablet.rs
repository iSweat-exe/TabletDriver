use crate::app::state::TabletMapperApp;
use crate::core::config::models::MappingConfig;
use crate::ui::theme::{ui_input_box, ui_section_header};
use eframe::egui;

pub fn render_tablet_section(app: &TabletMapperApp, ui: &mut egui::Ui, config: &mut MappingConfig) {
    ui_section_header(ui, "Tablet");

    let (phys_w, phys_h) = *app.shared.physical_size.read().unwrap();
    let tablet_data = app.shared.tablet_data.read().unwrap();

    egui::Frame::canvas(ui.style())
        .fill(crate::ui::theme::panel_bg(ui.visuals()))
        .stroke(egui::Stroke::new(
            1.0,
            crate::ui::theme::panel_border(ui.visuals()),
        ))
        .show(ui, |ui| {
            let available_w = ui.available_width();
            let viz_h = 250.0;
            let (rect, response) = ui.allocate_at_least(
                egui::vec2(available_w, viz_h),
                egui::Sense::click_and_drag(),
            );

            let scale = (rect.width() / phys_w).min(rect.height() / phys_h) * 0.8;
            let draw_w = phys_w * scale;
            let draw_h = phys_h * scale;
            let offset_x = rect.center().x - draw_w / 2.0;
            let offset_y = rect.center().y - draw_h / 2.0;

            let full_rect = egui::Rect::from_min_size(
                egui::pos2(offset_x, offset_y),
                egui::vec2(draw_w, draw_h),
            );
            ui.painter().rect_stroke(
                full_rect,
                0.0,
                egui::Stroke::new(1.0, crate::ui::theme::panel_border(ui.visuals())),
            );

            let aa_center_x = offset_x + config.active_area.x * scale;
            let aa_center_y = offset_y + config.active_area.y * scale;

            let half_w = (config.active_area.w * scale) / 2.0;
            let half_h = (config.active_area.h * scale) / 2.0;

            let mut points = vec![
                egui::pos2(-half_w, -half_h),
                egui::pos2(half_w, -half_h),
                egui::pos2(half_w, half_h),
                egui::pos2(-half_w, half_h),
            ];

            let rot_rad = config.active_area.rotation.to_radians();
            let (sin, cos) = rot_rad.sin_cos();

            for p in &mut points {
                let rx = p.x * cos - p.y * sin;
                let ry = p.x * sin + p.y * cos;
                *p = egui::pos2(rx + aa_center_x, ry + aa_center_y);
            }

            let stroke_color = if ui.visuals().dark_mode {
                egui::Color32::WHITE
            } else {
                egui::Color32::BLACK
            };
            ui.painter().add(egui::Shape::convex_polygon(
                points.clone(),
                crate::ui::theme::accent_bg(ui.visuals()),
                egui::Stroke::new(1.0, stroke_color),
            ));

            ui.painter()
                .circle_filled(egui::pos2(aa_center_x, aa_center_y), 1.5, stroke_color);

            if config.show_osu_playfield {
                // Dynamic Osu! playfield calculation using actual Target Area (Screen Resolution)
                // Proportionally maps 1316x1028 (at 1080p) to the user's specific monitor
                let target_w = config.target_area.w;
                let target_h = config.target_area.h;

                let (pf_w, pf_h, y_offset_mm) = if target_w > 0.0 && target_h > 0.0 {
                    let h = config.active_area.h * (1028.0 / target_h);
                    let w = (target_h * (1316.0 / 1080.0) / target_w) * config.active_area.w;
                    let offset = config.active_area.h * (18.0 / target_h);
                    (w, h, offset)
                } else {
                    (0.0, 0.0, 0.0)
                };

                let pf_half_w = (pf_w * scale) / 2.0;
                let pf_half_h = (pf_h * scale) / 2.0;
                let pf_offset_y = y_offset_mm * scale;

                let mut pf_points = vec![
                    egui::pos2(-pf_half_w, -pf_half_h + pf_offset_y),
                    egui::pos2(pf_half_w, -pf_half_h + pf_offset_y),
                    egui::pos2(pf_half_w, pf_half_h + pf_offset_y),
                    egui::pos2(-pf_half_w, pf_half_h + pf_offset_y),
                ];

                for p in &mut pf_points {
                    let rx = p.x * cos - p.y * sin;
                    let ry = p.x * sin + p.y * cos;
                    *p = egui::pos2(rx + aa_center_x, ry + aa_center_y);
                }

                ui.painter().add(egui::Shape::convex_polygon(
                    pf_points,
                    egui::Color32::from_rgba_unmultiplied(255, 105, 180, 40),
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 105, 180)),
                ));
            }

            let font_id = egui::FontId::proportional(11.0);
            let color = if ui.visuals().dark_mode {
                egui::Color32::from_gray(20)
            } else {
                egui::Color32::BLACK
            };

            let left_mid = egui::pos2(
                (points[0].x + points[3].x) / 2.0,
                (points[0].y + points[3].y) / 2.0,
            );
            let galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    format!("{:.2}mm", config.active_area.h).replace(".", ","),
                    font_id.clone(),
                    color,
                )
            });
            ui.painter().add(egui::epaint::TextShape {
                pos: left_mid
                    + egui::vec2(
                        5.0 * rot_rad.cos() - 2.0 * rot_rad.sin(),
                        5.0 * rot_rad.sin() + 2.0 * rot_rad.cos(),
                    ),
                galley,
                underline: egui::Stroke::NONE,
                override_text_color: None,
                angle: -std::f32::consts::FRAC_PI_2 + rot_rad,
                fallback_color: color,
                opacity_factor: 1.0,
            });

            let ratio = if config.active_area.h != 0.0 {
                config.active_area.w / config.active_area.h
            } else {
                0.0
            };
            ui.painter().text(
                egui::pos2(aa_center_x, aa_center_y + 12.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.4}", ratio).replace(".", ","),
                font_id.clone(),
                color,
            );

            let top_mid = egui::pos2(
                (points[0].x + points[1].x) / 2.0,
                (points[0].y + points[1].y) / 2.0,
            );
            ui.painter().text(
                top_mid + egui::vec2(5.0 * rot_rad.sin(), 5.0 * rot_rad.cos()),
                egui::Align2::CENTER_TOP,
                format!("{:.2}mm", config.active_area.w).replace(".", ","),
                font_id.clone(),
                color,
            );

            let drag_id = ui.id().with("tablet_drag");
            let mut current_drag_delta =
                ui.data_mut(|d| d.get_temp::<egui::Vec2>(drag_id).unwrap_or_default());

            if response.dragged() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let click_rect = egui::Rect::from_min_max(
                        points
                            .iter()
                            .fold(egui::pos2(f32::INFINITY, f32::INFINITY), |a, b| {
                                egui::pos2(a.x.min(b.x), a.y.min(b.y))
                            }),
                        points
                            .iter()
                            .fold(egui::pos2(f32::NEG_INFINITY, f32::NEG_INFINITY), |a, b| {
                                egui::pos2(a.x.max(b.x), a.y.max(b.y))
                            }),
                    );

                    if click_rect.expand(20.0).contains(pointer_pos) || response.drag_started() {
                        let delta = response.drag_delta() / scale;
                        current_drag_delta += delta;
                        ui.data_mut(|d| d.insert_temp(drag_id, current_drag_delta));

                        config.active_area.x = (config.active_area.x + delta.x).clamp(
                            config.active_area.w / 2.0,
                            phys_w - config.active_area.w / 2.0,
                        );
                        config.active_area.y = (config.active_area.y + delta.y).clamp(
                            config.active_area.h / 2.0,
                            phys_h - config.active_area.h / 2.0,
                        );
                    }
                }
            } else if response.drag_stopped() {
                ui.data_mut(|d| d.insert_temp(drag_id, egui::Vec2::ZERO));
            }

            if tablet_data.is_connected {
                let (max_w, max_h) = *app.shared.hardware_size.read().unwrap();
                let cx = offset_x + (tablet_data.x as f32 / max_w) * phys_w * scale;
                let cy = offset_y + (tablet_data.y as f32 / max_h) * phys_h * scale;
                if full_rect.contains(egui::pos2(cx, cy)) {
                    let cursor_color = if ui.visuals().dark_mode {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::BLACK
                    };
                    ui.painter()
                        .circle_filled(egui::pos2(cx, cy), 3.0, cursor_color);
                }
            }
        });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut config.lock_aspect_ratio, "Force Aspect Ratio");
                ui.add_space(10.0);
                ui.checkbox(&mut config.show_osu_playfield, "Show Osu!Playfield");
            });
            ui.add_space(5.0);
            egui::Grid::new("tablet_grid")
                .spacing(egui::vec2(10.0, 10.0))
                .show(ui, |ui| {
                    let mut w = config.active_area.w;
                    let mut h = config.active_area.h;

                    ui_input_box(ui, "Width", &mut w, "mm");
                    if w != config.active_area.w {
                        if config.lock_aspect_ratio {
                            let ratio = config.active_area.w / config.active_area.h;
                            config.active_area.w = w;
                            config.active_area.h = (w / ratio).clamp(1.0, phys_h);
                        } else {
                            config.active_area.w = w;
                        }
                    }

                    ui_input_box(ui, "Height", &mut h, "mm");
                    if h != config.active_area.h {
                        if config.lock_aspect_ratio {
                            let ratio = config.active_area.w / config.active_area.h;
                            config.active_area.h = h;
                            config.active_area.w = (h * ratio).clamp(1.0, phys_w);
                        } else {
                            config.active_area.h = h;
                        }
                    }

                    ui_input_box(ui, "X", &mut config.active_area.x, "mm");
                    ui_input_box(ui, "Y", &mut config.active_area.y, "mm");
                    ui_input_box(ui, "Rotation", &mut config.active_area.rotation, "°");
                    ui.end_row();

                    config.active_area.w = config.active_area.w.clamp(1.0, phys_w);
                    config.active_area.h = config.active_area.h.clamp(1.0, phys_h);
                    config.active_area.x = config.active_area.x.clamp(
                        config.active_area.w / 2.0,
                        phys_w - config.active_area.w / 2.0,
                    );
                    config.active_area.y = config.active_area.y.clamp(
                        config.active_area.h / 2.0,
                        phys_h - config.active_area.h / 2.0,
                    );
                });
        });
    });
}
