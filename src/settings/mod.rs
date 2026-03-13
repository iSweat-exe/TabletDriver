use crate::core::config::models::MappingConfig;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub fn get_settings_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "NextTabletDriver", "NextTabletReader") {
        let config_dir = proj_dirs.config_dir().join("Settings");
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        config_dir
    } else {
        PathBuf::from("Settings") // Fallback
    }
}

pub fn save_settings(name: &str, config: &MappingConfig) -> Result<(), String> {
    let dir = get_settings_dir();
    let filename = if name.ends_with(".json") {
        name.to_string()
    } else {
        format!("{}.json", name)
    };
    let path = dir.join(filename);

    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    log::info!(target: "Settings", "Saved preset '{}'", name);

    // Also update "last_session.json" to point to this or save state?
    // User requested "last settings applied". We can save a "last_session.json" with the current config content
    // OR a reference. Saving content is safer.
    save_last_session(config)?;

    Ok(())
}

pub fn save_last_session(config: &MappingConfig) -> Result<(), String> {
    let dir = get_settings_dir();
    let path = dir.join("last_session.json");
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_last_session() -> Option<MappingConfig> {
    let dir = get_settings_dir();
    let path = dir.join("last_session.json");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&content) {
                log::info!(target: "Settings", "Loaded last session");
                return Some(config);
            }
        }
    }
    None
}

pub fn load_settings_from_file(path: PathBuf) -> Result<MappingConfig, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    log::info!(target: "Settings", "Loaded settings from file");
    Ok(config)
}
