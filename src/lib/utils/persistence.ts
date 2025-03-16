import { setNotes } from '$lib/stores/notes';
import { invoke } from '@tauri-apps/api/core';

// Save a note to storage
export async function saveNote(tabIndex: number, content: string) {
  try {
    // Use local storage for quick saving (to reduce disk writes)
    localStorage.setItem(`jot-note-${tabIndex}`, content);
    
    // Debounced saving to filesystem via Tauri
    if (window.savingTimeout) {
      clearTimeout(window.savingTimeout);
    }
    
    window.savingTimeout = setTimeout(async () => {
      await invoke('save_note', { 
        tabIndex, 
        content 
      });
    }, 1000); // 1 second debounce
    
    return true;
  } catch (error) {
    console.error('Error saving note:', error);
    return false;
  }
}

// Load all notes from storage
export async function loadNotes() {
  try {
    // Try to load from localStorage first (faster startup)
    const notesData: Record<number, string> = {
      0: '', 1: '', 2: '', 3: '', 4: '', 5: '', 6: ''
    };
    
    // Check localStorage first
    for (let i = 0; i <= 6; i++) {
      const content = localStorage.getItem(`jot-note-${i}`);
      if (content) {
        notesData[i] = content;
      }
    }
    
    // Update the store with what we have so far
    setNotes(notesData);
    
    // Then load from the filesystem (which may update the notes again)
    const storedNotes = await invoke<Record<number, string>>('load_notes');
    
    // Merge with what we already have (filesystem takes precedence)
    for (const [tabIndex, content] of Object.entries(storedNotes)) {
      notesData[Number(tabIndex)] = content;
      // Also update localStorage
      localStorage.setItem(`jot-note-${tabIndex}`, content);
    }
    
    // Update the store
    setNotes(notesData);
    
    return true;
  } catch (error) {
    console.error('Error loading notes:', error);
    return false;
  }
}

// For TypeScript
declare global {
  interface Window {
    savingTimeout: number | null;
  }
}

// Initialize timeout property
window.savingTimeout = null;