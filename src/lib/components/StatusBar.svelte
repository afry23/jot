<script lang="ts">
  import { createBackup, loadBackups } from "$lib/stores/backupStore";
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
  import Help from "./Help.svelte";
  import Modal from "./Modal.svelte";
  import SyncIndicator from "./SyncIndicator.svelte";

  export let markdownEditorRef;
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
  let isCloudMenuOpen = false;

  // Language menu state
  let isLanguageMenuOpen = false;
  // AI menu state
  let isAIMenuOpen = false;

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
