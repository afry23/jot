import { writable } from 'svelte/store';

// Define theme type
type Theme = 'light' | 'dark';

// Initialize theme based on system preference
function initTheme(): Theme {
  // Default to system preference if available
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark';
  }
  return 'light';
}

// Create theme store
export const theme = writable<Theme>(initTheme());

// Toggle theme function
export function toggleTheme() {
  theme.update(current => (current === 'light' ? 'dark' : 'light'));
  
  // Save theme preference
  theme.subscribe(value => {
    localStorage.setItem('jot-theme', value);
  });
}

// Load settings on init
export async function loadSettings() {
  // First check localStorage for saved theme
  const savedTheme = localStorage.getItem('jot-theme');
  if (savedTheme === 'light' || savedTheme === 'dark') {
    theme.set(savedTheme);
  }
  
  // Listen for system theme changes
  if (window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', e => {
      // Only update if user hasn't set a preference
      if (!localStorage.getItem('jot-theme')) {
        theme.set(e.matches ? 'dark' : 'light');
      }
    });
  }
}