<!-- $lib/components/LogSettings.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import Button from "./Button.svelte";
  import LogViewer from "./LogViewer.svelte";
  import { logger } from "$lib/utils/logger";
  import LogLevelSettings from "./LogLevelSettings.svelte";

  // State
  let logEnabled: boolean = true;
  let logFiles: string[] = [];
  let totalLogSize: string = "Calculating...";

  // Load log settings on mount
  onMount(async () => {
    // Get log files
    logFiles = await logger.getLogFiles();

    // Calculate total log size
    totalLogSize = await calculateLogSize();
  });

  // Calculate the total size of all log files
  async function calculateLogSize(): Promise<string> {
    try {
      const size = await invoke<number>("calculate_log_size");

      // Format size
      if (size < 1024) {
        return `${size} B`;
      } else if (size < 1024 * 1024) {
        return `${(size / 1024).toFixed(2)} KB`;
      } else {
        return `${(size / (1024 * 1024)).toFixed(2)} MB`;
      }
    } catch (error) {
      console.error("Failed to calculate log size:", error);
      return "Unknown";
    }
  }

  // Clear all logs
  async function clearLogs(): Promise<void> {
    if (!confirm("Are you sure you want to clear all logs?")) {
      return;
    }

    try {
      await invoke("clear_logs");

      // Refresh log files and size
      logFiles = await logger.getLogFiles();
      totalLogSize = await calculateLogSize();
    } catch (error) {
      console.error("Failed to clear logs:", error);
    }
  }

  // Toggle logging
  function toggleLogging(): void {
    logEnabled = !logEnabled;
    logger.setEnabled(logEnabled);
  }
</script>

<div class="space-y-6">
  <!-- Log Level Settings -->
  <LogLevelSettings />

  <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-4">
      Log Files
    </h3>

    <div class="flex items-center justify-between mb-4">
      <span class="text-gray-700 dark:text-gray-300">Enable Logging</span>
      <Button
        variant={logEnabled ? "primary" : "outline"}
        on:click={toggleLogging}
      >
        {logEnabled ? "Enabled" : "Disabled"}
      </Button>
    </div>

    <div class="flex items-center justify-between mb-4">
      <span class="text-gray-700 dark:text-gray-300">Log Files</span>
      <span class="text-gray-700 dark:text-gray-300"
        >{logFiles.length} files ({totalLogSize})</span
      >
    </div>

    <Button variant="outline" on:click={clearLogs} class="mb-4">
      <FontAwesomeIcon icon="trash" class="mr-2" />
      Clear All Logs
    </Button>
  </div>

  <div>
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">
      Recent Logs
    </h3>
    <LogViewer maxLines={200} autoRefresh={logEnabled} />
  </div>
</div>
