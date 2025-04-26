<script lang="ts">
  import Editor from "./Editor.svelte";
  import { activeTab } from "$lib/stores/tabs";
  import { notes } from "$lib/stores/notes";
  import { emptyTabColor, tabColors } from "$lib/utils/colors";

  // Function to check if the current tab is empty
  $: isCurrentTabEmpty =
    !$notes[$activeTab] || $notes[$activeTab].trim() === "";

  // Get the appropriate color for the current tab
  $: currentTabColor = isCurrentTabEmpty
    ? emptyTabColor
    : tabColors[$activeTab];
</script>

<div class="editor-wrapper" style="--tab-color: {currentTabColor};">
  <div class="content-container">
    <Editor />
  </div>
</div>

<style>
  .editor-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .content-container {
    flex: 1;
    overflow: hidden;
  }
</style>
