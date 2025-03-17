<script lang="ts">
  import { activeTab } from '$lib/stores/tabs';
  import { notes } from '$lib/stores/notes';
  import { theme, toggleTheme } from '$lib/stores/settings';
  import { viewMode, toggleViewMode } from '$lib/stores/viewMode';
  import { countWords, countCharacters, countLines } from '$lib/utils/textFormatting';
  
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
  
  // Calculate text statistics
  $: currentContent = $notes[$activeTab] || '';
  $: wordCount = countWords(currentContent);
  $: characterCount = countCharacters(currentContent);
  $: lineCount = countLines(currentContent);
</script>

<footer>
  <div class="tab-indicator" style="--tab-color: {tabColors[$activeTab]};"></div>
  
  <div class="stats">
    <span>{lineCount} {lineCount === 1 ? 'line' : 'lines'}</span>
    <span>•</span>
    <span>{wordCount} {wordCount === 1 ? 'word' : 'words'}</span>
    <span>•</span>
    <span>{characterCount} {characterCount === 1 ? 'character' : 'characters'}</span>
  </div>
  
  <div class="actions">
    <button class="view-toggle" on:click={toggleViewMode} title="Toggle view mode">
      {$viewMode === 'edit' ? '👁️' : '✏️'}
    </button>
    <button class="theme-toggle" on:click={toggleTheme} title="Toggle theme">
      {$theme === 'dark' ? '☀️' : '🌙'}
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