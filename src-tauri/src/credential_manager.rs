#[cfg(not(target_os = "android"))]
use keyring::Entry;
use log::debug;
use tauri::{command, AppHandle};

// Service names for different credential types
const LANGUAGETOOL_SERVICE: &str = "jot.languagetool";
const CHATGPT_SERVICE: &str = "jot.chatgpt";

// Android: kein keyring-Store verfügbar → tauri-plugin-secure-storage (Android Keystore, AES-GCM)
#[cfg(target_os = "android")]
mod store {
    use super::*;
    use tauri_plugin_secure_storage::{OptionsRequest, SecureStorageExt};

    fn request(service: &str, username: &str, data: Option<String>) -> OptionsRequest {
        OptionsRequest {
            prefixed_key: Some(format!("{}/{}", service, username)),
            data,
            sync: None,
            keychain_access: None,
        }
    }

    pub fn set(app: &AppHandle, service: &str, username: &str, password: &str) -> Result<(), String> {
        app.secure_storage()
            .set_item(app.clone(), request(service, username, Some(password.to_string())))
            .map(|_| ())
            .map_err(|e| format!("Failed to store credential: {}", e))
    }

    pub fn get(app: &AppHandle, service: &str, username: &str) -> Result<String, String> {
        app.secure_storage()
            .get_item(app.clone(), request(service, username, None))
            .map_err(|e| format!("Failed to retrieve credential: {}", e))?
            .data
            .ok_or_else(|| "Failed to retrieve credential: no entry".to_string())
    }

    pub fn delete(app: &AppHandle, service: &str, username: &str) -> Result<(), String> {
        app.secure_storage()
            .remove_item(app.clone(), request(service, username, None))
            .map(|_| ())
            .map_err(|e| format!("Failed to delete credential: {}", e))
    }
}

// Desktop: System-Keychain via keyring
#[cfg(not(target_os = "android"))]
mod store {
    use super::*;

    fn create_entry(service: &str, username: &str) -> Result<Entry, String> {
        Entry::new(service, username).map_err(|e| format!("Keyring error: {}", e))
    }

    pub fn set(_app: &AppHandle, service: &str, username: &str, password: &str) -> Result<(), String> {
        let entry = create_entry(service, username)?;
        entry
            .set_password(password)
            .map_err(|e| format!("Failed to store credential: {}", e))?;
        // Verification step: immediately try to retrieve it
        match entry.get_password() {
            Ok(retrieved) if retrieved == password => Ok(()),
            Ok(_) => Err("Password was stored but verification failed".to_string()),
            Err(e) => Err(format!(
                "Password was seemingly stored but couldn't be retrieved: {}",
                e
            )),
        }
    }

    pub fn get(_app: &AppHandle, service: &str, username: &str) -> Result<String, String> {
        create_entry(service, username)?
            .get_password()
            .map_err(|e| format!("Failed to retrieve credential: {}", e))
    }

    pub fn delete(_app: &AppHandle, service: &str, username: &str) -> Result<(), String> {
        create_entry(service, username)?
            .delete_credential()
            .map_err(|e| format!("Failed to delete credential: {}", e))
    }
}

// Store a credential in the system keychain
pub fn store_credential(
    app: &AppHandle,
    service: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    debug!("Storing credential for service: {}, username: {}", service, username);
    store::set(app, service, username, password)
}

// Retrieve a credential from the system keychain
pub fn get_credential(app: &AppHandle, service: &str, username: &str) -> Result<String, String> {
    debug!("Retrieving credential for service: {}, username: {}", service, username);
    store::get(app, service, username)
}

// Delete a credential from the system keychain
#[allow(dead_code)]
pub fn delete_credential(app: &AppHandle, service: &str, username: &str) -> Result<(), String> {
    debug!("Deleting credential for service: {}, username: {}", service, username);
    store::delete(app, service, username)
}

// Tauri commands for frontend interaction
#[command]
pub fn store_languagetool_credential(app_handle: AppHandle, username: String, api_key: String) -> Result<(), String> {
    store_credential(&app_handle, LANGUAGETOOL_SERVICE, &username, &api_key)
}

#[command]
pub fn get_languagetool_credential(app_handle: AppHandle, username: String) -> Result<String, String> {
    get_credential(&app_handle, LANGUAGETOOL_SERVICE, &username)
}

#[command]
pub fn has_languagetool_credential(app_handle: AppHandle, username: String) -> bool {
    match get_credential(&app_handle, LANGUAGETOOL_SERVICE, &username) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[command]
pub fn store_chatgpt_credential(app_handle: AppHandle, api_key: String) -> Result<(), String> {
    let app_id = app_handle.config().identifier.clone();
    store_credential(&app_handle, CHATGPT_SERVICE, &app_id, &api_key)
}

#[command]
pub fn get_chatgpt_credential(app_handle: AppHandle) -> Result<String, String> {
    let app_id = app_handle.config().identifier.clone();
    get_credential(&app_handle, CHATGPT_SERVICE, &app_id)
}

#[command]
pub fn has_chatgpt_credential(app_handle: AppHandle) -> bool {
    let app_id = app_handle.config().identifier.clone();
    match get_credential(&app_handle, CHATGPT_SERVICE, &app_id) {
        Ok(_) => true,
        Err(_) => false,
    }
}
