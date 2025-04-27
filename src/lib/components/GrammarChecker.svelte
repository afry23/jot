<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { notes, notesLoaded } from "$lib/stores/notes";
  import { activeTab } from "$lib/stores/tabs";
  import { theme } from "$lib/stores/settings";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import {
    grammarCheckResults,
    checkGrammar,
    isGrammarCheckInProgress,
    clearGrammarCheckResults,
    grammarCheckError,
    type GrammarError,
    type GrammarCheckResult,
  } from "$lib/stores/languageServices";

  // Props
  export let onClose: () => void;

  // Local state
  let currentContent: string = "";
  let selectedLanguage: string = "auto";
  let debounceTimer: number | null = null;
  let localMatches: GrammarError[] = []; // Local copy of matches to modify without rechecking
  let lastCheckedContent: string = ""; // Keep track of last checked content
  let contentChanged = false;

  // Language options - valid LanguageTool language codes
  const languageOptions = [
    { value: "auto", label: "Auto-detect" },
    { value: "en-US", label: "English (US)" },
    { value: "en-GB", label: "English (UK)" },
    { value: "fr", label: "French" },
    { value: "de", label: "German" },
    { value: "es", label: "Spanish" },
    { value: "pt", label: "Portuguese" },
    { value: "it", label: "Italian" },
    { value: "nl", label: "Dutch" },
    { value: "pl", label: "Polish" },
    { value: "ru", label: "Russian" },
  ];

  // Handle grammar check results
  function handleGrammarResults(results: GrammarCheckResult) {
    if (results && results.matches) {
      localMatches = [...results.matches];
      lastCheckedContent = currentContent;
    }
  }

  // Subscribe to grammar check results
  $: if ($grammarCheckResults && $grammarCheckResults.matches) {
    handleGrammarResults($grammarCheckResults);
  }

  // Subscribe to active tab content changes
  $: if ($notesLoaded && $notes[$activeTab] !== undefined) {
    const newContent = $notes[$activeTab] || "";

    // Only update if content has changed to avoid unnecessary rerendering
    if (newContent !== currentContent) {
      currentContent = newContent;
      contentChanged = true;
    }
  }

  // Handle content changes separately to avoid circular dependency
  $: if (contentChanged) {
    contentChanged = false;

    // If content changes significantly, run a new check
    if (shouldRecheck(currentContent, lastCheckedContent)) {
      // Debounce grammar checking to avoid too many API calls
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }

      debounceTimer = setTimeout(() => {
        if (currentContent) {
          runGrammarCheck();
        } else {
          clearGrammarCheckResults();
          localMatches = [];
        }
      }, 1500);
    }
  }

  // Determine if we should recheck based on content changes
  function shouldRecheck(newContent: string, oldContent: string): boolean {
    // If there's no old content or it's substantially different (>30 chars), recheck
    if (!oldContent || Math.abs(newContent.length - oldContent.length) > 30) {
      return true;
    }

    // If a paragraph was added or removed, recheck
    const oldParagraphs = oldContent.split("\n").length;
    const newParagraphs = newContent.split("\n").length;
    if (Math.abs(oldParagraphs - newParagraphs) > 0) {
      return true;
    }

    // Default to not rechecking for small changes
    return false;
  }

  // Run grammar check on current content
  async function runGrammarCheck() {
    if (!currentContent.trim()) {
      clearGrammarCheckResults();
      localMatches = [];
      return;
    }

    try {
      const result = await checkGrammar(currentContent, selectedLanguage);
      handleGrammarResults(result);
    } catch (error) {
      console.error("Grammar check error:", error);
    }
  }

  // Apply suggestion to text without rechecking
  function applySuggestion(error: GrammarError, replacement: string) {
    if (!currentContent) return;

    const before = currentContent.substring(0, error.offset);
    const after = currentContent.substring(error.offset + error.length);

    // Update the content with the replacement
    const newContent = before + replacement + after;

    // Update the store
    notes.update((state) => {
      state[$activeTab] = newContent;
      return state;
    });

    // Update current content to match
    currentContent = newContent;
    contentChanged = true;

    // Remove this error from the local matches
    localMatches = localMatches.filter(
      (match) =>
        // Filter out the current error
        !(match.offset === error.offset && match.length === error.length)
    );

    // Adjust offsets for remaining errors that come after this replacement
    // This ensures positions remain correct for subsequent corrections
    const lengthDifference = replacement.length - error.length;
    if (lengthDifference !== 0) {
      localMatches = localMatches.map((match) => {
        if (match.offset > error.offset) {
          // Adjust the offset of subsequent errors
          return {
            ...match,
            offset: match.offset + lengthDifference,
          };
        }
        return match;
      });
    }
  }

  // Handle language selection change
  function handleLanguageChange() {
    runGrammarCheck();
  }

  onMount(() => {
    if (currentContent) {
      runGrammarCheck();
    }
  });

  onDestroy(() => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
  });
