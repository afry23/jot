use std::path::PathBuf;
use tauri::{command, AppHandle, Runtime};

use crate::nextcloud::client::NextcloudClient;
use crate::nextcloud::config::{get_nextcloud_config, save_nextcloud_config};
use crate::nextcloud::types::{NextcloudConfig, NoteStatus, SyncStatus};

// Helper function to get the path to a note file
fn get_note_path<R: Runtime>(app_handle: &AppHandle<R>, tab_index: usize) -> PathBuf {
    crate::storage_service::get_note_path(app_handle, tab_index)
}

// Tauri command: Save Nextcloud configuration
#[command]
pub async fn save_nextcloud_config_command<R: Runtime>(
    app_handle: AppHandle<R>,
    next_cloud_config: NextcloudConfig,
) -> Result<(), String> {
    log::debug!("Saving Nextcloud config...");

    if let Err(e) = next_cloud_config.save_credentials() {
        log::error!("Failed to save credentials: {}", e);
        return Err(format!("Failed to save credentials: {}", e));
    }

    let config_to_save = NextcloudConfig {
        password: String::new(), // Don't save password in config file
        ..next_cloud_config.clone()
    };

    if let Err(e) = save_nextcloud_config(&app_handle, &config_to_save) {
        return Err(format!("Failed to save config: {}", e));
    }

    Ok(())
}

// Tauri command: Get Nextcloud configuration
#[tauri::command]
pub fn get_nextcloud_config_command<R: Runtime>(app_handle: AppHandle<R>) -> NextcloudConfig {
    get_nextcloud_config(&app_handle)
}

// Tauri command: Test Nextcloud connection
#[tauri::command]
pub async fn test_nextcloud_connection<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<bool, String> {
    let config = get_nextcloud_config(&app_handle);
    // Create client
    let client = match NextcloudClient::new(config) {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create Nextcloud client: {}", e);
            return Err(format!("Failed to create Nextcloud client: {}", e));
        }
    };

    // Test connection
    client
        .test_connection()
        .await
        .map_err(|e| format!("Connection test failed: {}", e))
}

// Tauri command: Sync all notes
#[tauri::command]
pub async fn sync_all_notes<R: Runtime>(app_handle: AppHandle<R>) -> Result<SyncStatus, String> {
    let app_handle_clone = app_handle.clone();
    let mut config = get_nextcloud_config(&app_handle);

    // Create a backup before syncing
    match crate::backup_service::create_backup(app_handle.clone()).await {
        Ok(backup_path) => {
            log::info!("Created backup before sync: {}", backup_path);
            // Emit backup created event
            tauri::Emitter::emit(&app_handle, "backup-created", backup_path).unwrap();
        }
        Err(e) => {
            log::warn!("Warning: Failed to create backup before sync: {}", e);
            // Continue with sync despite backup failure
        }
    }

    // Emit started event
    tauri::Emitter::emit(&app_handle, "sync-started", ()).unwrap();

    // Create the client
    let client = match NextcloudClient::new(config.clone()) {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to create Nextcloud client: {}", e);
            // Emit error event
            tauri::Emitter::emit(&app_handle, "sync-error", &error_msg).unwrap();
            return Err(error_msg);
        }
    };

    // Function to get note path (to be passed to sync_all_notes)
    let note_path_fn = move |tab_index| get_note_path(&app_handle_clone, tab_index);

    // Perform sync
    let sync_result = match client.sync_all_notes(note_path_fn, true).await {
        Ok(status) => status,
        Err(e) => {
            let error_msg = format!("Sync failed: {}", e);
            // Emit error event
            tauri::Emitter::emit(&app_handle, "sync-error", &error_msg).unwrap();
            return Err(error_msg);
        }
    };

    // Update last sync time
    config.last_sync = sync_result.last_sync;
    if let Err(e) = save_nextcloud_config(&app_handle, &config) {
        log::warn!("Failed to save last sync time: {}", e);
    }

    // Emit completed event
    tauri::Emitter::emit(&app_handle, "sync-completed", ()).unwrap();

    Ok(sync_result)
}

// Tauri command: Get sync status
#[tauri::command]
pub fn get_sync_status<R: Runtime>(app_handle: AppHandle<R>) -> SyncStatus {
    let config = get_nextcloud_config(&app_handle);

    SyncStatus {
        last_sync: config.last_sync,
        syncing: false,
        error: None,
        notes_status: std::collections::HashMap::new(), // Empty until sync is performed
    }
}

