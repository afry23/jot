import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

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

export interface LanguageServicesConfig {
  languagetool_api_key: string | null;
  languagetool_username: string | null;
  languagetool_endpoint: string;
}

export const languageConfig = writable<LanguageServicesConfig>({
  languagetool_api_key: null,
  languagetool_username: null,
  languagetool_endpoint: "https://api.languagetoolplus.com/v2/check",
});

export const isGrammarCheckInProgress = writable<boolean>(false);
export const grammarCheckError = writable<string | null>(null);
export const grammarCheckResults = writable<GrammarCheckResult>({ matches: [] });

// Spell check toggle — persisted to localStorage
export const spellCheckEnabled = writable<boolean>(
  typeof localStorage !== "undefined"
    ? localStorage.getItem("jot-spell-check") === "true"
    : false
);

if (typeof localStorage !== "undefined") {
  spellCheckEnabled.subscribe((val) => {
    localStorage.setItem("jot-spell-check", String(val));
  });
}

export function toggleSpellCheck() {
  spellCheckEnabled.update((v) => !v);
}

export async function loadLanguageServicesConfig(): Promise<void> {
  try {
    const config = await invoke<LanguageServicesConfig>(
      "get_language_services_config"
    );
    languageConfig.set(config);
  } catch (error) {
    console.error("Failed to load language services config:", error);
  }
}

export async function saveLanguageToolConfig(
  apiKey?: string,
  username?: string,
  endpoint?: string
): Promise<boolean> {
  try {
    await invoke("save_language_tool_config", {
      apiKey,
      username,
      endpoint,
    });

    languageConfig.update((config) => ({
      ...config,
      languagetool_username: username ?? config.languagetool_username,
      languagetool_endpoint: endpoint ?? config.languagetool_endpoint,
    }));

    return true;
  } catch (error) {
    console.error("Failed to save LanguageTool config:", error);
    return false;
  }
}

export async function checkGrammar(
  text: string,
  language: string = "auto"
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

export function clearGrammarCheckResults(): void {
  grammarCheckResults.set({ matches: [] });
  grammarCheckError.set(null);
}
