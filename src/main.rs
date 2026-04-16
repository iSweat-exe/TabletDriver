//! # NextTabletDriver Entry Point
//!
//! This is the main executable for the NextTabletDriver application.
//! It initializes logging, checks for single-instance enforcement,
//! configures the window properties, and launches the `eframe` (egui) graphical interface.

#![windows_subsystem = "windows"]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use eframe::egui;
use next_tablet_driver::app::TabletMapperApp;
use next_tablet_driver::logger;

/// The main entry point of the application.
///
/// # Platform Specifics
/// - **Windows**: Creates a named Mutex to ensure only one instance is running.
///   This mutex is also checked by the Inno Setup installer.
/// - **Linux**: Uses a file lock in `$XDG_RUNTIME_DIR` (or `/tmp`) for single-instance enforcement.
///
/// # Execution Flow
/// 1. Verifies no other instance is running.
/// 2. Initializes the application logger.
/// 3. Configures the GUI window options (icon, dimensions, title).
/// 4. Enters the `eframe::run_native` GUI event loop.
fn main() -> eframe::Result {
    // --- Single Instance Check ---

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
        use windows_sys::Win32::System::Threading::CreateMutexW;

        log::debug!(target: "Startup", "Checking for single instance (Windows Mutex)...");
        let mutex_name: Vec<u16> = "NextTabletDriverMutex\0".encode_utf16().collect();
        let handle: HANDLE = unsafe { CreateMutexW(std::ptr::null(), 1, mutex_name.as_ptr()) };
        if handle.is_null() {
            log::error!(target: "Startup", "Failed to create mutex handle");
            return Ok(());
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            log::error!(target: "Startup", "Another instance of NextTabletDriver is already running.");
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    let _lock_file = {
        use std::fs;
        use std::io::Write;

        log::debug!(target: "Startup", "Checking for single instance (Linux flock)...");
        // Determine the lock file path: prefer $XDG_RUNTIME_DIR, fallback to /tmp
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        let lock_path = std::path::PathBuf::from(runtime_dir).join("nexttabletdriver.lock");

        // Try to create/open the lock file
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_path);

        match file {
            Ok(mut f) => {
                use std::os::unix::io::AsRawFd;

                // Try to acquire an exclusive non-blocking lock
                let fd = f.as_raw_fd();
                let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
                if ret != 0 {
                    log::error!(target: "Startup", "Another instance of NextTabletDriver is already running (PID locked).");
                    std::process::exit(1);
                }

                // Write our PID for debugging purposes
                let _ = write!(f, "{}", std::process::id());
                Some(f) // Keep the file handle alive to hold the lock
            }
            Err(e) => {
                log::warn!(target: "Startup", "Could not create lock file at {:?}: {}", lock_path, e);
                None
            }
        }
    };

    // Start logger
    logger::init();

    log::info!(target: "Startup", "NextTabletDriver v{} starting on {} ({})", 
        next_tablet_driver::VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let icon_data =
        eframe::icon_data::from_png_bytes(include_bytes!("../resources/icon.png")).unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon_data)
            .with_inner_size([1000.0, 850.0])
            // .with_decorations(false)
            // .with_transparent(true)
            .with_title(format!("NextTabletDriver v{}", next_tablet_driver::VERSION)),
        ..Default::default()
    };

    eframe::run_native(
        &format!("NextTabletDriver v{}", next_tablet_driver::VERSION),
        options,
        Box::new(|cc| Ok(Box::new(TabletMapperApp::new(cc.egui_ctx.clone())))),
    )
}
