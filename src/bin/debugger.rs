#![windows_subsystem = "windows"]

use crate::drivers::detect_tablet;
use eframe::egui;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tablet_driver::drivers::{self, TabletData};
use tablet_driver::input::SharedState;
use tablet_driver::ui::panels::render_debugger_panel;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 750.0])
            .with_title("TD - Tablet Debugger"),
        ..Default::default()
    };

    eframe::run_native(
        "TD - Tablet Debugger",
        options,
        Box::new(|_cc| {
            _cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(TabletApp::new()))
        }),
    )
}

struct TabletApp {
    shared: Arc<SharedState>,
    api: hidapi::HidApi,
    device: Option<hidapi::HidDevice>,
    driver: Option<Box<dyn drivers::TabletDriver>>,

    // Metrics
    last_update: Instant,
    last_hz_update: Instant,
    last_packet_count: u32,
    displayed_hz: f32,
}

impl TabletApp {
    fn new() -> Self {
        let api_start = Instant::now();
        let api = hidapi::HidApi::new().unwrap();
        log::info!(target: "Detect", "HID API initialized in {:.2?}", api_start.elapsed());

        let dummy_config = tablet_driver::domain::MappingConfig {
            mode: tablet_driver::domain::DriverMode::Absolute,
            active_area: tablet_driver::domain::ActiveArea {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
                rotation: 0.0,
            },
            target_area: tablet_driver::domain::TargetArea {
                x: 0.0,
                y: 0.0,
                w: 1920.0,
                h: 1080.0,
            },
            relative_config: tablet_driver::domain::RelativeConfig::default(),
            tip_threshold: 0,
            eraser_threshold: 0,
            disable_pressure: false,
            disable_tilt: false,
            tip_binding: "None".to_string(),
            eraser_binding: "None".to_string(),
            pen_button_bindings: vec!["None".to_string(), "None".to_string()],
            run_at_startup: false,
            enable_telemetry: false,
            websocket: Default::default(),
            antichatter: Default::default(),
        };

        let shared = Arc::new(SharedState {
            config: RwLock::new(dummy_config),
            config_version: AtomicU32::new(0),
            tablet_data: RwLock::new(TabletData::default()),
            tablet_name: RwLock::new("No Tablet Detected".to_string()),
            tablet_vid: RwLock::new(0),
            tablet_pid: RwLock::new(0),
            physical_size: RwLock::new((160.0, 100.0)),
            hardware_size: RwLock::new((32767.0, 32767.0)),
            is_first_run: RwLock::new(false),
            packet_count: AtomicU32::new(0),
        });

        Self {
            shared,
            api,
            device: None,
            driver: None,
            last_update: Instant::now(),
            last_hz_update: Instant::now(),
            last_packet_count: 0,
            displayed_hz: 0.0,
        }
    }
}

impl eframe::App for TabletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- AUTO-RECONNECT & USB READ ---
        if self.device.is_none() && self.last_update.elapsed().as_secs() >= 1 {
            if let Some((dev, drv, vid, pid)) = detect_tablet(&self.api) {
                let _ = dev.set_blocking_mode(false);
                let specs = drv.get_specs();
                {
                    let mut hw = self.shared.hardware_size.write().unwrap();
                    *hw = (specs.0, specs.1);
                    let mut name = self.shared.tablet_name.write().unwrap();
                    *name = drv.get_name().to_string();
                    let mut vid_lock = self.shared.tablet_vid.write().unwrap();
                    *vid_lock = vid;
                    let mut pid_lock = self.shared.tablet_pid.write().unwrap();
                    *pid_lock = pid;
                }
                self.device = Some(dev);
                self.driver = Some(drv);
            }
            self.last_update = Instant::now();
        }

        if let (Some(dev), Some(drv)) = (&self.device, &self.driver) {
            let mut buf = [0u8; 64];
            while let Ok(len) = dev.read(&mut buf) {
                if len == 0 {
                    break;
                }
                if let Some(data) = drv.parse(&buf[..len]) {
                    {
                        let mut shared_data = self.shared.tablet_data.write().unwrap();
                        *shared_data = data;
                        self.shared.packet_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }

        // --- HZ CALCULATION ---
        let current_packets = self.shared.packet_count.load(Ordering::Relaxed);
        let elapsed_hz = self.last_hz_update.elapsed();
        if elapsed_hz >= Duration::from_millis(200) {
            let delta = current_packets.saturating_sub(self.last_packet_count);
            let hz = delta as f32 / elapsed_hz.as_secs_f32();
            self.displayed_hz += (hz - self.displayed_hz) * 0.3;
            self.last_packet_count = current_packets;
            self.last_hz_update = Instant::now();
        }

        // --- GUI ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let name = self.shared.tablet_name.read().unwrap().clone();
                ui.add_space(5.0);
                ui.heading(egui::RichText::new(name).strong().extra_letter_spacing(1.5));
            });

            render_debugger_panel(self.shared.clone(), self.displayed_hz, ui);
        });

        ctx.request_repaint();
    }
}
