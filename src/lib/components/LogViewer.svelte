<!-- $lib/components/LogViewer.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { logger } from "$lib/utils/logger";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import Button from "./Button.svelte";

  // Props
  export let maxLines: number = 100;
  export let autoRefresh: boolean = false;

  // State
  let logs: string = "Loading logs...";
  let logFiles: string[] = [];
  let selectedFile: string | null = null;
  let refreshing: boolean = false;
  let refreshInterval: number | null = null;

  // Load logs on mount
  onMount(async () => {
    await loadLogs();

    // List available log files
    logFiles = await logger.getLogFiles();

    // Set up auto-refresh if enabled
    if (autoRefresh) {
      refreshInterval = setInterval(loadLogs, 5000) as unknown as number;
    }
  });

  // Clean up on unmount
  onDestroy(() => {
    if (refreshInterval !== null) {
      clearInterval(refreshInterval);
    }
  });

  // Load logs from backend
  async function loadLogs(): Promise<void> {
    refreshing = true;
    try {
      logs = await logger.getLatestLogs(maxLines);
    } catch (error) {
      logs = `Failed to load logs: ${error}`;
    } finally {
      refreshing = false;
    }
  }

  // Force refresh logs
  async function refreshLogs(): Promise<void> {
    await loadLogs();
  }

  // Scroll to bottom when logs change
  $: if (logs) {
    setTimeout(() => {
      const logsContainer = document.getElementById("logs-container");
      if (logsContainer) {
        logsContainer.scrollTop = logsContainer.scrollHeight;
      }
    }, 100);
  }
</script>

<div class="log-viewer">
  <div class="log-controls flex justify-between items-center mb-2">
    <div class="flex items-center gap-2">
      <Button
        variant="outline"
        size="sm"
        on:click={refreshLogs}
        disabled={refreshing}
      >
        <FontAwesomeIcon
          icon={refreshing ? "spinner" : "sync"}
          spin={refreshing}
        />
        <span class="ml-1">Refresh</span>
      </Button>

      <label
        class="flex items-center text-sm text-gray-600 dark:text-gray-400 ml-2"
      >
        <input type="checkbox" bind:checked={autoRefresh} class="mr-2" />
        Auto-refresh
      </label>
    </div>

    <select
      class="text-sm p-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800"
      bind:value={selectedFile}
    >
      <option value={null}>Latest logs</option>
      {#each logFiles as file}
        <option value={file}>{file}</option>
      {/each}
    </select>
  </div>

  <div
    id="logs-container"
    class="logs-container bg-gray-100 dark:bg-gray-900 rounded border border-gray-300 dark:border-gray-700 p-3 text-sm font-mono h-80 overflow-y-auto whitespace-pre"
  >
    {logs}
  </div>
</div>

<style>
  .logs-container {
    font-family: "Courier New", Courier, monospace;
  }
</style>
