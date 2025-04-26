use crate::credential_manager::get_nextcloud_credential;
use crate::nextcloud::error::SyncError;
use crate::nextcloud::types::NextcloudConfig;
use log::debug;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

pub fn get_config_path<R: Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
    debug!("Getting Nextcloud config path");
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    app_dir.join("nextcloud_config.json")
}

// Get Nextcloud configuration
pub fn get_nextcloud_config<R: Runtime>(app_handle: &AppHandle<R>) -> NextcloudConfig {
    debug!("Loading Nextcloud config");
    let config_path = get_config_path(app_handle);

    let mut config = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("Error parsing Nextcloud config: {}", e);
                    return NextcloudConfig::default();
                }
            },
            Err(e) => {
                log::warn!("Error reading Nextcloud config: {}", e);
                return NextcloudConfig::default();
            }
        }
    } else {
        NextcloudConfig::default()
    };

    if let Err(e) = config.load_credentials() {
        log::warn!("Error loading credentials: {}", e);
    }

    // Load sensitive data from keychain when needed
    if !config.username.is_empty() {
        match get_nextcloud_credential(config.username.clone()) {
            Ok(password) => {
                config.password = password;
            }
            Err(e) => {
                // If it doesn't exist in keychain yet, that's ok
                if !e.contains("not found") {
                    log::warn!("Error loading Nextcloud credentials: {}", e);
                }
            }
        }
    }

    config
}

// Save Nextcloud configuration
pub fn save_nextcloud_config<R: Runtime>(
    app_handle: &AppHandle<R>,
    config: &NextcloudConfig,
) -> Result<(), SyncError> {
    let config_path = get_config_path(app_handle);

    // Convert to JSON string
    let json_str = serde_json::to_string_pretty(&config)?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Write to file
    std::fs::write(config_path, json_str)?;

    Ok(())
}
