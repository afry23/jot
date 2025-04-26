// $lib/stores/nextcloudSync.ts
import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { logger } from "$lib/utils/logger";
import { NextCloudCommands } from "$lib/utils/tauriCommands";

export type SyncStatus = "idle" | "syncing" | "success" | "error";

// Nextcloud sync configuration interface
export interface NextcloudConfig {
  server_url: string;
  username: string;
  password: string;
  sync_folder: string;
  last_sync: number | null;
  auto_sync: boolean;
  sync_on_startup: boolean;
  sync_interval_minutes: number;
  show_sync_status: boolean;
}

// Stores for sync state
export const isSyncing = writable<boolean>(false);
export const syncStatus = writable<SyncStatus>("idle");
export const syncError = writable<string | null>(null);
export const lastSyncTime = writable<number | null>(null);
export const nextcloudConfig = writable<NextcloudConfig | null>(null);
export const showSyncStatus = writable<boolean>(true);

// Initialize Nextcloud sync
export async function initSync(): Promise<void> {
  logger.info("Initializing Nextcloud sync");

  try {
    // Load Nextcloud configuration
    const config = await getNextcloudConfig();
    nextcloudConfig.set(config);
    showSyncStatus.set(config.show_sync_status);

    // Set last sync time if available
    if (config.last_sync) {
      lastSyncTime.set(config.last_sync);
    }

    // Set up event listeners for sync status
    await listen("sync-started", () => {
      logger.info("Sync started");
      isSyncing.set(true);
      syncStatus.set("syncing");
      syncError.set(null);
    });

    await listen("sync-completed", (event) => {
      logger.info("Sync completed");
      isSyncing.set(false);
      syncStatus.set("success");

      // Update last sync time
      const timestamp = Date.now();
      lastSyncTime.set(timestamp);

      // Store last sync time in local storage for quick access
      localStorage.setItem("jot-last-sync-time", timestamp.toString());
    });

    await listen("sync-error", (event) => {
      const errorMessage = event.payload as string;
      logger.error("Sync error:", errorMessage);
      isSyncing.set(false);
      syncStatus.set("error");
      syncError.set(errorMessage);
    });

    // Try to get last sync time from localStorage (faster than waiting for config load)
    const storedLastSync = localStorage.getItem("jot-last-sync-time");
    if (storedLastSync) {
      lastSyncTime.set(parseInt(storedLastSync));
    }
  } catch (error) {
    logger.error("Failed to initialize Nextcloud sync:", error);
  }
}

// Get Nextcloud configuration
export async function getNextcloudConfig(): Promise<NextcloudConfig> {
  try {
    return await invoke<NextcloudConfig>(
      NextCloudCommands.GET_NEXTCLOUD_CONFIG
    );
  } catch (error) {
    logger.error("Failed to get Nextcloud config:", error);
    return {
      server_url: "",
      username: "",
      password: "",
      sync_folder: "/jot",
      last_sync: null,
      auto_sync: false,
      show_sync_status: false,
      sync_interval_minutes: 30,
      sync_on_startup: true,
    };
  }
}

// Save Nextcloud configuration
export async function saveNextcloudConfig(
  config: Partial<NextcloudConfig>
): Promise<boolean> {
  try {
    await invoke(NextCloudCommands.SAVE_NEXTCLOUD_CONFIG, {
      nextCloudConfig: config,
    });

    // Update the store
    const currentConfig = get(nextcloudConfig);
    if (currentConfig) {
      nextcloudConfig.set({
        ...currentConfig,
        ...config,
      });
    }

    return true;
  } catch (error) {
    logger.error("Failed to save Nextcloud config:", error);
    return false;
  }
}

// Test Nextcloud connection
export async function testConnection(): Promise<boolean> {
  try {
    return await invoke<boolean>(NextCloudCommands.TEST_NEXTCLOUD_CONNECTION);
  } catch (error) {
    logger.error("Failed to test Nextcloud connection:", error);
    return false;
  }
}

// Trigger manual sync
export async function triggerSync(): Promise<void> {
  try {
    if (get(isSyncing)) {
      logger.info("Sync already in progress");
      return;
    }
    // Update sync status
    isSyncing.set(true);
    syncStatus.set("syncing");
    syncError.set(null);

    logger.info("Triggering manual sync");
    await invoke<void>(NextCloudCommands.TRIGGER_SYNC);
  } catch (error) {
    logger.error("Failed to trigger sync:", error);
    isSyncing.set(false);
    syncStatus.set("error");
    syncError.set(String(error));
  }
}

export async function uploadAllNotes(): Promise<void> {
  try {
    if (get(isSyncing)) {
      logger.info("Sync already in progress");
      return;
    }

    // Update sync status
    isSyncing.set(true);
    syncStatus.set("syncing");
    syncError.set(null);

    logger.info("Triggering upload of all notes");
    await invoke<void>(NextCloudCommands.UPLOAD_ALL_NOTES);
  } catch (error) {
    logger.error("Failed to upload notes:", error);
    isSyncing.set(false);
    syncStatus.set("error");
    syncError.set(String(error));
  }
}

export async function downloadAllNotes(): Promise<void> {
  try {
    if (get(isSyncing)) {
      logger.info("Sync already in progress");
      return;
    }

    // Update sync status
    isSyncing.set(true);
    syncStatus.set("syncing");
    syncError.set(null);

    logger.info("Triggering download of all notes");
    await invoke<void>(NextCloudCommands.DOWNLOAD_ALL_NOTES);
  } catch (error) {
    logger.error("Failed to download notes:", error);
    isSyncing.set(false);
    syncStatus.set("error");
    syncError.set(String(error));
  }
}

// Get sync status
export async function getSyncStatus(): Promise<any> {
  try {
    return await invoke(NextCloudCommands.GET_SYNC_STATUS);
  } catch (error) {
    logger.error("Failed to get sync status:", error);
    return { status: "unknown" };
  }
}

export async function saveShowSyncStatus(show: boolean): Promise<boolean> {
  try {
    // Get current config
    const config = await getNextcloudConfig();
    if (!config) return false;

    // Save with updated setting
    await invoke(NextCloudCommands.SAVE_NEXTCLOUD_CONFIG, {
      server_url: config.server_url,
      username: config.username,
      password: config.password,
      syncFolder: config.sync_folder,
      autoSync: config.auto_sync,
      syncOnStartup: config.sync_on_startup,
      syncIntervalMinutes: config.sync_interval_minutes,
      showSyncStatus: show,
    });

    // Update the local store
    showSyncStatus.set(show);
    return true;
  } catch (error) {
    console.error("Failed to save sync status setting:", error);
    return false;
  }
}
