// backup_service.rs
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, Runtime};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// Create a backup of all note files
#[tauri::command]
pub async fn create_backup<R: Runtime>(app_handle: AppHandle<R>) -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    // Create backups directory if it doesn't exist
    let backups_dir = app_dir.join("backups");
    if !backups_dir.exists() {
        fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;
    }

    // Format current date/time for filename
    let datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_filename = format!("jot_backup_{}_{}.zip", datetime, timestamp);
    let backup_path = backups_dir.join(&backup_filename);

    // Create the zip file
    let file = fs::File::create(&backup_path)
        .map_err(|e| format!("Failed to create backup file: {}", e))?;

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Add all note files to the zip
    let mut added_files = 0;
    for i in 0..7 {
        let note_path = crate::storage_service::get_note_path(&app_handle, i);
        if note_path.exists() {
            let note_content = fs::read_to_string(&note_path)
                .map_err(|e| format!("Failed to read note {}: {}", i, e))?;

            // Add file to zip
            zip.start_file(format!("note_{}.md", i), options)
                .map_err(|e| format!("Failed to add note {} to backup: {}", i, e))?;

            zip.write_all(note_content.as_bytes())
                .map_err(|e| format!("Failed to write note {} content: {}", i, e))?;

            added_files += 1;
        }
    }

    // Add a metadata file with timestamp
    zip.start_file("backup_info.txt", options)
        .map_err(|e| format!("Failed to add metadata to backup: {}", e))?;

    let metadata = format!(
        "Backup created: {}\nTimestamp: {}\nFiles: {}",
        datetime, timestamp, added_files
    );

    zip.write_all(metadata.as_bytes())
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    // Finalize the zip file
    zip.finish()
        .map_err(|e| format!("Failed to finalize backup: {}", e))?;

    // Return the path to the backup file
    Ok(backup_path.to_string_lossy().to_string())
}

// Get a list of available backups
#[tauri::command]
pub fn list_backups(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let backups_dir = app_dir.join("backups");
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(backups_dir)
        .map_err(|e| format!("Failed to read backups directory: {}", e))?;

    let backups: Vec<String> = {
        let mut temp: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                path.is_file() && matches!(path.extension(), Some(ext) if ext == "zip")
            })
            .map(|entry| entry.path().to_string_lossy().to_string())
            .collect();

        temp.sort_by(|a, b| b.cmp(a));
        temp
    };

    Ok(backups)
}

// Restore from a backup file
#[tauri::command]
pub async fn restore_backup(app_handle: AppHandle, backup_path: String) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let backup_path = Path::new(&backup_path);
    if !backup_path.exists() {
        return Err(format!("Backup file not found: {}", backup_path.display()));
    }

    // Open the zip file
    let file =
        fs::File::open(backup_path).map_err(|e| format!("Failed to open backup file: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read backup archive: {}", e))?;

    // Extract each note file
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access backup file entry: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => {
                if path.to_string_lossy().ends_with(".md") {
                    app_dir.join(path)
                } else {
                    // Skip non-markdown files (like the metadata file)
                    continue;
                }
            }
            None => continue,
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
        }

        // Extract the file
        let mut outfile = fs::File::create(&outpath)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        io::copy(&mut file, &mut outfile)
            .map_err(|e| format!("Failed to copy file data: {}", e))?;

        // Emit an event to update the UI for this note
        if let Some(filename) = outpath.file_name() {
            if let Some(note_name) = filename.to_string_lossy().strip_prefix("note_") {
                if let Some(index_str) = note_name.strip_suffix(".md") {
                    if let Ok(index) = index_str.parse::<usize>() {
                        let outpath = crate::storage_service::get_note_path(&app_handle, index);
                        let content = fs::read_to_string(&outpath)
                            .map_err(|e| format!("Failed to read restored note: {}", e))?;

                        // Emit an event to update the UI
                        tauri::Emitter::emit(
                            &app_handle,
                            &format!("note-updated-{}", index),
                            content,
                        )
                        .map_err(|e| format!("Failed to emit update event: {}", e))?;
                    }
                }
            }
        }
    }

    // Return success
    Ok(())
}

// Delete a backup file
#[tauri::command]
pub fn delete_backup(backup_path: String) -> Result<(), String> {
    let path = Path::new(&backup_path);
    if !path.exists() {
        return Err(format!("Backup file not found: {}", path.display()));
    }

    fs::remove_file(path).map_err(|e| format!("Failed to delete backup: {}", e))?;

    Ok(())
}

// Count the number of backups
#[tauri::command]
pub fn count_backups(app_handle: AppHandle) -> Result<usize, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let backups_dir = app_dir.join("backups");
    if !backups_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(backups_dir)
        .map_err(|e| format!("Failed to read backups directory: {}", e))?;

    let count = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().is_file() && matches!(entry.path().extension(), Some(ext) if ext == "zip")
        })
        .count();

    Ok(count)
}

// Prune old backups, keeping only the specified number of recent backups
#[tauri::command]
pub fn prune_backups(app_handle: AppHandle, keep_count: usize) -> Result<usize, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let backups_dir = app_dir.join("backups");
    if !backups_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(&backups_dir)
        .map_err(|e| format!("Failed to read backups directory: {}", e))?;

    // Collect and sort backup files by modified time (newest first)
    let mut backups: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && matches!(path.extension(), Some(ext) if ext == "zip"))
        .collect();

    backups.sort_by(|a, b| {
        let a_time = fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());
        let b_time = fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());
        b_time.cmp(&a_time)
    });

    // Keep the specified number of recent backups, delete the rest
    let mut deleted_count = 0;
    if backups.len() > keep_count {
        for backup_path in backups.iter().skip(keep_count) {
            if fs::remove_file(backup_path).is_ok() {
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}
