<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import Button from "./Button.svelte";
  import RestoreConfirmation from "./RestoreConfirmation.svelte";
  import "../fa-icons";
  import {
    backups,
    isCreatingBackup,
    isRestoringBackup,
    backupError,
    backupSuccess,
    restoreError,
    restoreSuccess,
    selectedBackupPath,
    selectedBackup,
    loadBackups,
    createBackup,
    restoreBackup,
    deleteBackup,
    countBackups,
    initBackupSystem,
    type Backup
  } from "$lib/stores/backupStore";
  import { invoke } from "@tauri-apps/api/core";

  // Local state
  let backupCount: number = 0;
  let keepBackupCount: number = 10; // Default number of backups to keep
  let showRestoreConfirmation: boolean = false;
  
  // Load backup count on mount
  onMount(async () => {
    // Initialize backup system
    await initBackupSystem();
    
    // Get backup count
    backupCount = await countBackups();
  });
  
  // Subscribe to backups store to update count
  $: if ($backups) {
    backupCount = $backups.length;
  }
  
  // Handle backup creation
  async function handleCreateBackup() {
    await createBackup();
  }
  
  // Show confirmation before restoring
  function confirmRestore() {
    if ($selectedBackup) {
      showRestoreConfirmation = true;
    }
  }
  
  // Handle backup restoration after confirmation
  async function handleRestoreBackup() {
    if ($selectedBackupPath) {
      await restoreBackup($selectedBackupPath);
      showRestoreConfirmation = false;
    }
  }
  
  // Handle backup deletion
  async function handleDeleteBackup(backup: Backup) {
    await deleteBackup(backup.path);
  }
  
  // Prune old backups
  async function pruneBackups() {
    try {
      const deletedCount = await invoke<number>("prune_backups", { keepCount: keepBackupCount });
      
      if (deletedCount > 0) {
        // Reload the backup list
        await loadBackups();
      }
    } catch (error) {
      console.error("Failed to prune backups:", error);
    }
  }
  
  // Cancel restoration
  function cancelRestore() {
    showRestoreConfirmation = false;
  }
</script>

<div class="space-y-6">
  <section>
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">Note Backups</h3>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
      Backup your notes to protect against data loss or sync issues.
    </p>

    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <Button 
            variant="primary" 
            on:click={handleCreateBackup} 
            loading={$isCreatingBackup}
            disabled={$isCreatingBackup}
          >
            <FontAwesomeIcon icon="save" class="mr-2" />
            Create Backup Now
          </Button>
          
          {#if $backupSuccess}
            <div class="mt-2 text-green-600 dark:text-green-400 text-sm flex items-center">
              <FontAwesomeIcon icon="check-circle" class="mr-1" />
              <span>Backup created successfully</span>
            </div>
          {/if}
          
          {#if $backupError}
            <div class="mt-2 text-red-600 dark:text-red-400 text-sm flex items-center">
              <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
              <span>{$backupError}</span>
            </div>
          {/if}
        </div>
        
        <div class="text-sm text-gray-600 dark:text-gray-400">
          {backupCount} {backupCount === 1 ? 'backup' : 'backups'} available
        </div>
      </div>
    
      <!-- Backup retention settings -->
      <div class="flex items-center gap-4 mt-4">
        <label for="keep-count" class="text-sm font-medium text-gray-700 dark:text-gray-300">Keep</label>
        <input
          id="keep-count"
          type="number"
          bind:value={keepBackupCount}
          min="1"
          max="50"
          class="w-16 px-2 py-1 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
        />
        <span class="text-sm text-gray-700 dark:text-gray-300">most recent backups</span>
        
        <Button
          variant="outline"
          size="sm"
          on:click={pruneBackups}
          disabled={$backups.length <= keepBackupCount}
        >
          Apply
        </Button>
      </div>
      
      <!-- Auto-backup before sync info -->
      <div class="p-3 bg-blue-50 dark:bg-blue-900 dark:bg-opacity-20 border border-blue-100 dark:border-blue-800 rounded text-sm text-blue-800 dark:text-blue-300 mt-2">
        <div class="flex items-start">
          <FontAwesomeIcon icon="info-circle" class="mt-0.5 mr-2 flex-shrink-0" />
          <span>Backups are automatically created before each Nextcloud sync to protect your data.</span>
        </div>
      </div>
    </div>
  </section>

  <!-- Available backups -->
  <section class="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-4">Available Backups</h3>
    
    {#if $backups.length === 0}
      <div class="p-4 bg-gray-50 dark:bg-gray-800 rounded-md text-center text-gray-500 dark:text-gray-400">
        No backups available yet. Create your first backup to protect your notes.
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-2 mb-4">
        {#each $backups as backup}
          <div 
            class="p-3 border rounded-md flex justify-between items-center cursor-pointer transition-colors"
            class:bg-gray-50={$selectedBackupPath !== backup.path}
            class:dark:bg-gray-800={$selectedBackupPath !== backup.path}
            class:bg-blue-50={$selectedBackupPath === backup.path}
            class:dark:bg-blue-900={$selectedBackupPath === backup.path}
            class:dark:bg-opacity-20={$selectedBackupPath === backup.path}
            class:border-gray-200={$selectedBackupPath !== backup.path}
            class:dark:border-gray-700={$selectedBackupPath !== backup.path}
            class:border-blue-300={$selectedBackupPath === backup.path}
            class:dark:border-blue-700={$selectedBackupPath === backup.path}
            on:click={() => selectedBackupPath.set(backup.path)}
          >
            <div>
              <div class="font-medium text-gray-900 dark:text-gray-100">{backup.formattedDate}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400">{backup.filename}</div>
            </div>
            
            <button
              class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 p-1"
              on:click|stopPropagation={() => handleDeleteBackup(backup)}
              title="Delete backup"
            >
              <FontAwesomeIcon icon="trash" />
            </button>
          </div>
        {/each}
      </div>
      
      <div class="mt-4">
        <Button
          variant={$selectedBackupPath ? "primary" : "default"}
          on:click={confirmRestore}
          disabled={!$selectedBackupPath || $isRestoringBackup}
          loading={$isRestoringBackup}
        >
          <FontAwesomeIcon icon="undo" class="mr-2" />
          Restore Selected Backup
        </Button>
        
        {#if $restoreSuccess}
          <div class="mt-2 text-green-600 dark:text-green-400 text-sm flex items-center">
            <FontAwesomeIcon icon="check-circle" class="mr-1" />
            <span>Backup restored successfully</span>
          </div>
        {/if}
        
        {#if $restoreError}
          <div class="mt-2 text-red-600 dark:text-red-400 text-sm flex items-center">
            <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
            <span>{$restoreError}</span>
          </div>
        {/if}
      </div>
    {/if}
  </section>
  
  <div class="bg-yellow-50 dark:bg-yellow-900 dark:bg-opacity-20 rounded-md p-4 border border-yellow-100 dark:border-yellow-800 mt-4">
    <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-300 mb-2 flex items-center">
      <FontAwesomeIcon icon="exclamation-triangle" class="mr-2" />
      Important Information
    </h3>
    <p class="text-sm text-yellow-700 dark:text-yellow-400">
      Restoring a backup will replace all your current notes with the content from the backup.
      Make sure to create a new backup of your current state before restoring if you want to keep your changes.
    </p>
  </div>
</div>

{#if showRestoreConfirmation && $selectedBackup}
  <RestoreConfirmation 
    onConfirm={handleRestoreBackup} 
    onCancel={cancelRestore} 
    backupDate={$selectedBackup.formattedDate}
  />
{/if}