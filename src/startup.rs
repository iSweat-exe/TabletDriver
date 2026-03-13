//! # Startup Management
//!
//! This module provides utilities to manage the application's lifecycle on Windows,
//! specifically handling the "run at startup" functionality by managing
//! shell shortcuts in the user's Startup folder.
//!
//! This is achieved by creating or removing a Windows Shortcut (`.lnk`) file
//! pointing to the current executable.

use directories::UserDirs;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// The name of the application used for shortcut naming.
/// This will result in a file named `NextTabletDriver.lnk`.
const APP_NAME: &str = "NextTabletDriver";

/// Locates the Windows Startup folder for the current user.
///
/// # Internal Logic
/// It constructs the path: `C:\Users\<User>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`
///
/// # Returns
/// * `Some(PathBuf)` containing the absolute path to the startup folder.
/// * `None` if the home directory cannot be determined via OS APIs.
fn get_startup_folder() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| {
        let mut path = dirs.home_dir().to_path_buf();
        path.push("AppData");
        path.push("Roaming");
        path.push("Microsoft");
        path.push("Windows");
        path.push("Start Menu");
        path.push("Programs");
        path.push("Startup");
        path
    })
}

/// Returns the full path where the application shortcut should be located.
///
/// # Example
/// ```no_run
/// let path = get_shortcut_path().unwrap();
/// println!("Shortcut will be at: {:?}", path);
/// ```
fn get_shortcut_path() -> Option<PathBuf> {
    get_startup_folder().map(|mut p| {
        p.push(format!("{}.lnk", APP_NAME));
        p
    })
}

/// Enables or disables the application's automatic launch at Windows startup.
///
/// # Arguments
/// * `enabled` - If `true`, creates a Windows shortcut (.lnk). If `false`, removes it.
///
/// # Example
/// ```no_run
/// match set_run_at_startup(true) {
///     Ok(_) => println!("Successfully registered at startup"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Technical Details
/// Windows `.lnk` files are a proprietary binary format. To avoid complex binary encoding,
/// this function:
/// 1. Generates a temporary **VBScript** file.
/// 2. Uses the `WScript.Shell` COM object to create the shortcut.
/// 3. Executes the script via `wscript.exe`.
/// 4. Deletes the temporary script file.
///
/// # Errors
/// Returns an error if:
/// * The startup path cannot be determined.
/// * The current executable path is invalid or inaccessible.
/// * Writing the temporary VBScript to the `%TEMP%` directory fails.
/// * The system fails to execute the shortcut creation command.
pub fn set_run_at_startup(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut_path = get_shortcut_path().ok_or("Could not determine startup folder path")?;

    if enabled {
        let exe_path = env::current_exe()?;
        let exe_path_str = exe_path.to_str().ok_or("Invalid executable path")?;
        let shortcut_path_str = shortcut_path.to_str().ok_or("Invalid shortcut path")?;

        // Create a temporary VBScript to create the shortcut
        let vbs_content = format!(
            r#"Set oWS = WScript.CreateObject("WScript.Shell")
            Set oLink = oWS.CreateShortcut("{}")
            oLink.TargetPath = "{}"
            oLink.WorkingDirectory = "{}"
            oLink.Save"#,
            shortcut_path_str.replace("\\", "\\\\"),
            exe_path_str.replace("\\", "\\\\"),
            exe_path
                .parent()
                .unwrap_or(&exe_path)
                .to_str()
                .unwrap_or("")
                .replace("\\", "\\\\")
        );

        let temp_vbs = env::temp_dir().join("create_shortcut.vbs");
        fs::write(&temp_vbs, vbs_content)?;

        let status = Command::new("wscript").arg(&temp_vbs).status()?;

        // Clean up the temporary script regardless of the outcome
        let _ = fs::remove_file(temp_vbs);

        if !status.success() {
            return Err("Failed to create startup shortcut".into());
        }

        log::info!(target: "Startup", "Created startup shortcut: {:?}", shortcut_path);
    } else if shortcut_path.exists() {
        fs::remove_file(&shortcut_path)?;
        log::info!(target: "Startup", "Removed startup shortcut: {:?}", shortcut_path);
    }
    Ok(())
}

/// Checks if the application is currently configured to run at startup.
///
/// # Returns
/// * `true` if the shortcut file (`NextTabletDriver.lnk`) exists in the Startup folder.
/// * `false` otherwise or if the path cannot be determined.
///
/// # Example
/// ```no_run
/// if is_run_at_startup_registered() {
///     println!("The app will run on next boot.");
/// }
/// ```
pub fn is_run_at_startup_registered() -> bool {
    get_shortcut_path().map(|p| p.exists()).unwrap_or(false)
}
