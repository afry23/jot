<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import FormField from "./FormField.svelte";
  import Button from "./Button.svelte";
  import {
    showSyncStatus,
    saveShowSyncStatus,
    saveNextcloudConfig,
    getNextcloudConfig,
  } from "$lib/stores/nextcloudSync";
  import { NextCloudCommands } from "$lib/utils/tauriCommands";

  // Types for Nextcloud sync
  interface NextcloudConfig {
    server_url: string;
    username: string;
    password: string;
    syncFolder: string;
    lastSync: number | null;
    autoSync: boolean;
    syncOnStartup: boolean;
    syncIntervalMinutes: number;
    showSyncStatus: boolean;
  }

  // Local state
  let server_url: string = "";
  let username: string = "";
  let password: string = "";
  let syncFolder: string = "/jot";
  let autoSync: boolean = false;
  let syncOnStartup: boolean = false;
  let syncIntervalMinutes: number = 30;

  // Connection test state
  let testingConnection: boolean = false;
  let connectionSuccess: boolean | null = null;
  let connectionError: string | null = null;

  // Save state
  let saveSuccess: boolean = false;
  let saveError: string | null = null;
  let saveTimeout: number | null = null;

  // Load current configuration
  onMount(async () => {
    try {
      //const config = await invoke<NextcloudConfig>("get_nextcloud_config_command");
      const config = await getNextcloudConfig();

      server_url = config.server_url;
      username = config.username;
      password = config.password;
      syncFolder = config.sync_folder;
      autoSync = config.auto_sync;
      syncOnStartup = config.sync_on_startup;
      syncIntervalMinutes = config.sync_interval_minutes;
      showSyncStatus.set(config.show_sync_status);
    } catch (error) {
      console.error("Failed to load Nextcloud config:", error);
    }
  });

  // Clear timeouts on destroy
  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
  });

  // Save Nextcloud configuration
  async function handleSaveConfig() {
    saveSuccess = false;
    saveError = null;

    try {
      await saveNextcloudConfig({
        server_url,
        username,
        password,
        sync_folder: syncFolder,
        auto_sync: autoSync,
        sync_on_startup: syncOnStartup,
        sync_interval_minutes: syncIntervalMinutes,
        show_sync_status: $showSyncStatus,
      });

      saveSuccess = true;
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = setTimeout(() => {
        saveSuccess = false;
      }, 3000);
    } catch (error) {
      saveError = `Error: ${error}`;
    }
  }

  // Test Nextcloud connection
  async function testConnection() {
    testingConnection = true;
    connectionSuccess = null;
    connectionError = null;

    // Save config first to ensure we're testing with current values
    try {
      await handleSaveConfig();
      const success = await invoke<boolean>(
        NextCloudCommands.TEST_NEXTCLOUD_CONNECTION
      );
      connectionSuccess = success;

      if (!success) {
        connectionError =
          "Connection failed. Please check your credentials and server URL.";
      }
    } catch (error) {
      connectionSuccess = false;
      connectionError = `Error: ${error}`;
    } finally {
      testingConnection = false;
    }
  }

  // Handle auto sync checkbox
  function handleAutoSyncChange(event: Event) {
    const target = event.target as HTMLInputElement;
    autoSync = target.checked;

    // If auto sync is disabled, also disable sync on startup
    if (!autoSync) {
      syncOnStartup = false;
    }
  }

  async function handleShowSyncStatusChange(event: Event) {
    const target = event.target as HTMLInputElement;
    try {
      await saveShowSyncStatus(target.checked);
    } catch (error) {
      console.error("Failed to save sync status setting:", error);
    }
  }
</script>

