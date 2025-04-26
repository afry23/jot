use crate::credential_manager;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager};

// Configuration for ChatGPT API
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatGPTConfig {
    #[serde(skip_serializing, skip_deserializing)]
    api_key: Option<String>,
    model: String,
    endpoint: String,
    max_tokens: u32,
    temperature: f32,
}

impl Default for ChatGPTConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: String::from("gpt-3.5-turbo"),
            endpoint: String::from("https://api.openai.com/v1/chat/completions"),
            max_tokens: 500,
            temperature: 0.7,
        }
    }
}

impl ChatGPTConfig {
    // Load API key from secure storage
    pub fn load_api_key(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        match credential_manager::get_chatgpt_credential(app_handle.clone()) {
            Ok(api_key) => {
                self.api_key = Some(api_key);
                Ok(())
            }
            Err(e) => {
                if e.contains("not found") {
                    // Not an error if key doesn't exist yet
                    Ok(())
                } else {
                    Err(format!("Failed to load ChatGPT API key: {}", e))
                }
            }
        }
    }
}

// ChatGPT message types
#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

// ChatGPT request structure
#[derive(Serialize, Deserialize, Debug)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

// ChatGPT response structures
#[derive(Serialize, Deserialize, Debug)]
struct ChatChoice {
    message: ChatMessage,
    finish_reason: String,
    index: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    usage: ChatUsage,
    choices: Vec<ChatChoice>,
}

// Get ChatGPT configuration from settings
fn get_chatgpt_config(app_handle: &AppHandle) -> ChatGPTConfig {
    // Get app data directory
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let config_path = app_dir.join("chatgpt_config.json");

    let mut config = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ChatGPTConfig::default(),
        }
    } else {
        ChatGPTConfig::default()
    };

    // Load API key from secure storage
    let _ = config.load_api_key(app_handle);

    config
}

// Save ChatGPT configuration
fn save_chatgpt_config(app_handle: &AppHandle, config: &ChatGPTConfig) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let config_path = app_dir.join("chatgpt_config.json");

    // Convert to JSON string (API key will be skipped due to serde annotations)
    let json_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Write to file
    std::fs::write(config_path, json_str).map_err(|e| format!("Failed to save config: {}", e))
}

#[command]
pub async fn chat_with_gpt(
    app_handle: AppHandle,
    prompt: String,
    system_message: String,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<serde_json::Value, String> {
    if prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    let mut config = get_chatgpt_config(&app_handle);

    // Load API key if not already loaded
    if config.api_key.is_none() {
        config.load_api_key(&app_handle)?;
    }

    // Ensure API key is set
    let api_key = match &config.api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => return Err("ChatGPT API key is not configured".to_string()),
    };

    let client = Client::new();

    // Build messages array
    let mut messages = Vec::new();

    // Add system message if provided
    if !system_message.is_empty() {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_message,
        });
    }

    // Add user message (the prompt)
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: prompt,
    });

    // Build request
    let request = ChatRequest {
        model: model.unwrap_or(config.model),
        messages,
        max_tokens: max_tokens.unwrap_or(config.max_tokens),
        temperature: temperature.unwrap_or(config.temperature),
    };

    // Make the API request
    let response = client
        .post(&config.endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("ChatGPT API request failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read API response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "ChatGPT API error: HTTP {} - {}",
            status, response_text
        ));
    }

    // Parse and return the response
    serde_json::from_str::<serde_json::Value>(&response_text)
        .map_err(|e| format!("Failed to parse API response: {}", e))
}

#[command]
pub fn get_chatgpt_config_command(app_handle: AppHandle) -> ChatGPTConfig {
    get_chatgpt_config(&app_handle)
}

#[command]
pub fn save_chatgpt_config_command(
    app_handle: AppHandle,
    api_key: Option<String>,
    model: Option<String>,
    endpoint: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<(), String> {
    let mut config = get_chatgpt_config(&app_handle);

    // Store API key in secure storage if provided
    if let Some(key) = api_key {
        if !key.is_empty() {
            credential_manager::store_chatgpt_credential(app_handle.clone(), key.clone())?;
            config.api_key = Some(key);
        }
    }

    // Update other config values
    if let Some(model_name) = model {
        config.model = model_name;
    }

    if let Some(url) = endpoint {
        config.endpoint = url;
    }

    if let Some(tokens) = max_tokens {
        config.max_tokens = tokens;
    }

    if let Some(temp) = temperature {
        config.temperature = temp;
    }

    // Save non-sensitive config to file
    save_chatgpt_config(&app_handle, &config)
}
