use eframe::egui;

pub fn render_support_panel(_app: &crate::app::state::TabletMapperApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading("Support & Contribution");
        ui.add_space(10.0);

        ui.label(
            egui::RichText::new(
                "Thank you for considering a contribution! Your support helps keep this project alive.",
            )
            .size(16.0),
        );

        ui.add_space(3.0);

        ui.label(
            egui::RichText::new(
                "If you have Rust development skills, contributing directly to the project code \
                is one of the best ways to help and is often more valuable than a donation.",
            )
            .size(14.0)
            .color(egui::Color32::from_rgb(200, 000, 000)),
        );

        // TODO: Add crypto donations
        // ui.add_space(20.0);
        // ui.separator();
        // ui.add_space(20.0);

        // ui.heading("Crypto Donations");
        // ui.add_space(10.0);

        // egui::Grid::new("crypto_donations")
        //     .num_columns(3)
        //     .spacing([10.0, 10.0])
        //     .show(ui, |ui| {
        //         render_crypto_row(ui, "BTC", "EX");
        //         render_crypto_row(ui, "ETH", "EX");
        //         render_crypto_row(ui, "LTC", "EX");
        //     });
    });
}

// fn render_crypto_row(ui: &mut egui::Ui, name: &str, address: &str) {
//     ui.label(egui::RichText::new(name).strong());
//     ui.label(egui::RichText::new(address).monospace());
//     if ui.button("Copy").clicked() {
//         ui.ctx().copy_text(address.to_string());
//     }
//     ui.end_row();
// }
