<!-- StorageSettings.svelte - Using direct text input instead of dialog -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import { listen } from "@tauri-apps/api/event";
  import { logger } from "$lib/utils/logger";
  import Button from "./Button.svelte";
  import { loadNotes } from "$lib/utils/persistence";
  import { notes } from "$lib/stores/notes";

  // Local state
  let customStoragePath: string = "";
  let defaultStoragePath: string = "Loading...";
  let isUsingCustomPath: boolean = false;
  let saveSuccess: boolean = false;
  let saveError: string | null = null;
  let saveTimeout: number | null = null;
  let isApplyingChanges: boolean = false;
  let unlisten: (() => void) | null = null;
  let invalidPath: boolean = false;

  // Load current storage settings
  onMount(async () => {
    try {
      const settings = await invoke<{
        customPath: string | null;
        defaultPath: string;
        isUsingCustom: boolean;
      }>("get_storage_settings");
      
      customStoragePath = settings.customPath || "";
      defaultStoragePath = settings.defaultPath;
      isUsingCustomPath = settings.isUsingCustom;
      
      // Listen for storage-changed event
      unlisten = await listen("storage-changed", (event) => {
        loadNotes().then(() => {
          isApplyingChanges = false;
          saveSuccess = true;
          
          // Auto-dismiss success message
          if (saveTimeout) clearTimeout(saveTimeout);
          saveTimeout = setTimeout(() => {
            saveSuccess = false;
          }, 3000) as unknown as number;
        });
      });
    } catch (error) {
      logger.error("Failed to load storage settings:", error);
      saveError = `Failed to load storage settings: ${error}`;
    }
  });

  // Clean up on destroy
  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
    if (unlisten) unlisten();
  });

  // Validate path
  function validatePath() {
    // Very basic validation
    invalidPath = false;
    
    if (isUsingCustomPath && !customStoragePath.trim()) {
      invalidPath = true;
    }
    
    // Check for invalid characters based on OS
    // This is a simple implementation - in a real app you'd want more thorough validation
    const hasInvalidChars = /[<>:"|?*]/.test(customStoragePath);
    if (hasInvalidChars) {
      invalidPath = true;
    }
    
    return !invalidPath;
  }

  // Save storage settings
  async function saveStorageSettings() {
    try {
      if (isUsingCustomPath && !validatePath()) {
        saveError = "Invalid path. Please enter a valid directory path.";
        return;
      }
      
      saveSuccess = false;
      saveError = null;
      isApplyingChanges = true;
      
      await invoke("set_storage_path", {
        path: isUsingCustomPath ? customStoragePath : null
      });
      
      // Note: We don't set saveSuccess here as it's handled by the storage-changed event
    } catch (error) {
      logger.error("Failed to save storage settings:", error);
      saveError = `Error: ${error}`;
      isApplyingChanges = false;
    }
  }

  // Reset to default storage location
  function useDefaultStorage() {
    isUsingCustomPath = false;
    invalidPath = false;
  }
</script>

<div class="space-y-6">
  <section>
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">Note Storage Location</h3>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
      Choose where your notes are stored. Application settings and backups will remain in the default location.
    </p>

    <div class="space-y-4">
      <div class="flex items-start gap-2">
        <input
          type="radio"
          id="default-storage"
          value="default"
          checked={!isUsingCustomPath}
          on:change={useDefaultStorage}
          class="mt-1 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
        />
        <div>
          <label for="default-storage" class="font-medium text-gray-700 dark:text-gray-300">Use default storage location</label>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {defaultStoragePath}
          </p>
        </div>
      </div>

      <div class="flex items-start gap-2">
        <input
          type="radio"
          id="custom-storage"
          value="custom"
          checked={isUsingCustomPath}
          on:change={() => (isUsingCustomPath = true)}
          class="mt-1 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
        />
        <div class="flex-1">
          <label for="custom-storage" class="font-medium text-gray-700 dark:text-gray-300">Use custom storage location</label>
          
          <div class="flex mt-2 mb-1 gap-2">
            <input
              type="text"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 {invalidPath ? 'border-red-500 dark:border-red-400' : ''}"
              placeholder="Enter a directory path"
              bind:value={customStoragePath}
              disabled={!isUsingCustomPath}
            />
          </div>
          
          {#if invalidPath}
            <p class="text-sm text-red-500 dark:text-red-400 mt-1">
              Please enter a valid directory path
            </p>
          {/if}
          
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Notes will be stored as markdown files in this folder
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3 mt-6">
        <Button 
          variant="primary" 
          on:click={saveStorageSettings}
          loading={isApplyingChanges}
          disabled={isApplyingChanges || (isUsingCustomPath && !customStoragePath)}
        >
          <FontAwesomeIcon icon="save" class="mr-2" />
          Apply Changes
        </Button>
        
        {#if saveSuccess}
          <div class="text-green-600 dark:text-green-400 flex items-center">
            <FontAwesomeIcon icon="check-circle" class="mr-1" />
            <span>Storage location updated</span>
          </div>
        {/if}
        
        {#if saveError}
          <div class="text-red-600 dark:text-red-400 flex items-center">
            <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
            <span>{saveError}</span>
          </div>
        {/if}
      </div>
    </div>
  </section>

  <div class="bg-yellow-50 dark:bg-yellow-900/20 rounded-md p-4 border border-yellow-100 dark:border-yellow-800 mt-4">
    <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-300 mb-2 flex items-center">
      <FontAwesomeIcon icon="exclamation-triangle" class="mr-2" />
      Important Note
    </h3>
    <p class="text-sm text-yellow-700 dark:text-yellow-400 mb-2">
      Changing the storage location will move your existing notes to the new location. This operation:
    </p>
    <ul class="list-disc pl-5 text-sm text-yellow-700 dark:text-yellow-400 space-y-1">
      <li>Will create a backup of your notes before moving them</li>
      <li>May take a moment to complete, especially with many notes</li>
      <li>Should not be interrupted to prevent data loss</li>
    </ul>
  </div>
</div>