<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import Button from "./Button.svelte";
    import { writable } from "svelte/store";
    
    // Types
    interface NoteStatus {
      tab_index: number;
      local_modified: number;
      remote_modified: number | null;
      synced: boolean;
      conflict: boolean;
    }
    
    interface SyncStatus {
      last_sync: number | null;
      syncing: boolean;
      error: string | null;
      notes_status: Record<number, NoteStatus>;
    }
    
    // Props
    export let embedded: boolean = false; // If true, shows a compact version
    export let onClose: (() => void) | null = null;
    
    // State
    const syncStatus = writable<SyncStatus>({
      last_sync: null,
      syncing: false,
      error: null,
      notes_status: {},
    });
    let isSyncing = false;
    let error: string | null = null;
    let syncingMessage = "Syncing...";
    let unlisten: (() => void) | null = null;
    
    // Format date
    function formatDate(timestamp: number | null): string {
      if (!timestamp) return "Never";
      
      const date = new Date(timestamp * 1000);
      return date.toLocaleString();
    }
    
    // Get sync status
    async function getSyncStatus() {
      try {
        const status = await invoke<SyncStatus>("get_sync_status");
        syncStatus.set(status);
      } catch (err) {
        console.error("Failed to get sync status:", err);
        error = `Error: ${err}`;
      }
    }
    
    // Sync notes
    async function syncNotes() {
      if (isSyncing) return;
      
      try {
        isSyncing = true;
        error = null;
        syncingMessage = "Preparing to sync...";
        
        // Update status to show sync in progress
        syncStatus.update(s => ({
          ...s,
          syncing: true,
          error: null
        }));
        
        // Different sync stages with messages
        const stages = [
          "Connecting to Nextcloud...",
          "Checking for changes...",
          "Synchronizing notes...",
          "Finalizing sync..."
        ];
        
        // Simulate stages for better UX
        for (const message of stages) {
          syncingMessage = message;
          await new Promise(r => setTimeout(r, 500)); // Add a small delay between stages
        }
        
        // Actual sync operation
        const result = await invoke<SyncStatus>("sync_all_notes");
        
        // Update with the result
        syncStatus.set(result);
        
        // Force UI update with status
        await getSyncStatus();
        
      } catch (err) {
        console.error("Sync failed:", err);
        error = `Sync failed: ${err}`;
        
        // Update status to show error
        syncStatus.update(s => ({
          ...s,
          syncing: false,
          error: error
        }));
      } finally {
        isSyncing = false;
      }
    }
    
    // Manual trigger sync
    async function triggerSync() {
      if (isSyncing) return;
      
      try {
        await invoke("trigger_sync_command");
        // Status will be updated via the event listener
      } catch (err) {
        console.error("Failed to trigger sync:", err);
        error = `Error: ${err}`;
      }
    }
    
    onMount(async () => {
      // Get initial sync status
      await getSyncStatus();
      
      // Listen for sync completed events
      unlisten = await listen("sync-completed", () => {
        getSyncStatus();
      });
    });
    
    onDestroy(() => {
      // Cleanup event listener
      if (unlisten) unlisten();
    });
    
    // Subscribe to sync status changes
    $: lastSyncTime = formatDate($syncStatus.last_sync);
    $: syncError = $syncStatus.error || error;
    $: isInSync = Object.values($syncStatus.notes_status).every(note => note.synced);
    $: unsynced = Object.values($syncStatus.notes_status).filter(note => !note.synced);
    $: hasConflicts = Object.values($syncStatus.notes_status).some(note => note.conflict);
  </script>
  
  {#if embedded}
    <!-- Compact embedded view -->
    <div class="flex items-center gap-2 py-1">
      <div class="flex-1 flex items-center gap-2">
        {#if isSyncing}
          <FontAwesomeIcon icon="sync" spin class="text-blue-500 dark:text-blue-400" />
          <span class="text-sm">{syncingMessage}</span>
        {:else if syncError}
          <FontAwesomeIcon icon="exclamation-circle" class="text-red-500 dark:text-red-400" />
          <span class="text-sm truncate">{syncError}</span>
        {:else if hasConflicts}
          <FontAwesomeIcon icon="exclamation-triangle" class="text-yellow-500 dark:text-yellow-400" />
          <span class="text-sm">Sync conflicts detected</span>
        {:else if isInSync}
          <FontAwesomeIcon icon="check-circle" class="text-green-500 dark:text-green-400" />
          <span class="text-sm">In sync</span>
        {:else if unsynced.length > 0}
          <FontAwesomeIcon icon="info-circle" class="text-blue-500 dark:text-blue-400" />
          <span class="text-sm">{unsynced.length} {unsynced.length === 1 ? 'note' : 'notes'} need syncing</span>
        {:else}
          <FontAwesomeIcon icon="cloud" class="text-gray-500 dark:text-gray-400" />
          <span class="text-sm">Last synced: {lastSyncTime}</span>
        {/if}
      </div>
      
      <Button 
        variant="ghost" 
        size="sm" 
        icon="sync" 
        loading={isSyncing}
        disabled={isSyncing}
        on:click={syncNotes}
      >
        Sync
      </Button>
    </div>
  {:else}
    <!-- Full sync panel -->
    <div class="flex flex-col h-full bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200">
      <div class="flex justify-between items-center p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-medium flex items-center gap-2">
          <FontAwesomeIcon icon="cloud" />
          <span>Nextcloud Sync</span>
        </h2>
  
        {#if onClose}
          <button 
            class="w-8 h-8 flex items-center justify-center rounded-full bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
            on:click={onClose} 
            title="Close"
          >
            <FontAwesomeIcon icon="times" />
          </button>
        {/if}
      </div>
  
      <div class="p-4 overflow-y-auto flex-1">
        <div 
          class="flex p-4 rounded-lg border mb-5 {
            isSyncing ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800' : 
            syncError ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' : 
            hasConflicts ? 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800' : 
            isInSync ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800' :
            'bg-gray-50 dark:bg-gray-900/20 border-gray-200 dark:border-gray-700'
          }"
        >
          <div class="flex items-center justify-center w-12 h-12 rounded-full bg-white dark:bg-gray-800 mr-4 text-2xl {
            isSyncing ? 'text-blue-500 dark:text-blue-400' : 
            syncError ? 'text-red-500 dark:text-red-400' : 
            hasConflicts ? 'text-yellow-500 dark:text-yellow-400' : 
            isInSync ? 'text-green-500 dark:text-green-400' :
            'text-gray-500 dark:text-gray-400'
          }">
            {#if isSyncing}
              <FontAwesomeIcon icon="sync" spin />
            {:else if syncError}
              <FontAwesomeIcon icon="exclamation-circle" />
            {:else if hasConflicts}
              <FontAwesomeIcon icon="exclamation-triangle" />
            {:else if isInSync}
              <FontAwesomeIcon icon="check-circle" />
            {:else}
              <FontAwesomeIcon icon="cloud" />
            {/if}
          </div>
          
          <div class="flex-1">
            <h3 class="text-lg font-medium mb-1 {
              isSyncing ? 'text-blue-700 dark:text-blue-300' : 
              syncError ? 'text-red-700 dark:text-red-300' : 
              hasConflicts ? 'text-yellow-700 dark:text-yellow-300' : 
              isInSync ? 'text-green-700 dark:text-green-300' :
              'text-gray-700 dark:text-gray-300'
            }">
              {#if isSyncing}
                Syncing...
              {:else if syncError}
                Sync Error
              {:else if hasConflicts}
                Sync Conflicts
              {:else if isInSync}
                In Sync
              {:else if unsynced.length > 0}
                Changes to Sync
              {:else}
                Sync Status
              {/if}
            </h3>
            
            <p class="text-gray-600 dark:text-gray-400">
              {#if isSyncing}
                {syncingMessage}
              {:else if syncError}
                {syncError}
              {:else if hasConflicts}
                There are conflicts that need to be resolved
              {:else if isInSync}
                All notes are synchronized with Nextcloud
              {:else if unsynced.length > 0}
                {unsynced.length} {unsynced.length === 1 ? 'note' : 'notes'} need to be synchronized
              {:else}
                Last synchronized: {lastSyncTime}
              {/if}
            </p>
          </div>
        </div>
  
        <div class="mb-5">
          <Button 
            variant="primary" 
            icon="sync" 
            loading={isSyncing}
            disabled={isSyncing}
            fullWidth={true}
            on:click={syncNotes}
          >
            {isSyncing ? 'Syncing...' : 'Sync Now'}
          </Button>
        </div>
  
        {#if Object.keys($syncStatus.notes_status).length > 0}
          <div class="mt-6">
            <h3 class="text-base font-medium mb-2">Notes Status</h3>
            
            <div class="border border-gray-200 dark:border-gray-700 rounded-md overflow-hidden">
              {#each Object.entries($syncStatus.notes_status) as [index, note]}
                {@const tabIndex = parseInt(index)}
                <div class="flex justify-between items-center p-3 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 last:border-b-0">
                  <div class="flex items-center gap-2">
                    <span 
                      class="inline-block w-3 h-3 rounded-full" 
                      style="background-color: var(--color-tab-{tabIndex})"
                    ></span>
                    <span>Tab {tabIndex + 1}</span>
                  </div>
                  
                  <div>
                    {#if note.conflict}
                      <span class="inline-flex items-center gap-1 px-2 py-1 text-xs rounded bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300">
                        <FontAwesomeIcon icon="exclamation-triangle" />
                        Conflict
                      </span>
                    {:else if note.synced}
                      <span class="inline-flex items-center gap-1 px-2 py-1 text-xs rounded bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300">
                        <FontAwesomeIcon icon="check-circle" />
                        Synced
                      </span>
                    {:else}
                      <span class="inline-flex items-center gap-1 px-2 py-1 text-xs rounded bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300">
                        <FontAwesomeIcon icon="clock" />
                        Needs Sync
                      </span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}