<script lang="ts">
  import { activeTab } from '$lib/stores/tabs';
  import { notes } from '$lib/stores/notes';
  import { theme, toggleTheme } from '$lib/stores/settings';
  
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
  
  // Count words properly using DOM parsing
  function countWords(html: string): number {
    // Create a temporary DOM element to parse the HTML
    const tempElement = document.createElement('div');
    tempElement.innerHTML = html;
    
    // Get the text content (without HTML tags)
    const text = tempElement.textContent || '';
    
    // Count words by splitting on whitespace
    return text.trim().split(/\s+/).filter(word => word.length > 0).length;
  }
  
  // Count characters properly using DOM parsing
  function countCharacters(html: string): number {
    // Create a temporary DOM element to parse the HTML
    const tempElement = document.createElement('div');
    tempElement.innerHTML = html;
    
    // Get the text content (without HTML tags)
    const text = tempElement.textContent || '';
    
    // Return the character count
    return text.length;
  }
  
  // Count lines by counting block elements
  function countLines(html: string): number {
    // Create a temporary DOM element to parse the HTML
    const tempElement = document.createElement('div');
    tempElement.innerHTML = html;
    
    // Count block elements that create new lines
    const blockElements = tempElement.querySelectorAll('p, div, li, h1, h2, h3, h4, h5, h6, blockquote, pre, br');
    
    // Count <br> tags separately
    const brTags = tempElement.querySelectorAll('br');
    
    // Handle case where content exists but no block elements
    if (blockElements.length === 0) {
      // If there's content but no block elements, count as 1 line
      return tempElement.textContent?.trim() ? 1 : 0;
    }
    
    // Return the count of block elements plus <br> tags
    // We subtract the br tags count once since they're already counted in blockElements
    //return blockElements.length - brTags.length + (brTags.length > 0 ? brTags.length + 1 : 0);
    return blockElements.length;
  }
  
  // Compute the statistics
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