import { writable } from 'svelte/store';

// Store cursor positions for each tab and editor mode
// Structure: cursorPositions[tabIndex][mode] = position
export const cursorPositions = writable<Record<number, Record<string, number>>>({});

// Store scroll positions for each tab and editor mode
// Structure: scrollPositions[tabIndex][mode] = scrollTop
export const scrollPositions = writable<Record<number, Record<string, number>>>({});