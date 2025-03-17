<script lang="ts">
  import { onMount } from 'svelte';
  import { theme, toggleTheme } from '$lib/stores/settings';

  let isOpen = false;
  let fontSizes = ['Small', 'Medium', 'Large'];
  let selectedFontSize = 'Medium';
  
  // Keyboard shortcuts info
  const shortcuts = [
    { key: 'Ctrl+1-7', action: 'Switch to tab 1-7' }
  ];
  
  // Markdown syntax help
  const markdownHelp = [
    { syntax: '# Heading', description: 'Level 1 heading' },
    { syntax: '## Heading', description: 'Level 2 heading' },
    { syntax: '### Heading', description: 'Level 3 heading' },
    { syntax: '**bold text**', description: 'Bold text' },
    { syntax: '*italic text*', description: 'Italic text' },
    { syntax: '- item', description: 'List item' },
    { syntax: '1. item', description: 'Numbered list item' },
    { syntax: '> quote', description: 'Blockquote' },
    { syntax: '`code`', description: 'Inline code' },
    { syntax: '```code block```', description: 'Code block' },
    { syntax: '[text](url)', description: 'Link' },
    { syntax: '---', description: 'Horizontal rule' }
  ];

  export function open() {
    isOpen = true;
  }

  export function close() {
    isOpen = false;
  }
  
  function handleOutsideClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (isOpen && !target.closest('.settings-panel')) {
      close();
    }
  }

  // Change font size (to be implemented)
  function changeFontSize(size: string) {
    selectedFontSize = size;
    // TODO: Implement font size change logic
  }
  
  onMount(() => {
    document.addEventListener('click', handleOutsideClick);
    return () => {
      document.removeEventListener('click', handleOutsideClick);
    };
  });
</script>

{#if isOpen}
  <div class="settings-overlay">
    <div class="settings-panel">
      <h2>Settings</h2>
      
      <div class="settings-section">
        <h3>Appearance</h3>
        
        <div class="setting-item">
          <span>Theme</span>
          <button class="theme-toggle" on:click={toggleTheme}>
            {$theme === 'dark' ? 'Dark' : 'Light'}
          </button>
        </div>
        
        <div class="setting-item">
          <span>Font Size</span>
          <div class="font-size-options">
            {#each fontSizes as size}
              <button 
                class="font-size-option" 
                class:selected={selectedFontSize === size}
                on:click={() => changeFontSize(size)}
              >
                {size}
              </button>
            {/each}
          </div>
        </div>
      </div>
      
      <div class="settings-section">
        <h3>Keyboard Shortcuts</h3>
        <div class="shortcuts-list">
          {#each shortcuts as shortcut}
            <div class="shortcut-item">
              <kbd>{shortcut.key}</kbd>
              <span>{shortcut.action}</span>
            </div>
          {/each}
        </div>
      </div>
      
      <div class="settings-section">
        <h3>Markdown Formatting</h3>
        <div class="markdown-help">
          {#each markdownHelp as help}
            <div class="markdown-item">
              <code>{help.syntax}</code>
              <span>{help.description}</span>
            </div>
          {/each}
        </div>
      </div>
      
      <div class="footer">
        <span>Jot v1.0.0</span>
        <button class="close-btn" on:click={close}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease;
  }
  
  .settings-panel {
    width: 450px;
    max-width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    background-color: var(--bg-color);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    padding: 20px;
    animation: slideIn 0.3s ease;
  }
  
  h2 {
    margin-top: 0;
    margin-bottom: 20px;
    color: var(--text-color);
    font-size: 20px;
    font-weight: 600;
  }
  
  h3 {
    font-size: 16px;
    margin-bottom: 10px;
    color: var(--text-color);
  }
  
  .settings-section {
    margin-bottom: 24px;
  }
  
  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  
  button {
    background-color: rgba(0, 0, 0, 0.05);
    border: none;
    border-radius: 4px;
    padding: 8px 12px;
    cursor: pointer;
    color: var(--text-color);
  }
  
  button:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }
  
  .font-size-options {
    display: flex;
    gap: 8px;
  }
  
  .font-size-option {
    padding: 4px 8px;
    border-radius: 4px;
  }
  
  .font-size-option.selected {
    background-color: var(--accent-color);
    color: white;
  }
  
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .shortcut-item {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  kbd {
    display: inline-block;
    padding: 3px 6px;
    background-color: rgba(0, 0, 0, 0.05);
    border-radius: 3px;
    font-family: monospace;
    font-size: 14px;
  }
  
  .markdown-help {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow-y: auto;
    max-height: 200px;
  }
  
  .markdown-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px;
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.02);
  }
  
  code {
    font-family: monospace;
    background-color: rgba(0, 0, 0, 0.05);
    padding: 2px 4px;
    border-radius: 3px;
    font-size: 13px;
  }
  
  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 20px;
    font-size: 14px;
    opacity: 0.7;
  }
  
  .close-btn {
    background-color: transparent;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  @keyframes slideIn {
    from { transform: translateY(20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
</style>