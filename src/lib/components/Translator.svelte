<script lang="ts">
  import { onMount } from "svelte";
  import { notes } from "$lib/stores/notes";
  import { activeTab } from "$lib/stores/tabs";
  import { theme } from "$lib/stores/settings";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import {
    translateText,
    isTranslationInProgress,
    translationResult,
    translationError,
    clearTranslationResults,
    loadLanguageServicesConfig,
  } from "$lib/stores/languageServices";

  // Props
  export let onClose: () => void;

  // Local state
  let selectedText: string = "";
  let targetLanguage: string = "EN-US";
  let sourceLanguage: string = "auto";
  let showSettings: boolean = false;
  let apiConfigured: boolean = false;

  // Language options
  const targetLanguageOptions = [
    { value: "EN-US", label: "English (US)" },
    { value: "EN-GB", label: "English (UK)" },
    { value: "FR", label: "French" },
    { value: "DE", label: "German" },
    { value: "ES", label: "Spanish" },
    { value: "PT-PT", label: "Portuguese" },
    { value: "IT", label: "Italian" },
    { value: "NL", label: "Dutch" },
    { value: "PL", label: "Polish" },
    { value: "RU", label: "Russian" },
    { value: "JA", label: "Japanese" },
    { value: "ZH", label: "Chinese" },
  ];

  const sourceLanguageOptions = [
    { value: "auto", label: "Auto-detect" },
    ...targetLanguageOptions,
  ];

  // Initialize with current content
  onMount(async () => {
    // Check if API is configured
    await loadLanguageServicesConfig();

    // Get selection or full text
    const selection = window.getSelection();
    if (selection && selection.toString()) {
      selectedText = selection.toString();
    } else if ($notes[$activeTab]) {
      selectedText = $notes[$activeTab];
    }
  });

  // Run translation
  async function runTranslation() {
    if (!selectedText.trim()) {
      clearTranslationResults();
      return;
    }

    await translateText(
      selectedText,
      targetLanguage,
      sourceLanguage !== "auto" ? sourceLanguage : undefined
    );
  }

  // Replace selected text with translation
  function applyTranslation() {
    if (!$translationResult) return;

    const selection = window.getSelection();
    if (selection && selection.toString()) {
      // Replace just the selected text
      const range = selection.getRangeAt(0);
      const start = range.startOffset;
      const end = range.endOffset;

      const parentElement = range.startContainer.parentElement;
      if (parentElement && parentElement.tagName === "TEXTAREA") {
        const textarea = parentElement as HTMLTextAreaElement;
        const currentValue = textarea.value;

        textarea.value =
          currentValue.substring(0, start) +
          $translationResult +
          currentValue.substring(end);
      } else {
        // If not directly in a textarea, replace the full content
        notes.update((state) => {
          state[$activeTab] = state[$activeTab].replace(
            selectedText,
            $translationResult
          );
          return state;
        });
      }
    } else {
      // Replace the full note content
      notes.update((state) => {
        state[$activeTab] = $translationResult;
        return state;
      });
    }

    onClose();
  }

  // Swap source and target languages
  function swapLanguages() {
    if (sourceLanguage !== "auto") {
      const temp = sourceLanguage;
      sourceLanguage = targetLanguage;
      targetLanguage = temp;
    }
  }
</script>

