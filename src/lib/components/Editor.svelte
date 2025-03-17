<script lang="ts">
  import { onMount } from 'svelte';
  import { activeTab } from '$lib/stores/tabs';
  import { notes, updateNote } from '$lib/stores/notes';

  let editorElement: HTMLTextAreaElement;
  let tabColors = [
    '#F6C046', // Yellow
    '#F0874F', // Orange 
    '#F25D66', // Red
    '#7D7E8A', // Gray
    '#7E7E9A', // Blue-Gray
    '#9D7E9A', // Purple
    '#7E9D7E'  // Green
  ];
  
  function handleInput() {
    if (editorElement) {
      // Update the store with the current content
      updateNote($activeTab, editorElement.value);
    }
  }
  
  // Key shortcuts for tab navigation
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+1-7: Change tab
    if (event.ctrlKey && event.key >= '1' && event.key <= '7') {
      event.preventDefault();
      const tabIndex = parseInt(event.key) - 1;
      if (tabIndex >= 0 && tabIndex < 7) {
        activeTab.set(tabIndex);
      }
    }
  }
  
  let previousTab = -1;
  
  // Update editor content when active tab changes
  $: if (editorElement && $notes[$activeTab] !== undefined) {
    // Only update content if we've changed tabs
    if (previousTab !== $activeTab) {
      const content = $notes[$activeTab] || '';
      editorElement.value = content;
      previousTab = $activeTab;
      
      // Focus and place cursor at the end
      setTimeout(() => {
        editorElement.focus();
        editorElement.selectionStart = editorElement.value.length;
        editorElement.selectionEnd = editorElement.value.length;
      }, 10);
    }
  }
  
  onMount(() => {
    if (editorElement) {
      // Set initial content
      editorElement.value = $notes[$activeTab] || '';
      // Focus the editor
      editorElement.focus();
    }
  });
</script>

<div 
  class="editor-container"
  style="--tab-color: {tabColors[$activeTab]};"
>
  <textarea
    class="editor"
    bind:this={editorElement}
    on:input={handleInput}
    on:keydown={handleKeydown}
    on:blur={handleInput}
    spellcheck="true"
  ></textarea>
</div>

<style>
  .editor-container {
    height: 100%;
    padding: 0;
    margin: 0;
    overflow: hidden;
  }

  .editor {
    width: 100%;
    height: 100%;
    outline: none;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
    font-size: 16px;
    line-height: 1.5;
    color: var(--text-color);
    background-color: transparent;
    border: none;
    resize: none;
    padding: 20px;
    box-sizing: border-box;
    caret-color: var(--tab-color);
  }
</style>