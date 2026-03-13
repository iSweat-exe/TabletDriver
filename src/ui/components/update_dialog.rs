use crate::app::state::TabletMapperApp;
use eframe::egui;

pub fn render_update_dialog(app: &mut TabletMapperApp, ctx: &egui::Context) {
    let mut update_action = None;

    if let crate::app::autoupdate::UpdateStatus::Available(release) = &app.update_status {
        // --- 1. Fullscreen Dark Overlay ---
        let screen_rect = ctx.screen_rect();
        egui::Area::new(egui::Id::new("update_overlay"))
            .interactable(true)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_black_alpha(180), // Dimming effect
                );
            });

        // --- 2. The Centered Dialog Window ---
        let mut open = true;
        let version = release.tag_name.clone();
        let body = release
            .body
            .clone()
            .unwrap_or_else(|| "No changelog provided.".to_string());

        egui::Window::new("Update Available")
            .title_bar(false) // Hide standard title bar for custom styling
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER) // Center the window exactly
            .default_pos(screen_rect.center()) // Center position
            .fixed_size([450.0, 350.0])
            .open(&mut open)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(20, 22, 25)) // Dark modern background
                    .rounding(4.0)
                    .inner_margin(0.0)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60))),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Header Area
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 34, 40))
                        .rounding(egui::Rounding {
                            nw: 12.0,
                            ne: 12.0,
                            sw: 0.0,
                            se: 0.0,
                        })
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("Update Available!")
                                        .size(24.0)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                );
                                ui.add_space(5.0);
                                ui.label(
                                    egui::RichText::new(format!("Version {}", version))
                                        .size(14.0)
                                        .color(egui::Color32::from_gray(180)),
                                );
                            });
                        });

                    ui.add_space(15.0);

                    // Body Area (Changelog)
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(20.0, 0.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("What's new:")
                                    .strong()
                                    .size(16.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                            ui.add_space(8.0);

                            // Scrollable area for the changelog
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(body)
                                            .size(13.0)
                                            .color(egui::Color32::from_gray(160)),
                                    );
                                });
                        });

                    ui.add_space(30.0);

                    // Actions Area
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(20.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let later_btn = egui::Button::new(
                                    egui::RichText::new("Remind Me Later")
                                        .size(14.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                                .min_size(egui::vec2(120.0, 36.0));

                                if ui.add(later_btn).clicked() {
                                    update_action = Some(false);
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let update_btn = egui::Button::new(
                                            egui::RichText::new("Install Update")
                                                .size(14.0)
                                                .strong()
                                                .color(egui::Color32::BLACK),
                                        )
                                        .fill(egui::Color32::from_rgb(0, 120, 215)) // #0078d7
                                        .rounding(6.0)
                                        .min_size(egui::vec2(160.0, 36.0));

                                        if ui.add(update_btn).clicked() {
                                            update_action = Some(true);
                                        }
                                    },
                                );
                            });
                        });

                    ui.add_space(5.0);
                });
            });
    }

    if let Some(install) = update_action {
        if install {
            if let Some(release) = app.update_status.as_release() {
                let release_clone = release.clone();
                std::thread::spawn(move || {
                    let _ = crate::app::autoupdate::download_and_install(release_clone);
                });
                app.update_status = crate::app::autoupdate::UpdateStatus::Downloading(0.0);
            }
        } else {
            app.update_status = crate::app::autoupdate::UpdateStatus::Idle;
        }
    }
}
