// tabStyles.ts - New file to handle tab styling
import { derived } from "svelte/store";
import { activeTab } from "./tabs";
import { theme } from "./settings";
import { getTabBackgroundColor } from "$lib/utils/colors";

// Derived store that provides the current tab's background color
// This breaks the circular dependency by being in a separate file
export const activeTabBackground = derived(
  [activeTab, theme],
  ([$activeTab, $theme]) => getTabBackgroundColor($activeTab, $theme)
);
