import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface ChatGPTConfig {
  model: string;
  endpoint: string;
  max_tokens: number;
  temperature: number;
  provider: string;
  default_rewrite_style: string;
  api_key_stored: boolean;
}

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export const REWRITE_STYLES: Record<
  string,
  { label: string; systemPrompt: string }
> = {
  formal_sie: {
    label: "Formal (Sie)",
    systemPrompt:
      "Formuliere den folgenden Text formell um und verwende 'Sie' als Anrede. Gib ausschließlich den umformulierten Text zurück, ohne Erklärungen oder Kommentare.",
  },
  friendly_du: {
    label: "Freundlich (du)",
    systemPrompt:
      "Formuliere den folgenden Text freundlich und persönlich um, verwende 'du' als Anrede. Gib ausschließlich den umformulierten Text zurück.",
  },
  kollegial: {
    label: "Kollegial",
    systemPrompt:
      "Formuliere den folgenden Text in einem kollegialen, professionellen Arbeitsumgangston um. Gib ausschließlich den umformulierten Text zurück.",
  },
  professionell: {
    label: "Professionell",
    systemPrompt:
      "Formuliere den folgenden Text in einem klaren, sachlichen und professionellen Ton um. Gib ausschließlich den umformulierten Text zurück.",
  },
  praegnant: {
    label: "Prägnant",
    systemPrompt:
      "Verdichte den folgenden Text auf das Wesentliche. Kürze ohne Inhaltsverlust. Gib ausschließlich den gekürzten Text zurück.",
  },
  custom: {
    label: "Eigene Anweisung",
    systemPrompt: "",
  },
};

export const TRANSLATE_LANGUAGES: Record<string, string> = {
  de: "Deutsch",
  en: "Englisch",
  fr: "Französisch",
  es: "Spanisch",
  it: "Italienisch",
  pt: "Portugiesisch",
  nl: "Niederländisch",
  pl: "Polnisch",
};

const defaultConfig: ChatGPTConfig = {
  model: "gpt-4o-mini",
  endpoint: "https://api.openai.com/v1/chat/completions",
  max_tokens: 1000,
  temperature: 0.7,
  provider: "openai",
  default_rewrite_style: "formal_sie",
  api_key_stored: false,
};

export const chatGPTConfig = writable<ChatGPTConfig>(defaultConfig);
export const isProcessing = writable<boolean>(false);
export const chatGPTError = writable<string | null>(null);

export async function loadChatGPTConfig(): Promise<void> {
  try {
    const config = await invoke<ChatGPTConfig>("get_chatgpt_config");
    chatGPTConfig.set(config);
  } catch (error) {
    console.error("Failed to load ChatGPT config:", error);
  }
}

export async function saveChatGPTConfig(options: {
  apiKey?: string;
  model?: string;
  endpoint?: string;
  maxTokens?: number;
  temperature?: number;
  provider?: string;
  defaultRewriteStyle?: string;
}): Promise<boolean> {
  try {
    await invoke("save_chatgpt_config", {
      apiKey: options.apiKey ?? null,
      model: options.model ?? null,
      endpoint: options.endpoint ?? null,
      maxTokens: options.maxTokens ?? null,
      temperature: options.temperature ?? null,
      provider: options.provider ?? null,
      defaultRewriteStyle: options.defaultRewriteStyle ?? null,
    });

    chatGPTConfig.update((config) => ({
      ...config,
      model: options.model ?? config.model,
      endpoint: options.endpoint ?? config.endpoint,
      max_tokens: options.maxTokens ?? config.max_tokens,
      temperature: options.temperature ?? config.temperature,
      provider: options.provider ?? config.provider,
      default_rewrite_style:
        options.defaultRewriteStyle ?? config.default_rewrite_style,
      api_key_stored: options.apiKey ? true : config.api_key_stored,
    }));

    return true;
  } catch (error) {
    console.error("Failed to save ChatGPT config:", error);
    return false;
  }
}

export async function processText(
  selectedText: string,
  action: "rewrite" | "expand" | "translate",
  options: { styleKey?: string; customPrompt?: string; targetLang?: string } = {}
): Promise<string> {
  const config = get(chatGPTConfig);

  isProcessing.set(true);
  chatGPTError.set(null);

  try {
    let systemMessage: string;

    if (action === "rewrite") {
      const styleKey = options.styleKey ?? config.default_rewrite_style;
      if (styleKey === "custom" && options.customPrompt) {
        systemMessage = `${options.customPrompt} Gib ausschließlich das Ergebnis zurück, ohne Erklärungen.`;
      } else {
        systemMessage =
          REWRITE_STYLES[styleKey]?.systemPrompt ??
          REWRITE_STYLES["formal_sie"].systemPrompt;
      }
    } else if (action === "expand") {
      systemMessage =
        "Erweitere den folgenden Text mit mehr Details, Kontext und Ausführlichkeit. Behalte den ursprünglichen Ton und Stil bei. Gib ausschließlich den erweiterten Text zurück.";
    } else {
      const lang = TRANSLATE_LANGUAGES[options.targetLang ?? "en"] ?? "Englisch";
      systemMessage = `Übersetze den folgenden Text ins ${lang}. Gib ausschließlich die Übersetzung zurück, ohne Erklärungen.`;
    }

    const result = await invoke<string>("chat_with_gpt", {
      prompt: selectedText,
      systemMessage,
      model: config.model,
      maxTokens: config.max_tokens,
      temperature: config.temperature,
    });

    return result;
  } catch (error) {
    const msg = String(error);
    chatGPTError.set(msg);
    throw new Error(msg);
  } finally {
    isProcessing.set(false);
  }
}
