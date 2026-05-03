use crate::app::state::{TabletMapperApp, UiSnapshot};
use eframe::egui;
use std::sync::atomic::Ordering;

pub fn render_developer_panel(app: &mut TabletMapperApp, ui: &mut egui::Ui, snapshot: &UiSnapshot) {
    ui.add_space(5.0);

    // 1. Thread & System Info
    crate::ui::theme::ui_card(ui, "System & Process", egui_phosphor::regular::CPU, |ui| {
        egui::Grid::new("sys_info_grid")
            .num_columns(4)
            .spacing([30.0, 8.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Target Triple:").weak().size(12.0));
                ui.label(
                    egui::RichText::new(format!(
                        "{}-{}",
                        std::env::consts::ARCH,
                        std::env::consts::OS
                    ))
                    .monospace()
                    .size(12.0),
                );

                ui.label(egui::RichText::new("Process ID:").weak().size(12.0));
                ui.label(
                    egui::RichText::new(format!("{}", std::process::id()))
                        .monospace()
                        .size(12.0),
                );
                ui.end_row();

                ui.label(egui::RichText::new("Threads:").weak().size(12.0));
                ui.label(
                    egui::RichText::new(format!(
                        "{}",
                        std::thread::available_parallelism()
                            .map(|n| n.get())
                            .unwrap_or(1)
                    ))
                    .monospace()
                    .size(12.0),
                );

                ui.label(egui::RichText::new("Driver Version:").weak().size(12.0));
                ui.label(egui::RichText::new(crate::VERSION).monospace().size(12.0));
                ui.end_row();
            });
    });
    ui.add_space(8.0);

    // 2. Live Pipeline Inspector
    crate::ui::theme::ui_card(
        ui,
        "Pipeline Math Operations",
        egui_phosphor::regular::MATH_OPERATIONS,
        |ui| {
            let tx_time = snapshot.debug_transform_time_ns as f32 / 1000.0;
            let fil_time = snapshot.debug_filter_time_ns as f32 / 1000.0;
            let pipe_time = snapshot.debug_pipeline_time_ns as f32 / 1000.0;
            let inject_count = snapshot.debug_inject_count;
            let stage = &snapshot.debug_pipeline_stage;

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Status:").weak().size(12.0));
                ui.label(
                    egui::RichText::new(stage)
                        .color(egui::Color32::from_rgb(137, 180, 250))
                        .strong()
                        .size(12.0),
                );

                ui.add_space(20.0);
                ui.label(egui::RichText::new("Event Injections:").weak().size(12.0));
                ui.label(
                    egui::RichText::new(format!("{}", inject_count))
                        .monospace()
                        .color(egui::Color32::LIGHT_GREEN)
                        .size(12.0),
                );

                ui.add_space(20.0);
                ui.label(egui::RichText::new("Latency Pipeline:").weak().size(12.0));
                ui.label(
                    egui::RichText::new(format!("{:.1} µs", pipe_time))
                        .monospace()
                        .strong()
                        .size(12.0),
                );
            });

            ui.add_space(10.0);

            let last_uv = snapshot.debug_last_uv;
            let fuv = snapshot.debug_last_filtered_uv;
            let screen = snapshot.debug_last_screen;

            egui::Frame::new()
                .fill(ui.visuals().window_fill.gamma_multiply(0.4))
                .stroke(ui.visuals().window_stroke)
                .corner_radius(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::Grid::new("dev_pipeline_grid")
                        .num_columns(4)
                        .spacing([40.0, 10.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Raw Normalized UV").strong().size(11.0));
                            ui.label(
                                egui::RichText::new(format!(
                                    "({:.4}, {:.4})",
                                    last_uv.0, last_uv.1
                                ))
                                .monospace(),
                            );
                            ui.label(egui::RichText::new("Transform Time:").weak().size(11.0));
                            ui.label(egui::RichText::new(format!("{:.1} µs", tx_time)).monospace());
                            ui.end_row();

                            ui.label(egui::RichText::new("Filtered UV").strong().size(11.0));
                            ui.label(
                                egui::RichText::new(format!("({:.4}, {:.4})", fuv.0, fuv.1))
                                    .monospace(),
                            );
                            ui.label(egui::RichText::new("Filter Time:").weak().size(11.0));
                            ui.label(
                                egui::RichText::new(format!("{:.1} µs", fil_time)).monospace(),
                            );
                            ui.end_row();

                            ui.label(
                                egui::RichText::new("Projected Screen Px")
                                    .strong()
                                    .size(11.0),
                            );
                            ui.label(
                                egui::RichText::new(format!("({:.1}, {:.1})", screen.0, screen.1))
                                    .monospace()
                                    .color(egui::Color32::from_rgb(166, 227, 161)),
                            );
                            ui.label(egui::RichText::new("Pipeline Execution:").weak().size(11.0));
                            ui.label(
                                egui::RichText::new(format!("{:.1} µs", pipe_time)).monospace(),
                            );
                            ui.end_row();
                        });
                });
        },
    );
    ui.add_space(8.0);

    // 3. Raw HID Buffer
    crate::ui::theme::ui_card(
        ui,
        "HID Buffer Monitor",
        egui_phosphor::regular::WIFI_HIGH,
        |ui| {
            ui.horizontal(|ui| {
                let pause_icon = if app.dev_pause_pipeline {
                    egui_phosphor::regular::PLAY
                } else {
                    egui_phosphor::regular::PAUSE
                };
                let pause_text = if app.dev_pause_pipeline {
                    "Resume Buffer"
                } else {
                    "Pause Buffer"
                };

                if ui
                    .button(format!("{} {}", pause_icon, pause_text))
                    .clicked()
                {
                    app.dev_pause_pipeline = !app.dev_pause_pipeline;
                }

                if ui
                    .button(format!("{} Clear", egui_phosphor::regular::TRASH))
                    .clicked()
                {
                    app.dev_raw_hid_history.clear();
                }
            });
            ui.add_space(5.0);

            egui::Frame::new()
                .fill(egui::Color32::from_gray(15))
                .corner_radius(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            let font_id = egui::FontId::monospace(11.0);
                            if app.dev_raw_hid_history.is_empty() {
                                ui.label(
                                    egui::RichText::new("No packet history...")
                                        .font(font_id)
                                        .color(egui::Color32::DARK_GRAY),
                                );
                            } else {
                                for pkt in &app.dev_raw_hid_history {
                                    ui.label(
                                        egui::RichText::new(pkt)
                                            .font(font_id.clone())
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                }
                            }
                        });
                });
        },
    );
    ui.add_space(8.0);

    // 4. Shared State Raw Dump
    crate::ui::theme::ui_card(
        ui,
        "Engine Memory State",
        egui_phosphor::regular::DATABASE,
        |ui| {
            let tname = &snapshot.tablet_name;
            let tvid = snapshot.tablet_vid;
            let tpid = snapshot.tablet_pid;
            let pcount = snapshot.packet_count;
            let cver = app.shared.config_version.load(Ordering::Relaxed);

            egui::Grid::new("dev_memory_grid")
                .num_columns(4)
                .spacing([30.0, 10.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("USB Interface:").weak().size(12.0));
                    ui.label(egui::RichText::new(tname).monospace().size(12.0));

                    ui.label(egui::RichText::new("VID:PID:").weak().size(12.0));
                    ui.label(
                        egui::RichText::new(format!("{:04X}:{:04X}", tvid, tpid))
                            .monospace()
                            .size(12.0),
                    );
                    ui.end_row();

                    ui.label(egui::RichText::new("Total Packets:").weak().size(12.0));
                    ui.label(
                        egui::RichText::new(format!("{}", pcount))
                            .monospace()
                            .size(12.0),
                    );

                    ui.label(egui::RichText::new("Config Version:").weak().size(12.0));
                    ui.label(
                        egui::RichText::new(format!("v{}", cver))
                            .monospace()
                            .size(12.0),
                    );
                    ui.end_row();
                });
        },
    );
    ui.add_space(8.0);

    // 5. Config Viewer
    crate::ui::theme::ui_card(
        ui,
        "Configuration Tree",
        egui_phosphor::regular::TREE_STRUCTURE,
        |ui| {
            let btn_text = if app.dev_show_full_config {
                format!("{} Hide Raw JSON", egui_phosphor::regular::CARET_UP)
            } else {
                format!("{} Expand Raw JSON", egui_phosphor::regular::CARET_DOWN)
            };

            if ui.button(btn_text).clicked() {
                app.dev_show_full_config = !app.dev_show_full_config;
            }

            if app.dev_show_full_config {
                ui.add_space(5.0);
                if let Ok(json) = serde_json::to_string_pretty(&snapshot.config) {
                    egui::Frame::new()
                        .fill(egui::Color32::from_gray(18))
                        .corner_radius(4.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            egui::ScrollArea::vertical()
                                .max_height(250.0)
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(json)
                                            .monospace()
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(205, 214, 244)),
                                    );
                                });
                        });
                }
            }
        },
    );
}