<div class="space-y-6">
  <section>
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">
      Nextcloud Connection
    </h3>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
      Configure your Nextcloud server to synchronize your notes across devices.
    </p>

    <div class="space-y-4">
      <FormField
        id="server-url"
        type="text"
        bind:value={server_url}
        placeholder="https://your-nextcloud-instance.com"
        label="Server URL"
        hint="Your Nextcloud server address"
      />

      <FormField
        id="username"
        type="text"
        bind:value={username}
        placeholder="Your Nextcloud username"
        label="Username"
      />

      <FormField
        id="password"
        type="password"
        bind:value={password}
        placeholder="Your Nextcloud password or app password"
        label="Password"
        hint="App passwords are recommended for better security"
      />

      <FormField
        id="sync-folder"
        type="text"
        bind:value={syncFolder}
        placeholder="/jot"
        label="Sync Folder"
        hint="The folder path in your Nextcloud where notes will be stored"
      />

      <div class="mt-6 flex items-center gap-3">
        <Button variant="primary" icon="save" on:click={handleSaveConfig}>
          Save Settings
        </Button>

        <Button
          variant={connectionSuccess ? "success" : "default"}
          icon={testingConnection ? "spinner" : "sync"}
          loading={testingConnection}
          on:click={testConnection}
        >
          Test Connection
        </Button>

        {#if saveSuccess}
          <div class="text-green-600 dark:text-green-400 flex items-center">
            <FontAwesomeIcon icon="check-circle" class="mr-1" />
            <span class="text-sm">Settings saved</span>
          </div>
        {/if}

        {#if saveError}
          <div class="text-red-600 dark:text-red-400 flex items-center">
            <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
            <span class="text-sm">{saveError}</span>
          </div>
        {/if}
      </div>

      {#if connectionSuccess !== null}
        <div
          class={`p-3 rounded-md ${connectionSuccess ? "bg-green-50 dark:bg-green-900/20 border border-green-100 dark:border-green-800" : "bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800"}`}
        >
          {#if connectionSuccess}
            <div class="flex items-center text-green-700 dark:text-green-400">
              <FontAwesomeIcon icon="check-circle" class="mr-2" />
              <span
                >Connection successful! Your Nextcloud server is correctly
                configured.</span
              >
            </div>
          {:else}
            <div class="flex items-center text-red-700 dark:text-red-400">
              <FontAwesomeIcon icon="exclamation-circle" class="mr-2" />
              <span
                >{connectionError ||
                  "Connection failed. Please check your settings."}</span
              >
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </section>

  <section class="pt-4 border-t border-gray-200 dark:border-gray-700">
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">
      Sync Settings
    </h3>

    <div class="space-y-4">
      <div class="flex items-start gap-2">
        <input
          type="checkbox"
          id="show-sync-status"
          bind:checked={$showSyncStatus}
          on:change={handleShowSyncStatusChange}
          class="mt-1 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
        />
        <div>
          <label
            for="show-sync-status"
            class="font-medium text-gray-700 dark:text-gray-300"
            >Show sync status</label
          >
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Display sync status in the app
          </p>
        </div>
      </div>
      <div class="flex items-start gap-2">
        <input
          type="checkbox"
          id="auto-sync"
          bind:checked={autoSync}
          on:change={handleAutoSyncChange}
          class="mt-1 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
        />
        <div>
          <label
            for="auto-sync"
            class="font-medium text-gray-700 dark:text-gray-300"
            >Enable automatic sync</label
          >
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Periodically sync your notes in the background
          </p>
        </div>
      </div>

      {#if autoSync}
        <div class="ml-6 space-y-4">
          <div class="flex items-start gap-2">
            <input
              type="checkbox"
              id="sync-on-startup"
              bind:checked={syncOnStartup}
              class="mt-1 h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
            />
            <div>
              <label
                for="sync-on-startup"
                class="font-medium text-gray-700 dark:text-gray-300"
                >Sync on startup</label
              >
              <p class="text-sm text-gray-500 dark:text-gray-400">
                Automatically sync when the application starts
              </p>
            </div>
          </div>

          <div>
            <label
              for="sync-interval"
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >Sync interval (minutes)</label
            >
            <input
              type="number"
              id="sync-interval"
              bind:value={syncIntervalMinutes}
              min="5"
              max="1440"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
              How often to automatically sync your notes (minimum 5 minutes)
            </p>
          </div>
        </div>
      {/if}
    </div>
  </section>

  <div
    class="bg-blue-50 dark:bg-blue-900/20 rounded-md p-4 border border-blue-100 dark:border-blue-800 mt-6"
  >
    <h3 class="text-sm font-medium text-blue-800 dark:text-blue-300 mb-2">
      About Nextcloud Sync
    </h3>
    <p class="text-sm text-blue-700 dark:text-blue-400 mb-2">
      Nextcloud sync allows you to keep your notes in sync across multiple
      devices:
    </p>
    <ul
      class="list-disc pl-5 text-sm text-blue-700 dark:text-blue-400 space-y-1 mb-2"
    >
      <li>
        Your notes are stored in the specified folder on your Nextcloud server
      </li>
      <li>Two-way sync ensures changes from any device are propagated</li>
      <li>
        For better security, consider using an app password from your Nextcloud
        security settings
      </li>
      <li>Make sure WebDAV is enabled on your Nextcloud server</li>
    </ul>
  </div>
</div>
