import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { activeTab } from "./tabs";

// Theme options
type Theme = "light" | "dark";
// Font size options
export type FontSize = "small" | "medium" | "large";

// Create stores
export const theme = writable<Theme>("light");
export const fontSize = writable<FontSize>("medium");

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

// Save settings to storage
export async function saveSettings() {
  let currentTheme: Theme;
  let currentFontSize: FontSize;
  let currentTab: number;

  // Get current values from stores
  theme.subscribe((value) => (currentTheme = value))();
  fontSize.subscribe((value) => (currentFontSize = value))();
  activeTab.subscribe((value) => (currentTab = value))();

  // Save to localStorage first (for quick access)
  localStorage.setItem("jot-theme", currentTheme);
  localStorage.setItem("jot-font-size", currentFontSize);
  localStorage.setItem("jot-active-tab", currentTab.toString());

  // Save to filesystem via Tauri
  try {
    await invoke("save_settings", {
      settings: {
        theme: currentTheme,
        fontSize: currentFontSize,
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
  const storedTab = localStorage.getItem("jot-active-tab");

  if (storedTheme) {
    theme.set(storedTheme);
  }

  if (storedFontSize) {
    fontSize.set(storedFontSize);
  }

  // Then try to load from the filesystem
  try {
    const settings = await invoke<{
      theme?: Theme;
      fontSize?: FontSize;
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
