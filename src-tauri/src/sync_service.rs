use crate::nextcloud::client::NextcloudClient;
use crate::nextcloud::config::{get_nextcloud_config, save_nextcloud_config};
use crate::nextcloud::types::SyncStatus;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::{mpsc, Mutex};

pub struct SyncState {
    running: bool,
    last_sync_attempt: Option<Instant>,
    tx: Option<mpsc::Sender<SyncCommand>>,
}

#[derive(Debug)]
enum SyncCommand {
    Sync,
    Stop,
}

// Helper function to get the path to a note file
fn get_note_path_fn<R: Runtime>(app_handle: &AppHandle<R>) -> impl Fn(usize) -> PathBuf + '_ {
    move |tab_index| crate::storage_service::get_note_path(app_handle, tab_index)
}

// Initialize the sync service
pub fn init_sync_service<R: Runtime>(
    app: &tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.app_handle();

    // Create channel for communication with the sync service
    let (tx, mut rx) = mpsc::channel::<SyncCommand>(10);

    // Store the sender in app state using tokio's Mutex
    app.manage(Arc::new(Mutex::new(SyncState {
        running: true,
        last_sync_attempt: None,
        tx: Some(tx),
    })));

    // Extract config params once at startup
    let config = get_nextcloud_config(app_handle);

    let sync_interval_minutes = config.sync_interval_minutes;
    let auto_sync = config.auto_sync;
    let sync_on_startup = config.sync_on_startup;

    // Create channel for results
    let (result_tx, mut result_rx) = mpsc::channel::<Result<SyncStatus, String>>(10);

    // Clone app_handle for background task
    let app_handle_clone = app_handle.clone();

    // Spawn the background sync task
    tauri::async_runtime::spawn(async move {
        let sync_interval = Duration::from_secs(sync_interval_minutes as u64 * 60);
        let mut interval_timer = tokio::time::interval(Duration::from_secs(60)); // Check every minute

        let state_arc = app_handle_clone.state::<Arc<Mutex<SyncState>>>();

        // If sync_on_startup, trigger an initial sync
        if sync_on_startup {
            log::info!("Performing initial sync on startup");

            let result = perform_sync(&app_handle_clone).await;
            let _ = result_tx.send(result).await;

            // Update last sync time
            let mut state = state_arc.lock().await;
            state.last_sync_attempt = Some(Instant::now());

            log::info!("Initial sync completed");
        }

        loop {
            // Wait for either the interval or a command
            tokio::select! {
                _ = interval_timer.tick() => {
                    // Check if we should auto-sync
                    let should_sync = {
                        let state = state_arc.lock().await;
                        if !state.running {
                            break; // Exit loop if not running
                        }

                        auto_sync && match state.last_sync_attempt {
                            Some(last) => last.elapsed() >= sync_interval,
                            None => true
                        }
                    };

                    if should_sync {
                        log::info!("Auto sync triggered");
                        let result = perform_sync(&app_handle_clone).await;
                        let _ = result_tx.send(result).await;

                        // Update last sync time
                        let mut state = state_arc.lock().await;
                        state.last_sync_attempt = Some(Instant::now());

                        log::info!("Auto sync completed");
                    }
                }

                Some(cmd) = rx.recv() => {
                    match cmd {
                        SyncCommand::Sync => {
                            log::info!("Manual sync triggered");
                            let result = perform_sync(&app_handle_clone).await;
                            let _ = result_tx.send(result).await;

                            // Update last sync time
                            let mut state = state_arc.lock().await;
                            state.last_sync_attempt = Some(Instant::now());

                            log::info!("Manual sync completed");
                        }
                        SyncCommand::Stop => {
                            log::info!("Stopping sync service");
                            let mut state = state_arc.lock().await;
                            state.running = false;
                            break;
                        }
                    }
                }
            }
        }

        log::info!("Sync service stopped");
    });

    // Handle sync results
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(result) = result_rx.recv().await {
            match result {
                Ok(_) => {
                    // Emit event to notify frontend
                    let _ = tauri::Emitter::emit(&app_handle_clone, "sync-completed", ());
                }
                Err(e) => {
                    log::error!("Sync error: {}", e);
                    // Emit error event
                    let _ = tauri::Emitter::emit(&app_handle_clone, "sync-error", e);
                }
            }
        }
    });

    Ok(())
}

// Perform sync operation using the Nextcloud client
async fn perform_sync<R: Runtime>(app_handle: &AppHandle<R>) -> Result<SyncStatus, String> {
    // Create a backup before syncing
    match crate::backup_service::create_backup(app_handle.clone()).await {
        Ok(backup_path) => {
            log::info!("Created backup before sync: {}", backup_path);
            // Emit backup created event
            tauri::Emitter::emit(app_handle, "backup-created", backup_path).unwrap();
        }
        Err(e) => {
            log::warn!("Warning: Failed to create backup before sync: {}", e);
            // Continue with sync despite backup failure
        }
    }

    // Emit started event
    tauri::Emitter::emit(app_handle, "sync-started", ()).unwrap();

    let config = get_nextcloud_config(app_handle);

    // Create the Nextcloud client
    let client = match NextcloudClient::new(config.clone()) {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("Failed to create Nextcloud client: {}", e);
            // Emit error event
            tauri::Emitter::emit(app_handle, "sync-error", &error_msg).unwrap();
            return Err(error_msg);
        }
    };

    // Create a note path function
    let note_path_fn = get_note_path_fn(app_handle);

    // Perform the sync
    let sync_result = match client.sync_all_notes(note_path_fn, true).await {
        Ok(status) => status,
        Err(e) => {
            let error_msg = format!("Sync failed: {}", e);
            // Emit error event
            tauri::Emitter::emit(app_handle, "sync-error", &error_msg).unwrap();
            return Err(error_msg);
        }
    };

    // Update last sync time in config
    let mut updated_config = config.clone();
    updated_config.last_sync = sync_result.last_sync;
    if let Err(e) = save_nextcloud_config(app_handle, &updated_config) {
        log::warn!("Failed to save last sync time: {}", e);
    }

    // Emit completed event
    tauri::Emitter::emit(app_handle, "sync-completed", ()).unwrap();

    Ok(sync_result)
}

// Trigger a manual sync
#[tauri::command]
pub async fn trigger_sync_command(app_handle: AppHandle) -> Result<(), String> {
    let state_arc = app_handle.state::<Arc<Mutex<SyncState>>>();
    let state = state_arc.lock().await;

    if let Some(tx) = &state.tx {
        let tx_clone = tx.clone();
        drop(state); // Release the lock before the await

        tx_clone
            .send(SyncCommand::Sync)
            .await
            .map_err(|e| format!("Failed to trigger sync: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_sync_command(app_handle: AppHandle) -> Result<(), String> {
    let state_arc = app_handle.state::<Arc<Mutex<SyncState>>>();
    let state = state_arc.lock().await;

    if let Some(tx) = &state.tx {
        let tx_clone = tx.clone();
        drop(state); // Release the lock before the await

        tx_clone
            .send(SyncCommand::Stop)
            .await
            .map_err(|e| format!("Failed to stop sync: {}", e))?;
    }

    Ok(())
}
