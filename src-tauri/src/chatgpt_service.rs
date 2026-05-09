use crate::credential_manager;
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

const OPENAI_DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const CHATGPT_SERVICE: &str = "jot.chatgpt";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatGPTConfig {
    pub endpoint: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub provider: String,
    pub default_rewrite_style: String,
    pub api_key_stored: bool,
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
pub fn get_chatgpt_config(app_handle: AppHandle) -> Result<ChatGPTConfig, String> {
    let settings = read_settings(&app_handle);
    let app_id = app_handle.config().identifier.clone();
    let api_key_stored = credential_manager::get_credential(CHATGPT_SERVICE, &app_id).is_ok();

    Ok(ChatGPTConfig {
        endpoint: settings["gpt_endpoint"]
            .as_str()
            .unwrap_or(OPENAI_DEFAULT_ENDPOINT)
            .to_string(),
        model: settings["gpt_model"]
            .as_str()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        max_tokens: settings["gpt_max_tokens"].as_u64().unwrap_or(1000) as u32,
        temperature: settings["gpt_temperature"].as_f64().unwrap_or(0.7) as f32,
        provider: settings["gpt_provider"]
            .as_str()
            .unwrap_or("openai")
            .to_string(),
        default_rewrite_style: settings["gpt_default_style"]
            .as_str()
            .unwrap_or("formal_sie")
            .to_string(),
        api_key_stored,
    })
}

#[command]
pub fn save_chatgpt_config(
    app_handle: AppHandle,
    endpoint: Option<String>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider: Option<String>,
    default_rewrite_style: Option<String>,
    api_key: Option<String>,
) -> Result<(), String> {
    let mut settings = read_settings(&app_handle);

    if let Some(ref ep) = endpoint {
        settings["gpt_endpoint"] = serde_json::json!(ep);
    }
    if let Some(ref m) = model {
        settings["gpt_model"] = serde_json::json!(m);
    }
    if let Some(mt) = max_tokens {
        settings["gpt_max_tokens"] = serde_json::json!(mt);
    }
    if let Some(t) = temperature {
        settings["gpt_temperature"] = serde_json::json!(t);
    }
    if let Some(ref p) = provider {
        settings["gpt_provider"] = serde_json::json!(p);
    }
    if let Some(ref s) = default_rewrite_style {
        settings["gpt_default_style"] = serde_json::json!(s);
    }

    write_settings(&app_handle, &settings)?;

    if let Some(ref key) = api_key {
        let app_id = app_handle.config().identifier.clone();
        credential_manager::store_credential(CHATGPT_SERVICE, &app_id, key)?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[command]
pub async fn chat_with_gpt(
    app_handle: AppHandle,
    prompt: String,
    system_message: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
) -> Result<String, String> {
    let settings = read_settings(&app_handle);
    let endpoint = settings["gpt_endpoint"]
        .as_str()
        .unwrap_or(OPENAI_DEFAULT_ENDPOINT)
        .to_string();
    let provider = settings["gpt_provider"]
        .as_str()
        .unwrap_or("openai")
        .to_string();

    let client = Client::new();
    let mut request_builder = client
        .post(&endpoint)
        .header("Content-Type", "application/json");

    if provider != "ollama" {
        let app_id = app_handle.config().identifier.clone();
        match credential_manager::get_credential(CHATGPT_SERVICE, &app_id) {
            Ok(key) => {
                request_builder =
                    request_builder.header("Authorization", format!("Bearer {}", key));
            }
            Err(_) => {
                return Err(
                    "Kein API-Key konfiguriert. Bitte API-Key in Einstellungen → AI eintragen."
                        .to_string(),
                );
            }
        }
    }

    let body = ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_message,
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        max_tokens,
        temperature,
    };

    debug!("AI request → {}", endpoint);

    let response = request_builder
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anfrage fehlgeschlagen: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        error!("API error {}: {}", status, body_text);
        return Err(format!("API-Fehler {}: {}", status, body_text));
    }

    let result: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Antwort konnte nicht gelesen werden: {}", e))?;

    let content = result
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "Leere Antwort von der API".to_string())?;

    debug!("AI response ({} chars)", content.len());

    Ok(content)
}
