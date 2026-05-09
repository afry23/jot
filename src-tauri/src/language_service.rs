use crate::credential_manager;
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

const LT_DEFAULT_ENDPOINT: &str = "https://api.languagetoolplus.com/v2/check";
const LANGUAGETOOL_SERVICE: &str = "jot.languagetool";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LtReplacement {
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LtRule {
    pub id: String,
    pub description: String,
    #[serde(rename = "issueType")]
    pub issue_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LtMatch {
    pub message: String,
    pub offset: usize,
    pub length: usize,
    pub replacements: Vec<LtReplacement>,
    pub rule: LtRule,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrammarCheckResult {
    pub matches: Vec<LtMatch>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LanguageServicesConfig {
    pub languagetool_endpoint: String,
    pub languagetool_username: Option<String>,
    pub languagetool_api_key: Option<String>,
}

fn get_settings_path(app_handle: &AppHandle) -> PathBuf {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
    }
    app_dir.join("settings.json")
}

fn read_settings(app_handle: &AppHandle) -> serde_json::Value {
    let path = get_settings_path(app_handle);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str(&content) {
                return json;
            }
        }
    }
    serde_json::json!({})
}

fn write_settings(app_handle: &AppHandle, settings: &serde_json::Value) -> Result<(), String> {
    let path = get_settings_path(app_handle);
    let json_str = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(path, json_str).map_err(|e| format!("Failed to write settings: {}", e))
}

#[command]
pub fn get_language_services_config(
    app_handle: AppHandle,
) -> Result<LanguageServicesConfig, String> {
    let settings = read_settings(&app_handle);
    let endpoint = settings["lt_endpoint"]
        .as_str()
        .unwrap_or(LT_DEFAULT_ENDPOINT)
        .to_string();
    let username = settings["lt_username"].as_str().map(|s| s.to_string());

    // Indicate if API key is stored (without exposing the key itself)
    let has_key = username.as_deref().map_or(false, |uname| {
        credential_manager::get_credential(LANGUAGETOOL_SERVICE, uname).is_ok()
    });

    Ok(LanguageServicesConfig {
        languagetool_endpoint: endpoint,
        languagetool_username: username,
        languagetool_api_key: if has_key {
            Some("stored".to_string())
        } else {
            None
        },
    })
}

#[command]
pub fn save_language_tool_config(
    app_handle: AppHandle,
    username: Option<String>,
    api_key: Option<String>,
    endpoint: Option<String>,
) -> Result<(), String> {
    let mut settings = read_settings(&app_handle);

    if let Some(ref ep) = endpoint {
        settings["lt_endpoint"] = serde_json::json!(ep);
    }
    if let Some(ref uname) = username {
        settings["lt_username"] = serde_json::json!(uname);
    }

    write_settings(&app_handle, &settings)?;

    if let (Some(ref uname), Some(ref key)) = (&username, &api_key) {
        credential_manager::store_credential(LANGUAGETOOL_SERVICE, uname, key)?;
    }

    Ok(())
}

#[command]
pub async fn check_grammar(
    app_handle: AppHandle,
    text: String,
    language: String,
) -> Result<GrammarCheckResult, String> {
    let settings = read_settings(&app_handle);
    let endpoint = settings["lt_endpoint"]
        .as_str()
        .unwrap_or(LT_DEFAULT_ENDPOINT)
        .to_string();
    let username = settings["lt_username"].as_str().map(|s| s.to_string());

    let client = Client::new();
    let mut params: Vec<(String, String)> = vec![
        ("text".to_string(), text),
        ("language".to_string(), language),
    ];

    if let Some(ref uname) = username {
        if let Ok(api_key) = credential_manager::get_credential(LANGUAGETOOL_SERVICE, uname) {
            params.push(("username".to_string(), uname.clone()));
            params.push(("apiKey".to_string(), api_key));
        }
    }

    debug!("Grammar check via {}", endpoint);

    let response = client
        .post(&endpoint)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("LanguageTool API error {}: {}", status, body);
        return Err(format!("API error {}: {}", status, body));
    }

    let result: GrammarCheckResult = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    debug!("Grammar check: {} matches", result.matches.len());

    Ok(result)
}
