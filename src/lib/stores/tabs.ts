import { getTabBackgroundColor, tabColors } from "$lib/utils/colors";
import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";

// Store for active tab index (0-6)
export const activeTab = writable<number>(0);

export const activeTabColor = derived(
  activeTab,
  ($activeTab) => tabColors[$activeTab]
);

// Function to set active tab
export function setActiveTab(index: number) {
  if (index >= 0 && index <= 6) {
    activeTab.set(index);
    saveActiveTab(index);
  }
}

export async function saveActiveTab(index: number) {
  try {
    // Save to localStorage for quick access
    localStorage.setItem("jot-active-tab", index.toString());

    // Save to filesystem via Tauri
    await invoke("save_active_tab", { tabIndex: index });
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
      if (tabIndex >= 0 && tabIndex <= 6) {
        activeTab.set(tabIndex);
      }
    }

    // Then try to load from the filesystem
    const settings = await invoke<{ activeTab?: number }>("load_settings");

    if (settings && settings.activeTab !== undefined) {
      const tabIndex = settings.activeTab;
      if (tabIndex >= 0 && tabIndex <= 6) {
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
