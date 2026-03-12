use crate::drivers::detect_tablet;
use crate::engine::injector::Injector;
use crate::engine::pipeline::Pipeline;
use crate::engine::state::SharedState;
use crossbeam_channel::Sender;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn run_manager(shared: Arc<SharedState>, ctx: egui::Context, tablet_sender: Sender<crate::drivers::TabletData>) {
    let hid_api = hidapi::HidApi::new().unwrap();
    let mut injector = Injector::new();
    let mut pipeline = Pipeline::new();

    let mut local_config = shared.config.read().unwrap().clone();
    let mut local_config_version = shared.config_version.load(Ordering::Relaxed);
    let mut last_config_check = Instant::now();

    let mut filters = crate::filters::FilterPipeline::new();
    filters.add(Box::new(crate::filters::antichatter::DevocubAntichatter::new()));
    filters.add(Box::new(crate::filters::stats::SpeedStatsFilter::new(Arc::clone(&shared))));
    filters.update_config(&local_config);

    loop {
        if let Some((device, driver, vid, pid)) = detect_tablet(&hid_api) {
            {
                let mut name = shared.tablet_name.write().unwrap();
                *name = driver.get_name().to_string();
                *shared.tablet_vid.write().unwrap() = vid;
                *shared.tablet_pid.write().unwrap() = pid;

                let mut size = shared.physical_size.write().unwrap();
                *size = driver.get_physical_specs();
                let mut hw_size = shared.hardware_size.write().unwrap();
                let (mw, mh, _) = driver.get_specs();
                *hw_size = (mw, mh);

                let mut is_first = shared.is_first_run.write().unwrap();
                if *is_first {
                    let mut config = shared.config.write().unwrap();
                    config.active_area.w = size.0;
                    config.active_area.h = size.1;
                    config.active_area.x = size.0 / 2.0;
                    config.active_area.y = size.1 / 2.0;
                    *is_first = false;
                    local_config = config.clone();
                    shared.config_version.fetch_add(1, Ordering::SeqCst);
                }
            }

            let mut buf = [0u8; 64];
            loop {
                match device.read_timeout(&mut buf, 1000) {
                    Ok(len) if len > 0 => {
                        if let Some(data) = driver.parse(&buf[..len]) {
                            shared.packet_count.fetch_add(1, Ordering::Relaxed);

                            let now = Instant::now();
                            if now.duration_since(last_config_check) > Duration::from_millis(50) {
                                let cv = shared.config_version.load(Ordering::Relaxed);
                                if cv != local_config_version {
                                    local_config = shared.config.read().unwrap().clone();
                                    local_config_version = cv;
                                    filters.update_config(&local_config);
                                }
                                last_config_check = now;
                            }

                            // Inject UI updates
                            let _ = tablet_sender.send(data.clone());
                            ctx.request_repaint();

                            // Process the packet and apply coordinate transforms + OS injection
                            pipeline.process(&data, driver.as_ref(), &local_config, &mut injector, &mut filters);
                        }
                    }
                    Ok(_) => {
                        let cv = shared.config_version.load(Ordering::Relaxed);
                        if cv != local_config_version {
                            local_config = shared.config.read().unwrap().clone();
                            local_config_version = cv;
                            filters.update_config(&local_config);
                        }
                    }
                    Err(_) => break, // Disconnected
                }
            }
        }

        // On Disconnect
        {
            *shared.tablet_name.write().unwrap() = "No Tablet Detected".to_string();
            *shared.tablet_vid.write().unwrap() = 0;
            *shared.tablet_pid.write().unwrap() = 0;
            *shared.tablet_data.write().unwrap() = crate::drivers::TabletData::default();
        }
        thread::sleep(Duration::from_secs(1));
    }
}
