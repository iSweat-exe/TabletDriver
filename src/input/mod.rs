use crate::domain::MappingConfig;
use crate::drivers::{detect_tablet, TabletData};
use eframe::egui;
use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

pub struct SharedState {
    pub config: RwLock<MappingConfig>,
    pub config_version: AtomicU32,
    pub tablet_data: RwLock<TabletData>,
    pub tablet_name: RwLock<String>,
    pub tablet_vid: RwLock<u16>,
    pub tablet_pid: RwLock<u16>,
    pub physical_size: RwLock<(f32, f32)>,
    pub hardware_size: RwLock<(f32, f32)>,
    pub is_first_run: RwLock<bool>,
    pub packet_count: AtomicU32,
}

pub fn input_loop(
    shared: Arc<SharedState>,
    ctx: egui::Context,
    tablet_sender: crossbeam_channel::Sender<TabletData>,
) {
    let api_start = Instant::now();
    let hid_api = hidapi::HidApi::new().unwrap();
    log::info!(target: "Detect", "HID API initialized in {:.2?}", api_start.elapsed());

    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let mut last_pressure_down = false;

    let mut local_config = shared.config.read().unwrap().clone();
    let mut local_config_version = shared.config_version.load(Ordering::Relaxed);
    let mut last_config_check = Instant::now();

    let mut last_screen_x = -1.0;
    let mut last_screen_y = -1.0;
    let mut last_packet_time = Instant::now();

    let mut filters = crate::filters::FilterPipeline::new();
    filters.add(Box::new(
        crate::filters::antichatter::DevocubAntichatter::new(),
    ));

    loop {
        // 1. Connection handling
        if let Some((device, driver, vid, pid)) = detect_tablet(&hid_api) {
            // Keep blocking mode (default) to let the OS sleep the thread
            {
                let mut name = shared.tablet_name.write().unwrap();
                *name = driver.get_name().to_string();
                let mut vid_lock = shared.tablet_vid.write().unwrap();
                *vid_lock = vid;
                let mut pid_lock = shared.tablet_pid.write().unwrap();
                *pid_lock = pid;

                let mut size = shared.physical_size.write().unwrap();
                *size = driver.get_physical_specs();
                let mut hw_size = shared.hardware_size.write().unwrap();
                let (mw, mh, mp) = driver.get_specs();
                *hw_size = (mw, mh);

                log::info!(target: "Input", "Connected to {} ({:04x}:{:04x}). Physical: {}x{}mm, Hardware: {}x{}, Max Pressure: {}", 
                    driver.get_name(), vid, pid, size.0, size.1, mw, mh, mp);

                // Dynamic Area: If first run (no settings loaded), set to full physical size
                let mut is_first = shared.is_first_run.write().unwrap();
                if *is_first {
                    let mut config = shared.config.write().unwrap();
                    config.active_area.w = size.0;
                    config.active_area.h = size.1;
                    config.active_area.x = size.0 / 2.0;
                    config.active_area.y = size.1 / 2.0;
                    *is_first = false;
                    log::info!(target: "Input", "First run detected: Automatically set active area to full: {}x{}mm", size.0, size.1);
                    // Update local config immediately
                    local_config = config.clone();
                    shared.config_version.fetch_add(1, Ordering::SeqCst);
                }
            }

            let mut buf = [0u8; 64];
            let mut first_packet = true;
            let mut packet_log_count = Box::new(0);
            loop {
                // Blocking read with 1s timeout to check for config updates periodically
                match device.read_timeout(&mut buf, 1000) {
                    Ok(len) if len > 0 => {
                        if let Some(data) = driver.parse(&buf[..len]) {
                            // Update tablet data for UI
                            if first_packet || *packet_log_count < 10 {
                                log::info!(target: "InputDebug", "Packet: x={}, y={}, p={}, status='{}', connected={}, raw='{}'", 
                                    data.x, data.y, data.pressure, data.status, data.is_connected, data.raw_data);
                                *packet_log_count += 1;
                            }

                            // Increment packet count
                            shared.packet_count.fetch_add(1, Ordering::Relaxed);

                            let now = Instant::now();

                            // 1. Update Mapping Config ONLY if changed or periodic check
                            if now.duration_since(last_config_check) > Duration::from_millis(50) {
                                let current_version = shared.config_version.load(Ordering::Relaxed);
                                if current_version != local_config_version {
                                    local_config = shared.config.read().unwrap().clone();
                                    local_config_version = current_version;
                                }
                                last_config_check = now;
                            }

                            // 2. Send data to UI thread via channel
                            let _ = tablet_sender.send(data.clone());
                            ctx.request_repaint(); // Trigger UI refresh

                            if first_packet {
                                log::debug!(target: "Input", "First packet received and UI notified");
                                first_packet = false;
                            }

                            if data.is_connected {
                                // 0. Convert Raw to Millimeters using local specs to avoid locks
                                let (max_w, max_h, _max_p) = driver.get_specs();
                                let (phys_w, phys_h) = driver.get_physical_specs();

                                let x_mm = (data.x as f32 / max_w) * phys_w;
                                let y_mm = (data.y as f32 / max_h) * phys_h;

                                // 1. Normalize (Tablet Space in mm) - Center Based
                                let mut u = (x_mm - local_config.active_area.x)
                                    / local_config.active_area.w
                                    + 0.5;
                                let mut v = (y_mm - local_config.active_area.y)
                                    / local_config.active_area.h
                                    + 0.5;

                                if local_config.active_area.rotation != 0.0 {
                                    let rad = -local_config.active_area.rotation.to_radians();
                                    let (sin, cos) = rad.sin_cos();
                                    let cu = u - 0.5;
                                    let cv = v - 0.5;
                                    u = cu * cos - cv * sin + 0.5;
                                    v = cu * sin + cv * cos + 0.5;
                                }

                                // 1.5. Apply Filters
                                let (nx, ny) = filters.process(u, v, &local_config);
                                u = nx;
                                v = ny;

                                // 2. Project to Screen Space
                                u = u.clamp(0.0, 1.0);
                                v = v.clamp(0.0, 1.0);

                                match local_config.mode {
                                    crate::domain::DriverMode::Absolute => {
                                        let screen_x = local_config.target_area.x
                                            + u * local_config.target_area.w;
                                        let screen_y = local_config.target_area.y
                                            + v * local_config.target_area.h;

                                        // 3. Inject only if changed (sub-pixel optimization)
                                        if (screen_x - last_screen_x).abs() > 0.1
                                            || (screen_y - last_screen_y).abs() > 0.1
                                        {
                                            let _ = enigo.move_mouse(
                                                screen_x as i32,
                                                screen_y as i32,
                                                Coordinate::Abs,
                                            );
                                            last_screen_x = screen_x;
                                            last_screen_y = screen_y;
                                        }
                                    }
                                    crate::domain::DriverMode::Relative => {
                                        // Reset handling: if too much time passes between packets, reset the starting point
                                        if now.duration_since(last_packet_time)
                                            > Duration::from_millis(
                                                local_config.relative_config.reset_time_ms as u64,
                                            )
                                        {
                                            last_screen_x = -1.0;
                                            last_screen_y = -1.0;
                                        }
                                        last_packet_time = now;

                                        if last_screen_x != -1.0 && last_screen_y != -1.0 {
                                            // 1. Calculate delta in Millimeters
                                            let mut dx_mm = x_mm - last_screen_x;
                                            let mut dy_mm = y_mm - last_screen_y;

                                            // 2. Apply Rotation (to the delta vector)
                                            if local_config.relative_config.rotation != 0.0 {
                                                let rad = local_config
                                                    .relative_config
                                                    .rotation
                                                    .to_radians();
                                                let (sin, cos) = rad.sin_cos();
                                                let rx = dx_mm * cos - dy_mm * sin;
                                                let ry = dx_mm * sin + dy_mm * cos;
                                                dx_mm = rx;
                                                dy_mm = ry;
                                            }

                                            // 3. Apply X/Y Sensitivities (px/mm)
                                            let dx_px =
                                                dx_mm * local_config.relative_config.x_sensitivity;
                                            let dy_px =
                                                dy_mm * local_config.relative_config.y_sensitivity;

                                            if dx_px.abs() > 0.01 || dy_px.abs() > 0.01 {
                                                let _ = enigo.move_mouse(
                                                    dx_px as i32,
                                                    dy_px as i32,
                                                    Coordinate::Rel,
                                                );
                                            }
                                        }

                                        // Reset handling: if too much time passes between packets, reset the starting point
                                        // This prevents huge jumps if the driver/tablet stuttered
                                        // But here we use last_screen_x as the mm position.
                                        last_screen_x = x_mm;
                                        last_screen_y = y_mm;
                                    }
                                }

                                // 4. Click Handling (Tip)
                                let pressure = if local_config.disable_pressure {
                                    _max_p as u16
                                } else {
                                    data.pressure
                                };
                                let threshold_raw =
                                    (local_config.tip_threshold as f32 / 100.0) * _max_p;
                                let is_down = pressure as f32 > threshold_raw;

                                if is_down && !last_pressure_down {
                                    let _ = enigo.button(Button::Left, Direction::Press);
                                } else if !is_down && last_pressure_down {
                                    let _ = enigo.button(Button::Left, Direction::Release);
                                }
                                last_pressure_down = is_down;

                                // 5. Eraser Handling (Simple for now)
                                // We check if eraser bit is set or eraser pressure exceeds threshold
                                let _eraser_threshold_raw =
                                    (local_config.eraser_threshold as f32 / 100.0) * _max_p;
                                let eraser_down = data.eraser;

                                if eraser_down {
                                    // Future: Handle eraser binding
                                }
                            } else {
                                // If pen leaves range, ensure we release the button
                                if last_pressure_down {
                                    let _ = enigo.button(Button::Left, Direction::Release);
                                    last_pressure_down = false;
                                }
                                // Reset last position when pen leaves range to avoid large jumps in relative mode
                                last_screen_x = -1.0;
                                last_screen_y = -1.0;
                                filters.reset();
                            }
                        }
                    }
                    Ok(_) => {
                        // Timeout - check for config updates
                        let current_version = shared.config_version.load(Ordering::Relaxed);
                        if current_version != local_config_version {
                            local_config = shared.config.read().unwrap().clone();
                            local_config_version = current_version;
                        }
                    }
                    Err(_) => break, // Disconnected
                }
            }
        }

        // Reset status on disconnect
        {
            let mut name_lock = shared.tablet_name.write().unwrap();
            if *name_lock != "No Tablet Detected" {
                log::info!(target: "Input", "Tablet disconnected");
            }
            *name_lock = "No Tablet Detected".to_string();
            *shared.tablet_vid.write().unwrap() = 0;
            *shared.tablet_pid.write().unwrap() = 0;
            let mut shared_data = shared.tablet_data.write().unwrap();
            *shared_data = TabletData::default();
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
