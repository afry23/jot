import { derived, type Writable } from "svelte/store";

// Tab primary colors
export const tabColors = [
  "#F6C046", // Yellow
  "#F0874F", // Orange
  "#F25D66", // Red
  "#7D7E8A", // Gray
  "#7E7E9A", // Blue-Gray
  "#9D7E9A", // Purple
  "#7E9D7E", // Green
];

// Light theme background colors (pastel versions)
export const lightTabBackgroundColors = [
  "#FFF8E6", // Light Yellow
  "#FFF1EB", // Light Orange
  "#FFEAEC", // Light Red
  "#F0F0F2", // Light Gray
  "#EFEFFF", // Light Blue-Gray
  "#F8EFFF", // Light Purple
  "#EFFFEF", // Light Green
];

// Dark theme background colors (slightly darker pastel versions)
export const darkTabBackgroundColors = [
  "#3A3522", // Dark Yellow
  "#3A2E27", // Dark Orange
  "#3A2828", // Dark Red
  "#2E2E32", // Dark Gray
  "#2E2E3A", // Dark Blue-Gray
  "#332E3A", // Dark Purple
  "#2E3A2E", // Dark Green
];

export const emptyTabColor = "#4A4A4A";

// Function to get the background color for a specific tab and theme
export function getTabBackgroundColor(
  tabIndex: number,
  currentTheme: string
): string {
  if (tabIndex < 0 || tabIndex >= tabColors.length) {
    tabIndex = 0; // Default to first color if invalid index
  }

  return currentTheme === "dark"
    ? darkTabBackgroundColors[tabIndex]
    : lightTabBackgroundColors[tabIndex];
}

// Create a derived store that provides the current background color based on active tab
// This function now requires the theme to be passed in
export function createBackgroundColorStore(
  activeTabStore: Writable<number>,
  themeStore: Writable<string>
) {
  return derived([activeTabStore, themeStore], ([$activeTab, $theme]) =>
    getTabBackgroundColor($activeTab, $theme)
  );
}

// Helper function to adjust color opacity/transparency
export function withOpacity(color: string, opacity: number): string {
  // Check if color is in hex format
  if (color.startsWith("#")) {
    // Convert hex to rgb
    const r = parseInt(color.slice(1, 3), 16);
    const g = parseInt(color.slice(3, 5), 16);
    const b = parseInt(color.slice(5, 7), 16);

    return `rgba(${r}, ${g}, ${b}, ${opacity})`;
  }

  // If already in rgb/rgba format, parse and modify
  if (color.startsWith("rgb")) {
    // Extract just the numbers
    const rgbMatch = color.match(/\d+/g);
    if (rgbMatch && rgbMatch.length >= 3) {
      const [r, g, b] = rgbMatch.map(Number);
      return `rgba(${r}, ${g}, ${b}, ${opacity})`;
    }
  }

  // If color format is unknown, return as is
  return color;
}
