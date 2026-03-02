use hidapi::{HidApi, HidDevice};
use std::fs;
use std::path::Path;
use std::collections::HashSet;
use include_dir::{include_dir, Dir, DirEntry};

pub mod config;
pub mod generic;
pub mod parsers;

use config::TabletConfiguration;
use generic::GenericTabletDriver;

#[derive(Debug, Clone, Default)]
pub struct TabletData {
    pub status: String,
    pub x: u16,
    pub y: u16,
    pub pressure: u16,
    pub tilt_x: i8,
    pub tilt_y: i8,
    pub buttons: u8, // bitmask
    pub eraser: bool,
    pub hover_distance: u8,
    pub raw_data: String,
    pub is_connected: bool,
}

pub trait TabletDriver {
    fn get_name(&self) -> &str;
    fn get_specs(&self) -> (f32, f32, f32); // Max X, Max Y, Max Pressure
    fn get_physical_specs(&self) -> (f32, f32); // Physical Width (mm), Physical Height (mm)
    fn parse(&self, data: &[u8]) -> Option<TabletData>;
}

static TABLET_CONFIGS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/tablets");

lazy_static::lazy_static! {
    static ref LOADED_CONFIGS: Vec<TabletConfiguration> = load_configurations();
}

/// Charge les configurations depuis le dossier local (prioritaire) et les ressources embarquées
fn load_configurations() -> Vec<TabletConfiguration> {
    let mut configs = Vec::new();
    let mut loaded_names = HashSet::new();

    // 1. Scan récursif du dossier "tablets" sur le disque (Overrides)
    let local_dir = Path::new("tablets");
    if local_dir.exists() {
        load_from_disk_recursive(local_dir, &mut configs, &mut loaded_names);
    }

    // 2. Scan récursif des ressources embarquées (Baseline)
    load_embedded_recursive(&TABLET_CONFIGS_DIR, &mut configs, &mut loaded_names);

    log::debug!(target: "Config", "Total configurations loaded: {}", configs.len());
    configs
}

/// Parcourt récursivement les dossiers EMBARQUÉS (fix pour les sous-dossiers de marques)
fn load_embedded_recursive(dir: &Dir, configs: &mut Vec<TabletConfiguration>, names: &mut HashSet<String>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub_dir) => {
                load_embedded_recursive(sub_dir, configs, names);
            }
            DirEntry::File(file) => {
                if file.path().extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(content_str) = file.contents_utf8() {
                        if let Ok(config) = serde_json::from_str::<TabletConfiguration>(content_str) {
                            if !names.contains(&config.name) {
                                log::debug!(target: "Config", "Loaded embedded config: {}", config.name);
                                configs.push(config);
                            } else {
                                log::debug!(target: "Config", "Embedded config {} is shadowed", config.name);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Parcourt récursivement les dossiers sur le DISQUE
fn load_from_disk_recursive(path: &Path, configs: &mut Vec<TabletConfiguration>, names: &mut HashSet<String>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                load_from_disk_recursive(&p, configs, names);
            } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(config) = serde_json::from_str::<TabletConfiguration>(&content) {
                        log::debug!(target: "Config", "Loaded override config: {} from {:?}", config.name, p.display());
                        names.insert(config.name.clone());
                        configs.push(config);
                    }
                }
            }
        }
    }
}

pub fn detect_tablet(api: &HidApi) -> Option<(HidDevice, Box<dyn TabletDriver>)> {
    let start = std::time::Instant::now();
    log::debug!(target: "Detect", "Starting tablet detection...");
    
    let configs = &*LOADED_CONFIGS;
    let devices: Vec<_> = api.device_list().collect();

    for config in configs {
        for digitizer in &config.digitizer_identifiers {
            for device_info in &devices {
                if device_info.vendor_id() == digitizer.vendor_id && device_info.product_id() == digitizer.product_id {
                    let interface = device_info.interface_number();
                    
                    if let Ok(device) = api.open_path(device_info.path()) {
                        let mut init_success = true;
                        use base64::{Engine as _, engine::general_purpose};

                        if let Some(reports) = &digitizer.feature_init_report {
                            for report_str in reports {
                                if let Ok(data) = general_purpose::STANDARD.decode(report_str) {
                                    if let Err(e) = device.send_feature_report(&data) {
                                        log::error!(target: "Detect", "Failed feature report ({}): {}", config.name, e);
                                        init_success = false;
                                        break;
                                    }
                                }
                            }
                        }

                        if !init_success { continue; }

                        if let Some(reports) = &digitizer.output_init_report {
                            for report_str in reports {
                                if let Ok(data) = general_purpose::STANDARD.decode(report_str) {
                                    if let Err(e) = device.write(&data) {
                                        log::error!(target: "Detect", "Failed output report ({}): {}", config.name, e);
                                        init_success = false;
                                        break;
                                    }
                                }
                            }
                        }

                        if !init_success { continue; }

                        log::info!(target: "Detect", "Initialized {} (Intf {}) in {:.2?}", config.name, interface, start.elapsed());
                        return Some((device, Box::new(GenericTabletDriver::new(config.clone()))));
                    }
                }
            }
        }
    }

    log::debug!(target: "Detect", "No tablet found ({:.2?})", start.elapsed());
    None
}