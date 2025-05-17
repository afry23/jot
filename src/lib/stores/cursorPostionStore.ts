import { writable } from 'svelte/store';

// Store cursor positions for each tab
export const cursorPositions = writable<Record<number, number>>({});

// Store scroll positions for each tab
export const scrollPositions = writable<Record<number, number>>({});