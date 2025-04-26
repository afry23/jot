// src/storage_service.rs - Using settings.json for configuration
use log::{error, info, warn};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager, Runtime};

// Get default storage directory
pub fn get_default_storage_dir<R: Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory")
}

// Get settings file path
fn get_settings_path<R: Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    app_dir.join("settings.json")
}

// Validate a user-provided storage path
fn validate_storage_path(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);

    // Check if path exists, if not try to create it
    if !path_buf.exists() {
        match std::fs::create_dir_all(&path_buf) {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to create directory: {}", e)),
        }
    }

    // Check if path is a directory
    if !path_buf.is_dir() {
        return Err("Specified path is not a directory".to_string());
    }

    // Check if path is writable
    // Create a temporary file to verify
    let temp_file_path = path_buf.join(".jot_write_test");
    match std::fs::write(&temp_file_path, "test") {
        Ok(_) => {
            // Clean up the test file
            let _ = std::fs::remove_file(temp_file_path);
        }
        Err(e) => return Err(format!("Directory is not writable: {}", e)),
    }

    Ok(path_buf)
}

// Get the current storage directory based on configuration
pub fn get_current_storage_dir<R: Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
    // Load settings from the main settings file
    let settings_path = get_settings_path(app_handle);

    if settings_path.exists() {
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check if using custom storage
                if let Some(using_custom) = settings["using_custom_storage"].as_bool() {
                    if using_custom {
                        // Get custom path if set
                        if let Some(path) = settings["custom_storage_path"].as_str() {
                            if !path.is_empty() {
                                let custom_path = PathBuf::from(path);
                                if custom_path.exists() || fs::create_dir_all(&custom_path).is_ok()
                                {
                                    return custom_path;
                                } else {
                                    warn!("Custom storage path is invalid or cannot be created, falling back to default");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Default path if not using custom or if custom path is invalid
    get_default_storage_dir(app_handle)
}

// Get the path to a specific note file
pub fn get_note_path<R: Runtime>(app_handle: &AppHandle<R>, tab_index: usize) -> PathBuf {
    let storage_dir = get_current_storage_dir(app_handle);
    storage_dir.join(format!("note_{}.md", tab_index))
}

// Move notes to a new location
pub async fn migrate_notes<R: Runtime>(
    app_handle: &AppHandle<R>,
    old_dir: &PathBuf,
    new_dir: &PathBuf,
) -> Result<(), String> {
    info!("Migrating notes from {:?} to {:?}", old_dir, new_dir);

    // Create a backup before migration
    match crate::backup_service::create_backup(app_handle.clone()).await {
        Ok(backup_path) => {
            info!("Created backup before migration: {}", backup_path);
            // Emit backup created event
            tauri::Emitter::emit(app_handle, "backup-created", backup_path).unwrap();
        }
        Err(e) => {
            error!("Warning: Failed to create backup before migration: {}", e);
            // Continue with migration despite backup failure
            // But we should probably notify the user
        }
    }

    // Create the new directory if it doesn't exist
    if !new_dir.exists() {
        fs::create_dir_all(new_dir)
            .map_err(|e| format!("Failed to create new storage directory: {}", e))?;
    }

    // Copy all note files to the new location
    for tab_index in 0..7 {
        let old_note_path = old_dir.join(format!("note_{}.md", tab_index));
        let new_note_path = new_dir.join(format!("note_{}.md", tab_index));

        if old_note_path.exists() {
            // Read the old note
            let content = fs::read_to_string(&old_note_path)
                .map_err(|e| format!("Failed to read note {}: {}", tab_index, e))?;

            // Write to the new location
            fs::write(&new_note_path, &content).map_err(|e| {
                format!("Failed to write note {} to new location: {}", tab_index, e)
            })?;

            info!("Migrated note {} to new location", tab_index);
        }
    }

    // Emit event to notify UI that storage location has changed
    tauri::Emitter::emit(app_handle, "storage-changed", ()).unwrap();

    Ok(())
}

// Tauri commands
#[tauri::command]
pub fn get_storage_settings<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<serde_json::Value, String> {
    let settings_path = get_settings_path(&app_handle);
    let default_path = get_default_storage_dir(&app_handle)
        .to_string_lossy()
        .to_string();

    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;

        return Ok(serde_json::json!({
            "customPath": settings["custom_storage_path"],
            "defaultPath": default_path,
            "isUsingCustom": settings["using_custom_storage"].as_bool().unwrap_or(false)
        }));
    }

    // Return default values if settings file doesn't exist
    Ok(serde_json::json!({
        "customPath": null,
        "defaultPath": default_path,
        "isUsingCustom": false
    }))
}

#[tauri::command]
pub async fn set_storage_path<R: Runtime>(
    app_handle: AppHandle<R>,
    path: Option<String>,
) -> Result<(), String> {
    let old_storage_dir = get_current_storage_dir(&app_handle);

    // Load current settings
    let settings_path = get_settings_path(&app_handle);
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?
    } else {
        serde_json::json!({
            "theme": "light",
            "fontSize": "medium",
            "activeTab": 0
        })
    };

    // Validate path if provided
    if let Some(path_str) = path.clone() {
        // Validate the path before setting it
        match validate_storage_path(&path_str) {
            Ok(_) => {
                settings["custom_storage_path"] = serde_json::json!(path_str);
                settings["using_custom_storage"] = serde_json::json!(true);
            }
            Err(e) => {
                return Err(format!("Invalid storage path: {}", e));
            }
        }
    } else {
        settings["custom_storage_path"] = serde_json::json!(null);
        settings["using_custom_storage"] = serde_json::json!(false);
    }

    // Save updated settings
    let json_str = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&settings_path, json_str).map_err(|e| format!("Failed to save settings: {}", e))?;

    // If path changed, migrate notes
    let new_storage_dir = get_current_storage_dir(&app_handle);

    // Only migrate if the directories are different
    if old_storage_dir != new_storage_dir {
        migrate_notes(&app_handle, &old_storage_dir, &new_storage_dir).await?;
    }

    Ok(())
}
