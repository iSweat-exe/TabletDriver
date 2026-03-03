use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use directories::UserDirs;

const APP_NAME: &str = "TabletDriver";

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

fn get_shortcut_path() -> Option<PathBuf> {
    get_startup_folder().map(|mut p| {
        p.push(format!("{}.lnk", APP_NAME));
        p
    })
}

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
            exe_path.parent().unwrap_or(&exe_path).to_str().unwrap_or("").replace("\\", "\\\\")
        );

        let temp_vbs = env::temp_dir().join("create_shortcut.vbs");
        fs::write(&temp_vbs, vbs_content)?;

        let status = Command::new("wscript")
            .arg(&temp_vbs)
            .status()?;

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

pub fn is_run_at_startup_registered() -> bool {
    get_shortcut_path().map(|p| p.exists()).unwrap_or(false)
}
