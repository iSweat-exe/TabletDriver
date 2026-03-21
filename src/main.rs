//! # NextTabletDriver Entry Point
//!
//! This is the main executable for the NextTabletDriver application.
//! It initializes logging, checks for single-instance via a mutex (on Windows),
//! configures the window properties, and launches the `eframe` (egui) graphical interface.

#![windows_subsystem = "windows"]

use eframe::egui;
use next_tablet_driver::app::TabletMapperApp;
use next_tablet_driver::logger;

use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

/// The main entry point of the application.
///
/// # Platform Specifics
/// On Windows, it creates a named Mutex to ensure only one instance of the
/// application is running at any given time. This mutex is also checked by the Inno Setup installer.
///
/// # Execution Flow
/// 1. Verifies no other instance is running.
/// 2. Initializes the application logger.
/// 3. Configures the GUI window options (icon, dimensions, title).
/// 4. Enters the `eframe::run_native` GUI event loop.
fn main() -> eframe::Result {
    // --- Single Instance Mutex ---
    // This allows Inno Setup to know if the app is running and allows us to release it before update.
    let mutex_name: Vec<u16> = "NextTabletDriverMutex\0".encode_utf16().collect();
    let handle: HANDLE = unsafe { CreateMutexW(std::ptr::null(), 1, mutex_name.as_ptr()) };
    if handle == 0 {
        return Ok(());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        log::error!("Another instance of NextTabletDriver is already running.");
        return Ok(());
    }

    // Start logger
    logger::init();

    log::info!(target: "Detect", "Application starting...");

    let icon_data =
        eframe::icon_data::from_png_bytes(include_bytes!("../resources/icon.png")).unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon_data)
            .with_inner_size([1000.0, 850.0])
            .with_title(format!("NextTabletDriver v{}", next_tablet_driver::VERSION)),
        ..Default::default()
    };

    eframe::run_native(
        &format!("NextTabletDriver v{}", next_tablet_driver::VERSION),
        options,
        Box::new(|cc| Ok(Box::new(TabletMapperApp::new(cc.egui_ctx.clone())))),
    )
}
