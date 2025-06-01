// $lib/stores/history.ts
import { writable } from "svelte/store";
import { activeTab } from "./tabs";
import { get } from "svelte/store";

// Define maximum history size per tab
const MAX_HISTORY_SIZE = 100;

// Store for history stacks (one for each tab)
export const undoHistory = writable<Record<number, string[]>>({
  0: [],
  1: [],
  2: [],
  3: [],
  4: [],
  5: [],
  6: [],
});

// Store for redo stacks (one for each tab)
export const redoHistory = writable<Record<number, string[]>>({
  0: [],
  1: [],
  2: [],
  3: [],
  4: [],
  5: [],
  6: [],
});

// Store for last saved content to compare changes
export const lastSavedContent = writable<Record<number, string>>({
  0: "",
  1: "",
  2: "",
  3: "",
  4: "",
  5: "",
  6: "",
});

// Add content to history
export function pushToHistory(tabIndex: number, content: string): void {
  const currentLastSaved = get(lastSavedContent)[tabIndex];

  // Only save if content has changed
  if (currentLastSaved !== content) {
    undoHistory.update((state) => {
      // Push current content to history
      if (state[tabIndex].length === 0 || state[tabIndex][state[tabIndex].length - 1] !== currentLastSaved) {
        state[tabIndex].push(currentLastSaved);
      }

      // Limit history size
      if (state[tabIndex].length > MAX_HISTORY_SIZE) {
        state[tabIndex] = state[tabIndex].slice(-MAX_HISTORY_SIZE);
      }

      return state;
    });

    // Clear redo history for this tab since we have a new change
    redoHistory.update((state) => {
      state[tabIndex] = [];
      return state;
    });

    // Update last saved content
    lastSavedContent.update((state) => {
      state[tabIndex] = content;
      return state;
    });
  }
}

// Undo action
export function myundo(tabIndex: number): string | null {
  let result = null;

  undoHistory.update((undoState) => {
    if (undoState[tabIndex].length > 0) {
      // Get the last content from history
      const previousContent = undoState[tabIndex].pop();

      // Add current content to redo stack
      redoHistory.update((redoState) => {
        redoState[tabIndex].push(get(lastSavedContent)[tabIndex]);
        return redoState;
      });

      // Update result to return
      result = previousContent;

      // Update last saved content
      lastSavedContent.update((state) => {
        if (previousContent !== undefined) {
          state[tabIndex] = previousContent;
        }
        return state;
      });
    }

    return undoState;
  });

  return result;
}

// Redo action
export function myredo(tabIndex: number): string | null {
  let result = null;

  redoHistory.update((redoState) => {
    if (redoState[tabIndex].length > 0) {
      // Get the last content from redo history
      const nextContent = redoState[tabIndex].pop();

      // Add current content to undo stack
      undoHistory.update((undoState) => {
        undoState[tabIndex].push(get(lastSavedContent)[tabIndex]);
        return undoState;
      });

      // Update result to return
      result = nextContent;

      // Update last saved content
      lastSavedContent.update((state) => {
        if (nextContent !== undefined) {
          state[tabIndex] = nextContent;
        }
        return state;
      });
    }

    return redoState;
  });

  return result;
}

// Clear history for a tab
export function clearHistory(tabIndex: number): void {
  undoHistory.update((state) => {
    state[tabIndex] = [];
    return state;
  });

  redoHistory.update((state) => {
    state[tabIndex] = [];
    return state;
  });
}

// Initialize history with current note content
export function initializeHistory(content: Record<number, string>): void {
  lastSavedContent.set({ ...content });
}