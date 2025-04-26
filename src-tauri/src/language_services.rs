use crate::credential_manager;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, AppHandle, Manager};

// Configuration for API keys
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LanguageConfig {
    #[serde(skip_serializing, skip_deserializing)]
    languagetool_api_key: Option<String>,
    languagetool_username: Option<String>,
    languagetool_endpoint: String,
    #[serde(skip_serializing, skip_deserializing)]
    deepl_api_key: Option<String>,
    deepl_endpoint: String,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            languagetool_api_key: None,
            languagetool_username: None,
            languagetool_endpoint: String::from("https://api.languagetool.org/v2/check"),
            deepl_api_key: None,
            deepl_endpoint: String::from("https://api-free.deepl.com/v2/translate"),
        }
    }
}

// LanguageTool Types
#[derive(Serialize, Deserialize, Debug)]
struct LanguageToolMatch {
    message: String,
    offset: usize,
    length: usize,
    replacements: Vec<LanguageToolReplacement>,
    rule: LanguageToolRule,
}

#[derive(Serialize, Deserialize, Debug)]
struct LanguageToolReplacement {
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LanguageToolRule {
    id: String,
    description: String,
    #[serde(rename = "issueType")]
    issue_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LanguageToolResponse {
    matches: Vec<LanguageToolMatch>,
}

// DeepL Types
#[derive(Serialize, Deserialize, Debug)]
struct DeepLRequest {
    text: Vec<String>,
    target_lang: String,
    source_lang: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeepLTranslation {
    detected_source_language: String,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

impl LanguageConfig {
    // Load credentials from secure storage
    pub fn load_credentials(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        // Load LanguageTool API key if username is set
        if let Some(username) = &self.languagetool_username {
            if !username.is_empty() {
                match credential_manager::get_languagetool_credential(username.clone()) {
                    Ok(api_key) => self.languagetool_api_key = Some(api_key),
                    Err(e) => {
                        if !e.contains("not found") {
                            log::warn!("Error loading LanguageTool API key: {}", e);
                        }
                    }
                }
            }
        }

        // Load DeepL API key
        match credential_manager::get_deepl_credential(app_handle.clone()) {
            Ok(api_key) => self.deepl_api_key = Some(api_key),
            Err(e) => {
                if !e.contains("not found") {
                    log::warn!("Error loading DeepL API key: {}", e);
                }
            }
        }

        Ok(())
    }
}

// Get language config from settings
fn get_language_config(app_handle: &AppHandle) -> LanguageConfig {
    // Get app data directory
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let config_path = app_dir.join("language_config.json");

    let mut config = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => LanguageConfig::default(),
        }
    } else {
        LanguageConfig::default()
    };

    // Load secure credentials
    let _ = config.load_credentials(app_handle);

    config
}

// Save language config
fn save_language_config(app_handle: &AppHandle, config: &LanguageConfig) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let config_path = app_dir.join("language_config.json");

    // Convert to JSON string
    let json_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Write to file
    std::fs::write(config_path, json_str).map_err(|e| format!("Failed to save config: {}", e))
}

#[command]
pub async fn check_grammar(
    app_handle: AppHandle,
    text: String,
    language: Option<String>,
) -> Result<serde_json::Value, String> {
    if text.trim().is_empty() {
        return Ok(serde_json::json!({ "matches": [] }));
    }

    let config = get_language_config(&app_handle);
    let client = Client::new();

    // Build request parameters - LanguageTool requires specific parameters
    let mut params = HashMap::new();
    params.insert("text", text);

    // Set language parameter - must be a valid language code
    let lang = match language {
        Some(lang) if !lang.is_empty() && lang != "auto" => lang,
        _ => "en-US".to_string(), // Default to English if auto
    };
    params.insert("language", lang);

    // Handle API authentication
    if let Some(api_key) = &config.languagetool_api_key {
        if !api_key.is_empty() {
            // For API key to work, username must be set or it must be used as a token
            if let Some(username) = &config.languagetool_username {
                if !username.is_empty() {
                    // Use username + apiKey for authentication
                    params.insert("username", username.clone());
                    params.insert("apiKey", api_key.clone());
                } else {
                    // Use apiKey as token for bearer authentication or remove it
                    // Adjust based on how your LanguageTool API expects authentication
                    // params.insert("token", api_key.clone());
                }
            } else {
                // If no username is set, don't use the API key to avoid errors
                // Try using without authentication (limited usage)
                log::info!("Warning: API key provided but no username. Using public API.");
            }
        }
    }

    // Add required parameters
    params.insert("enabledOnly", "false".to_string());

    // Make the API request
    let response = client
        .post(&config.languagetool_endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("LanguageTool API request failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read API response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "LanguageTool API error: HTTP {} - {}",
            status, response_text
        ));
    }

    // Parse and return the response
    serde_json::from_str::<serde_json::Value>(&response_text)
        .map_err(|e| format!("Failed to parse API response: {}", e))
}

#[command]
pub async fn translate_text(
    app_handle: AppHandle,
    text: String,
    target_lang: String,
    source_lang: Option<String>,
) -> Result<serde_json::Value, String> {
    if text.trim().is_empty() {
        return Ok(serde_json::json!({ "translations": [] }));
    }

    let config = get_language_config(&app_handle);

    // Ensure we have an API key for DeepL
    let api_key = match &config.deepl_api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => return Err("DeepL API key is not configured".to_string()),
    };

    let client = Client::new();

    // Build request body
    let request = DeepLRequest {
        text: vec![text],
        target_lang,
        source_lang,
    };

    // Make the API request
    let response = client
        .post(&config.deepl_endpoint)
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("DeepL API request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("DeepL API error: HTTP {}", status));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read API response: {}", e))?;

    // Parse and return the response
    serde_json::from_str::<serde_json::Value>(&response_text)
        .map_err(|e| format!("Failed to parse API response: {}", e))
}

#[command]
pub fn save_language_tool_config(
    app_handle: AppHandle,
    api_key: Option<String>,
    username: Option<String>,
    endpoint: Option<String>,
) -> Result<(), String> {
    let mut config = get_language_config(&app_handle);

    // Update non-sensitive config values
    if let Some(user) = username {
        config.languagetool_username = Some(user.clone());

        // If username and API key are provided, save API key securely
        if let Some(key) = &api_key {
            if !key.is_empty() && !user.is_empty() {
                credential_manager::store_languagetool_credential(user, key.clone())?;
            }
        }
    }

    if let Some(url) = endpoint {
        config.languagetool_endpoint = url;
    }

    // Save non-sensitive config to file
    save_language_config(&app_handle, &config)
}

#[command]
pub fn save_deepl_config(
    app_handle: AppHandle,
    api_key: Option<String>,
    endpoint: Option<String>,
) -> Result<(), String> {
    let mut config = get_language_config(&app_handle);

    // Store API key securely if provided
    if let Some(key) = api_key {
        if !key.is_empty() {
            credential_manager::store_deepl_credential(app_handle.clone(), key)?;
        }
    }

    if let Some(url) = endpoint {
        config.deepl_endpoint = url;
    }

    // Save non-sensitive config to file
    save_language_config(&app_handle, &config)
}

#[command]
pub fn get_language_services_config(app_handle: AppHandle) -> LanguageConfig {
    get_language_config(&app_handle)
}
