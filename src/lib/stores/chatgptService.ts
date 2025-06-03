// $lib/stores/chatgptService.ts
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

// Types for ChatGPT integration
export interface ChatGPTConfig {
  api_key: string | null;
  model: string;
  endpoint: string;
  max_tokens: number;
  temperature: number;
}

export interface ChatGPTRequest {
  messages: ChatMessage[];
  model: string;
  max_tokens: number;
  temperature: number;
}

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface ChatGPTResponse {
  choices: {
    message: {
      content: string;
      role: string;
    };
  }[];
  usage: {
    total_tokens: number;
  };
}

// Default configuration
const defaultConfig: ChatGPTConfig = {
  api_key: null,
  model: "gpt-3.5-turbo",
  endpoint: "https://api.openai.com/v1/chat/completions",
  max_tokens: 500,
  temperature: 0.7,
};

// Store for ChatGPT configuration
export const chatGPTConfig = writable<ChatGPTConfig>(defaultConfig);

// Store for current response and state
export const chatGPTResponse = writable<string>("");
export const isProcessing = writable<boolean>(false);
export const chatGPTError = writable<string | null>(null);

// Load ChatGPT configuration
export async function loadChatGPTConfig(): Promise<void> {
  try {
    const config = await invoke<ChatGPTConfig>("get_chatgpt_config_command");
    chatGPTConfig.set(config);
    return;
  } catch (error) {
    console.error("Failed to load ChatGPT config:", error);
  }
}

// Save ChatGPT configuration
export async function saveChatGPTConfig(
  apiKey?: string,
  model?: string,
  endpoint?: string,
  maxTokens?: number,
  temperature?: number
): Promise<boolean> {
  try {
    const currentConfig = get(chatGPTConfig);

    await invoke("save_chatgpt_config_command", {
      apiKey: apiKey,
      model: model,
      endpoint: endpoint,
      maxTokens: maxTokens,
      temperature: temperature,
    });

    // Update local store
    chatGPTConfig.update((config) => ({
      ...config,
      model: model ?? config.model,
      endpoint: endpoint ?? config.endpoint,
      max_tokens: maxTokens ?? config.max_tokens,
      temperature: temperature ?? config.temperature,
    }));

    return true;
  } catch (error) {
    console.error("Failed to save ChatGPT config:", error);
    return false;
  }
}

// Send request to ChatGPT
export async function sendToChatGPT(
  prompt: string,
  action: "summarize" | "chat" = "chat"
): Promise<string> {
  if (!prompt.trim()) {
    return "";
  }

  const config = get(chatGPTConfig);

  isProcessing.set(true);
  chatGPTError.set(null);
  chatGPTResponse.set("");

  try {
    // Customize system message based on action
    let systemMessage = "";
    if (action === "summarize") {
      systemMessage = "Summarize the following text concisely:";
    } else {
      systemMessage = "You are a helpful assistant.";
    }

    const response = await invoke<ChatGPTResponse>("chat_with_gpt", {
      prompt,
      systemMessage,
      model: config.model,
      maxTokens: config.max_tokens,
      temperature: config.temperature,
    });

    if (response.choices && response.choices.length > 0) {
      const responseText = response.choices[0].message.content;
      chatGPTResponse.set(responseText);
      return responseText;
    }

    return "";
  } catch (error) {
    const errorMessage = `ChatGPT request failed: ${error}`;
    console.error(errorMessage);
    chatGPTError.set(errorMessage);
    return "";
  } finally {
    isProcessing.set(false);
  }
}

// Summarize text with ChatGPT
export async function summarizeWithChatGPT(text: string): Promise<string> {
  return sendToChatGPT(text, "summarize");
}

// Clear ChatGPT response and error
export function clearChatGPTState(): void {
  chatGPTResponse.set("");
  chatGPTError.set(null);
}
