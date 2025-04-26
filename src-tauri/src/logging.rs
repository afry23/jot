// src/logging.rs
use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    None = 5,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::None => LevelFilter::Off,
        }
    }
}

impl From<LevelFilter> for LogLevel {
    fn from(filter: LevelFilter) -> Self {
        match filter {
            LevelFilter::Trace => LogLevel::Trace,
            LevelFilter::Debug => LogLevel::Debug,
            LevelFilter::Info => LogLevel::Info,
            LevelFilter::Warn => LogLevel::Warn,
            LevelFilter::Error => LogLevel::Error,
            LevelFilter::Off => LogLevel::None,
        }
    }
}

// Custom logger structure
#[derive(Debug)]
pub struct FileLogger {
    file: Arc<Mutex<File>>,
    level: AtomicUsize,
}

impl FileLogger {
    pub fn new(log_path: PathBuf, level: LogLevel) -> Result<Self, std::io::Error> {
        // Create directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            create_dir_all(parent)?;
        }

        // Open log file with append mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(FileLogger {
            file: Arc::new(Mutex::new(file)),
            level: AtomicUsize::new(level as usize),
        })
    }

    pub fn set_level(&self, level: LogLevel) {
        self.level.store(level as usize, Ordering::SeqCst);
        log::set_max_level(level.into());
    }

    // Get the current log level
    pub fn get_level(&self) -> LogLevel {
        let level_usize = self.level.load(Ordering::SeqCst);
        match level_usize {
            0 => LogLevel::Trace,
            1 => LogLevel::Debug,
            2 => LogLevel::Info,
            3 => LogLevel::Warn,
            4 => LogLevel::Error,
            5 => LogLevel::None,
            _ => LogLevel::Info, // Default to INFO for unexpected values
        }
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let current_level: LogLevel = self.get_level();
        let current_level_filter: LevelFilter = current_level.into();
        metadata.level() <= current_level_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
            let log_message = format!(
                "[{}] [{}] [{}] {}\n",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            );

            if let Ok(mut file) = self.file.lock() {
                let _ = file.write_all(log_message.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

// Initialize the logger
pub static LOGGER: once_cell::sync::OnceCell<Arc<FileLogger>> = once_cell::sync::OnceCell::new();

// Initialize the logger
pub fn init_logger(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let log_dir = app_dir.join("logs");
    create_dir_all(&log_dir)?;

    let now = Local::now();
    let log_filename = format!("jot_{}.log", now.format("%Y%m%d"));
    let log_path = log_dir.join(log_filename);

    // Default to INFO level
    let logger = FileLogger::new(log_path.clone(), LogLevel::Info)?;
    let logger_arc = Arc::new(logger);

    // Store in global static
    LOGGER
        .set(logger_arc.clone())
        .expect("Failed to set global logger");

    log::set_boxed_logger(Box::new(logger_arc.clone()))?;
    log::set_max_level(LevelFilter::Info); // Default to Info

    // Log startup information
    log::info!("Jot application started");
    log::info!("Log file: {}", log_path.display());

    Ok(())
}

// Helper function to get the log directory
pub fn get_log_dir(app_handle: &AppHandle) -> PathBuf {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");
    app_dir.join("logs")
}

// Helper function to get all log files
pub fn get_log_files(app_handle: &AppHandle) -> Vec<PathBuf> {
    let log_dir = get_log_dir(app_handle);

    if !log_dir.exists() {
        return Vec::new();
    }

    let entries = std::fs::read_dir(log_dir).expect("Failed to read log directory");

    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            if let Ok(file_type) = entry.file_type() {
                file_type.is_file() && entry.path().extension().unwrap_or_default() == "log"
            } else {
                false
            }
        })
        .map(|entry| entry.path())
        .collect()
}

// Tauri command to get log content
#[tauri::command]
pub fn get_latest_logs(app_handle: AppHandle, max_lines: Option<usize>) -> Result<String, String> {
    let log_files = get_log_files(&app_handle);

    if log_files.is_empty() {
        return Ok("No logs found".to_string());
    }

    // Find the most recent log file (should be the latest by filename)
    let latest_log = log_files
        .iter()
        .max_by(|a, b| {
            std::fs::metadata(a)
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&std::fs::metadata(b).unwrap().modified().unwrap())
        })
        .ok_or_else(|| "Cannot find the latest log file".to_string())?;

    // Read the log file
    let content = std::fs::read_to_string(latest_log)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    // Return the last N lines if specified
    if let Some(max) = max_lines {
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > max {
            lines.len() - max
        } else {
            0
        };
        Ok(lines[start..].join("\n"))
    } else {
        Ok(content)
    }
}

// Tauri command to get all log files
#[tauri::command]
pub fn list_log_files(app_handle: AppHandle) -> Result<Vec<String>, String> {
    let log_files = get_log_files(&app_handle);

    Ok(log_files
        .iter()
        .filter_map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect())
}

#[tauri::command]
pub fn log_from_frontend(app_handle: AppHandle, logs: Vec<String>) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let log_dir = app_dir.join("logs");
    let frontend_log_path = log_dir.join("frontend.log");

    // Ensure log directory exists
    if let Err(e) = create_dir_all(&log_dir) {
        return Err(format!("Failed to create log directory: {}", e));
    }

    // Append logs to the frontend log file
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&frontend_log_path)
    {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open log file: {}", e)),
    };

    for log in logs {
        if let Err(e) = writeln!(file, "{}", log) {
            return Err(format!("Failed to write log: {}", e));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn calculate_log_size(app_handle: AppHandle) -> Result<u64, String> {
    let log_files = get_log_files(&app_handle);

    let total_size = log_files
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok())
        .map(|metadata| metadata.len())
        .sum();

    Ok(total_size)
}

#[tauri::command]
pub fn clear_logs(app_handle: AppHandle) -> Result<(), String> {
    let log_files = get_log_files(&app_handle);

    for log_file in log_files {
        if let Err(e) = std::fs::remove_file(&log_file) {
            return Err(format!(
                "Failed to remove log file {}: {}",
                log_file.display(),
                e
            ));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_log_level() -> LogLevel {
    if let Some(logger) = LOGGER.get() {
        logger.get_level()
    } else {
        LogLevel::Info // Default if logger not initialized
    }
}

// Tauri command to set log level
#[tauri::command]
pub fn set_log_level(level: LogLevel) -> bool {
    if let Some(logger) = LOGGER.get() {
        logger.set_level(level);
        log::info!("Log level set to: {:?}", level);
        true
    } else {
        false
    }
}