</script>

<div class="grammar-checker" class:dark-theme={$theme === "dark"}>
  <div class="header">
    <h2>
      <FontAwesomeIcon icon="check-circle" />
      <span>Grammar Checker</span>
    </h2>

    <div class="actions">
      <select
        bind:value={selectedLanguage}
        on:change={handleLanguageChange}
        title="Select language"
      >
        {#each languageOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>

      <button
        on:click={runGrammarCheck}
        disabled={$isGrammarCheckInProgress}
        title="Check grammar"
      >
        <FontAwesomeIcon
          icon={$isGrammarCheckInProgress ? "spinner" : "sync"}
          spin={$isGrammarCheckInProgress}
        />
      </button>

      <button class="close-btn" on:click={onClose} title="Close">
        <FontAwesomeIcon icon="times" />
      </button>
    </div>
  </div>

  <div class="checker-content">
    {#if $isGrammarCheckInProgress}
      <div class="loading">
        <FontAwesomeIcon icon="spinner" spin />
        <span>Checking grammar...</span>
      </div>
    {:else if $grammarCheckError}
      <div class="error-message">
        <FontAwesomeIcon icon="exclamation-triangle" />
        <span>{$grammarCheckError}</span>
        <div class="error-help">
          <p>Tips to resolve this issue:</p>
          <ul>
            <li>Make sure you've selected the correct language</li>
            <li>Try with a shorter text sample</li>
            <li>Check your API key in Language Settings</li>
            <li>The free LanguageTool API has request limits</li>
          </ul>
        </div>
      </div>
    {:else if localMatches.length === 0}
      <div class="no-issues">
        <FontAwesomeIcon icon="check-circle" />
        <span>No grammar issues found</span>
      </div>
    {:else}
      <div class="issues-list">
        <h3>Suggestions ({localMatches.length})</h3>
        {#each localMatches as error, i}
          <div class="issue-item">
            <div class="issue-header">
              <span class="issue-type"
                >{error.rule ? error.rule.issueType : "Issue"}</span
              >
              <span class="issue-message">{error.message}</span>
            </div>
            <div class="context">
              <span class="context-text">
                {currentContent.substring(
                  Math.max(0, error.offset - 15),
                  error.offset
                )}
                <mark
                  >{currentContent.substring(
                    error.offset,
                    error.offset + error.length
                  )}</mark
                >
                {currentContent.substring(
                  error.offset + error.length,
                  error.offset + error.length + 15
                )}...
              </span>
            </div>
            {#if error.replacements && error.replacements.length > 0}
              <div class="suggestions">
                {#each error.replacements.slice(0, 3) as replacement}
                  <button
                    class="suggestion-btn"
                    on:click={() => applySuggestion(error, replacement.value)}
                  >
                    {replacement.value}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="footer">
    <div class="refresh-hint">
      <FontAwesomeIcon icon="sync" />
      <span>Click refresh to recheck text</span>
    </div>
    <span class="powered-by">Powered by LanguageTool</span>
  </div>
</div>

<style>
  .grammar-checker {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 350px;
    background-color: white;
    border-left: 1px solid #ddd;
    display: flex;
    flex-direction: column;
    z-index: 10;
    box-shadow: -2px 0 10px rgba(0, 0, 0, 0.1);
    animation: slideIn 0.3s ease;
  }

  .dark-theme {
    background-color: #333;
    color: #e0e0e0;
    border-left: 1px solid #555;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 15px;
    border-bottom: 1px solid #eee;
  }

  .dark-theme .header {
    border-bottom: 1px solid #555;
  }

  h2 {
    margin: 0;
    font-size: 16px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  select {
    padding: 6px;
    border-radius: 4px;
    border: 1px solid #ddd;
    background-color: #f5f5f5;
    font-size: 14px;
  }

  .dark-theme select {
    background-color: #444;
    border-color: #555;
    color: #e0e0e0;
  }

  button {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    color: inherit;
    opacity: 0.8;
    transition: all 0.2s;
  }

  button:hover:not(:disabled) {
    opacity: 1;
    background-color: rgba(0, 0, 0, 0.05);
  }

  .dark-theme button:hover:not(:disabled) {
    background-color: rgba(255, 255, 255, 0.1);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checker-content {
    flex: 1;
    overflow-y: auto;
    padding: 15px;
  }

  .loading,
  .no-issues,
  .error-message {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100px;
    gap: 10px;
    color: #666;
    flex-direction: column;
  }

  .dark-theme .loading,
  .dark-theme .no-issues,
  .dark-theme .error-message {
    color: #aaa;
  }

  .error-message {
    color: #e74c3c;
    align-items: flex-start;
  }

  .dark-theme .error-message {
    color: #ff7675;
  }

  .error-help {
    margin-top: 15px;
    padding: 12px;
    border-radius: 6px;
    background-color: rgba(231, 76, 60, 0.1);
    font-size: 14px;
    width: 100%;
  }

  .error-help p {
    margin: 0 0 8px 0;
    font-weight: 500;
  }

  .error-help ul {
    margin: 0;
    padding-left: 20px;
  }

  .error-help li {
    margin-bottom: 4px;
  }

  .dark-theme .error-help {
    background-color: rgba(231, 76, 60, 0.2);
  }

  .issues-list {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  h3 {
    margin: 0 0 10px 0;
    font-size: 15px;
    color: #666;
  }

  .dark-theme h3 {
    color: #aaa;
  }

  .issue-item {
    padding: 10px;
    border-radius: 4px;
    background-color: #f8f8f8;
    border: 1px solid #eee;
  }

  .dark-theme .issue-item {
    background-color: #3a3a3a;
    border-color: #555;
  }

  .issue-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .issue-type {
    font-size: 12px;
    background-color: #e74c3c;
    color: white;
    padding: 2px 6px;
    border-radius: 3px;
    text-transform: capitalize;
  }

  .issue-message {
    font-size: 14px;
    color: #333;
  }

  .dark-theme .issue-message {
    color: #ddd;
  }

  .context {
    font-size: 13px;
    margin-bottom: 8px;
    line-height: 1.4;
    color: #666;
    background-color: #f0f0f0;
    padding: 8px;
    border-radius: 3px;
  }

  .dark-theme .context {
    background-color: #444;
    color: #ccc;
  }

  mark {
    background-color: rgba(231, 76, 60, 0.2);
    text-decoration: wavy underline #e74c3c;
    padding: 0 2px;
  }

  .dark-theme mark {
    background-color: rgba(231, 76, 60, 0.3);
  }

  .suggestions {
    display: flex;
    gap: 5px;
    flex-wrap: wrap;
  }

  .suggestion-btn {
    width: auto;
    height: auto;
    padding: 3px 8px;
    font-size: 13px;
    background-color: #3498db;
    color: white;
    border-radius: 3px;
    opacity: 1;
  }

  .suggestion-btn:hover {
    background-color: #2980b9;
  }

  .dark-theme .suggestion-btn {
    background-color: #2980b9;
  }

  .dark-theme .suggestion-btn:hover {
    background-color: #3498db;
  }

  .footer {
    padding: 10px;
    border-top: 1px solid #eee;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dark-theme .footer {
    border-top: 1px solid #555;
  }

  .refresh-hint {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: #888;
  }

  .dark-theme .refresh-hint {
    color: #777;
  }

  .powered-by {
    font-size: 11px;
    color: #999;
  }

  .dark-theme .powered-by {
    color: #777;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
</style>
