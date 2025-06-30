import { tabColors } from "$lib/utils/colors";
import { invoke } from "@tauri-apps/api/core";
import { derived, get, writable } from "svelte/store";

const MAX_TAB_INDEX = 6; // Maximum tab index (0-6)

interface Settings {
  activeTab?: number; // Optional active tab index
}

// Store for active tab index (0-6)
export const activeTab = writable<number>(0);

export const activeTabColor = derived(
  activeTab,
  ($activeTab) => tabColors[$activeTab]
);

// Function to set active tab
export async function setActiveTab(tabIndex: number) {
  if (tabIndex >= 0 && tabIndex <= MAX_TAB_INDEX) {
    console.log(`Setting active tab from ${get(activeTab)} to ${tabIndex}`);
    activeTab.set(tabIndex);
    await saveActiveTab(tabIndex);
    return true;
  }
  return false;
}

export async function saveActiveTab(tabIndex: number) {
  try {
    // Save to localStorage for quick access
    localStorage.setItem("jot-active-tab", tabIndex.toString());

    // Save to filesystem via Tauri
    await invoke("save_active_tab", { tabIndex: tabIndex });
    return true;
  } catch (error) {
    console.error("Error saving active tab:", error);
    return false;
  }
}

// Load active tab from storage
export async function loadActiveTab() {
  try {
    // Try to load from localStorage first (faster startup)
    const storedTab = localStorage.getItem("jot-active-tab");

    if (storedTab) {
      const tabIndex = parseInt(storedTab, 10);
      if (tabIndex >= 0 && tabIndex <= MAX_TAB_INDEX) {
        activeTab.set(tabIndex);
      }
    }

    // Then try to load from the filesystem
    const settings = await invoke<Settings>("load_settings");

    if (settings && settings.activeTab !== undefined) {
      const tabIndex = settings.activeTab;
      if (tabIndex >= 0 && tabIndex <= MAX_TAB_INDEX) {
        activeTab.set(tabIndex);
        // Update localStorage
        localStorage.setItem("jot-active-tab", tabIndex.toString());
      }
    }

    return true;
  } catch (error) {
    console.error("Error loading active tab:", error);
    return false;
  }
}
