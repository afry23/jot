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
    
    function handleInput() {
      if (editorElement) {
        updateNote($activeTab, editorElement.innerHTML);
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
    
    // Update editor content when tab changes
    $: if (editorElement && $notes[$activeTab] !== undefined) {
      editorElement.innerHTML = $notes[$activeTab] || '';
    }
    
    // Set focus on tab change
    $: if (editorElement && $activeTab !== undefined) {
      setTimeout(() => {
        editorElement.focus();
      }, 0);
    }
    
    onMount(() => {
      if (editorElement) {
        editorElement.innerHTML = $notes[$activeTab] || '';
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