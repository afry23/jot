<script lang="ts">
  import { onMount, afterUpdate } from 'svelte';
  import { activeTab } from '$lib/stores/tabs';
  import { notes, updateNote } from '$lib/stores/notes';
  import { formatBold, formatItalic, formatList } from '$lib/utils/textFormatting';

  let editorElement: HTMLDivElement;
  let tabColors = [
    '#F6C046', // Yellow
    '#F0874F', // Orange 
    '#F25D66', // Red
    '#7D7E8A', // Gray
    '#7E7E9A', // Blue-Gray
    '#9D7E9A', // Purple
    '#7E9D7E'  // Green
  ];
  
  function handleInput(event) {
    if (editorElement) {
      // Store the current selection position
      const selection = window.getSelection();
      const range = selection?.getRangeAt(0);
      
      // Get the content directly from the editor element
      const content = editorElement.innerHTML;
      
      // Update the store with the current content
      updateNote($activeTab, content);
      
      // Prevent any default browser behavior that might affect cursor
      event.stopPropagation();
    }
  }
  
  // Key shortcuts for formatting
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+B: Bold
    if (event.ctrlKey && event.key === 'b') {
      event.preventDefault();
      formatBold();
    }
    // Ctrl+I: Italic
    else if (event.ctrlKey && event.key === 'i') {
      event.preventDefault();
      formatItalic();
    }
    // Ctrl+L: List
    else if (event.ctrlKey && event.key === 'l') {
      event.preventDefault();
      formatList();
    }
    // Ctrl+1-7: Change tab
    else if (event.ctrlKey && event.key >= '1' && event.key <= '7') {
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
      editorElement.innerHTML = content;
      previousTab = $activeTab;
      
      // Wait for DOM update, then focus at the end
      setTimeout(() => {
        // Place cursor at the end
        editorElement.focus();
        
        // Use Selection API to move cursor to end
        const selection = window.getSelection();
        if (selection) {
          selection.removeAllRanges();
          const range = document.createRange();
          range.selectNodeContents(editorElement);
          range.collapse(false); // collapse to end
          selection.addRange(range);
        }
      }, 10);
    }
  }
  
  // Remove reactive focus setter since we handle this in the tab change watcher now
  
  onMount(() => {
    if (editorElement) {
      // Set initial content - using insertAdjacentHTML instead of innerHTML
      // to avoid potential DOM nesting issues
      editorElement.innerHTML = '';
      if ($notes[$activeTab]) {
        editorElement.innerHTML = $notes[$activeTab];
      }
      
      // Simple focus without cursor manipulation
      editorElement.focus();
    }
  });
</script>

<div 
  class="editor-container"
  style="--tab-color: {tabColors[$activeTab]};"
>
  <div
    class="editor"
    bind:this={editorElement}
    contenteditable="true"
    on:input={handleInput}
    on:keydown={handleKeydown}
    on:blur={handleInput} 
    on:focus={() => {}} 
    spellcheck="true"
  ></div>
</div>

<style>
  .editor-container {
    height: 100%;
    overflow-y: auto;
    padding: 20px;
  }

  .editor {
    min-height: 100%;
    outline: none;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
    font-size: 16px;
    line-height: 1.5;
    color: var(--text-color);
    white-space: pre-wrap; /* Preserve whitespace but allow wrapping */
    word-break: break-word; /* Prevent overflow by breaking words */
    direction: ltr; /* Explicitly set text direction */
    caret-color: var(--text-color); /* Explicit caret color */
  }

  .editor :global(h1), .editor :global(h2), .editor :global(h3) {
    color: var(--tab-color);
    margin-top: 1em;
    margin-bottom: 0.5em;
  }

  .editor :global(strong) {
    font-weight: 600;
  }

  .editor :global(ul), .editor :global(ol) {
    padding-left: 1.5em;
    margin: 0.5em 0;
  }

  .editor :global(li) {
    margin-bottom: 0.25em;
  }
</style>