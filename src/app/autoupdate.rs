//! # Automatic Updates
//!
//! This module handles checking for, downloading, and launching new releases of
//! the application from GitHub. It operates in a background thread to prevent pausing
//! the main UI or input processing.

use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Represents a GitHub Release object returned from the API.
#[derive(Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

/// Represents an individual file (asset) attached to a GitHub Release.
#[derive(Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

/// Represents the current phase/status of the update process.
/// This enum is passed via channels from the update thread to the main UI thread.
#[derive(Clone)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available(Release),
    Downloading(f32),
    ReadyToInstall(PathBuf),
    Error(String),
}

impl UpdateStatus {
    pub fn as_release(&self) -> Option<&Release> {
        if let Self::Available(release) = self {
            Some(release)
        } else {
            None
        }
    }
}

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/isweat-exe/NextTabletDriver/releases/latest";

/// Queries the GitHub API to check if a newer version is available.
///
/// # Returns
/// - `Ok(Some(Release))` if the remote tag name is different from the local `VERSION`.
/// - `Ok(None)` if the current version matches the latest remote version.
/// - `Err` if the network request or parsing fails.
pub fn check_for_updates() -> Result<Option<Release>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("NextTabletDriver-AutoUpdate")
        .build()?;

    let release: Release = client.get(GITHUB_API_URL).send()?.json()?;

    let remote_version = release.tag_name.trim_start_matches('v');
    let local_version = crate::VERSION;

    if remote_version != local_version {
        log::info!(target: "Update", "New version available: {} (local version: {})", remote_version, local_version);
        Ok(Some(release))
    } else {
        log::info!(target: "Update", "No new updates found. You are on the latest version ({})", local_version);
        Ok(None)
    }
}

/// Downloads the specified release installer to the system's temporary directory
/// and launches it.
///
/// # Process
/// 1. Finds an asset ending in `.exe` (or falls back to the first asset).
/// 2. Downloads the binary directly to `%TEMP%\Next_Tablet_Driver_Setup.exe`.
/// 3. Spawns the installer process.
/// 4. Exits the current application instance so the installer can overwrite files.
pub fn download_and_install(release: Release) -> Result<(), Box<dyn std::error::Error>> {
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".exe"))
        .or_else(|| release.assets.first())
        .ok_or("No suitable installer asset found in release")?;
    let download_url = &asset.browser_download_url;

    log::info!(target: "Update", "Downloading update from {}", download_url);

    let client = reqwest::blocking::Client::builder()
        .user_agent("NextTabletDriver-AutoUpdate")
        .build()?;
    let mut response = client.get(download_url).send()?;
    let mut temp_path = env::temp_dir();
    temp_path.push("Next_Tablet_Driver_Setup.exe");

    {
        let mut file = fs::File::create(&temp_path)?;
        response.copy_to(&mut file)?;
    } // File handle is closed here

    log::info!(target: "Update", "Download complete settings setup at {:?}", temp_path);

    // Launch installer
    let status = Command::new(&temp_path).spawn();

    match status {
        Ok(_) => {
            log::info!(target: "Update", "Installer launched, exiting...");
            std::process::exit(0);
        }
        Err(e) => {
            log::error!(target: "Update", "Failed to launch installer: {}", e);
            Err(e.into())
        }
    }
}
