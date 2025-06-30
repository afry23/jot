import { activeTab, setActiveTab } from "./tabs";
import { toggleViewMode } from "./viewMode";
import { toggleTheme } from "./settings";
import { get } from "svelte/store";
import { myredo, myundo } from "./history";
import { notes, updateNote } from "./notes";
import { register } from "@tauri-apps/plugin-global-shortcut";
import { Window } from "@tauri-apps/api/window";
import { logger } from "$lib/utils/logger";
import { saveNote } from "$lib/utils/persistence";

async function toggleWindow() {
  console.log("Toggling window visibility");
  try {
    let visible = await Window.getCurrent().isVisible();
    if (visible) {
      console.log("Hiding window");
      await Window.getCurrent().hide();
    } else {
      console.log("Showing window");
      await Window.getCurrent().show();
      await Window.getCurrent().setFocus();
    }
  } catch (error) {
    console.error("Failed to hide window:", error);
  }
}

// Safe tab switching that ensures current content is saved first
async function safeSwitchTab(newTabIndex: number) {
  // Dispatch a custom event to request current content from the editor
  const saveContentEvent = new CustomEvent("save-current-content", {
    detail: { newTabIndex },
  });

  // Let the editor handle saving, then it will call setActiveTab
  window.dispatchEvent(saveContentEvent);
}

// Function to setup global keyboard shortcuts
export async function setupKeyboardShortcuts() {
  // Only set up shortcuts once
  if (typeof window === "undefined") return;

  // Avoid duplicate listeners
  window.removeEventListener("keydown", handleKeydown);
  window.addEventListener("keydown", handleKeydown);
  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+1-7: Change tab (works even in textarea)
    if (event.ctrlKey && event.key >= "1" && event.key <= "7") {
      event.preventDefault();
      const tabIndex = parseInt(event.key) - 1;
      if (tabIndex >= 0 && tabIndex <= 6) {
        // Blur any focused editor elements to ensure content is saved properly
        const activeElement = document.activeElement as HTMLElement;
        if (
          activeElement &&
          (activeElement.classList.contains("markdown-textarea") ||
            activeElement.classList.contains("ProseMirror"))
        ) {
          activeElement.blur();
        }

        // Small delay to ensure content is properly saved before switching
        setTimeout(() => {
          setActiveTab(tabIndex);
        }, 10);
      }
      return;
    }

    // Ctrl+E: Toggle edit/preview mode (works even when editing)
    if (event.ctrlKey && event.key === "e") {
      event.preventDefault();
      toggleViewMode();
      return;
    }

    // Ctrl+D: Toggle dark/light mode
    if (event.ctrlKey && event.key === "d") {
      event.preventDefault();
      toggleTheme();
      return;
    }

    //if (event.ctrlKey && !event.shiftKey && event.key === 'z') {
    //  event.preventDefault();
    //  const currentTab = get(activeTab);
    //  const previousContent = undo(currentTab);

    //  if (previousContent !== null) {
    // Use a different function to update notes to avoid circular updates
    //   notes.update((state) => {
    //      state[currentTab] = previousContent;
    // return state;
    // });
    // }
    // return;
    // }

    // Ctrl+Shift+Z or Ctrl+Y: Redo
    // if ((event.ctrlKey && event.shiftKey && event.key === 'z') ||
    // (event.ctrlKey && event.key === 'y')) {
    // event.preventDefault();
    // const currentTab = get(activeTab);
    // const nextContent = redo(currentTab);

    // if (nextContent !== null) {
    // Use a different function to update notes to avoid circular updates
    // notes.update((state) => {
    // state[currentTab] = nextContent;
    // return state;
    // });
    // }
    // return;
    // }
  }

  // Clean up on teardown
  return () => {
    window.removeEventListener("keydown", handleKeydown);
  };
}
