use hidapi::{HidApi, HidDevice};
use include_dir::{include_dir, Dir, DirEntry};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

pub mod config;
pub mod generic;
pub mod parsers;

use config::TabletConfiguration;
use generic::GenericNextTabletDriver;

#[derive(Debug, Clone, Default)]
pub struct TabletData {
    pub status: String,
    pub x: u16,
    pub y: u16,
    pub pressure: u16,
    pub tilt_x: i8,
    pub tilt_y: i8,
    pub buttons: u8,
    pub eraser: bool,
    pub hover_distance: u8,
    pub raw_data: String,
    pub is_connected: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DriverStats {
    pub handspeed: f32,
    pub total_distance_mm: f32,
}

pub trait NextTabletDriver {
    fn get_name(&self) -> &str;
    fn get_specs(&self) -> (f32, f32, f32);
    fn get_physical_specs(&self) -> (f32, f32);
    fn get_vid_pid(&self) -> (u16, u16);
    fn parse(&self, data: &[u8]) -> Option<TabletData>;
}

static TABLET_CONFIGS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/tablets");

lazy_static::lazy_static! {
    static ref LOADED_CONFIGS: Vec<TabletConfiguration> = load_configurations();
}

fn load_configurations() -> Vec<TabletConfiguration> {
    let mut configs = Vec::new();
    let mut loaded_names = HashSet::new();

    let local_dir = Path::new("tablets");
    if local_dir.exists() {
        load_from_disk_recursive(local_dir, &mut configs, &mut loaded_names);
    }

    load_embedded_recursive(&TABLET_CONFIGS_DIR, &mut configs, &mut loaded_names);
    configs
}

fn load_embedded_recursive(
    dir: &Dir,
    configs: &mut Vec<TabletConfiguration>,
    names: &mut HashSet<String>,
) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub_dir) => {
                load_embedded_recursive(sub_dir, configs, names);
            }
            DirEntry::File(file) => {
                if file.path().extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(content_str) = file.contents_utf8() {
                        if let Ok(config) = serde_json::from_str::<TabletConfiguration>(content_str)
                        {
                            if !names.contains(&config.name) {
                                configs.push(config);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn load_from_disk_recursive(
    path: &Path,
    configs: &mut Vec<TabletConfiguration>,
    names: &mut HashSet<String>,
) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                load_from_disk_recursive(&p, configs, names);
            } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(config) = serde_json::from_str::<TabletConfiguration>(&content) {
                        names.insert(config.name.clone());
                        configs.push(config);
                    }
                }
            }
        }
    }
}

pub fn detect_tablet(api: &HidApi) -> Option<(HidDevice, Box<dyn NextTabletDriver>, u16, u16)> {
    let global_start = Instant::now();
    let enum_start = Instant::now();
    let devices: Vec<_> = api.device_list().collect();
    let enum_duration = enum_start.elapsed();

    if enum_duration > Duration::from_millis(500) {
        log::warn!(target: "Detect", "HID Enumeration SLOW: {:.2?}", enum_duration);
    }

    let configs = &*LOADED_CONFIGS;

    for config in configs {
        for digitizer in &config.digitizer_identifiers {
            for device_info in &devices {
                if device_info.vendor_id() == digitizer.vendor_id
                    && device_info.product_id() == digitizer.product_id
                {
                    let interface = device_info.interface_number();
                    let open_start = Instant::now();

                    match api.open_path(device_info.path()) {
                        Ok(device) => {
                            let open_duration = open_start.elapsed();
                            let mut init_success = true;
                            use base64::{engine::general_purpose, Engine as _};

                            let init_start = Instant::now();

                            // Feature Reports
                            if let Some(reports) = &digitizer.feature_init_report {
                                for report_str in reports {
                                    if let Ok(data) = general_purpose::STANDARD.decode(report_str) {
                                        if let Err(e) = device.send_feature_report(&data) {
                                            log::error!(target: "Detect", "Init Error (Feature): {}", e);
                                            init_success = false;
                                            break;
                                        }
                                    }
                                }
                            }

                            // Output Reports
                            if init_success {
                                if let Some(reports) = &digitizer.output_init_report {
                                    for report_str in reports {
                                        if let Ok(data) =
                                            general_purpose::STANDARD.decode(report_str)
                                        {
                                            if let Err(e) = device.write(&data) {
                                                log::error!(target: "Detect", "Init Error (Output): {}", e);
                                                init_success = false;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }

                            if !init_success {
                                continue;
                            }

                            log::info!(target: "Detect",
                                "{} | Enum: {:.2?} | Open: {:.2?} | Init: {:.2?} | Total: {:.2?}",
                                config.name, enum_duration, open_duration, init_start.elapsed(), global_start.elapsed()
                            );

                            return Some((
                                device,
                                Box::new(GenericNextTabletDriver::new(
                                    config.clone(),
                                    digitizer.vendor_id,
                                    digitizer.product_id,
                                )),
                                digitizer.vendor_id,
                                digitizer.product_id,
                            ));
                        }
                        Err(e) => {
                            log::trace!(target: "Detect", "Interface {} busy: {}", interface, e);
                        }
                    }
                }
            }
        }
    }
    None
}
