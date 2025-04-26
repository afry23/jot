// $lib/stores/backupStore.ts
import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Types
export interface Backup {
  path: string;
  date: Date;
  filename: string;
  formattedDate: string;
}

// Stores
export const backups = writable<Backup[]>([]);
export const isCreatingBackup = writable<boolean>(false);
export const isRestoringBackup = writable<boolean>(false);
export const backupError = writable<string | null>(null);
export const backupSuccess = writable<boolean>(false);
export const restoreError = writable<string | null>(null);
export const restoreSuccess = writable<boolean>(false);
export const selectedBackupPath = writable<string | null>(null);

// Derived store for selected backup
export const selectedBackup = derived(
  [backups, selectedBackupPath],
  ([$backups, $selectedBackupPath]) => {
    if (!$selectedBackupPath) return null;
    return $backups.find(backup => backup.path === $selectedBackupPath) || null;
  }
);

// Parse backup filenames and format dates
function parseBackups(paths: string[]): Backup[] {
  return paths.map(path => {
    // Extract the filename from the path
    const filename = path.split('/').pop()?.split('\\').pop() || path;
    
    // Try to extract the date from the filename (jot_backup_YYYY-MM-DD_HH-MM-SS_timestamp.zip)
    let date = new Date();
    let formattedDate = filename;
    
    const dateMatch = filename.match(/jot_backup_(\d{4}-\d{2}-\d{2})_(\d{2}-\d{2}-\d{2})_(\d+)\.zip/);
    if (dateMatch) {
      const [_, datePart, timePart, timestamp] = dateMatch;
      
      // Parse date from filename
      const dateStr = `${datePart}T${timePart.replace(/-/g, ':')}`;
      date = new Date(dateStr);
      
      // Format the date for display
      formattedDate = date.toLocaleString();
    }
    
    return {
      path,
      date,
      filename,
      formattedDate
    };
  });
}

// Load all available backups
export async function loadBackups(): Promise<Backup[]> {
  try {
    const paths = await invoke<string[]>("list_backups");
    const parsedBackups = parseBackups(paths);
    
    // Sort backups by date (newest first)
    parsedBackups.sort((a, b) => b.date.getTime() - a.date.getTime());
    
    // Update the store
    backups.set(parsedBackups);
    return parsedBackups;
  } catch (error) {
    console.error("Failed to load backups:", error);
    backupError.set(`Failed to load backups: ${error}`);
    return [];
  }
}

// Create a new backup
export async function createBackup(): Promise<string | null> {
  try {
    backupSuccess.set(false);
    backupError.set(null);
    isCreatingBackup.set(true);
    
    const backupPath = await invoke<string>("create_backup");
    
    // Reload the backup list
    await loadBackups();
    
    backupSuccess.set(true);
    // Reset success flag after 3 seconds
    setTimeout(() => backupSuccess.set(false), 3000);
    
    return backupPath;
  } catch (error) {
    console.error("Failed to create backup:", error);
    backupError.set(`${error}`);
    return null;
  } finally {
    isCreatingBackup.set(false);
  }
}

// Restore from a backup
export async function restoreBackup(backupPath: string): Promise<boolean> {
  try {
    restoreSuccess.set(false);
    restoreError.set(null);
    isRestoringBackup.set(true);
    
    await invoke("restore_backup", { backupPath });
    
    restoreSuccess.set(true);
    // Reset success flag after 3 seconds
    setTimeout(() => restoreSuccess.set(false), 3000);
    
    return true;
  } catch (error) {
    console.error("Failed to restore backup:", error);
    restoreError.set(`${error}`);
    return false;
  } finally {
    isRestoringBackup.set(false);
  }
}

// Delete a backup
export async function deleteBackup(backupPath: string): Promise<boolean> {
  try {
    await invoke("delete_backup", { backupPath });
    
    // If this was the selected backup, deselect it
    selectedBackupPath.update(current => current === backupPath ? null : current);
    
    // Reload the backup list
    await loadBackups();
    
    return true;
  } catch (error) {
    console.error("Failed to delete backup:", error);
    backupError.set(`Failed to delete backup: ${error}`);
    return false;
  }
}

// Count backups
export async function countBackups(): Promise<number> {
  try {
    return await invoke<number>("count_backups");
  } catch (error) {
    console.error("Failed to count backups:", error);
    return 0;
  }
}

// Setup backup event listeners
export async function setupBackupListeners(): Promise<UnlistenFn[]> {
  const listeners: UnlistenFn[] = [];
  
  // Listen for backup created events
  const backupCreatedListener = await listen("backup-created", (event) => {
    console.log("Backup created:", event.payload);
    loadBackups();
  });
  
  listeners.push(backupCreatedListener);
  
  return listeners;
}

// Initialize backup system
export async function initBackupSystem(): Promise<void> {
  // Load initial backup list
  await loadBackups();
  
  // Setup event listeners
  const listeners = await setupBackupListeners();
  
  // Cleanup listeners
  listeners.forEach(unlisten => unlisten());
}