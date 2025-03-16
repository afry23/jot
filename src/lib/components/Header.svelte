<script lang="ts">
    import ColorDot from './ColorDot.svelte';
    import { activeTab, setActiveTab } from '$lib/stores/tabs';
    import { invoke } from '@tauri-apps/api/tauri';
  
    // Tab colors
    const tabColors = [
      '#F6C046', // Yellow
      '#F0874F', // Orange 
      '#F25D66', // Red
      '#7D7E8A', // Gray
      '#7E7E9A', // Blue-Gray
      '#9D7E9A', // Purple
      '#7E9D7E'  // Green
    ];
  
    // Handle tab click
    function handleTabClick(index: number) {
      setActiveTab(index);
    }
  
    // Close window function
    async function closeWindow() {
      await invoke('close_window');
    }
  
    // Open settings
    function openSettings() {
      // Will be implemented
    }
  </script>
  
  <header>
    <div class="left">
      <button class="close-btn" on:click={closeWindow}>×</button>
    </div>
    
    <div class="tabs">
      {#each tabColors as color, i}
        <ColorDot 
          {color} 
          active={$activeTab === i} 
          onClick={() => handleTabClick(i)} 
        />
      {/each}
    </div>
    
    <div class="right">
      <button class="settings-btn" on:click={openSettings}>⚙</button>
    </div>
  </header>
  
  <style>
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 10px 15px;
      background-color: rgba(0, 0, 0, 0.1);
      -webkit-app-region: drag; /* Makes the header draggable */
    }
  
    .tabs {
      display: flex;
      gap: 10px;
      margin: 0 auto;
    }
  
    .left, .right {
      width: 30px;
      display: flex;
      justify-content: center;
    }
  
    button {
      -webkit-app-region: no-drag; /* Buttons should not be draggable */
      background: none;
      border: none;
      font-size: 20px;
      cursor: pointer;
      width: 24px;
      height: 24px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--text-color);
      opacity: 0.7;
      transition: opacity 0.2s;
    }
  
    button:hover {
      opacity: 1;
    }
  
    .close-btn {
      font-size: 24px;
    }
  
    .settings-btn {
      font-size: 16px;
    }
  </style>