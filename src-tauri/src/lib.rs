// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

// Get the path to a specific note file
fn get_note_path(app_handle: &AppHandle, tab_index: usize) -> PathBuf {
    let mut path = get_notes_dir(app_handle);
    path.push(format!("note_{}.html", tab_index));
    path
}

#[tauri::command]
fn save_note(app_handle: AppHandle, tab_index: usize, content: String) -> Result<(), String> {
    let path = get_note_path(&app_handle, tab_index);
    
    fs::write(path, content)
        .map_err(|e| format!("Failed to save note: {}", e.to_string()))
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
                    return Err(format!("Failed to load note {}: {}", tab_index, e.to_string()));
                }
            }
        }
    }
    
    Ok(notes)
}

#[tauri::command]
fn close_window(app_handle: AppHandle) {
    // Get the main window (using get_webview_window in newer Tauri versions)
    if let Some(window) = app_handle.get_webview_window("main") {
        // Close the window
        window.close().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_note,
            load_notes,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}