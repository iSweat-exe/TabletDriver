//! # Automatic Updates
//!
//! This module handles checking for, downloading, and launching new releases of
//! the application from GitHub. It operates in a background thread to prevent pausing
//! the main UI or input processing.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

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

const OWNER: &str = "Next-Tablet-Driver";
const REPO: &str = "NextTabletDriver";

fn github_api_url() -> String {
    format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        OWNER, REPO
    )
}

/// Queries the GitHub API to check if a newer version is available.
///
/// # Returns
/// - `Ok(Some(Release))` if the remote tag name is different from the local `VERSION`.
/// - `Ok(None)` if the current version matches the latest remote version.
/// - `Err` if the network request or parsing fails.
pub fn check_for_updates() -> Result<Option<Release>, Box<dyn std::error::Error>> {
    let url = github_api_url();
    let response = ureq::get(&url)
        .set("User-Agent", "NextTabletDriver-AutoUpdate")
        .call()?;

    if response.status() != 200 {
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let release: Release = response.into_json()?;

    let remote_version = release.tag_name.trim_start_matches('v');
    let local_version = crate::VERSION;

    if remote_version != local_version {
        log::info!(
            target: "Update",
            "New version available: {} (local version: {})",
            remote_version,
            local_version
        );
        Ok(Some(release))
    } else {
        log::info!(
            target: "Update",
            "No new updates found. You are on the latest version ({})",
            local_version
        );
        Ok(None)
    }
}

/// Downloads the specified release installer and launches it.
///
/// # Process
/// 1. Finds a suitable asset for the current platform.
/// 2. Downloads the binary to a temporary location.
/// 3. Spawns the installer/updater process.
/// 4. Exits the current application instance so the installer can overwrite files.
///
/// # Platform Specifics
/// - **Windows**: Looks for `.exe` assets, saves to `%TEMP%\Next_Tablet_Driver_Setup.exe`.
/// - **Linux**: Looks for `.AppImage` or `.tar.gz` assets, saves to `/tmp/`.
pub fn download_and_install(release: Release) -> Result<(), Box<dyn std::error::Error>> {
    let asset = find_platform_asset(&release)?;

    let download_url = &asset.browser_download_url;

    log::info!(
        target: "Update",
        "Downloading update from {}",
        download_url
    );

    let response = ureq::get(download_url)
        .set("User-Agent", "NextTabletDriver-AutoUpdate")
        .call()?;

    if response.status() != 200 {
        return Err(format!("Download failed: {}", response.status()).into());
    }

    let mut temp_path = env::temp_dir();
    temp_path.push(&asset.name);

    {
        let mut file = fs::File::create(&temp_path)?;
        std::io::copy(&mut response.into_reader(), &mut file)?;
    }

    log::info!(
        target: "Update",
        "Download complete, saved to {:?}",
        temp_path
    );

    // Make the file executable on Linux
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755));
    }

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

/// Finds the appropriate release asset for the current platform.
fn find_platform_asset(release: &Release) -> Result<&Asset, Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".exe"))
            .or_else(|| release.assets.first())
            .ok_or_else(|| "No suitable installer asset found in release".into())
    }

    #[cfg(target_os = "linux")]
    {
        release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".AppImage"))
            .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".tar.gz")))
            .or_else(|| release.assets.first())
            .ok_or_else(|| "No suitable Linux asset found in release".into())
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        release
            .assets
            .first()
            .ok_or_else(|| "No assets found in release".into())
    }
}
