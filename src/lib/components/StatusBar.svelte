<script lang="ts">
  import { createBackup, loadBackups } from "$lib/stores/backupStore";
  import { loadChatGPTConfig } from "$lib/stores/chatgptService";
  import { loadLanguageServicesConfig } from "$lib/stores/languageServices";
  import {
    downloadAllNotes,
    initSync,
    isSyncing,
    showSyncStatus,
    triggerSync,
    uploadAllNotes,
  } from "$lib/stores/nextcloudSync";
  import { notes } from "$lib/stores/notes";
  import { theme, toggleTheme } from "$lib/stores/settings";
  import { activeTab } from "$lib/stores/tabs";
  import { activeTabBackground } from "$lib/stores/tabStyles";
  import { toggleViewMode, viewMode } from "$lib/stores/viewMode";
  import { emptyTabColor, tabColors } from "$lib/utils/colors";
  import {
    countCharacters,
    countLines,
    countWords,
  } from "$lib/utils/textFormatting";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import "../fa-icons";
  import BackupStatus from "./BackupStatus.svelte";
  import ChatGPT from "./ChatGPT.svelte";
  import GrammarChecker from "./GrammarChecker.svelte";
  import Help from "./Help.svelte";
  import Modal from "./Modal.svelte";
  import SyncIndicator from "./SyncIndicator.svelte";
  import Translator from "./Translator.svelte";

  export let showBackupStatus: boolean = false; // Prop to control backup status visibility
  let syncOperation: "sync" | "upload" | "download" = "sync";

  // Calculate text statistics
  $: currentContent = $notes[$activeTab] || "";
  $: wordCount = countWords(currentContent);
  $: characterCount = countCharacters(currentContent);
  $: lineCount = countLines(currentContent);

  // Check if the current tab is empty
  $: isCurrentTabEmpty = !currentContent || currentContent.trim() === "";

  // Get the appropriate color for the current tab
  $: currentTabColor = isCurrentTabEmpty
    ? emptyTabColor
    : tabColors[$activeTab];

  // Reference to components
  let helpComponent: Help;
  let settingsModal: Modal;
  let isGrammarCheckerVisible = false;
  let isTranslatorVisible = false;
  let isChatGPTVisible = false;
  let isCloudMenuOpen = false;

  // Language menu state
  let isLanguageMenuOpen = false;
  // AI menu state
  let isAIMenuOpen = false;

  // AI Mode state
  let chatGPTMode: "chat" | "summarize" = "chat";
  let selectedText: string = "";

  function openHelp() {
    if (helpComponent) {
      helpComponent.open();
    }
  }

  function syncNow() {
    isCloudMenuOpen = false;
    syncOperation = "sync";
    triggerSync()
      .then(() => {
        console.log("Sync triggered successfully");
      })
      .catch((error) => {
        console.error("Error triggering sync:", error);
      });
  }

  function uploadNotes() {
    isCloudMenuOpen = false;
    syncOperation = "upload";
    uploadAllNotes()
      .then(() => {
        console.log("Upload triggered successfully");
      })
      .catch((error) => {
        console.error("Error triggering upload:", error);
      });
  }

  function downloadNotes() {
    isCloudMenuOpen = false;
    syncOperation = "download";
    downloadAllNotes()
      .then(() => {
        console.log("Download triggered successfully");
      })
      .catch((error) => {
        console.error("Error triggering download:", error);
      });
  }
  function toggleLanguageMenu() {
    isLanguageMenuOpen = !isLanguageMenuOpen;
    if (isLanguageMenuOpen) {
      isAIMenuOpen = false;
      isCloudMenuOpen = false;
    }
  }

  function toggleAIMenu() {
    isAIMenuOpen = !isAIMenuOpen;
    if (isAIMenuOpen) {
      isLanguageMenuOpen = false;
      isCloudMenuOpen = false;
    }
  }

  function showGrammarChecker() {
    isGrammarCheckerVisible = true;
    isLanguageMenuOpen = false;
  }

  function hideGrammarChecker() {
    isGrammarCheckerVisible = false;
  }

  function showTranslator() {
    isTranslatorVisible = true;
    isLanguageMenuOpen = false;
  }

  function hideTranslator() {
    isTranslatorVisible = false;
  }

  function showChatGPT(mode: "chat" | "summarize" = "chat") {
    isChatGPTVisible = true;
    isAIMenuOpen = false;
    chatGPTMode = mode;

    // Get selected text if any
    const selection = window.getSelection ? window.getSelection() : null;
    if (selection) {
      selectedText = selection.toString() || currentContent;
    } else {
      selectedText = currentContent;
    }
  }

  function hideChatGPT() {
    isChatGPTVisible = false;
  }

  function toggleCloudMenu() {
    isCloudMenuOpen = !isCloudMenuOpen;
    if (isCloudMenuOpen) {
      isLanguageMenuOpen = false;
      isAIMenuOpen = false;
    }
  }

  // Close menus when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (
      isLanguageMenuOpen &&
      !target.closest(".language-button") &&
      !target.closest(".language-menu")
    ) {
      isLanguageMenuOpen = false;
    }

    if (
      isAIMenuOpen &&
      !target.closest(".ai-button") &&
      !target.closest(".ai-menu")
    ) {
      isAIMenuOpen = false;
    }

    if (
      isCloudMenuOpen &&
      !target.closest(".cloud-button") &&
      !target.closest(".cloud-menu")
    ) {
      isCloudMenuOpen = false;
    }
  }

  // Handle keyboard shortcuts for AI features
  function handleKeydown(event: KeyboardEvent) {
    // Skip if inside input/textarea for certain shortcuts
    const target = event.target as HTMLElement;
    const isEditing =
      target.tagName === "INPUT" || target.tagName === "TEXTAREA";

    // Only handle global shortcuts if not in an input/textarea
    if (!isEditing) {
      // Ctrl+Shift+G: Open ChatGPT Chat with selected text
      if (event.ctrlKey && event.shiftKey && event.key === "G") {
        event.preventDefault();
        showChatGPT("chat");
        return;
      }

      // Ctrl+Shift+S: Summarize with ChatGPT
      if (event.ctrlKey && event.shiftKey && event.key === "S") {
        event.preventDefault();
        showChatGPT("summarize");
        return;
      }

      // Ctrl+Shift+B: Create backup
      if (event.ctrlKey && event.shiftKey && event.key === "B") {
        event.preventDefault();
        createManualBackup();
        return;
      }
    }
  }

  // Create a manual backup and update status
  async function createManualBackup() {
    try {
      isCloudMenuOpen = false;
      await createBackup();
      const timestamp = Date.now();
      localStorage.setItem("jot-last-backup-time", timestamp.toString());
    } catch (error) {
      console.error("Failed to create manual backup:", error);
    }
  }

  onMount(() => {
    const initialize = async () => {
      // Load language services configuration
      await loadLanguageServicesConfig();

      // Load ChatGPT configuration
      await loadChatGPTConfig();

      // Initialize Nextcloud sync
      await initSync();

      // Load existing backups
      await loadBackups();
    };

    initialize();

    const setupSyncListeners = async () => {
      await listen("sync-started", (event) => {
        const operation = event.payload as string;
        if (operation === "upload") {
          syncOperation = "upload";
        } else if (operation === "download") {
          syncOperation = "download";
        } else {
          syncOperation = "sync";
        }
      });
    };

    setupSyncListeners();

    // Add click outside handler for menus
    document.addEventListener("click", handleClickOutside);

    // Add keyboard shortcut handler
    document.addEventListener("keydown", handleKeydown);

    return () => {
      document.removeEventListener("click", handleClickOutside);
      document.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<footer
  style="--tab-color: {currentTabColor}; --status-background: {$activeTabBackground}; background-color: {$activeTabBackground};"
  class="flex justify-between items-center px-4 py-2 border-t border-black/10 dark:border-white/10 text-sm transition-colors"
  class:text-gray-800={$theme === "light"}
  class:text-gray-200={$theme === "dark"}
>
  <div class="flex items-center gap-2">
    <div
      class="w-2 h-2 rounded-full"
      style="background-color: {currentTabColor};"
    ></div>
    <div class="stats flex gap-2 opacity-85">
      <span>{lineCount} {lineCount === 1 ? "line" : "lines"}</span>
      <span>•</span>
      <span>{wordCount} {wordCount === 1 ? "word" : "words"}</span>
      <span>•</span>
      <span
        >{characterCount}
        {characterCount === 1 ? "character" : "characters"}</span
      >
    </div>
  </div>

  <div class="flex items-center gap-2">
    <!-- Backup status -->
    {#if showBackupStatus}
      <div class="hidden md:flex items-center">
        <BackupStatus compact={true} />
      </div>
    {/if}
    <!-- Sync status -->
    {#if $showSyncStatus}
      <div class="hidden md:flex items-center">
        <SyncIndicator />
      </div>
    {/if}

    <!-- AI menu dropdown -->
    <div class="relative">
      <button
        class="ai-button flex items-center justify-center px-2.5 py-1 rounded-full bg-black/5 dark:bg-white/10 hover:bg-black/10 dark:hover:bg-white/15 transition-colors"
        on:click|stopPropagation={toggleAIMenu}
        title="AI Tools"
      >
        <span>AI</span>
      </button>

      {#if isAIMenuOpen}
        <div
          class="ai-menu absolute bottom-10 right-0 bg-white dark:bg-gray-800 rounded-md shadow-lg w-56 py-1 z-50 animate-fadeIn"
        >
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={() => showChatGPT("chat")}
          >
            <FontAwesomeIcon
              icon="robot"
              class="text-gray-500 dark:text-gray-400"
            />
            <span>Chat with ChatGPT</span>
          </button>
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={() => showChatGPT("summarize")}
          >
            <FontAwesomeIcon
              icon="robot"
              class="text-gray-500 dark:text-gray-400"
            />
            <span>Summarize with ChatGPT</span>
          </button>
        </div>
      {/if}
    </div>

    <div class="relative">
      <button
        class="cloud-button w-8 h-8 rounded-full flex items-center justify-center hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
        on:click|stopPropagation={toggleCloudMenu}
        title="Sync & Backup"
      >
        <FontAwesomeIcon
          icon={$isSyncing ? "sync" : "cloud"}
          spin={$isSyncing}
        />
      </button>

      {#if isCloudMenuOpen}
        <div
          class="cloud-menu absolute bottom-10 right-0 bg-white dark:bg-gray-800 rounded-md shadow-lg w-56 py-1 z-50 animate-fadeIn"
        >
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={() => syncNow()}
            disabled={$isSyncing}
          >
            <FontAwesomeIcon
              icon="sync"
              class="text-gray-500 dark:text-gray-400"
              spin={$isSyncing}
            />
            <span>{$isSyncing ? "Syncing..." : "Sync Now"}</span>
          </button>
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={downloadNotes}
          >
            <FontAwesomeIcon
              icon="cloud-download-alt"
              class="text-gray-500 dark:text-gray-400"
              spin={$isSyncing && syncOperation === "download"}
            />
            <span
              >{$isSyncing && syncOperation === "download"
                ? "Downloading..."
                : "Download All Notes"}</span
            >
          </button>
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={uploadNotes}
          >
            <FontAwesomeIcon
              icon="cloud-upload-alt"
              class="text-gray-500 dark:text-gray-400"
              spin={$isSyncing && syncOperation === "upload"}
            />
            <span
              >{$isSyncing && syncOperation === "sync"
                ? "Uploading..."
                : "Upload All Notes"}</span
            >
          </button>
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={createManualBackup}
          >
            <FontAwesomeIcon
              icon="save"
              class="text-gray-500 dark:text-gray-400"
            />
            <span>Create Backup</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Language Tools dropdown -->
    <div class="relative">
      <button
        class="language-button w-8 h-8 rounded-full flex items-center justify-center hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
        on:click|stopPropagation={toggleLanguageMenu}
        title="Language Tools"
      >
        <FontAwesomeIcon icon="language" />
      </button>

      {#if isLanguageMenuOpen}
        <div
          class="language-menu absolute bottom-10 right-0 bg-white dark:bg-gray-800 rounded-md shadow-lg w-56 py-1 z-50 animate-fadeIn"
        >
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={showGrammarChecker}
          >
            <FontAwesomeIcon
              icon="check-circle"
              class="text-gray-500 dark:text-gray-400"
            />
            <span>Grammar Checker</span>
          </button>
          <button
            class="menu-item w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-2"
            on:click={showTranslator}
          >
            <FontAwesomeIcon
              icon="exchange-alt"
              class="text-gray-500 dark:text-gray-400"
            />
            <span>Translator</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Help button -->
    <button
      class="w-8 h-8 rounded-full flex items-center justify-center hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
      on:click|stopPropagation={openHelp}
      title="Help"
    >
      <FontAwesomeIcon icon="question" />
    </button>

    <!-- View mode toggle - Show either plain text or preview -->
    <button
      class="w-8 h-8 rounded-full flex items-center justify-center hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
      on:click={toggleViewMode}
      title="Toggle view mode (Ctrl+E)"
    >
      {#if $viewMode === "edit"}
        <FontAwesomeIcon icon="eye" />
      {:else}
        <FontAwesomeIcon icon="edit" />
      {/if}
    </button>

    <!-- Theme toggle -->
    <button
      class="w-8 h-8 rounded-full flex items-center justify-center hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
      on:click={toggleTheme}
      title="Toggle theme (Ctrl+D)"
    >
      {#if $theme === "dark"}
        <FontAwesomeIcon icon="sun" />
      {:else}
        <FontAwesomeIcon icon="moon" />
      {/if}
    </button>
  </div>
</footer>

<Help bind:this={helpComponent} />

<!-- Grammar checker panel -->
{#if isGrammarCheckerVisible}
  <GrammarChecker onClose={hideGrammarChecker} />
{/if}

<!-- Translator panel -->
{#if isTranslatorVisible}
  <Translator onClose={hideTranslator} />
{/if}

<!-- ChatGPT panel -->
{#if isChatGPTVisible}
  <ChatGPT
    onClose={hideChatGPT}
    initialText={selectedText}
    mode={chatGPTMode}
  />
{/if}

<style>
  /* Custom animation for dropdown menus */
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(5px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-fadeIn {
    animation: fadeIn 0.2s ease-out;
  }
</style>
