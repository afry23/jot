<script lang="ts">
  import Editor from "./Editor.svelte";
  import MarkdownEditor from "./MarkdownEditor.svelte";
  import { activeTab } from "$lib/stores/tabs";
  import { notes, updateNote } from "$lib/stores/notes";
  import { emptyTabColor, tabColors } from "$lib/utils/colors";

  // Function to check if the current tab is empty
  $: isCurrentTabEmpty =
    !$notes[$activeTab] || $notes[$activeTab].trim() === "";

  // Get the appropriate color for the current tab
  $: currentTabColor = isCurrentTabEmpty
    ? emptyTabColor
    : tabColors[$activeTab];
  
  // This function would handle content updates from MarkdownEditor.
  // MarkdownEditor.svelte needs to be modified to dispatch a 'change' event.
  function handleMarkdownEditorChange(event: CustomEvent<string>) {
    if ($activeTab !== undefined && event.detail !== undefined) {
      // Assuming event.detail contains the new markdown string
      updateNote($activeTab, event.detail);
    }
  }
</script>

<div class="editor-wrapper" style="--tab-color: {currentTabColor};">
  <div class="content-container">
    {#key $activeTab}
    <MarkdownEditor
      initialContent={$notes[$activeTab] || ""}
      on:change={handleMarkdownEditorChange}
    />
    {/key}
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
