import { writable } from 'svelte/store';

// Store for active tab index (0-6)
export const activeTab = writable<number>(0);

// Function to set active tab
export function setActiveTab(index: number) {
  if (index >= 0 && index <= 6) {
    activeTab.set(index);
  }
}