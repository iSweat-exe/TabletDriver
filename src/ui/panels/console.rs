use crate::app::state::TabletMapperApp;
use eframe::egui;

pub fn render_console_panel(_app: &TabletMapperApp, ui: &mut egui::Ui) {
    ui.add_space(5.0);
    let logs = crate::logger::LOG_BUFFER.read().unwrap();

    egui::Frame::new()
        .fill(ui.visuals().window_fill)
        .inner_margin(0.0)
        .show(ui, |ui| {
            use egui_extras::{Column, TableBuilder};

            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .vscroll(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(100.0).at_least(80.0)) // Time
                .column(Column::initial(60.0).at_least(50.0)) // Level
                .column(Column::initial(150.0).at_least(100.0)) // Group
                .column(Column::remainder().at_least(200.0)) // Message
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Time");
                    });
                    header.col(|ui| {
                        ui.strong("Level");
                    });
                    header.col(|ui| {
                        ui.strong("Group");
                    });
                    header.col(|ui| {
                        ui.strong("Message");
                    });
                })
                .body(|body| {
                    let len = logs.len();
                    body.rows(20.0, len, |mut row| {
                        let row_index = row.index();
                        // Reverse index: newest first
                        let log_index = len - 1 - row_index;
                        if let Some(log) = logs.get(log_index) {
                            row.col(|ui: &mut egui::Ui| {
                                ui.label(&log.time);
                            });
                            row.col(|ui: &mut egui::Ui| {
                                ui.label(&log.level);
                            });
                            row.col(|ui: &mut egui::Ui| {
                                ui.label(&log.group);
                            });
                            row.col(|ui: &mut egui::Ui| {
                                ui.label(&log.message);
                            });
                        }
                    });
                });
        });

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked()
                && let Ok(mut entries) = crate::logger::LOG_BUFFER.write()
            {
                entries.clear();
            }

            if ui.button("Copy All").clicked() {
                let full_log = logs
                    .iter()
                    .map(|l| format!("[{}] {} [{}] {}", l.time, l.level, l.group, l.message))
                    .collect::<Vec<_>>()
                    .join("\n");
                ui.output_mut(|o| o.commands.push(egui::OutputCommand::CopyText(full_log)));
            }
        });
    });
}
