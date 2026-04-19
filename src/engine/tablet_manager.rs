//! # Tablet Device Manager
//!
//! This module is the execution environment for the background USB polling thread.
//! It handles detecting devices, reading raw USB packets, checking for configuration
//! updates, and feeding data to the UI thread and `Pipeline`.

use crate::drivers::detect_tablet;
use crate::engine::injector::Injector;
use crate::engine::pipeline::Pipeline;
use crate::engine::state::SharedState;
use crossbeam_channel::Sender;
use eframe::egui;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

/// The main background loop for device interaction.
///
/// This function runs indefinitely in its own thread, blocking on USB reads.
///
/// # Lifecycle
/// 1. **Initialization**: Sets up `hidapi`, `Injector`, `Pipeline`, and default `Filters`.
/// 2. **Detection Loop**: Periodically scans USB buses for a supported device.
/// 3. **Connection Hook**: Upon connection, populates `SharedState` with device metadata
///    (Name, VID, PID, Physical Size) and applies default area mappings if it's the first run.
/// 4. **Polling Loop**: Continuously reads from the USB endpoint.
///    - Parses raw bytes via the specific vendor driver.
///    - Updates packet statistics.
///    - Checks if the user changed the configuration in the UI (throttled to 50ms checks).
///    - Sends the parsed data to the GUI thread for rendering.
///    - Passes the data to the `Pipeline` for math/injection.
/// 5. **Disconnection Hook**: If reading fails, cleans up state and drops back to detection loop.
pub fn run_manager(
    shared: Arc<SharedState>,
    _ctx: egui::Context,
    tablet_sender: Sender<crate::drivers::TabletData>,
) {
    log::info!(target: "TabletManager", "Starting device manager thread");
    let hid_init_start = Instant::now();
    let hid_api = hidapi::HidApi::new().unwrap();
    log::info!(target: "TabletManager", "HID API initialized in {:.2?}", hid_init_start.elapsed());
    let mut injector = Injector::new();
    let mut pipeline = Pipeline::new();

    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
        };
        // Set engine thread to Time Critical for minimum scheduling jitter
        if SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL) != 0 {
            log::info!(target: "TabletManager", "Thread priority set to TIME_CRITICAL");
        } else {
            log::warn!(target: "TabletManager", "Failed to set thread priority to TIME_CRITICAL");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Attempt to lower nice value for higher scheduling priority.
        // This may silently fail without CAP_SYS_NICE — that's acceptable,
        // the driver will still work at normal priority.
        unsafe {
            let ret = libc::nice(-11);
            if ret == -1 {
                log::info!(target: "TabletManager", "Running at normal priority (CAP_SYS_NICE not available)");
            } else {
                log::info!(target: "TabletManager", "Thread priority increased (nice -11)");
            }
        }
    }

    let mut local_config = shared.config.read().unwrap().clone();
    let mut local_config_version = shared.config_version.load(Ordering::Relaxed);
    let mut last_config_check = Instant::now();
    let mut last_stats_update = Instant::now();

    let mut filters = crate::filters::FilterPipeline::new();
    log::debug!(target: "TabletManager", "Initializing filters...");
    filters.add(Box::new(
        crate::filters::antichatter::DevocubAntichatter::new(),
    ));
    filters.add(Box::new(crate::filters::stats::SpeedStatsFilter::new(
        Arc::clone(&shared),
    )));
    filters.update_config(&local_config);
    log::info!(target: "TabletManager", "Filter pipeline ready");

    loop {
        if let Some((device, driver, vid, pid)) = detect_tablet(&hid_api) {
            {
                let mut name = shared.tablet_name.write().unwrap();
                *name = driver.get_name().to_string();
                log::info!(target: "TabletManager", "Tablet detected: {} ({:04x}:{:04x})", *name, vid, pid);
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

            // Drain stale packets left by init sequence to prevent cursor teleport
            let mut drain_buf = [0u8; 64];
            while device.read_timeout(&mut drain_buf, 10).unwrap_or(0) > 0 {}
            pipeline.reset_relative();

            let mut buf = [0u8; 64];
            loop {
                let hid_read_start = Instant::now();
                match device.read_timeout(&mut buf, 1000) {
                    Ok(len) if len > 0 => {
                        let hid_read_duration = hid_read_start.elapsed();
                        let parse_start = Instant::now();
                        if let Some(mut data) = driver.parse(&buf[..len]) {
                            let parse_duration = parse_start.elapsed();
                            data.receive_time = Some(hid_read_start);
                            data.parser_time = parse_duration;

                            pipeline.process(
                                &data,
                                driver.as_ref(),
                                &local_config,
                                &mut injector,
                                &mut filters,
                                &shared,
                            );

                            shared.packet_count.fetch_add(1, Ordering::Relaxed);

                            let now = Instant::now();
                            if now.duration_since(last_stats_update) > Duration::from_millis(16)
                                && let Ok(mut stats) = shared.stats.write()
                            {
                                last_stats_update = now;
                                stats.total_packets =
                                    shared.packet_count.load(Ordering::Relaxed) as u64;

                                let hr_ms = hid_read_duration.as_secs_f32() * 1000.0;
                                stats.hid_read_ms = hr_ms;
                                stats.min_hid_read_ms = stats.min_hid_read_ms.min(hr_ms);
                                stats.max_hid_read_ms = stats.max_hid_read_ms.max(hr_ms);
                                stats.avg_hid_read_ms += (hr_ms - stats.avg_hid_read_ms) * 0.05;

                                let p_ms = parse_duration.as_secs_f32() * 1000.0;
                                stats.parser_ms = p_ms;
                                stats.min_parser_ms = stats.min_parser_ms.min(p_ms);
                                stats.max_parser_ms = stats.max_parser_ms.max(p_ms);
                                stats.avg_parser_ms += (p_ms - stats.avg_parser_ms) * 0.05;
                            }

                            let now = Instant::now();
                            if now.duration_since(last_config_check) > Duration::from_millis(50) {
                                let cv = shared.config_version.load(Ordering::Relaxed);
                                if cv != local_config_version {
                                    local_config = shared.config.read().unwrap().clone();
                                    log::info!(target: "TabletManager", "Configuration updated (version {})", cv);
                                    local_config_version = cv;
                                    filters.update_config(&local_config);
                                }
                                last_config_check = now;
                            }

                            let _ = tablet_sender.send(data.clone());
                        }
                    }
                    Ok(_) => {
                        let out_of_range = crate::drivers::TabletData {
                            status: "Out of Range".to_string(),
                            ..Default::default()
                        };
                        pipeline.process(
                            &out_of_range,
                            driver.as_ref(),
                            &local_config,
                            &mut injector,
                            &mut filters,
                            &shared,
                        );
                        let _ = tablet_sender.send(out_of_range);

                        let cv = shared.config_version.load(Ordering::Relaxed);
                        if cv != local_config_version {
                            local_config = shared.config.read().unwrap().clone();
                            local_config_version = cv;
                            filters.update_config(&local_config);
                        }
                    }
                    Err(e) => {
                        log::error!(target: "TabletManager", "HID Read Error: {}", e);
                        log::warn!(target: "TabletManager", "Tablet disconnected or bus reset");
                        break;
                    } // Disconnected
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
