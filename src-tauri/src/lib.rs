// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#![allow(deprecated)]

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{App, AppHandle, Manager};

mod backup_service;
mod chatgpt_service;
mod credential_manager;
mod language_services;
mod logging;
mod nextcloud;
mod storage_service;
mod sync_service; // Add the sync service module

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppSettings {
    theme: String,
    font_size: String,
    active_tab: Option<usize>,
    custom_storage_path: Option<String>,
    using_custom_storage: bool,
}

fn get_note_path(app_handle: &AppHandle, tab_index: usize) -> PathBuf {
    storage_service::get_note_path(app_handle, tab_index)
}

// Get the path to the notes directory
fn get_notes_dir(app_handle: &AppHandle) -> PathBuf {
    // In newer Tauri versions, we use app_handle.path() instead of path_resolver
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    // Create the directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
    }

    app_dir
}

fn get_settings_path(app_handle: &AppHandle) -> PathBuf {
    let mut path = get_notes_dir(app_handle);
    path.push("settings.json");
    path
}

#[tauri::command]
fn save_settings(app_handle: AppHandle, settings: serde_json::Value) -> Result<(), String> {
    let path = get_settings_path(&app_handle);

    // Convert to pretty JSON string
    let json_str = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Write to file
    fs::write(path, json_str).map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
fn load_settings(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    let path = get_settings_path(&app_handle);

    if path.exists() {
        // Read the file content
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        // Parse JSON
        let settings: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;

        Ok(settings)
    } else {
        // Return default settings if file doesn't exist
        let default_settings = serde_json::json!({
            "theme": "light",
            "fontSize": "medium",
            "activeTab": 0
        });

        Ok(default_settings)
    }
}

#[tauri::command]
fn save_active_tab(app_handle: AppHandle, tab_index: usize) -> Result<(), String> {
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
            "fontSize": "medium"
        })
    };

    // Update activeTab
    settings["activeTab"] = serde_json::json!(tab_index);

    // Save back to file
    let json_str = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(settings_path, json_str).map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
fn save_note(app_handle: AppHandle, tab_index: usize, content: String) -> Result<(), String> {
    let path = get_note_path(&app_handle, tab_index);

    fs::write(path, content).map_err(|e| format!("Failed to save note: {}", e))
}

#[tauri::command]
fn load_notes(app_handle: AppHandle) -> Result<HashMap<usize, String>, String> {
    let mut notes = HashMap::new();

    // Try to load notes for all 7 tabs
    for tab_index in 0..7 {
        let path = get_note_path(&app_handle, tab_index);

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    notes.insert(tab_index, content);
                }
                Err(e) => {
                    return Err(format!("Failed to load note {}: {}", tab_index, e));
                }
            }
        }
    }

    Ok(notes)
}

#[tauri::command]
fn close_window(app_handle: AppHandle) {
    info!("Closing the main window");
    if let Some(main_window) = app_handle.get_webview_window("main") {
        main_window
            .hide()
            .unwrap_or_else(|e| info!("Failed to hide window: {}", e));
    }
}

fn configure_tray_menu(app: &App) -> Result<(), tauri::Error> {
    let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;
    let toggle = MenuItemBuilder::new("Toggle").id("toggle").build(app)?;

    let tray_menu = MenuBuilder::new(app).items(&[&toggle, &quit]).build()?;

    let tray_icon = app.tray_by_id("main").unwrap();

    tray_icon.set_menu(Some(tray_menu))?;

    tray_icon.on_menu_event(|app, event| match event.id.as_ref() {
        "quit" => std::process::exit(0),
        "toggle" => toggle_window(app),
        _ => {}
    });

    tray_icon.on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            toggle_window(&app.app_handle());
        }
    });

    Ok(())
}

fn toggle_window(app: &AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    if window.is_visible().unwrap() {
        window.hide().unwrap();
    } else {
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();
    let _tauri_app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                    window.eval("setTimeout(() => { console.log('DevTools opened - Console ready!'); }, 100);").ok();
                }
            }
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

                let ctrl_j_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyJ);
                let app_handle = app.handle();
                app_handle.plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                    .with_handler({
                        let app_handle = app_handle.clone();
                        move |_app, shortcut, event| {
                            println!("{:?}", shortcut);
                            if shortcut == &ctrl_j_shortcut {
                                match event.state() {
                                    ShortcutState::Pressed => {
                                        println!("Ctrl-J Pressed!");
                                    }
                                    ShortcutState::Released => {
                                        println!("Ctrl-J Released!");
                                        toggle_window(&app_handle);
                                    }
                                }
                            }
                        }
                    })
                    .build(),
                )?;

                app_handle.global_shortcut().register(ctrl_j_shortcut)?;
            }
            logging::init_logger(app.app_handle())?;
            info!("Jot application starting up");
            configure_tray_menu(app).unwrap();
            // Initialize the sync service
            sync_service::init_sync_service(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_note,
            save_settings,
            load_settings,
            save_active_tab,
            load_notes,
            close_window,
            // Language services
            language_services::check_grammar,
            language_services::translate_text,
            language_services::save_language_tool_config,
            language_services::save_deepl_config,
            language_services::get_language_services_config,
            // ChatGPT services
            chatgpt_service::chat_with_gpt,
            chatgpt_service::get_chatgpt_config_command,
            chatgpt_service::save_chatgpt_config_command,
            // Nextcloud sync services
            nextcloud::commands::save_nextcloud_config_command,
            nextcloud::commands::get_nextcloud_config_command,
            nextcloud::commands::test_nextcloud_connection,
            nextcloud::commands::sync_all_notes,
            nextcloud::commands::get_sync_status,
            nextcloud::commands::upload_all_notes,
            nextcloud::commands::download_all_notes,
            // Sync service commands
            sync_service::trigger_sync_command,
            sync_service::stop_sync_command,
            // Backup service commands
            backup_service::create_backup,
            backup_service::list_backups,
            backup_service::restore_backup,
            backup_service::delete_backup,
            backup_service::count_backups,
            backup_service::prune_backups,
            // Logging commands
            logging::get_latest_logs,
            logging::list_log_files,
            logging::log_from_frontend,
            logging::calculate_log_size,
            logging::clear_logs,
            logging::set_log_level,
            logging::get_log_level,
            credential_manager::store_nextcloud_credential,
            credential_manager::get_nextcloud_credential,
            credential_manager::delete_nextcloud_credential,
            credential_manager::store_languagetool_credential,
            credential_manager::get_languagetool_credential,
            credential_manager::has_languagetool_credential,
            credential_manager::store_deepl_credential,
            credential_manager::get_deepl_credential,
            credential_manager::has_deepl_credential,
            credential_manager::store_chatgpt_credential,
            credential_manager::get_chatgpt_credential,
            storage_service::get_storage_settings,
            storage_service::set_storage_path
        ])
        .run(context)
        .expect("error while running tauri application");
}
