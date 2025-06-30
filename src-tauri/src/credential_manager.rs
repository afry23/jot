use keyring::Entry;
use log::debug;
use tauri::{command, AppHandle};

// Service names for different credential types
const NEXTCLOUD_SERVICE: &str = "jot.nextcloud";
const LANGUAGETOOL_SERVICE: &str = "jot.languagetool";
const DEEPL_SERVICE: &str = "jot.deepl";
const CHATGPT_SERVICE: &str = "jot.chatgpt";

// Helper function to create a keyring entry with consistent naming
fn create_entry(service: &str, username: &str) -> Result<Entry, String> {
    Entry::new(service, username).map_err(|e| format!("Keyring error: {}", e))
}

// Store a credential in the system keychain
pub fn store_credential(service: &str, username: &str, password: &str) -> Result<(), String> {
    debug!(
        "Storing credential for service: {}, username: {}",
        service, username
    );
    let entry = create_entry(service, username)?;
    match entry.set_password(password) {
        Ok(_) => {
            // Verification step: immediately try to retrieve it
            match entry.get_password() {
                Ok(retrieved) => {
                    if retrieved == password {
                        debug!(
                            "Successfully verified credential storage for {}/{}",
                            service, username
                        );
                        Ok(())
                    } else {
                        Err("Password was stored but verification failed".to_string())
                    }
                }
                Err(e) => Err(format!(
                    "Password was seemingly stored but couldn't be retrieved: {}",
                    e
                )),
            }
        }
        Err(e) => Err(format!("Failed to store credential: {}", e)),
    }
}

// Retrieve a credential from the system keychain
pub fn get_credential(service: &str, username: &str) -> Result<String, String> {
    debug!(
        "Retrieving credential for service: {}, username: {}",
        service, username
    );

    let entry = create_entry(service, username)?;

    match entry.get_password() {
        Ok(password) => {
            debug!(
                "Successfully retrieved credential for {}/{}",
                service, username
            );
            Ok(password)
        }
        Err(e) => Err(format!("Failed to retrieve credential: {}", e)),
    }
}

// Delete a credential from the system keychain
pub fn delete_credential(service: &str, username: &str) -> Result<(), String> {
    debug!(
        "Deleting credential for service: {}, username: {}",
        service, username
    );

    let entry = create_entry(service, username)?;

    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete credential: {}", e))
}

// Tauri commands for frontend interaction
#[command]
pub fn store_nextcloud_credential(username: String, password: String) -> Result<(), String> {
    // Check if the password is empty
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }
    store_credential(NEXTCLOUD_SERVICE, &username, &password)
}

#[command]
pub fn get_nextcloud_credential(username: String) -> Result<String, String> {
    get_credential(NEXTCLOUD_SERVICE, &username)
}

#[command]
pub fn delete_nextcloud_credential(username: String) -> Result<(), String> {
    delete_credential(NEXTCLOUD_SERVICE, &username)
}

#[command]
pub fn store_languagetool_credential(username: String, api_key: String) -> Result<(), String> {
    store_credential(LANGUAGETOOL_SERVICE, &username, &api_key)
}

#[command]
pub fn get_languagetool_credential(username: String) -> Result<String, String> {
    get_credential(LANGUAGETOOL_SERVICE, &username)
}

#[command]
pub fn has_languagetool_credential(username: String) -> bool {
    match get_credential(LANGUAGETOOL_SERVICE, &username) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[command]
pub fn store_deepl_credential(app_handle: AppHandle, api_key: String) -> Result<(), String> {
    // For services that don't have a username, we use a consistent identifier
    // Including the app handle info to make it unique per installation
    let app_id = app_handle.config().identifier.clone();
    store_credential(DEEPL_SERVICE, &app_id, &api_key)
}

#[command]
pub fn get_deepl_credential(app_handle: AppHandle) -> Result<String, String> {
    let app_id = app_handle.config().identifier.clone();
    get_credential(DEEPL_SERVICE, &app_id)
}

#[command]
pub fn has_deepl_credential(app_handle: AppHandle) -> bool {
    let app_id = app_handle.config().identifier.clone();
    match get_credential(DEEPL_SERVICE, &app_id) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[command]
pub fn store_chatgpt_credential(app_handle: AppHandle, api_key: String) -> Result<(), String> {
    let app_id = app_handle.config().identifier.clone();
    store_credential(CHATGPT_SERVICE, &app_id, &api_key)
}

#[command]
pub fn get_chatgpt_credential(app_handle: AppHandle) -> Result<String, String> {
    let app_id = app_handle.config().identifier.clone();
    get_credential(CHATGPT_SERVICE, &app_id)
}

#[command]
pub fn has_chatgpt_credential(app_handle: AppHandle) -> bool {
    let app_id = app_handle.config().identifier.clone();
    match get_credential(CHATGPT_SERVICE, &app_id) {
        Ok(_) => true,
        Err(_) => false,
    }
}
