<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { marked } from 'marked';
    import { writable } from 'svelte/store';
    
    // Import DOMPurify conditionally for SSR compatibility
    let DOMPurify: any;
    
    // Props with default values
    export let initialContent: string = '';
    export let placeholder: string = 'Start writing...';
    export let maxHeight: string = '80vh';
    export let autoSave: boolean = true;
    export let storageKey: string = 'minimalist-editor-content';
    export let theme: 'light' | 'dark' = 'light';
    
    // Internal stores
    const contentStore = writable(initialContent);
    const isDirty = writable(false);
    
    // State
    let content: string = initialContent;
    let isMarkdownMode: boolean = false;
    let editor: HTMLTextAreaElement;
    let previewElement: HTMLDivElement;
    let renderedContent: string = '';
    let isSyncing: boolean = false;
    let lastSaved: Date | null = null;
    let stats = {
      characters: 0,
      words: 0,
      lines: 0,
      readingTime: 0
    };
    
    // Autosave timer
    let autosaveInterval: number;
    
    // Sync content with store
    $: {
      $contentStore = content;
      isDirty.set(true);
    }
    
    // Marked configuration
    onMount(async () => {
      // Import DOMPurify dynamically on the client side
      if (typeof window !== 'undefined') {
        const domPurifyModule = await import('dompurify');
        DOMPurify = domPurifyModule.default;
      }
      
      // Set marked options
      marked.setOptions({
        gfm: true,
        breaks: true
      });
      
      // Focus editor
      if (editor) {
        editor.focus();
      }
      
      // Load content from storage if enabled
      if (autoSave) {
        const savedContent = localStorage.getItem(storageKey);
        if (savedContent) {
          content = savedContent;
          isDirty.set(false);
          lastSaved = new Date();
        }
        
        // Setup autosave
        autosaveInterval = window.setInterval(() => {
          if ($isDirty) {
            saveContent();
          }
        }, 5000); // Save every 5 seconds when content changes
      }
      
      // Set initial rendered content
      if (content) {
        updateRenderedContent();
        updateStats(content);
      }
      
      // Sync scroll positions between edit and preview
      if (editor && previewElement) {
        editor.addEventListener('scroll', syncScrollPosition);
      }
    });
    
    onDestroy(() => {
      if (autosaveInterval) {
        clearInterval(autosaveInterval);
      }
      
      if (editor && previewElement) {
        editor.removeEventListener('scroll', syncScrollPosition);
      }
    });
    
    // Update rendered markdown whenever content changes
    $: if (content !== undefined) {
      updateRenderedContent();
      updateStats(content);
      isDirty.set(true);
    }
    
    function updateRenderedContent() {
      if (content) {
        // Handle both synchronous and asynchronous returns from marked
        const result = marked(content, {
          gfm: true,
          breaks: true
        });
        
        if (result instanceof Promise) {
          result.then(html => {
            // Sanitize the HTML to prevent XSS (only if DOMPurify is available)
            renderedContent = typeof DOMPurify !== 'undefined' ? DOMPurify.sanitize(html) : html;
          });
        } else {
          // Sanitize the HTML to prevent XSS (only if DOMPurify is available)
          renderedContent = typeof DOMPurify !== 'undefined' ? DOMPurify.sanitize(result) : result;
        }
      } else {
        renderedContent = '';
      }
    }
    
    // Calculate text statistics
    function updateStats(text: string) {
      stats.characters = text.length;
      stats.words = text.trim() === '' ? 0 : text.trim().split(/\s+/).length;
      stats.lines = text === '' ? 0 : text.split('\n').length;
      // Calculate reading time (average reading speed: 200 words per minute)
      stats.readingTime = Math.ceil(stats.words / 200);
    }
    
    // Handle tab key in the editor
    function handleKeydown(event: KeyboardEvent) {
      // Tab key
      if (event.key === 'Tab') {
        event.preventDefault();
        
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        
        // If text is selected, indent or unindent the selection
        if (start !== end) {
          const selectedText = content.substring(start, end);
          
          if (event.shiftKey) {
            // Unindent selected lines
            const unindented = selectedText
              .split('\n')
              .map(line => line.startsWith('  ') ? line.substring(2) : line)
              .join('\n');
            
            content = content.substring(0, start) + unindented + content.substring(end);
            
            // Adjust selection
            setTimeout(() => {
              editor.selectionStart = start;
              editor.selectionEnd = start + unindented.length;
            }, 0);
          } else {
            // Indent selected lines
            const indented = selectedText
              .split('\n')
              .map(line => '  ' + line)
              .join('\n');
            
            content = content.substring(0, start) + indented + content.substring(end);
            
            // Adjust selection
            setTimeout(() => {
              editor.selectionStart = start;
              editor.selectionEnd = start + indented.length;
            }, 0);
          }
        } else {
          // No selection, just insert tab at cursor position
          content = content.substring(0, start) + '  ' + content.substring(end);
          
          // Move cursor after the inserted tab
          setTimeout(() => {
            editor.selectionStart = editor.selectionEnd = start + 2;
          }, 0);
        }
      }
      
      // Ctrl/Cmd+S to save
      if ((event.ctrlKey || event.metaKey) && event.key === 's') {
        event.preventDefault();
        saveContent();
      }
      
      // Ctrl/Cmd+M to toggle mode
      if ((event.ctrlKey || event.metaKey) && event.key === 'm') {
        event.preventDefault();
        toggleMode();
      }
    }
    
    // Save content to localStorage
    function saveContent() {
      if (autoSave) {
        localStorage.setItem(storageKey, content);
        lastSaved = new Date();
        isDirty.set(false);
      }
    }
    
    // Format relative time
    function formatRelativeTime(date: Date | null): string {
      if (!date) return 'Never';
      
      const now = new Date();
      const diffSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);
      
      if (diffSeconds < 60) return `${diffSeconds} seconds ago`;
      if (diffSeconds < 3600) return `${Math.floor(diffSeconds / 60)} minutes ago`;
      
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
    
    // Toggle between plain text and markdown mode
    function toggleMode() {
      isMarkdownMode = !isMarkdownMode;
    }
    
    // Sync scroll positions between edit and preview
    function syncScrollPosition(event: Event) {
      if (isSyncing || !isMarkdownMode) return;
      
      isSyncing = true;
      
      const textareaPercentage = editor.scrollTop / (editor.scrollHeight - editor.clientHeight);
      const previewScrollMax = previewElement.scrollHeight - previewElement.clientHeight;
      
      previewElement.scrollTop = Math.round(textareaPercentage * previewScrollMax);
      
      setTimeout(() => {
        isSyncing = false;
      }, 100);
    }
  </script>
  
  <div class="minimalist-editor-container" class:dark-theme={theme === 'dark'}>
    <div class="editor-header">
      <div class="left-controls">
        {#if autoSave}
          <div class="save-status">
            {#if $isDirty}
              <span class="unsaved">Unsaved changes</span>
            {:else}
              <span class="saved">Saved {lastSaved ? formatRelativeTime(lastSaved) : ''}</span>
            {/if}
          </div>
        {/if}
      </div>
      
      <div class="right-controls">
        <button 
          class="mode-toggle" 
          on:click={toggleMode} 
          aria-label={isMarkdownMode ? "Switch to edit mode" : "Switch to preview mode"}
          title={isMarkdownMode ? "Switch to edit mode (Ctrl+M)" : "Switch to preview mode (Ctrl+M)"}
        >
          {isMarkdownMode ? 'Edit' : 'Preview'}
        </button>
      </div>
    </div>
    
    <div class="editor-content" style="max-height: {maxHeight}">
      <!-- Plain text editing mode (always exists but may be hidden) -->
      <textarea
        bind:this={editor}
        bind:value={content}
        on:keydown={handleKeydown}
        placeholder={placeholder}
        spellcheck="true"
        aria-label="Text editor"
        class:hidden={isMarkdownMode}
      ></textarea>
      
      <!-- Markdown preview (read-only) -->
      {#if isMarkdownMode}
        <div 
          bind:this={previewElement}
          class="markdown-preview"
        >
          {#if content.trim() === ''}
            <div class="placeholder">{placeholder}</div>
          {:else}
            {@html renderedContent}
          {/if}
        </div>
      {/if}
    </div>
    
    <div class="editor-footer">
      <div class="stats">
        <span title="Character count">{stats.characters} character{stats.characters !== 1 ? 's' : ''}</span>
        <span title="Word count">{stats.words} word{stats.words !== 1 ? 's' : ''}</span>
        <span title="Line count">{stats.lines} line{stats.lines !== 1 ? 's' : ''}</span>
        {#if stats.words > 0}
          <span title="Estimated reading time">~{stats.readingTime} min read</span>
        {/if}
      </div>
      
      <div class="actions">
        {#if autoSave}
          <button 
            class="action-button"
            on:click={saveContent}
            disabled={!$isDirty}
            title="Save (Ctrl+S)"
          >
            Save
          </button>
        {/if}
      </div>
    </div>
  </div>
  
  <style>
    .minimalist-editor-container {
      display: flex;
      flex-direction: column;
      background-color: #ffffff;
      border-radius: 8px;
      box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
      width: 100%;
      height: 100%;
      overflow: hidden;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
        Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
      transition: background-color 0.3s, color 0.3s;
    }
    
    /* Dark theme */
    .dark-theme {
      background-color: #1e1e1e;
      color: #e0e0e0;
      box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
    }
    
    .editor-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 16px;
      border-bottom: 1px solid #f0f0f0;
    }
    
    .dark-theme .editor-header {
      border-bottom-color: #333;
    }
    
    .left-controls, .right-controls {
      display: flex;
      align-items: center;
    }
    
    .save-status {
      font-size: 12px;
      margin-right: 16px;
    }
    
    .unsaved {
      color: #e67e22;
    }
    
    .saved {
      color: #777;
    }
    
    .dark-theme .saved {
      color: #aaa;
    }
    
    .mode-toggle {
      background-color: #f5f5f5;
      border: none;
      border-radius: 4px;
      padding: 6px 12px;
      font-size: 14px;
      cursor: pointer;
      transition: background-color 0.2s;
      color: #333;
    }
    
    .dark-theme .mode-toggle {
      background-color: #333;
      color: #e0e0e0;
    }
    
    .mode-toggle:hover {
      background-color: #e9e9e9;
    }
    
    .dark-theme .mode-toggle:hover {
      background-color: #444;
    }
    
    .editor-content {
      flex-grow: 1;
      overflow-y: auto;
      position: relative;
    }
    
    textarea {
      width: 100%;
      height: 100%;
      border: none;
      padding: 20px;
      resize: none;
      font-family: 'SF Mono', SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
      font-size: 16px;
      line-height: 1.5;
      color: #333;
      outline: none;
      background-color: transparent;
    }
    
    .dark-theme textarea {
      color: #e0e0e0;
    }
    
    .hidden {
      display: none;
    }
    
    .markdown-preview {
      padding: 20px;
      line-height: 1.6;
      font-size: 16px;
      color: #333;
      height: 100%;
      overflow-y: auto;
    }
    
    .dark-theme .markdown-preview {
      color: #e0e0e0;
    }
    
    .placeholder {
      color: #aaa;
      font-style: italic;
    }
    
    .dark-theme .placeholder {
      color: #666;
    }
    
    .editor-footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 8px 16px;
      border-top: 1px solid #f0f0f0;
      font-size: 12px;
      color: #999;
    }
    
    .dark-theme .editor-footer {
      border-top-color: #333;
      color: #aaa;
    }
    
    .stats {
      display: flex;
      gap: 16px;
    }
    
    .actions {
      display: flex;
      gap: 8px;
    }
    
    .action-button {
      background-color: transparent;
      border: 1px solid #ddd;
      border-radius: 4px;
      padding: 4px 8px;
      font-size: 12px;
      cursor: pointer;
      color: #555;
      transition: all 0.2s;
    }
    
    .dark-theme .action-button {
      border-color: #444;
      color: #aaa;
    }
    
    .action-button:hover:not(:disabled) {
      background-color: #f5f5f5;
      color: #333;
    }
    
    .dark-theme .action-button:hover:not(:disabled) {
      background-color: #333;
      color: #fff;
    }
    
    .action-button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    
    /* Markdown styles */
    .markdown-preview :global(h1) {
      font-size: 1.8em;
      margin-top: 0.5em;
      margin-bottom: 0.5em;
      border-bottom: 1px solid #eee;
      padding-bottom: 0.3em;
    }
    
    .dark-theme .markdown-preview :global(h1) {
      border-bottom-color: #333;
    }
    
    .markdown-preview :global(h2) {
      font-size: 1.5em;
      margin-top: 0.5em;
      margin-bottom: 0.5em;
    }
    
    .markdown-preview :global(h3) {
      font-size: 1.3em;
      margin-top: 0.5em;
      margin-bottom: 0.5em;
    }
    
    .markdown-preview :global(p) {
      margin-bottom: 1em;
    }
    
    .markdown-preview :global(ul), 
    .markdown-preview :global(ol) {
      margin-bottom: 1em;
      margin-left: 2em;
    }
    
    .markdown-preview :global(blockquote) {
      border-left: 3px solid #eee;
      padding-left: 1em;
      margin-left: 0;
      color: #777;
    }
    
    .dark-theme .markdown-preview :global(blockquote) {
      border-left-color: #444;
      color: #aaa;
    }
    
    .markdown-preview :global(code) {
      font-family: 'SF Mono', SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
      background-color: #f5f5f5;
      padding: 0.2em 0.4em;
      border-radius: 3px;
      font-size: 85%;
    }
    
    .dark-theme .markdown-preview :global(code) {
      background-color: #2d2d2d;
    }
    
    .markdown-preview :global(pre) {
      background-color: #f5f5f5;
      padding: 1em;
      border-radius: 5px;
      overflow-x: auto;
      margin-bottom: 1em;
    }
    
    .dark-theme .markdown-preview :global(pre) {
      background-color: #2d2d2d;
    }
    
    .markdown-preview :global(pre code) {
      background-color: transparent;
      padding: 0;
      border-radius: 0;
    }
    
    .markdown-preview :global(a) {
      color: #0366d6;
      text-decoration: none;
    }
    
    .dark-theme .markdown-preview :global(a) {
      color: #58a6ff;
    }
    
    .markdown-preview :global(a:hover) {
      text-decoration: underline;
    }
    
    .markdown-preview :global(table) {
      border-collapse: collapse;
      margin: 1em 0;
      overflow-x: auto;
      display: block;
      width: 100%;
    }
    
    .markdown-preview :global(table th),
    .markdown-preview :global(table td) {
      border: 1px solid #ddd;
      padding: 8px 12px;
    }
    
    .dark-theme .markdown-preview :global(table th),
    .dark-theme .markdown-preview :global(table td) {
      border-color: #444;
    }
    
    .markdown-preview :global(table th) {
      padding-top: 12px;
      padding-bottom: 12px;
      text-align: left;
      background-color: #f8f8f8;
    }
    
    .dark-theme .markdown-preview :global(table th) {
      background-color: #333;
    }
    
    .markdown-preview :global(hr) {
      border: 0;
      height: 1px;
      background-color: #eee;
      margin: 1.5em 0;
    }
    
    .dark-theme .markdown-preview :global(hr) {
      background-color: #444;
    }
    
    .markdown-preview :global(img) {
      max-width: 100%;
      height: auto;
    }
    
    /* Responsive adjustments */
    @media (max-width: 600px) {
      .editor-header, .editor-footer {
        padding: 8px;
      }
      
      .editor-content {
        padding: 0;
      }
      
      textarea, .markdown-preview {
        font-size: 14px;
        padding: 12px;
      }
      
      .stats {
        gap: 8px;
        font-size: 11px;
      }
    }
  </style>
  