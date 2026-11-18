import { get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { activeTab } from "./tabs";

// Theme options
type Theme = "light" | "dark";
// Font size options
export type FontSize = "small" | "medium" | "large";
// Text width options
export type TextWidth = "medium" | "wide" | "full";

// Create stores
export const theme = writable<Theme>("light");
export const fontSize = writable<FontSize>("medium");
export const textWidth = writable<TextWidth>("wide");

// Toggle theme function
export function toggleTheme() {
  theme.update((current) => (current === "light" ? "dark" : "light"));

  // Save the theme setting
  saveSettings();
}

// Set font size function
export function setFontSize(size: FontSize) {
  fontSize.set(size);

  // Save to localStorage immediately for faster application
  localStorage.setItem("jot-font-size", size);

  // Dispatch a custom event for components that need to react
  if (typeof window !== "undefined") {
    window.dispatchEvent(
      new CustomEvent("jot:fontSizeChanged", { detail: size })
    );
  }

  // Save the font size setting
  saveSettings();
}

// Set text width function
export function setTextWidth(width: TextWidth) {
  textWidth.set(width);
  localStorage.setItem("jot-text-width", width);
  saveSettings();
}

// Save settings to storage
export async function saveSettings() {
  // Get current values from stores
  const currentTheme = get(theme);
  const currentFontSize = get(fontSize);
  const currentTextWidth = get(textWidth);
  const currentTab = get(activeTab);

  // Save to localStorage first (for quick access)
  localStorage.setItem("jot-theme", currentTheme);
  localStorage.setItem("jot-font-size", currentFontSize);
  localStorage.setItem("jot-text-width", currentTextWidth);
  localStorage.setItem("jot-active-tab", currentTab.toString());

  // Save to filesystem via Tauri
  try {
    await invoke("save_settings", {
      settings: {
        theme: currentTheme,
        fontSize: currentFontSize,
        textWidth: currentTextWidth,
        activeTab: currentTab,
      },
    });
    return true;
  } catch (error) {
    console.error("Error saving settings:", error);
    return false;
  }
}

// Load settings from storage
export async function loadSettings() {
  // Try to load from localStorage first (faster startup)
  const storedTheme = localStorage.getItem("jot-theme") as Theme;
  const storedFontSize = localStorage.getItem("jot-font-size") as FontSize;
  const storedTextWidth = localStorage.getItem(
    "jot-text-width"
  ) as TextWidth | null;
  const storedTab = localStorage.getItem("jot-active-tab");

  if (storedTheme) {
    theme.set(storedTheme);
  }

  if (storedFontSize) {
    fontSize.set(storedFontSize);
  }

  if (storedTextWidth) {
    textWidth.set(storedTextWidth);
  }

  // Then try to load from the filesystem
  try {
    const settings = await invoke<{
      theme?: Theme;
      fontSize?: FontSize;
      textWidth?: TextWidth;
      activeTab?: number;
    }>("load_settings");

    if (settings) {
      if (settings.theme) {
        theme.set(settings.theme);
        localStorage.setItem("jot-theme", settings.theme);
      }

      if (settings.fontSize) {
        fontSize.set(settings.fontSize);
        localStorage.setItem("jot-font-size", settings.fontSize);
      }

      if (settings.textWidth) {
        textWidth.set(settings.textWidth);
        localStorage.setItem("jot-text-width", settings.textWidth);
      }

      if (settings.activeTab !== undefined) {
        activeTab.set(settings.activeTab);
        localStorage.setItem("jot-active-tab", settings.activeTab.toString());
      }
    }

    return true;
  } catch (error) {
    console.error("Error loading settings:", error);
    return false;
  }
}
