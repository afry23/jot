import { writable } from 'svelte/store';

// Define view mode type
type ViewMode = 'edit' | 'preview';

// Create view mode store, default to edit mode
export const viewMode = writable<ViewMode>('edit');

// Toggle view mode function
export function toggleViewMode() {
  viewMode.update(current => (current === 'edit' ? 'preview' : 'edit'));
}

// Set specific view mode
export function setViewMode(mode: ViewMode) {
  viewMode.set(mode);
}