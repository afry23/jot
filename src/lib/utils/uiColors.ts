// uiColors.ts - Enhanced with Markdown-specific colors
import { derived } from "svelte/store";
import { theme } from "../stores/settings";

// Color palette for UI elements based on the current theme
export const uiColors = derived(theme, ($theme) => {
  // Base colors
  const isDark = $theme === "dark";
  const baseTextPrimary = isDark ? "#E0E0E0" : "#333333";
  const baseTextSecondary = isDark ? "#AAAAAA" : "#777777";
  const baseBgPrimary = isDark ? "#222222" : "#F5F5F5";
  const baseBgSecondary = isDark ? "#333333" : "#FFFFFF";

  return {
    // Text colors
    textPrimary: baseTextPrimary,
    textSecondary: baseTextSecondary,

    // Background colors
    bgPrimary: baseBgPrimary,
    bgSecondary: baseBgSecondary,

    // Border colors
    borderLight: isDark ? "rgba(255, 255, 255, 0.1)" : "rgba(0, 0, 0, 0.1)",
    borderMedium: isDark ? "rgba(255, 255, 255, 0.2)" : "rgba(0, 0, 0, 0.2)",

    // Interactive element colors
    buttonBg: isDark ? "rgba(255, 255, 255, 0.1)" : "rgba(0, 0, 0, 0.05)",
    buttonBgHover: isDark ? "rgba(255, 255, 255, 0.2)" : "rgba(0, 0, 0, 0.1)",

    // Status colors
    success: "#4CAF50",
    error: "#F44336",
    warning: "#FF9800",
    info: "#2196F3",

    // Markdown-specific colors (non-heading elements)
    markdown: {
      // Link colors
      linkText: isDark ? "#6B8AFF" : "#3366CC",
      linkHover: isDark ? "#92A9FF" : "#4477DD",
      linkVisited: isDark ? "#B088FF" : "#6655AA",

      // Code colors
      codeBackground: isDark ? "#333333" : "#F0F0F0",
      codeText: isDark ? "#E0E0E0" : "#333333",
      codeBorder: isDark ? "#444444" : "#DDDDDD",

      // Blockquote colors
      blockquoteBg: isDark ? "#2A2A2A" : "#F9F9F9",
      blockquoteBorder: isDark ? "#444444" : "#DDDDDD",
      blockquoteText: isDark ? "#BBBBBB" : "#666666",

      // List colors
      listMarker: isDark ? "#6B8AFF" : "#3366CC", // This can be updated to use tabColor

      // Table colors
      tableHeaderBg: isDark ? "#333333" : "#F0F0F0",
      tableRowEvenBg: isDark ? "#2A2A2A" : "#FFFFFF",
      tableRowOddBg: isDark ? "#252525" : "#F5F5F5",
      tableBorder: isDark ? "#444444" : "#DDDDDD",

      // Emphasis colors
      boldText: isDark ? "#FFFFFF" : "#111111",
      italicText: baseTextPrimary,

      // Horizontal rule
      hrColor: isDark ? "#444444" : "#DDDDDD",
    },
  };
});

// Helper function to adjust heading colors based on tabColor
export function getHeadingColorWithOpacity(
  tabColor: string,
  level: number
): string {
  // Calculate opacity based on heading level (higher level = lower opacity)
  const opacityFactors = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5]; // h1 to h6
  const opacity =
    opacityFactors[Math.min(level - 1, opacityFactors.length - 1)];

  // Parse the input color
  let r = 0,
    g = 0,
    b = 0;

  // Handle different color formats
  if (tabColor.startsWith("#")) {
    if (tabColor.length === 4) {
      // Convert #RGB to #RRGGBB
      r = parseInt(tabColor[1] + tabColor[1], 16);
      g = parseInt(tabColor[2] + tabColor[2], 16);
      b = parseInt(tabColor[3] + tabColor[3], 16);
    } else if (tabColor.length === 7) {
      // Standard #RRGGBB
      r = parseInt(tabColor.slice(1, 3), 16);
      g = parseInt(tabColor.slice(3, 5), 16);
      b = parseInt(tabColor.slice(5, 7), 16);
    }
  } else if (tabColor.startsWith("rgb")) {
    // Extract RGB values from rgb() or rgba()
    const rgbMatch = tabColor.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
    if (rgbMatch) {
      r = parseInt(rgbMatch[1], 10);
      g = parseInt(rgbMatch[2], 10);
      b = parseInt(rgbMatch[3], 10);
    }
  }

  // For lighter colors, darken as opacity decreases
  // For darker colors, lighten as opacity decreases
  const isLight = r * 0.299 + g * 0.587 + b * 0.114 > 128;

  if (isLight) {
    // Darken light colors
    r = Math.round(r * opacity);
    g = Math.round(g * opacity);
    b = Math.round(b * opacity);
  } else {
    // Lighten dark colors
    r = Math.round(r + (255 - r) * (1 - opacity));
    g = Math.round(g + (255 - g) * (1 - opacity));
    b = Math.round(b + (255 - b) * (1 - opacity));
  }

  // Convert back to hex format
  return `#${r.toString(16).padStart(2, "0")}${g
    .toString(16)
    .padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
}