<div class="translator" class:dark-theme={$theme === "dark"}>
  <div class="header">
    <h2>
      <FontAwesomeIcon icon="language" />
      <span>Translator</span>
    </h2>

    <div class="actions">
      <button class="close-btn" on:click={onClose} title="Close">
        <FontAwesomeIcon icon="times" />
      </button>
    </div>
  </div>

  <div class="translator-content">
    <div class="language-selectors">
      <div class="selector">
        <label for="source-lang">From:</label>
        <select id="source-lang" bind:value={sourceLanguage}>
          {#each sourceLanguageOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>

      <button
        class="swap-btn"
        on:click={swapLanguages}
        disabled={sourceLanguage === "auto"}
      >
        <FontAwesomeIcon icon="exchange-alt" />
      </button>

      <div class="selector">
        <label for="target-lang">To:</label>
        <select id="target-lang" bind:value={targetLanguage}>
          {#each targetLanguageOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="translation-area">
      <div class="text-area">
        <h3>Original Text</h3>
        <textarea
          bind:value={selectedText}
          placeholder="Enter text to translate"
          rows="6"
        ></textarea>
      </div>

      <div class="text-area">
        <h3>Translation</h3>
        {#if $isTranslationInProgress}
          <div class="loading-translation">
            <FontAwesomeIcon icon="spinner" spin />
            <span>Translating...</span>
          </div>
        {:else if $translationError}
          <div class="translation-error">
            <FontAwesomeIcon icon="exclamation-triangle" />
            <span>{$translationError}</span>
          </div>
        {:else}
          <textarea
            value={$translationResult}
            placeholder="Translation will appear here"
            rows="6"
            readonly
          ></textarea>
        {/if}
      </div>
    </div>

    <div class="translate-actions">
      <button
        class="translate-btn"
        on:click={runTranslation}
        disabled={$isTranslationInProgress || !selectedText.trim()}
      >
        <FontAwesomeIcon icon="language" />
        <span>Translate</span>
      </button>

      <button
        class="apply-btn"
        on:click={applyTranslation}
        disabled={!$translationResult}
      >
        <FontAwesomeIcon icon="check" />
        <span>Apply Translation</span>
      </button>
    </div>
  </div>

  <div class="footer">
    <span class="powered-by">Powered by DeepL</span>
  </div>
</div>

<style>
  .translator {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 700px;
    max-width: 90vw;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 5px 20px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    z-index: 1000;
    animation: fadeIn 0.3s ease;
  }

  .dark-theme {
    background-color: #333;
    color: #e0e0e0;
    box-shadow: 0 5px 20px rgba(0, 0, 0, 0.5);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    border-bottom: 1px solid #eee;
  }

  .dark-theme .header {
    border-bottom: 1px solid #555;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  button {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 6px 10px;
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

  .close-btn {
    width: 30px;
    height: 30px;
    padding: 0;
  }

  .translator-content {
    padding: 20px;
  }

  .language-selectors {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 20px;
  }

  .selector {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  label {
    font-size: 14px;
    min-width: 50px;
  }

  select {
    flex: 1;
    padding: 8px 10px;
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

  .swap-btn {
    background-color: #f0f0f0;
    width: 36px;
    height: 36px;
    border-radius: 50%;
  }

  .dark-theme .swap-btn {
    background-color: #444;
  }

  .swap-btn:hover:not(:disabled) {
    background-color: #e0e0e0;
  }

  .dark-theme .swap-btn:hover:not(:disabled) {
    background-color: #555;
  }

  .translation-area {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
  }

  .text-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  h3 {
    margin: 0;
    font-size: 14px;
    color: #666;
  }

  .dark-theme h3 {
    color: #aaa;
  }

  textarea {
    width: 100%;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 10px;
    font-size: 14px;
    resize: none;
    background-color: #f9f9f9;
  }

  .dark-theme textarea {
    background-color: #444;
    border-color: #555;
    color: #e0e0e0;
  }

  textarea:focus {
    outline: none;
    border-color: #3498db;
  }

  .dark-theme textarea:focus {
    border-color: #2980b9;
  }

  .loading-translation,
  .translation-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 150px;
    background-color: #f9f9f9;
    border: 1px solid #ddd;
    border-radius: 4px;
    color: #666;
  }

  .dark-theme .loading-translation,
  .dark-theme .translation-error {
    background-color: #444;
    border-color: #555;
    color: #aaa;
  }

  .translation-error {
    color: #e74c3c;
  }

  .dark-theme .translation-error {
    color: #ff7675;
  }

  .translate-actions {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }

  .translate-btn,
  .apply-btn {
    flex: 1;
    justify-content: center;
    padding: 10px;
    font-weight: 500;
    border-radius: 4px;
  }

  .translate-btn {
    background-color: #3498db;
    color: white;
  }

  .translate-btn:hover:not(:disabled) {
    background-color: #2980b9;
  }

  .apply-btn {
    background-color: #2ecc71;
    color: white;
  }

  .apply-btn:hover:not(:disabled) {
    background-color: #27ae60;
  }

  .dark-theme .translate-btn {
    background-color: #2980b9;
  }

  .dark-theme .translate-btn:hover:not(:disabled) {
    background-color: #3498db;
  }

  .dark-theme .apply-btn {
    background-color: #27ae60;
  }

  .dark-theme .apply-btn:hover:not(:disabled) {
    background-color: #2ecc71;
  }

  .footer {
    padding: 10px 20px;
    border-top: 1px solid #eee;
    display: flex;
    justify-content: flex-end;
  }

  .dark-theme .footer {
    border-top: 1px solid #555;
  }

  .powered-by {
    font-size: 12px;
    color: #999;
  }

  .dark-theme .powered-by {
    color: #777;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
