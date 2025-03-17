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
    
    .content-container {
      flex: 1;
      overflow: hidden;
    }
  </style>