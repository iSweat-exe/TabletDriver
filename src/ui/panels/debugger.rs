use std::sync::Arc;
use eframe::egui;
use crate::input::SharedState;

pub fn render_debugger_panel(shared: Arc<SharedState>, displayed_hz: f32, ui: &mut egui::Ui) {
    let tablet_data = shared.tablet_data.read().unwrap();
    let tablet_name = shared.tablet_name.read().unwrap();
    let is_detected = *tablet_name != "No Tablet Detected";
    
    let (max_x, max_y, max_p) = shared.hardware_size.read().map(|s| (s.0, s.1, 8192.0)).unwrap_or((32767.0, 32767.0, 8192.0));
    
    ui.add_space(10.0);
    
    // 1. VISUALIZER
    let available_width = ui.available_width();
    let desired_height = (available_width * (9.0 / 16.0)).min(300.0);
    
    let (rect, _) = ui.allocate_at_least(
        egui::vec2(available_width, desired_height), 
        egui::Sense::hover()
    );
    
    // Background
    ui.painter().rect_filled(rect, 8.0, egui::Color32::from_rgb(15, 15, 15));
    ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)));

    if is_detected && tablet_data.is_connected {
        let x_pct = tablet_data.x as f32 / max_x;
        let y_pct = tablet_data.y as f32 / max_y;
        
        let dot_pos = egui::pos2(
            rect.left() + x_pct * rect.width(),
            rect.top() + y_pct * rect.height()
        );

        if tablet_data.status == "Contact" || tablet_data.pressure > 0 {
            ui.painter().circle_filled(dot_pos, 10.0, egui::Color32::from_rgba_unmultiplied(0, 120, 255, 40));
            ui.painter().circle_filled(dot_pos, 4.0, egui::Color32::from_rgb(0, 150, 255));
        } else {
            ui.painter().circle_filled(dot_pos, 3.0, egui::Color32::from_gray(200));
        }
    } else {
        let status_text = if !is_detected {
            "NO USB DEVICE DETECTED"
        } else {
            "PEN OUT OF RANGE"
        };
        
        ui.painter().text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, 
            status_text, 
            egui::FontId::proportional(16.0), 
            egui::Color32::DARK_GRAY
        );
    }

    ui.add_space(20.0);

    // 2. DASHBOARD (Fixed columns instead of Grid)
    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            status_card(ui, "REPORT STATUS", &tablet_data.status, egui::Color32::LIGHT_GREEN);
            ui.add_space(10.0);
            status_card(ui, "COORDINATES", &format!("X: {}, Y: {}", tablet_data.x, tablet_data.y), egui::Color32::WHITE);
            ui.add_space(10.0);
            let tilt_str = format!("X: {}, Y: {}", tablet_data.tilt_x, tablet_data.tilt_y);
            status_card(ui, "PEN TILT", &tilt_str, egui::Color32::from_rgb(255, 100, 255));
        });
        cols[1].vertical(|ui| {
            status_card(ui, "REPORT RATE", &format!("{:.0} Hz", displayed_hz), egui::Color32::GOLD);
            ui.add_space(10.0);
            status_card(ui, "PRESSURE", &format!("{} / {}", tablet_data.pressure, max_p as u16), egui::Color32::LIGHT_BLUE);
            ui.add_space(10.0);
            let b1 = (tablet_data.buttons & 0x01) != 0;
            let b2 = (tablet_data.buttons & 0x02) != 0;
            let btn_str = format!("B1: {} | B2: {}", b1, b2);
            status_card(ui, "BUTTONS", &btn_str, if b1 || b2 { egui::Color32::from_rgb(255, 165, 0) } else { egui::Color32::from_gray(150) });
        });
    });

    ui.add_space(20.0);

    // 3. RAW DATA (Hexadecimal and Binary)
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_gray(30))
        .show(ui, |ui: &mut egui::Ui| {
            ui.set_width(ui.available_width());
            
            ui.label(egui::RichText::new("Raw Tablet Stream").weak().size(11.0));
            ui.label(egui::RichText::new(&tablet_data.raw_data).code().size(12.0).color(egui::Color32::LIGHT_GRAY));

            ui.add_space(20.0);

            ui.label(egui::RichText::new("Raw Tablet Stream (Binary)").weak().size(11.0));
            let binary_string = tablet_data.raw_data
                .split_whitespace()
                .filter_map(|hex| u8::from_str_radix(hex, 16).ok())
                .map(|byte| format!("{:08b}", byte))
                .collect::<Vec<String>>()
                .join(" ");

            ui.label(egui::RichText::new(binary_string).code().size(12.0).color(egui::Color32::LIGHT_GRAY));
        });
}

fn status_card(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(28))
        .rounding(6.0)
        .inner_margin(12.0)
        .show(ui, |ui: &mut egui::Ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(label).weak().size(10.0));
            ui.label(egui::RichText::new(value).color(color).strong().size(17.0));
        });
}
