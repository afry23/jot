<script lang="ts">
  import ColorDot from "./ColorDot.svelte";
  import Modal from "./Modal.svelte";
  import UnifiedSettings from "./UnifiedSettings.svelte";
  import { activeTab, setActiveTab } from "$lib/stores/tabs";
  import { notes } from "$lib/stores/notes"; // Import notes to check content
  import { Window } from "@tauri-apps/api/window";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import { tabColors } from "$lib/utils/colors";
  import { invoke } from "@tauri-apps/api/core";
  import {
    nextcloudConfig,
    type NextcloudConfig,
  } from "$lib/stores/nextcloudSync";

  // Reference to modal component
  let settingsModal: Modal;

  // Handle tab click
  async function handleTabClick(index: number) {
    await setActiveTab(index);
  }

  // Function to check if a tab is empty
  function isTabEmpty(index: number): boolean {
    return !$notes[index] || $notes[index].trim() === "";
  }

  async function testCommand() {
    try {
      const myNextcloudConfig: NextcloudConfig = {
        server_url: "https://example.com",
        username: "user",
        password: "password",
        sync_folder: "/path/to/sync/folder",
        sync_interval_minutes: 5,
        sync_on_startup: true,
        auto_sync: true,
        last_sync: null,
        show_sync_status: true,
      };
      await invoke("my_custom_command", { nextcloudConfig: myNextcloudConfig });
    } catch (error) {
      console.error("Error executing command:", error);
    }
  }

  // Close window function
  async function hideWindow() {
    try {
      await Window.getCurrent().hide();
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  }

  // Open settings
  function openSettings() {
    if (settingsModal) {
      settingsModal.open();
    }
  }
</script>

<header
  class="flex justify-between items-center px-4 py-2.5 bg-black/10 dark:bg-white/5"
>
  <div class="w-8 flex justify-center">
    <button
      class="w-6 h-6 rounded-full flex items-center justify-center text-gray-700 dark:text-gray-300 opacity-80 hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/15 transition-all -webkit-app-region-no-drag"
      on:click={hideWindow}
    >
      <FontAwesomeIcon icon="times" />
    </button>
  </div>

  <div class="flex gap-2.5 mx-auto">
    {#each tabColors as color, i}
      <ColorDot
        {color}
        active={$activeTab === i}
        isEmpty={isTabEmpty(i)}
        onClick={async () => await handleTabClick(i)}
      />
    {/each}
  </div>

  <div class="w-8 flex justify-center">
    <button
      class="w-6 h-6 rounded-full flex items-center justify-center text-gray-700 dark:text-gray-300 opacity-80 hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/15 transition-all -webkit-app-region-no-drag"
      on:click|stopPropagation={openSettings}
      title="Settings"
    >
      <FontAwesomeIcon icon="cog" />
    </button>
  </div>
</header>

<Modal title="Settings" icon="cog" bind:this={settingsModal} width="700px">
  <UnifiedSettings />
</Modal>

<style>
  /* Make the header draggable */
  header {
    -webkit-app-region: drag;
  }

  /* Cancel drag for buttons */
  :global(.webkit-app-region-no-drag) {
    -webkit-app-region: no-drag;
  }
</style>
