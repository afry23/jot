<script lang="ts">
    import { notes } from '$lib/stores/notes';
    import { activeTab } from '$lib/stores/tabs';
    import { formatMarkdown } from '$lib/utils/textFormatting';
  
    let tabColors = [
      '#F6C046', // Yellow
      '#F0874F', // Orange 
      '#F25D66', // Red
      '#7D7E8A', // Gray
      '#7E7E9A', // Blue-Gray
      '#9D7E9A', // Purple
      '#7E9D7E'  // Green
    ];
  
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
    // Get content for the current tab
    $: currentContent = $notes[$activeTab] || '';
    // Format the content for preview
    $: formattedContent = formatMarkdown(currentContent);
  </script>
  
  <div class="preview-container" style="--tab-color: {tabColors[$activeTab]};">
    <div class="preview-content">
      {@html formattedContent}
    </div>
  </div>
  
  <style>
    .preview-container {
      height: 100%;
      overflow-y: auto;
      padding: 20px;
    }
  
    .preview-content {
      line-height: 1.6;
      color: var(--text-color);
      font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
    }
  
    /* Style for headings in preview */
    .preview-content :global(h1),
    .preview-content :global(h2),
    .preview-content :global(h3) {
      color: var(--tab-color);
      margin-top: 1em;
      margin-bottom: 0.5em;
      font-weight: 600;
    }
  
    .preview-content :global(h1) {
      font-size: 1.8em;
    }
  
    .preview-content :global(h2) {
      font-size: 1.5em;
    }
  
    .preview-content :global(h3) {
      font-size: 1.2em;
    }
  
    /* Style for paragraphs */
    .preview-content :global(p) {
      margin-bottom: 1em;
    }
  
    /* Style for lists */
    .preview-content :global(ul), 
    .preview-content :global(ol) {
      margin-left: 2em;
      margin-bottom: 1em;
    }
  
    .preview-content :global(li) {
      margin-bottom: 0.5em;
    }
  
    /* Style for code blocks */
    .preview-content :global(pre) {
      background-color: rgba(0, 0, 0, 0.05);
      padding: 1em;
      border-radius: 4px;
      overflow-x: auto;
      margin-bottom: 1em;
      font-family: monospace;
    }
  
    .preview-content :global(code) {
      font-family: monospace;
      background-color: rgba(0, 0, 0, 0.05);
      padding: 0.2em 0.4em;
      border-radius: 3px;
    }
  
    /* Style for blockquotes */
    .preview-content :global(blockquote) {
      border-left: 4px solid var(--tab-color);
      padding-left: 1em;
      margin-left: 0;
      margin-bottom: 1em;
      font-style: italic;
    }
  
    /* Style for horizontal rules */
    .preview-content :global(hr) {
      border: none;
      border-top: 2px solid rgba(0, 0, 0, 0.1);
      margin: 1.5em 0;
    }
  
    /* Style for links */
    .preview-content :global(a) {
      color: var(--tab-color);
      text-decoration: none;
    }
  
    .preview-content :global(a:hover) {
      text-decoration: underline;
    }
  
    /* Style for bold and italic text */
    .preview-content :global(strong) {
      font-weight: 600;
    }
  
    .preview-content :global(em) {
      font-style: italic;
    }
  
    /* Dark theme adjustments */
    :global(.dark) .preview-content :global(pre),
    :global(.dark) .preview-content :global(code) {
      background-color: rgba(255, 255, 255, 0.1);
    }
  
    :global(.dark) .preview-content :global(hr) {
      border-top: 2px solid rgba(255, 255, 255, 0.1);
    }
  </style>