#[tauri::command]
pub async fn upload_all_notes<R: Runtime>(app_handle: AppHandle<R>) -> Result<SyncStatus, String> {
    let app_handle_clone = app_handle.clone();
    let config = get_nextcloud_config(&app_handle);

    // Create a backup before uploading
    match crate::backup_service::create_backup(app_handle.clone()).await {
        Ok(backup_path) => {
            log::info!("Created backup before upload: {}", backup_path);
            tauri::Emitter::emit(&app_handle, "backup-created", backup_path).unwrap();
        }
        Err(e) => {
            log::warn!("Warning: Failed to create backup before upload: {}", e);
            // Continue with upload despite backup failure
        }
    }

    // Emit started event with upload type
    tauri::Emitter::emit(&app_handle, "sync-started", "upload").unwrap();

    // Create the client
    let client = match NextcloudClient::new(config.clone()) {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to create Nextcloud client: {}", e);
            tauri::Emitter::emit(&app_handle, "sync-error", &error_msg).unwrap();
            return Err(error_msg);
        }
    };

    // Ensure remote directory exists
    if let Err(e) = client.ensure_remote_directory().await {
        let error_msg = format!("Failed to ensure remote directory: {}", e);
        tauri::Emitter::emit(&app_handle, "sync-error", &error_msg).unwrap();
        return Err(error_msg);
    }

    // Upload each note
    let mut notes_status = std::collections::HashMap::new();
    for tab_index in 0..7 {
        let note_path = get_note_path(&app_handle_clone, tab_index);

        if note_path.exists() {
            // Read the note content
            let content = match std::fs::read_to_string(&note_path) {
                Ok(content) => content,
                Err(e) => {
                    log::warn!("Failed to read note {}: {}", tab_index, e);
                    continue;
                }
            };

            // Upload the note
            match client.upload_note(tab_index, &content).await {
                Ok(_) => {
                    log::info!("Successfully uploaded note {}", tab_index);

                    // Get remote modified time to return in status
                    let remote_modified =
                        match client.get_remote_note_modified_time(tab_index).await {
                            Ok(time) => time,
                            Err(_) => (client.get_remote_mod_time_from_head(tab_index).await)
                                .unwrap_or_default(),
                        };

                    // Get local modified time
                    let local_modified = match std::fs::metadata(&note_path) {
                        Ok(metadata) => match metadata.modified() {
                            Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
                                Ok(duration) => duration.as_secs(),
                                Err(_) => 0,
                            },
                            Err(_) => 0,
                        },
                        Err(_) => 0,
                    };

                    notes_status.insert(
                        tab_index,
                        NoteStatus {
                            tab_index,
                            local_modified,
                            remote_modified,
                            synced: true,
                            conflict: false,
                        },
                    );
                }
                Err(e) => {
                    log::warn!("Failed to upload note {}: {}", tab_index, e);
                }
            }
        }
    }

    // Update last sync time
    let last_sync = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut updated_config = config.clone();
    updated_config.last_sync = Some(last_sync);
    if let Err(e) = save_nextcloud_config(&app_handle, &updated_config) {
        log::warn!("Failed to save last sync time: {}", e);
    }

    // Create sync status
    let sync_status = SyncStatus {
        last_sync: Some(last_sync),
        syncing: false,
        error: None,
        notes_status,
    };

    // Emit completed event
    tauri::Emitter::emit(&app_handle, "sync-completed", "upload").unwrap();

    Ok(sync_status)
}

#[tauri::command]
pub async fn download_all_notes<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<SyncStatus, String> {
    let app_handle_clone = app_handle.clone();
    let config = get_nextcloud_config(&app_handle);

    // Create a backup before downloading
    match crate::backup_service::create_backup(app_handle.clone()).await {
        Ok(backup_path) => {
            log::info!("Created backup before download: {}", backup_path);
            tauri::Emitter::emit(&app_handle, "backup-created", backup_path).unwrap();
        }
        Err(e) => {
            log::warn!("Warning: Failed to create backup before download: {}", e);
            // Continue with download despite backup failure
        }
    }

    // Emit started event with download type
    tauri::Emitter::emit(&app_handle, "sync-started", "download").unwrap();

    // Create the client
    let client = match NextcloudClient::new(config.clone()) {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to create Nextcloud client: {}", e);
            tauri::Emitter::emit(&app_handle, "sync-error", &error_msg).unwrap();
            return Err(error_msg);
        }
    };

    // Download each note
    let mut notes_status = std::collections::HashMap::new();
    for tab_index in 0..7 {
        // Check if remote note exists
        let remote_modified = match client.get_remote_note_modified_time(tab_index).await {
            Ok(time) => time,
            Err(_) => (client.get_remote_mod_time_from_head(tab_index).await).unwrap_or_default(),
        };

        if let Some(remote_mod_time) = remote_modified {
            // Remote note exists, download it
            match client.download_note(tab_index).await {
                Ok(content) => {
                    log::info!("Successfully downloaded note {}", tab_index);

                    // Get the note path
                    let note_path = get_note_path(&app_handle_clone, tab_index);

                    // Write the note to file
                    match std::fs::write(&note_path, &content) {
                        Ok(_) => {
                            log::info!("Successfully saved note {} to disk", tab_index);

                            // Get local modified time after writing
                            let local_modified = match std::fs::metadata(&note_path) {
                                Ok(metadata) => match metadata.modified() {
                                    Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
                                        Ok(duration) => duration.as_secs(),
                                        Err(_) => 0,
                                    },
                                    Err(_) => 0,
                                },
                                Err(_) => 0,
                            };

                            notes_status.insert(
                                tab_index,
                                NoteStatus {
                                    tab_index,
                                    local_modified,
                                    remote_modified: Some(remote_mod_time),
                                    synced: true,
                                    conflict: false,
                                },
                            );

                            // Emit note-updated event to update UI
                            tauri::Emitter::emit(
                                &app_handle,
                                &format!("note-updated-{}", tab_index),
                                content,
                            )
                            .unwrap();
                        }
                        Err(e) => {
                            log::warn!("Failed to save note {} to disk: {}", tab_index, e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to download note {}: {}", tab_index, e);
                }
            }
        }
    }

    // Update last sync time
    let last_sync = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut updated_config = config.clone();
    updated_config.last_sync = Some(last_sync);
    if let Err(e) = save_nextcloud_config(&app_handle, &updated_config) {
        log::warn!("Failed to save last sync time: {}", e);
    }

    // Create sync status
    let sync_status = SyncStatus {
        last_sync: Some(last_sync),
        syncing: false,
        error: None,
        notes_status,
    };

    // Emit completed event
    tauri::Emitter::emit(&app_handle, "sync-completed", "download").unwrap();

    Ok(sync_status)
}
