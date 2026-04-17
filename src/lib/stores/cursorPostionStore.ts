import { writable } from 'svelte/store';

// Store cursor and scroll positions per tab (one position per tab)
export const cursorPositions = writable<Record<number, number>>({});
export const scrollPositions = writable<Record<number, number>>({});
