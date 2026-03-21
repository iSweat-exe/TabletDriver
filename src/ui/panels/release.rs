use eframe::egui;

struct ReleaseEntry {
    version: &'static str,
    date: &'static str,
    additions: &'static [&'static str],
    removals: &'static [&'static str],
    fixes: &'static [&'static str],
    improvements: &'static [&'static str],
}

const RELEASES: &[ReleaseEntry] = &[
    ReleaseEntry {
        version: "1.26.2103.02",
        date: "21/03/2026",
        additions: &[],
        removals: &[],
        fixes: &[
            "Updated to Rust 2024 edition",
            "Updated all dependencies to their latest versions",
            "Fixed a security vulnerability detected in dependencies",
            "Adapted codebase to new APIs introduced by dependency updates",
        ],
        improvements: &[],
    },
    ReleaseEntry {
        version: "1.26.2103.01",
        date: "21/03/2026",
        additions: &[
            "Add: Added 4 Catppuccin themes (Latte, Frappe, Macchiato, Mocha)",
            "Add: Added Osu! Playfield preview in the mapping area",
        ],
        removals: &[],
        fixes: &[
            "Fix: Cleaned up and modernized the default egui UI design (borders, rounding, and hover effects)",
        ],
        improvements: &["Improve: Improved theme-awareness for custom UI components"],
    },
    ReleaseEntry {
        version: "1.26.2003.01",
        date: "20/03/2026",
        additions: &["Add: 'Theme' settings in 'Settings' tab"],
        removals: &[],
        fixes: &[],
        improvements: &["Improve: 'Theme' settings to allow changing the theme of the application"],
    },
    ReleaseEntry {
        version: "1.26.1903.03",
        date: "19/03/2026",
        additions: &[],
        removals: &["Remove: Powershell files, Payload.json"],
        fixes: &[],
        improvements: &[
            "Next Tablet Driver now has a GitHub organization, the project has also been cleaned up for a better presentation in the future.",
        ],
    },
    ReleaseEntry {
        version: "1.26.1303.03",
        date: "13/03/2026",
        additions: &[],
        removals: &["Remove: Telemetry System"],
        fixes: &[],
        improvements: &["Internal Documentation"],
    },
    ReleaseEntry {
        version: "1.26.1203.01",
        date: "12/03/2026",
        additions: &[
            "Add: 'Relative Mode' for pen input",
            "Add: 'Filters' tab and 'Devocub Antichatter' settings like Open Tablet Driver Filters and 'HandSpeed WebSocket' settings",
        ],
        removals: &["Remove: Crypto Donations", "Remove: 'Tools' tab"],
        fixes: &["Nothing"],
        improvements: &[
            "Improve: 'HandSpeed WebSocket' filter to send 'total_distance' in addition to 'handspeed'",
        ],
    },
    ReleaseEntry {
        version: "1.26.0503.01",
        date: "05/03/2026",
        additions: &[
            "Add: Telemetry System for improvement (you can disable it in 'Settings' tab)",
            "Add: 'Relative Mode' for pen input",
            "Info: The telemetry doesn't collect any personally identifiable information; it’s only there to improve the driver. An example of the shared data is available to view on GitHub.",
        ],
        removals: &["Nothing"],
        fixes: &["Change version format to European format instead of US format (MMDD -> DDMM)"],
        improvements: &["Nothing"],
    },
    ReleaseEntry {
        version: "1.26.0303.03",
        date: "03/03/2026",
        additions: &[
            "New 'Release' tab to track changes",
            "Added Support & Contribution panel with Crypto donations",
        ],
        removals: &["Nothing"],
        fixes: &["Fix all 'cargo clippy' issues and warnings (as mentioned in ISSUE#2)"],
        improvements: &[
            "Add 'CI/CD' pipeline for automated code quality checks (as mentioned in ISSUE#1)",
        ],
    },
    ReleaseEntry {
        version: "1.26.0301.05",
        date: "01/03/2026",
        additions: &["New 'Websocket Server' settings in 'Settings' tab"],
        removals: &["Nothing"],
        fixes: &[
            "Improved 'Run At Startup' feature, before it was not working properly and flagged by Windows Defender",
            "Improved HID API initialization performance",
        ],
        improvements: &["Event-driven architecture for reduced CPU usage"],
    },
];

pub fn render_release_panel(_app: &crate::app::state::TabletMapperApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading("Release History");
        ui.add_space(10.0);
    });

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(10.0);

            for release in RELEASES {
                render_release_entry(ui, release);
                ui.add_space(15.0);
            }

            ui.add_space(10.0);
        });
}

fn render_release_entry(ui: &mut egui::Ui, entry: &ReleaseEntry) {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("Next Tablet Driver | v{}", entry.version));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(entry.date).weak());
                    });
                });

                ui.add_space(8.0);

                if !entry.additions.is_empty() {
                    ui.label(egui::RichText::new("Additions").strong());
                    for add in entry.additions {
                        ui.label(format!("• {}", add));
                    }
                    ui.add_space(8.0);
                }

                if !entry.removals.is_empty() {
                    ui.label(egui::RichText::new("Removals").strong());
                    for rem in entry.removals {
                        ui.label(format!("• {}", rem));
                    }
                    ui.add_space(8.0);
                }

                if !entry.fixes.is_empty() {
                    ui.label(egui::RichText::new("Fixes").strong());
                    for fix in entry.fixes {
                        ui.label(format!("• {}", fix));
                    }
                    ui.add_space(8.0);
                }

                if !entry.improvements.is_empty() {
                    ui.label(egui::RichText::new("Improvements").strong());
                    for imp in entry.improvements {
                        ui.label(format!("• {}", imp));
                    }
                }
            });
        });
}
