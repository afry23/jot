use crate::credential_manager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Configuration for Nextcloud sync
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NextcloudConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub sync_folder: String,
    pub last_sync: Option<u64>,
    pub auto_sync: bool,
    pub sync_on_startup: bool,
    pub sync_interval_minutes: u32,
    pub show_sync_status: bool,
}

impl Default for NextcloudConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
            sync_folder: String::from("/jot"),
            last_sync: None,
            auto_sync: false,
            sync_on_startup: false,
            sync_interval_minutes: 30,
            show_sync_status: true,
        }
    }
}

impl NextcloudConfig {
    // Load sensitive data from keychain when needed
    pub fn load_credentials(&mut self) -> Result<(), String> {
        if !self.username.is_empty() {
            match credential_manager::get_nextcloud_credential(self.username.clone()) {
                Ok(password) => {
                    self.password = password;
                    Ok(())
                }
                Err(e) => {
                    // If it doesn't exist in keychain yet, that's ok
                    if e.contains("not found") {
                        Ok(())
                    } else {
                        log::error!("Error loading Nextcloud credentials: {}", e);
                        Err(e)
                    }
                }
            }
        } else {
            Ok(())
        }
    }

    // Save password to keychain
    pub fn save_credentials(&self) -> Result<(), String> {
        if !self.username.is_empty() && !self.password.is_empty() {
            credential_manager::store_nextcloud_credential(
                self.username.clone(),
                self.password.clone(),
            )
        } else {
            Ok(())
        }
    }
}

// Status of a note for sync purposes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoteStatus {
    pub tab_index: usize,
    pub local_modified: u64,
    pub remote_modified: Option<u64>,
    pub synced: bool,
    pub conflict: bool,
}

// Sync status information to return to frontend
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncStatus {
    pub last_sync: Option<u64>,
    pub syncing: bool,
    pub error: Option<String>,
    pub notes_status: HashMap<usize, NoteStatus>,
}
