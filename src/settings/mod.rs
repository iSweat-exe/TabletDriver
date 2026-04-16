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

    let json = serde_json::to_string_pretty(config).map_err(|e| {
        log::error!(target: "Settings", "Failed to serialize preset '{}': {}", name, e);
        e.to_string()
    })?;
    
    fs::write(&path, json).map_err(|e| {
        log::error!(target: "Settings", "Failed to write preset to {:?}: {}", path, e);
        e.to_string()
    })?;
    
    log::info!(target: "Settings", "Saved preset '{}' to {:?}", name, path);

    // Also update "last_session.json" to point to this or save state?
    // User requested "last settings applied". We can save a "last_session.json" with the current config content
    // OR a reference. Saving content is safer.
    save_last_session(config)?;

    Ok(())
}

pub fn save_last_session(config: &MappingConfig) -> Result<(), String> {
    let dir = get_settings_dir();
    let path = dir.join("last_session.json");
    let json = serde_json::to_string_pretty(config).map_err(|e| {
        log::error!(target: "Settings", "Failed to serialize last session: {}", e);
        e.to_string()
    })?;
    fs::write(&path, json).map_err(|e| {
        log::error!(target: "Settings", "Failed to write last session to {:?}: {}", path, e);
        e.to_string()
    })?;
    log::debug!(target: "Settings", "Last session persistent state updated");
    Ok(())
}

pub fn load_last_session() -> Option<MappingConfig> {
    let dir = get_settings_dir();
    let path = dir.join("last_session.json");
    if !path.exists() {
        log::debug!(target: "Settings", "No last session file found at {:?}", path);
        return None;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => {
                log::info!(target: "Settings", "Loaded last session from {:?}", path);
                Some(config)
            }
            Err(e) => {
                log::error!(target: "Settings", "Failed to parse last session JSON: {}", e);
                None
            }
        },
        Err(e) => {
            log::error!(target: "Settings", "Failed to read last session file: {}", e);
            None
        }
    }
}

pub fn load_settings_from_file(path: PathBuf) -> Result<MappingConfig, String> {
    let content = fs::read_to_string(&path).map_err(|e| {
        log::error!(target: "Settings", "Failed to read settings file {:?}: {}", path, e);
        e.to_string()
    })?;
    let config = serde_json::from_str(&content).map_err(|e| {
        log::error!(target: "Settings", "Failed to parse settings JSON from {:?}: {}", path, e);
        e.to_string()
    })?;
    log::info!(target: "Settings", "Loaded settings from {:?}", path);
    Ok(config)
}
