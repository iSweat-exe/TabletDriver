use eframe::egui;
use crate::app::state::TabletMapperApp;
use crate::domain::MappingConfig;
use crate::ui::theme::{ui_section_header, ui_input_box};

pub fn render_output_panel(app: &TabletMapperApp, ui: &mut egui::Ui, config: &mut MappingConfig, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
    ui.add_space(10.0);
    
    // === DISPLAY SECTION ===
    ui_section_header(ui, "Display");
    
    // Frame for Display Viz
    egui::Frame::canvas(ui.style()).stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200))).show(ui, |ui| {
        let available_w = ui.available_width();
        let viz_h = 200.0;
        let (rect, response) = ui.allocate_at_least(egui::vec2(available_w, viz_h), egui::Sense::click_and_drag());
        
        let desk_w = max_x - min_x;
        let desk_h = max_y - min_y;

         if desk_w > 0.0 {
             let scale = (rect.width() / desk_w).min(rect.height() / desk_h) * 0.9;
             let offset_x = rect.center().x - (desk_w * scale) / 2.0;
             let offset_y = rect.center().y - (desk_h * scale) / 2.0;
             
             // Draw Screens
             for d in app.displays.iter() {
                 let s_rect = egui::Rect::from_min_size(
                     egui::pos2(offset_x + (d.x as f32 - min_x) * scale, offset_y + (d.y as f32 - min_y) * scale),
                     egui::vec2(d.width as f32 * scale, d.height as f32 * scale)
                 );
                 
                 ui.painter().rect_stroke(s_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                 ui.painter().text(s_rect.center(), egui::Align2::CENTER_CENTER, format!("{}px", d.width), egui::FontId::proportional(10.0), egui::Color32::BLACK);
             }

             // Draw Target Area (Blue Overlay)
             let t_rect = egui::Rect::from_min_size(
                 egui::pos2(offset_x + (config.target_area.x - min_x) * scale, offset_y + (config.target_area.y - min_y) * scale),
                 egui::vec2(config.target_area.w * scale, config.target_area.h * scale)
             );
             ui.painter().rect_filled(t_rect, 0.0, egui::Color32::from_rgba_unmultiplied(137, 196, 244, 255)); // OTD Blueish
             ui.painter().rect_stroke(t_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));

             // Center Indicator (Dot)
             ui.painter().circle_filled(t_rect.center(), 1.5, egui::Color32::BLACK);

             // Labels Style
             let font_id = egui::FontId::proportional(12.0);
             let color = egui::Color32::BLACK;

             // Width (Top - Inside)
             ui.painter().text(t_rect.center_top() + egui::vec2(0.0, 5.0), egui::Align2::CENTER_TOP, format!("{}px", config.target_area.w as i32), font_id.clone(), color);

             // Height (Left - Inside) - Rotated
             let left_mid = t_rect.left_center();
             let galley = ui.fonts(|f| f.layout_no_wrap(format!("{}px", config.target_area.h as i32), font_id.clone(), color));
             ui.painter().add(egui::epaint::TextShape {
                 pos: left_mid + egui::vec2(5.0, 0.0),
                 galley,
                 underline: egui::Stroke::NONE,
                 override_text_color: None,
                 angle: -std::f32::consts::FRAC_PI_2,
                 fallback_color: color,
                 opacity_factor: 1.0,
             });

             // Aspect Ratio (Below center)
             let ratio = if config.target_area.h != 0.0 { config.target_area.w / config.target_area.h } else { 0.0 };
             ui.painter().text(t_rect.center() + egui::vec2(0.0, 12.0), egui::Align2::CENTER_CENTER, format!("{:.4}", ratio).replace(".", ","), font_id.clone(), color);

             // Drag Interaction
             if response.dragged() {
                 if let Some(pointer_pos) = response.interact_pointer_pos() {
                     if t_rect.expand(20.0).contains(pointer_pos) || response.drag_started() {
                         let delta = response.drag_delta();
                         config.target_area.x = (config.target_area.x + delta.x / scale).clamp(min_x, max_x - config.target_area.w);
                         config.target_area.y = (config.target_area.y + delta.y / scale).clamp(min_y, max_y - config.target_area.h);
                     }
                 }
             }
         }
    });

    ui.add_space(8.0);

    // Display Settings Inputs - Aligned Grid
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        egui::Grid::new("display_grid").spacing(egui::vec2(10.0, 10.0)).show(ui, |ui| {
         ui_input_box(ui, "Width", &mut config.target_area.w, "px");
         ui_input_box(ui, "Height", &mut config.target_area.h, "px");
         
         // Clamp dimensions to total workspace size
         config.target_area.w = config.target_area.w.clamp(10.0, max_x - min_x);
         config.target_area.h = config.target_area.h.clamp(10.0, max_y - min_y);

         // UI Display Values: Relative to leftmost/topmost, using center as reference
         let mut ui_x = config.target_area.x - min_x + config.target_area.w / 2.0;
         let mut ui_y = config.target_area.y - min_y + config.target_area.h / 2.0;
         
         let old_ui_x = ui_x;
         let old_ui_y = ui_y;

         ui_input_box(ui, "X", &mut ui_x, "px");
         ui_input_box(ui, "Y", &mut ui_y, "px");

         // If changed via input box, update config with clamping
         if (ui_x - old_ui_x).abs() > 0.1 {
             config.target_area.x = (ui_x - config.target_area.w / 2.0 + min_x).clamp(min_x, max_x - config.target_area.w);
         }
         if (ui_y - old_ui_y).abs() > 0.1 {
             config.target_area.y = (ui_y - config.target_area.h / 2.0 + min_y).clamp(min_y, max_y - config.target_area.h);
         }
         ui.end_row();
    });
});

    ui.add_space(20.0);
    
    // === TABLET SECTION ===
    ui_section_header(ui, "Tablet");

    let (phys_w, phys_h) = *app.shared.physical_size.read().unwrap();
    let tablet_data = app.shared.tablet_data.read().unwrap();

    egui::Frame::canvas(ui.style()).stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200))).show(ui, |ui| {
        let available_w = ui.available_width();
        let viz_h = 250.0;
        let (rect, response) = ui.allocate_at_least(egui::vec2(available_w, viz_h), egui::Sense::click_and_drag());
        
        let scale = (rect.width() / phys_w).min(rect.height() / phys_h) * 0.8; 
        let draw_w = phys_w * scale;
        let draw_h = phys_h * scale;
        let offset_x = rect.center().x - draw_w / 2.0;
        let offset_y = rect.center().y - draw_h / 2.0;
        
        let full_rect = egui::Rect::from_min_size(egui::pos2(offset_x, offset_y), egui::vec2(draw_w, draw_h));
        ui.painter().rect_stroke(full_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

        // Draw Active Area (Blue) with Rotation - Center based
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
        
        ui.painter().add(egui::Shape::convex_polygon(
            points.clone(),
            egui::Color32::from_rgba_unmultiplied(137, 196, 244, 255), 
            egui::Stroke::new(1.0, egui::Color32::BLACK)
        ));
        
        // Center Indicator (Dot)
        ui.painter().circle_filled(egui::pos2(aa_center_x, aa_center_y), 1.5, egui::Color32::BLACK);

        // Dimension Labels
        let font_id = egui::FontId::proportional(11.0);
        let color = egui::Color32::BLACK;
        
        // Height (Left) - Rotated text inside
        let left_mid = egui::pos2((points[0].x + points[3].x) / 2.0, (points[0].y + points[3].y) / 2.0);
        let galley = ui.fonts(|f| f.layout_no_wrap(format!("{:.2}mm", config.active_area.h).replace(".", ","), font_id.clone(), color));
        ui.painter().add(egui::epaint::TextShape {
            pos: left_mid + egui::vec2(5.0 * rot_rad.cos() - 2.0 * rot_rad.sin(), 5.0 * rot_rad.sin() + 2.0 * rot_rad.cos()), 
            galley,
            underline: egui::Stroke::NONE,
            override_text_color: None,
            angle: -std::f32::consts::FRAC_PI_2 + rot_rad,
            fallback_color: color,
            opacity_factor: 1.0,
        });

        // Aspect Ratio (Below center)
        let ratio = if config.active_area.h != 0.0 { config.active_area.w / config.active_area.h } else { 0.0 };
        ui.painter().text(egui::pos2(aa_center_x, aa_center_y + 12.0), egui::Align2::CENTER_CENTER, format!("{:.4}", ratio).replace(".", ","), font_id.clone(), color);

        // Width (Top) 
        let top_mid = egui::pos2((points[0].x + points[1].x) / 2.0, (points[0].y + points[1].y) / 2.0);
        ui.painter().text(top_mid + egui::vec2(5.0 * rot_rad.sin(), 5.0 * rot_rad.cos()), egui::Align2::CENTER_TOP, format!("{:.2}mm", config.active_area.w).replace(".", ","), font_id.clone(), color);

        // Drag Interaction
        if response.dragged() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                // Check rough bounds
                 let click_rect = egui::Rect::from_min_max(
                    points.iter().fold(egui::pos2(f32::INFINITY, f32::INFINITY), |a, b| egui::pos2(a.x.min(b.x), a.y.min(b.y))),
                    points.iter().fold(egui::pos2(f32::NEG_INFINITY, f32::NEG_INFINITY), |a, b| egui::pos2(a.x.max(b.x), a.y.max(b.y)))
                 );

                 if click_rect.expand(20.0).contains(pointer_pos) || response.drag_started() {
                     let delta = response.drag_delta();
                     config.active_area.x = (config.active_area.x + delta.x / scale).clamp(config.active_area.w / 2.0, phys_w - config.active_area.w / 2.0);
                     config.active_area.y = (config.active_area.y + delta.y / scale).clamp(config.active_area.h / 2.0, phys_h - config.active_area.h / 2.0);
                 }
            }
        }

        // Live Cursor
        if tablet_data.is_connected {
             let (max_w, max_h) = *app.shared.hardware_size.read().unwrap();
             let cx = offset_x + (tablet_data.x as f32 / max_w) * phys_w * scale;
             let cy = offset_y + (tablet_data.y as f32 / max_h) * phys_h * scale;
             if full_rect.contains(egui::pos2(cx, cy)) {
                 ui.painter().circle_filled(egui::pos2(cx, cy), 3.0, egui::Color32::BLACK);
             }
        }
    });
    
    ui.add_space(8.0);
    
    // Tablet Settings Inputs - Aligned Grid
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        egui::Grid::new("tablet_grid").spacing(egui::vec2(10.0, 10.0)).show(ui, |ui| {
            ui_input_box(ui, "Width", &mut config.active_area.w, "mm");
            ui_input_box(ui, "Height", &mut config.active_area.h, "mm");
            ui_input_box(ui, "X", &mut config.active_area.x, "mm");
            ui_input_box(ui, "Y", &mut config.active_area.y, "mm");
            ui_input_box(ui, "Rotation", &mut config.active_area.rotation, "°");
            ui.end_row();

            // Clamping
            config.active_area.w = config.active_area.w.clamp(1.0, phys_w);
            config.active_area.h = config.active_area.h.clamp(1.0, phys_h);
            config.active_area.x = config.active_area.x.clamp(config.active_area.w / 2.0, phys_w - config.active_area.w / 2.0);
            config.active_area.y = config.active_area.y.clamp(config.active_area.h / 2.0, phys_h - config.active_area.h / 2.0);
        });
    });
}
