<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    isSyncing,
    lastSyncTime,
    syncError,
  } from "$lib/stores/nextcloudSync";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  // We'll implement a simple time formatting function since we don't have date-fns

  export let compact: boolean = false; // Whether to show a compact version

  function formatDistanceToNow(
    date: Date,
    options?: { addSuffix: boolean },
  ): string {
    const now = new Date();
    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    // Format based on time difference
    let result = "";
    if (diffInSeconds < 60) {
      result = "less than a minute";
    } else if (diffInSeconds < 3600) {
      const minutes = Math.floor(diffInSeconds / 60);
      result = `${minutes} ${minutes === 1 ? "minute" : "minutes"}`;
    } else if (diffInSeconds < 86400) {
      const hours = Math.floor(diffInSeconds / 3600);
      result = `${hours} ${hours === 1 ? "hour" : "hours"}`;
    } else {
      const days = Math.floor(diffInSeconds / 86400);
      result = `${days} ${days === 1 ? "day" : "days"}`;
    }

    // Add suffix if requested
    if (options?.addSuffix) {
      result = `${result} ago`;
    }

    return result;
  }
  import { listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  // Refresh the "last synced" time periodically
  let timeInterval: number;
  let lastSyncFormatted: string = "";
  let unlisten: UnlistenFn | null = null;

  // Format the last sync time
  function formatLastSync() {
    if ($lastSyncTime) {
      const date = new Date($lastSyncTime);
      lastSyncFormatted = formatDistanceToNow(date, { addSuffix: true });
    } else {
      lastSyncFormatted = "Never";
    }
  }

  onMount(async () => {
    // Format on initial load
    formatLastSync();

    // Setup interval to update the "last synced" time
    timeInterval = setInterval(() => {
      formatLastSync();
    }, 30000); // Update every 30 seconds

    // Listen for sync-completed events from Tauri
    unlisten = await listen("sync-completed", () => {
      // This will trigger the reactive update as the store changes
    });
  });

  onDestroy(() => {
    if (timeInterval) clearInterval(timeInterval);
    if (unlisten) unlisten();
  });

  // Watch for changes to lastSyncTime
  $: if ($lastSyncTime) {
    formatLastSync();
  }
</script>

<div class="sync-indicator flex items-center text-sm">
  {#if $isSyncing}
    <div class="flex items-center text-blue-600 dark:text-blue-400">
      <FontAwesomeIcon icon="sync" spin class="mr-1.5" />
      <span>Syncing...</span>
    </div>
  {:else if $syncError}
    <div
      class="flex items-center text-red-600 dark:text-red-400 cursor-pointer"
      title={$syncError}
    >
      <FontAwesomeIcon icon="exclamation-circle" class="mr-1.5" />
      <span>Sync error</span>
    </div>
  {:else}
    <div class="flex items-center text-gray-600 dark:text-gray-400">
      <FontAwesomeIcon icon="cloud-upload-alt" class="mr-1.5" />
      {#if compact}
        <span>Synced</span>
      {:else}
        {#if $lastSyncTime}
          <span class="text-gray-900 dark:text-gray-100 font-medium">
            Synced: {lastSyncFormatted}
          </span>
        {:else}
          <span class="text-gray-500 dark:text-gray-400">Synced: Never</span>
        {/if}
      {/if}
    </div>
  {/if}
</div>
