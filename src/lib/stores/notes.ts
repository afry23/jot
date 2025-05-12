import { writable } from "svelte/store";
import { pushToHistory } from "./history";
import { saveNote } from "$lib/utils/persistence";

export const notesLoaded = writable<boolean>(false);

// Store for notes content
// Index 0-6 corresponds to the 7 tabs
export const notes = writable<Record<number, string>>({
  0: "",
  1: "",
  2: "",
  3: "",
  4: "",
  5: "",
  6: "",
});

// Update a specific note
export function updateNote(tabIndex: number, content: string) {
  // Push to history before updating
  pushToHistory(tabIndex, content);

  notes.update((state) => {
    state[tabIndex] = content;
    return state;
  });

  // If content is empty, ensure it gets saved
  if (content === "") {
    console.log(`Explicitly saving empty content for tab ${tabIndex}`);
    saveNote(tabIndex, "");
  }
}

// Reset notes (used when loading from storage)
export function setNotes(notesData: Record<number, string>) {
  notes.set(notesData);
  notesLoaded.set(true);
}
