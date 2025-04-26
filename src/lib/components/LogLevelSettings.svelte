<script lang="ts">
  import { onMount } from "svelte";
  import { logger, logLevelStore } from "$lib/utils/logger";
  import { LogLevel, logLevelLabels } from "$lib/utils/logLevels";

  // Create an array for the UI to iterate over
  const logLevelOptions = Object.values(LogLevel)
    .filter((value) => typeof value === "number")
    .map((level) => ({
      value: level as LogLevel,
      label: logLevelLabels[level as LogLevel],
    }));

  // Current selection
  let selectedLevel: LogLevel;

  // Initialize on mount
  onMount(() => {
    // Subscribe to the log level store
    const unsubscribe = logLevelStore.subscribe((level) => {
      selectedLevel = level;
    });

    return unsubscribe;
  });

  // Handle level change
  async function handleLevelChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const newLevel = parseInt(target.value, 10) as LogLevel;

    // Update level in logger
    await logger.setLogLevel(newLevel);

    // Log an info message about the level change
    logger.info(`Log level changed to: ${LogLevel[newLevel]}`);
  }
</script>

<div class="log-level-settings">
  <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-4">
    Log Level
  </h3>

  <div class="mb-4">
    <label
      for="log-level"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
    >
      Select Log Level
    </label>
    <select
      id="log-level"
      bind:value={selectedLevel}
      on:change={handleLevelChange}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      {#each logLevelOptions as option}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
    <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
      Higher log levels include all lower levels. For example, "Warning" will
      show both warnings and errors.
    </p>
  </div>

  <div
    class="bg-gray-50 dark:bg-gray-800 p-3 rounded-md border border-gray-200 dark:border-gray-700"
  >
    <h4 class="font-medium text-gray-700 dark:text-gray-300 mb-2">
      Log Level Details
    </h4>
    <ul class="space-y-1 text-sm">
      <li class="text-purple-600 dark:text-purple-400">
        <strong>Trace:</strong> Most detailed logging, includes all messages
      </li>
      <li class="text-blue-600 dark:text-blue-400">
        <strong>Debug:</strong> Detailed information for debugging
      </li>
      <li class="text-green-600 dark:text-green-400">
        <strong>Info:</strong> Normal application behavior
      </li>
      <li class="text-yellow-600 dark:text-yellow-400">
        <strong>Warning:</strong> Potential issues that aren't errors
      </li>
      <li class="text-red-600 dark:text-red-400">
        <strong>Error:</strong> Application errors only
      </li>
      <li class="text-gray-600 dark:text-gray-400">
        <strong>None:</strong> Disable all logging
      </li>
    </ul>
  </div>
</div>
