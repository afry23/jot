<script lang="ts">
    import Editor from './Editor.svelte';
    import Preview from './Preview.svelte';
    import { viewMode, toggleViewMode } from '$lib/stores/viewMode';
    import { activeTab } from '$lib/stores/tabs';
    
    // Tab colors for indicator
    let tabColors = [
      '#F6C046', // Yellow
      '#F0874F', // Orange 
      '#F25D66', // Red
      '#7D7E8A', // Gray
      '#7E7E9A', // Blue-Gray
      '#9D7E9A', // Purple
      '#7E9D7E'  // Green
    ];
  </script>
  
  <div class="editor-wrapper" style="--tab-color: {tabColors[$activeTab]};">
    <div class="toggle-bar">
      <button 
        class="toggle-button" 
        class:active={$viewMode === 'edit'} 
        on:click={() => toggleViewMode()} 
        aria-label="Toggle edit/preview mode"
      >
        {$viewMode === 'edit' ? 'Preview' : 'Edit'}
      </button>
    </div>
    
    <div class="content-container">
      {#if $viewMode === 'edit'}
        <Editor />
      {:else}
        <Preview />
      {/if}
    </div>
  </div>
  
  <style>
    .editor-wrapper {
      display: flex;
      flex-direction: column;
      height: 100%;
      overflow: hidden;
    }
    
    .toggle-bar {
      padding: 8px 16px;
      display: flex;
      justify-content: flex-end;
      background-color: rgba(0, 0, 0, 0.03);
      border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    }
    
    :global(.dark) .toggle-bar {
      background-color: rgba(255, 255, 255, 0.03);
      border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }
    
    .toggle-button {
      background-color: transparent;
      border: 1px solid var(--tab-color);
      border-radius: 4px;
      color: var(--tab-color);
      padding: 4px 12px;
      font-size: 12px;
      cursor: pointer;
      transition: all 0.2s ease;
    }
    
    .toggle-button:hover {
      background-color: rgba(0, 0, 0, 0.05);
    }
    
    :global(.dark) .toggle-button:hover {
      background-color: rgba(255, 255, 255, 0.05);
    }
    
    .content-container {
      flex: 1;
      overflow: hidden;
    }
  </style>