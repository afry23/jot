<script lang="ts">
  import {
    theme,
    toggleTheme,
    fontSize,
    setFontSize,
    type FontSize,
  } from "$lib/stores/settings";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import Button from "./Button.svelte";
  import BackupSettings from "./BackupSettings.svelte";
  import LogSettings from "./LogSettings.svelte";
  import LanguageSettings from "./LanguageSettings.svelte";
  import ChatGPTSettings from "./ChatGPTSettings.svelte";
  import NextcloudSettings from "./NextcloudSettings.svelte";
  import StorageSettings from "./StorageSettings.svelte";

  // Font size options
  const fontSizeOptions: { value: FontSize; label: string }[] = [
    { value: "small", label: "Small" },
    { value: "medium", label: "Medium" },
    { value: "large", label: "Large" },
  ];

  // Settings tabs
  type SettingsTab =
    | "appearance"
    | "storage"
    | "nextcloud"
    | "language"
    | "ai"
    | "backups"
    | "about"
    | "logs";
  let activeTab: SettingsTab = "appearance";

  function changeFontSize(size: FontSize) {
    setFontSize(size);
  }

  function setActiveTab(tab: SettingsTab) {
    activeTab = tab;
  }
</script>

<div class="settings-container">
  <!-- Left sidebar navigation -->
  <div class="sidebar">
    <nav class="sidebar-nav">
      <button
        class="sidebar-tab px-4 py-2 text-left"
        class:active={activeTab === "appearance"}
        on:click={() => setActiveTab("appearance")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="palette" />
        </div>
        <span>Appearance</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "storage"}
        on:click={() => setActiveTab("storage")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="folder" class="sidebar-icon" />
        </div>
        <span>Storage</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "nextcloud"}
        on:click={() => setActiveTab("nextcloud")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="cloud" class="sidebar-icon" />
        </div>
        <span>Nextcloud</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "language"}
        on:click={() => setActiveTab("language")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="language" class="sidebar-icon" />
        </div>
        <span>Language Tools</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "ai"}
        on:click={() => setActiveTab("ai")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="robot" class="sidebar-icon" />
        </div>
        <span>AI</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "backups"}
        on:click={() => setActiveTab("backups")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="archive" class="sidebar-icon" />
        </div>
        <span>Backups</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "logs"}
        on:click={() => setActiveTab("logs")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="file-alt" class="sidebar-icon" />
        </div>
        <span>Logs</span>
      </button>

      <button
        class="sidebar-tab"
        class:active={activeTab === "about"}
        on:click={() => setActiveTab("about")}
      >
        <div class="sidebar-icon">
          <FontAwesomeIcon icon="info-circle" class="sidebar-icon" />
        </div>
        <span>About</span>
      </button>
    </nav>
  </div>

  <!-- Content area -->
  <div class="content-container">
    <!-- Appearance settings tab -->
    {#if activeTab === "appearance"}
      <div class="tab-content">
        <div>
          <h3 class="text-lg font-medium mb-3 text-gray-800 dark:text-gray-200">
            Appearance
          </h3>

          <div class="flex justify-between items-center mb-4">
            <span class="text-gray-700 dark:text-gray-300">Theme</span>
            <Button variant="outline" on:click={toggleTheme}>
              {#if $theme === "dark"}
                <FontAwesomeIcon icon="sun" class="mr-2" />
                <span>Light Mode</span>
              {:else}
                <FontAwesomeIcon icon="moon" class="mr-2" />
                <span>Dark Mode</span>
              {/if}
            </Button>
          </div>
        </div>

        <div>
          <div class="flex justify-between items-center mb-3">
            <span class="text-gray-700 dark:text-gray-300">Font Size</span>
          </div>

          <div class="grid grid-cols-3 gap-3">
            {#each fontSizeOptions as option}
              <Button
                variant={$fontSize === option.value ? "primary" : "default"}
                on:click={() => changeFontSize(option.value)}
              >
                {option.label}
              </Button>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Storage Settings tab -->
    {#if activeTab === "storage"}
      <div class="tab-content">
        <StorageSettings />
      </div>
    {/if}

    <!-- Nextcloud tab -->
    {#if activeTab === "nextcloud"}
      <div class="tab-content">
        <NextcloudSettings />
      </div>
    {/if}

    <!-- Language Tools tab -->
    {#if activeTab === "language"}
      <div class="tab-content">
        <LanguageSettings />
      </div>
    {/if}

    <!-- AI tab -->
    {#if activeTab === "ai"}
      <div class="tab-content">
        <ChatGPTSettings />
      </div>
    {/if}

    <!-- Backups tab -->
    {#if activeTab === "backups"}
      <div class="tab-content">
        <BackupSettings />
      </div>
    {/if}

    <!-- Logs tab -->
    {#if activeTab === "logs"}
      <div class="tab-content">
        <LogSettings />
      </div>
    {/if}

    <!-- About tab -->
    {#if activeTab === "about"}
      <div class="tab-content">
        <div class="space-y-6">
          <div>
            <h3
              class="text-lg font-medium mb-3 text-gray-800 dark:text-gray-200"
            >
              About Jot
            </h3>
            <p class="text-gray-600 dark:text-gray-400 mb-2">
              A minimal note-taking app for quick thoughts and ideas.
            </p>
            <p class="text-gray-600 dark:text-gray-400">Version 1.0.0</p>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Overall settings container */
  .settings-container {
    display: flex;
    height: 100%;
    max-height: 400px;
  }

  /* Left sidebar */
  .sidebar {
    width: 200px;
    border-right: 1px solid var(--border-color, rgba(229, 231, 235, 1));
    padding-right: 8px;
    flex-shrink: 0;
    overflow-y: auto;
    padding-top: 4px;
  }

  :global(.dark) .sidebar {
    border-color: var(--dark-border-color, rgba(75, 85, 99, 1));
  }

  /* Sidebar scrollbar styling */
  .sidebar::-webkit-scrollbar {
    width: 4px;
  }

  .sidebar::-webkit-scrollbar-track {
    background: transparent;
  }

  .sidebar::-webkit-scrollbar-thumb {
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 2px;
  }

  :global(.dark) .sidebar::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.2);
  }

  /* Navigation container */
  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* Individual sidebar tab buttons */
  .sidebar-tab {
    display: flex;
    align-items: center;
    padding: 10px 12px;
    border-radius: 6px;
    color: var(--tab-inactive-color, rgba(107, 114, 128, 1));
    transition: all 0.2s ease;
    font-size: 0.875rem;
    font-weight: 500;
    text-align: left;
  }

  :global(.dark) .sidebar-tab {
    color: var(--dark-tab-inactive-color, rgba(156, 163, 175, 1));
  }

  .sidebar-tab:hover {
    color: var(--tab-hover-color, rgba(55, 65, 81, 1));
    background-color: rgba(0, 0, 0, 0.05);
  }

  :global(.dark) .sidebar-tab:hover {
    color: var(--dark-tab-hover-color, rgba(209, 213, 219, 1));
    background-color: rgba(255, 255, 255, 0.05);
  }

  .sidebar-tab.active {
    color: var(--accent-color, #f0a05a);
    background-color: rgba(240, 160, 90, 0.1);
    font-weight: 600;
  }

  :global(.dark) .sidebar-tab.active {
    background-color: rgba(240, 160, 90, 0.15);
  }

  /* Tab icon styling */
  .sidebar-icon {
    margin-right: 8px;
    text-align: center;
    width: 16px;
  }

  /* Content container */
  .content-container {
    flex: 1;
    overflow-y: auto;
    padding-left: 20px;
    padding-right: 8px;
  }

  /* Content container scrollbar */
  .content-container::-webkit-scrollbar {
    width: 6px;
  }

  .content-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .content-container::-webkit-scrollbar-thumb {
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }

  :global(.dark) .content-container::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.2);
  }

  /* Tab content */
  .tab-content {
    padding-top: 10px;
    padding-bottom: 20px;
  }
</style>
