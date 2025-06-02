<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { notes } from "$lib/stores/notes";
  import { activeTab } from "$lib/stores/tabs";
  import { theme } from "$lib/stores/settings";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import {
    chatGPTResponse,
    isProcessing,
    chatGPTError,
    sendToChatGPT,
    summarizeWithChatGPT,
    clearChatGPTState,
    loadChatGPTConfig,
  } from "$lib/stores/chatgptService";

  // Props
  export let onClose: () => void;
  export let initialText: string = "";
  export let mode: "chat" | "summarize" = "chat";
  export let markdownEditorRef;

  // Local state
  let userInput: string = initialText;
  let responseText: string = "";
  let apiConfigured: boolean = false;

  // Setup component
  onMount(async () => {
    // Clear previous state
    clearChatGPTState();

    // Load ChatGPT configuration
    await loadChatGPTConfig();

    // If mode is summarize and we have initial text, auto-summarize
    if (mode === "summarize" && initialText.trim()) {
      handleSummarize();
    }
  });

  // Subscribe to response changes
  $: responseText = $chatGPTResponse;

  // Handle sending prompt to ChatGPT
  async function handleSendPrompt() {
    if (!userInput.trim()) return;

    await sendToChatGPT(userInput);
  }

  // Handle summarization
  async function handleSummarize() {
    if (!userInput.trim()) return;

    await summarizeWithChatGPT(userInput);
  }

  // Apply response to the current note
  function applyToNote() {
    if (!responseText) return;

    const currentTab = get(activeTab);

    notes.update((state) => {
      // If we have any text selected, this will replace just that text
      const currentNote = state[currentTab] || "";

      // For summarize mode, we replace the entire note
      if (mode === "summarize") {
        state[currentTab] = responseText;
      } else {
        // For chat mode, we append the response
        state[currentTab] = currentNote + "\n\n" + responseText;
      }

      return state;
    });

    // Close the dialog
    onClose();
  }

  // Create a new note with the response
  function createNewNote() {
    if (!responseText) return;

    // Find the first empty tab or use the next tab
    const tabCount = 7; // Total number of tabs
    let targetTab = $activeTab;

    notes.update((state) => {
      // Try to find an empty tab first
      for (let i = 0; i < tabCount; i++) {
        if (!state[i] || state[i].trim() === "") {
          targetTab = i;
          break;
        }
      }

      // If no empty tab found, use the next tab (circular)
      if (targetTab === $activeTab) {
        targetTab = ($activeTab + 1) % tabCount;
      }

      // Set the content to the response
      state[targetTab] = responseText;
      return state;
    });

    // Switch to the new tab
    activeTab.set(targetTab);

    // Close the dialog
    onClose();
  }
</script>

