// $lib/stores/languageServices.ts
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

// Define types for grammar checking
export interface GrammarError {
  message: string;
  offset: number;
  length: number;
  replacements: { value: string }[];
  rule: {
    id: string;
    description: string;
    issueType: string;
  };
}

export interface GrammarCheckResult {
  matches: GrammarError[];
}

// Define types for translation
export interface TranslationResult {
  translations: {
    detected_source_language: string;
    text: string;
  }[];
}

// Language service configuration
export interface LanguageServicesConfig {
  languagetool_api_key: string | null;
  languagetool_username: string | null; // Added username field
  languagetool_endpoint: string;
  deepl_api_key: string | null;
  deepl_endpoint: string;
}

// Store for language service configuration
export const languageConfig = writable<LanguageServicesConfig>({
  languagetool_api_key: null,
  languagetool_username: null, // Added username field
  languagetool_endpoint: "https://api.languagetoolplus.com/v2/check",
  deepl_api_key: null,
  deepl_endpoint: "https://api-free.deepl.com/v2/translate",
});

// Store for grammar check results
export const grammarCheckResults = writable<GrammarCheckResult>({
  matches: [],
});

// Store for translation results
export const translationResult = writable<string>("");

// Store for language service states
export const isGrammarCheckInProgress = writable<boolean>(false);
export const isTranslationInProgress = writable<boolean>(false);
export const grammarCheckError = writable<string | null>(null);
export const translationError = writable<string | null>(null);

// Load language services configuration
export async function loadLanguageServicesConfig(): Promise<void> {
  try {
    const config = await invoke<LanguageServicesConfig>(
      "get_language_services_config"
    );
    languageConfig.set(config);
    return;
  } catch (error) {
    console.error("Failed to load language services config:", error);
  }
}

// Save LanguageTool configuration
export async function saveLanguageToolConfig(
  apiKey?: string,
  username?: string, // Added username parameter
  endpoint?: string
): Promise<boolean> {
  try {
    await invoke("save_language_tool_config", {
      apiKey,
      username, // Added username parameter
      endpoint,
    });

    // Update local store
    languageConfig.update((config) => ({
      ...config,
      languagetool_username: username ?? config.languagetool_username, // Added username update
      languagetool_endpoint: endpoint ?? config.languagetool_endpoint,
    }));

    return true;
  } catch (error) {
    console.error("Failed to save LanguageTool config:", error);
    return false;
  }
}

// Save DeepL configuration
export async function saveDeepLConfig(
  apiKey?: string,
  endpoint?: string
): Promise<boolean> {
  try {
    await invoke("save_deepl_config", {
      apiKey,
      endpoint,
    });

    // Update local store
    languageConfig.update((config) => ({
      ...config,
      deepl_endpoint: endpoint ?? config.deepl_endpoint,
    }));

    return true;
  } catch (error) {
    console.error("Failed to save DeepL config:", error);
    return false;
  }
}

// Check grammar with LanguageTool
export async function checkGrammar(
  text: string,
  language: string = "en-US"
): Promise<GrammarCheckResult> {
  if (!text.trim()) {
    return { matches: [] };
  }

  isGrammarCheckInProgress.set(true);
  grammarCheckError.set(null);

  try {
    const result = await invoke<GrammarCheckResult>("check_grammar", {
      text,
      language,
    });

    grammarCheckResults.set(result);
    return result;
  } catch (error) {
    const errorMessage = `Grammar check failed: ${error}`;
    console.error(errorMessage);
    grammarCheckError.set(errorMessage);
    return { matches: [] };
  } finally {
    isGrammarCheckInProgress.set(false);
  }
}

// Translate text with DeepL
export async function translateText(
  text: string,
  targetLang: string,
  sourceLang?: string
): Promise<string> {
  if (!text.trim()) {
    return "";
  }

  isTranslationInProgress.set(true);
  translationError.set(null);

  try {
    const result = await invoke<TranslationResult>("translate_text", {
      text,
      targetLang,
      sourceLang,
    });

    if (result.translations && result.translations.length > 0) {
      const translatedText = result.translations[0].text;
      translationResult.set(translatedText);
      return translatedText;
    }

    return "";
  } catch (error) {
    const errorMessage = `Translation failed: ${error}`;
    console.error(errorMessage);
    translationError.set(errorMessage);
    return "";
  } finally {
    isTranslationInProgress.set(false);
  }
}

// Clear grammar check results
export function clearGrammarCheckResults(): void {
  grammarCheckResults.set({ matches: [] });
  grammarCheckError.set(null);
}

// Clear translation results
export function clearTranslationResults(): void {
  translationResult.set("");
  translationError.set(null);
}
