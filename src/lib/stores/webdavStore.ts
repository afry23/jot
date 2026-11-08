// #lib/stores/webdavStore.ts
import { get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { loadNotes } from "#lib/utils/persistence";
import { logger } from "#lib/utils/logger";

export interface WebDavConfig {
  enabled: boolean;
  url: string;
  username: string;
  remoteDir: string;
}

interface SyncResult {
  pulled: number[];
  pushed: number[];
  conflicts: number[];
}

const defaultConfig: WebDavConfig = {
  enabled: false,
  url: "",
  username: "",
  remoteDir: "jot",
};

export const webdavConfig = writable<WebDavConfig>({ ...defaultConfig });
export const isSyncing = writable<boolean>(false);
export const lastSync = writable<Date | null>(null);
export const lastSyncResult = writable<SyncResult | null>(null);
export const syncError = writable<string | null>(null);

const SYNC_DEBOUNCE_MS = 5000;
let syncTimeout: ReturnType<typeof setTimeout> | null = null;

export async function loadWebdavConfig(): Promise<WebDavConfig> {
  try {
    const config = await invoke<WebDavConfig>("get_webdav_config");
    const merged = { ...defaultConfig, ...config };
    webdavConfig.set(merged);
    return merged;
  } catch (error) {
    logger.error("Failed to load WebDAV config:", error);
    return get(webdavConfig);
  }
}

export async function hasWebdavPassword(username: string): Promise<boolean> {
  try {
    return await invoke<boolean>("has_webdav_credential", { username });
  } catch {
    return false;
  }
}

export async function saveWebdavConfig(
  config: WebDavConfig,
  password?: string
): Promise<void> {
  await invoke("save_webdav_config", {
    config,
    password: password || null,
  });
  webdavConfig.set(config);
}

export async function testWebdavConnection(
  config: WebDavConfig,
  password?: string
): Promise<void> {
  await invoke("test_webdav_connection", {
    config,
    password: password || null,
  });
}

// Run a sync now. Reloads notes if remote changes were applied locally.
export async function syncNow(): Promise<boolean> {
  if (get(isSyncing) || !get(webdavConfig).enabled) return false;

  isSyncing.set(true);
  syncError.set(null);
  try {
    const result = await invoke<SyncResult>("sync_webdav");
    lastSyncResult.set(result);
    lastSync.set(new Date());
    if (result.pulled.length > 0) {
      await loadNotes();
    }
    return true;
  } catch (error) {
    logger.error("WebDAV sync failed:", error);
    syncError.set(`${error}`);
    return false;
  } finally {
    isSyncing.set(false);
  }
}

// Sync shortly after the last local save
export function scheduleSync(): void {
  if (!get(webdavConfig).enabled) return;
  if (syncTimeout) clearTimeout(syncTimeout);
  syncTimeout = setTimeout(() => {
    syncTimeout = null;
    syncNow();
  }, SYNC_DEBOUNCE_MS);
}

// Initial sync at startup and sync after note saves
export async function initWebdavSync(): Promise<() => void> {
  await loadWebdavConfig();
  const onSaved = () => scheduleSync();
  window.addEventListener("jot:note-saved", onSaved);
  syncNow();
  return () => {
    window.removeEventListener("jot:note-saved", onSaved);
    if (syncTimeout) clearTimeout(syncTimeout);
  };
}