<div class="chatgpt-dialog" class:dark-theme={$theme === "dark"}>
  <div class="header">
    <h2>
      <svg class="chatgpt-icon" viewBox="0 0 24 24" width="24" height="24">
        <path
          fill="currentColor"
          d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"
        />
      </svg>
      <span
        >{mode === "summarize"
          ? "Summarize with ChatGPT"
          : "Chat with ChatGPT"}</span
      >
    </h2>

    <div class="actions">
      <button class="close-btn" on:click={onClose} title="Close">
        <FontAwesomeIcon icon="times" />
      </button>
    </div>
  </div>

  <div class="chatgpt-content">
    <div class="input-area">
      <h3>{mode === "summarize" ? "Text to Summarize" : "Your Prompt"}</h3>
      <textarea
        bind:value={userInput}
        placeholder={mode === "summarize"
          ? "Enter text to summarize"
          : "Enter your prompt for ChatGPT"}
        rows="6"
      ></textarea>
    </div>

    <div class="action-buttons">
      {#if mode === "summarize"}
        <button
          class="action-btn"
          on:click={handleSummarize}
          disabled={$isProcessing || !userInput.trim()}
        >
          {#if $isProcessing}
            <FontAwesomeIcon icon="spinner" spin />
            <span>Summarizing...</span>
          {:else}
            <FontAwesomeIcon icon="sync" />
            <span>Summarize</span>
          {/if}
        </button>
      {:else}
        <button
          class="action-btn"
          on:click={handleSendPrompt}
          disabled={$isProcessing || !userInput.trim()}
        >
          {#if $isProcessing}
            <FontAwesomeIcon icon="spinner" spin />
            <span>Processing...</span>
          {:else}
            <FontAwesomeIcon icon="paper-plane" />
            <span>Send</span>
          {/if}
        </button>
      {/if}
    </div>

    <div class="response-area">
      <h3>Response</h3>
      {#if $isProcessing}
        <div class="loading">
          <FontAwesomeIcon icon="spinner" spin />
          <span>Waiting for ChatGPT response...</span>
        </div>
      {:else if $chatGPTError}
        <div class="error-message">
          <FontAwesomeIcon icon="exclamation-triangle" />
          <span>{$chatGPTError}</span>

          <div class="error-info">
            <p>Please make sure you have:</p>
            <ul>
              <li>Configured your OpenAI API key in settings</li>
              <li>Stable internet connection</li>
              <li>Sufficient API credits</li>
            </ul>
          </div>
        </div>
      {:else if responseText}
        <div class="response-content">
          <textarea value={responseText} readonly rows="8"></textarea>

          <div class="response-actions">
            <button class="apply-btn" on:click={applyToNote}>
              <FontAwesomeIcon icon="file-import" />
              <span
                >{mode === "summarize"
                  ? "Replace with Summary"
                  : "Append to Note"}</span
              >
            </button>

            <button class="new-note-btn" on:click={createNewNote}>
              <FontAwesomeIcon icon="file-plus" />
              <span>Create New Note</span>
            </button>
          </div>
        </div>
      {:else}
        <div class="empty-response">
          <p>Your response will appear here</p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .chatgpt-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 700px;
    max-width: 90vw;
    max-height: 85vh;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 5px 20px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    z-index: 1000;
    animation: fadeIn 0.3s ease;
    overflow: hidden;
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

  .chatgpt-icon {
    width: 24px;
    height: 24px;
    color: #10a37f;
  }

  .dark-theme .chatgpt-icon {
    color: #19c37d;
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
    padding: 8px 12px;
    border-radius: 4px;
    color: inherit;
    transition: all 0.2s;
    font-size: 14px;
  }

  button:hover:not(:disabled) {
    opacity: 0.9;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: 50%;
    background-color: rgba(0, 0, 0, 0.05);
  }

  .dark-theme .close-btn {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .close-btn:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  .dark-theme .close-btn:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  .chatgpt-content {
    padding: 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  h3 {
    margin: 0 0 10px 0;
    font-size: 15px;
    font-weight: 500;
    color: #666;
  }

  .dark-theme h3 {
    color: #aaa;
  }

  .input-area,
  .response-area {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  textarea {
    width: 100%;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 12px;
    font-size: 14px;
    resize: none;
    background-color: #f5f5f5;
    font-family: inherit;
  }

  .dark-theme textarea {
    background-color: #444;
    border-color: #555;
    color: #e0e0e0;
  }

  textarea:focus {
    outline: none;
    border-color: #10a37f;
  }

  .dark-theme textarea:focus {
    border-color: #19c37d;
  }

  textarea[readonly] {
    background-color: #f8f8f8;
    border-color: #ddd;
  }

  .dark-theme textarea[readonly] {
    background-color: #3a3a3a;
    border-color: #555;
  }

  .action-buttons {
    display: flex;
    justify-content: center;
  }

  .action-btn {
    background-color: #10a37f;
    color: white;
    padding: 10px 20px;
    font-weight: 500;
    border-radius: 4px;
    transition: background-color 0.2s;
    min-width: 150px;
  }

  .action-btn:hover:not(:disabled) {
    background-color: #0d8c6d;
  }

  .dark-theme .action-btn {
    background-color: #19c37d;
  }

  .dark-theme .action-btn:hover:not(:disabled) {
    background-color: #10a37f;
  }

  .loading,
  .empty-response,
  .error-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 40px 20px;
    border: 1px dashed #ddd;
    border-radius: 4px;
    color: #888;
    text-align: center;
  }

  .dark-theme .loading,
  .dark-theme .empty-response {
    border-color: #555;
    color: #aaa;
  }

  .error-message {
    border-color: #e74c3c;
    color: #e74c3c;
    text-align: left;
    align-items: flex-start;
  }

  .dark-theme .error-message {
    border-color: #e57373;
    color: #e57373;
  }

  .error-info {
    margin-top: 10px;
    font-size: 13px;
    width: 100%;
  }

  .error-info p {
    margin: 0 0 5px 0;
  }

  .error-info ul {
    margin: 0;
    padding-left: 20px;
  }

  .response-content {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .response-actions {
    display: flex;
    gap: 10px;
    justify-content: center;
  }

  .apply-btn,
  .new-note-btn {
    background-color: #3498db;
    color: white;
    padding: 10px 16px;
    font-weight: 500;
    min-width: 180px;
    justify-content: center;
  }

  .apply-btn:hover {
    background-color: #2980b9;
  }

  .new-note-btn {
    background-color: #9b59b6;
  }

  .new-note-btn:hover {
    background-color: #8e44ad;
  }

  .dark-theme .apply-btn {
    background-color: #2980b9;
  }

  .dark-theme .apply-btn:hover {
    background-color: #3498db;
  }

  .dark-theme .new-note-btn {
    background-color: #8e44ad;
  }

  .dark-theme .new-note-btn:hover {
    background-color: #9b59b6;
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
