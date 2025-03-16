<script lang="ts">
    import { activeTab } from '$lib/stores/tabs';
    import { notes } from '$lib/stores/notes';
    import { theme, toggleTheme } from '$lib/stores/settings';
    
    // Calculate text statistics
    $: currentContent = $notes[$activeTab] || '';
    $: plainText = currentContent.replace(/<[^>]*>/g, '');
    $: characterCount = plainText.length;
    $: wordCount = plainText.trim().split(/\s+/).filter(word => word.length > 0).length;
    $: lineCount = plainText.split('\n').length;
    
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
  
  <footer>
    <div class="tab-indicator" style="--tab-color: {tabColors[$activeTab]};"></div>
    
    <div class="stats">
      <span>{lineCount} lines</span>
      <span>•</span>
      <span>{wordCount} words</span>
      <span>•</span>
      <span>{characterCount} characters</span>
    </div>
    
    <div class="actions">
      <button class="theme-toggle" on:click={toggleTheme}>
        {$theme === 'dark' ? '☀️' : '🌙'}
      </button>
      <button class="font-toggle">
        A
      </button>
    </div>
  </footer>
  
  <style>
    footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 15px;
      background-color: rgba(0, 0, 0, 0.07);
      border-top: 1px solid rgba(0, 0, 0, 0.1);
      font-size: 14px;
    }
  
    .app.dark footer {
      background-color: rgba(0, 0, 0, 0.2);
      border-top: 1px solid rgba(255, 255, 255, 0.1);
    }
  
    .tab-indicator {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background-color: var(--tab-color);
    }
  
    .stats {
      display: flex;
      gap: 8px;
      color: var(--text-color);
      opacity: 0.7;
    }
  
    .actions {
      display: flex;
      gap: 8px;
    }
  
    button {
      background: none;
      border: none;
      cursor: pointer;
      width: 24px;
      height: 24px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      opacity: 0.7;
      transition: opacity 0.2s;
    }
  
    button:hover {
      opacity: 1;
    }
  </style>