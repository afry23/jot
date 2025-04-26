<script lang="ts">
    import { onMount } from "svelte";
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import { countBackups } from "$lib/stores/backupStore";
    import { theme } from "$lib/stores/settings";
    
    // Props
    export let compact: boolean = false; // Whether to show a compact version
    
    // Local state
    let backupCount: number = 0;
    let lastBackupDate: string | null = null;
    
    onMount(async () => {
      // Get backup count
      backupCount = await countBackups();
      
      // Get last backup date from local storage
      const lastBackupTimestamp = localStorage.getItem("jot-last-backup-time");
      if (lastBackupTimestamp) {
        const timestamp = parseInt(lastBackupTimestamp);
        if (!isNaN(timestamp)) {
          const date = new Date(timestamp);
          lastBackupDate = date.toLocaleString();
        }
      }
    });
  </script>
  
  {#if compact}
    <!-- Compact version for status bar -->
    <div class="flex items-center text-xs">
      <FontAwesomeIcon icon="archive" class="mr-1 text-gray-500 dark:text-gray-400" />
      <span class="text-gray-700 dark:text-gray-300">{backupCount} backups</span>
      {#if lastBackupDate}
        <span class="text-gray-500 dark:text-gray-500 mx-1">•</span>
        <span class="text-gray-500 dark:text-gray-400">Last: {lastBackupDate}</span>
      {/if}
    </div>
  {:else}
    <!-- Full version for modal -->
    <div class="p-4 bg-white dark:bg-gray-800 rounded-lg shadow">
      <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-3">Backup Status</h3>
      
      <div class="space-y-3">
        <div class="flex justify-between items-center pb-2 border-b border-gray-200 dark:border-gray-700">
          <span class="text-gray-600 dark:text-gray-400">Total backups</span>
          <span class="text-gray-900 dark:text-gray-100 font-medium">{backupCount}</span>
        </div>
        
        {#if lastBackupDate}
          <div class="flex justify-between items-center pb-2 border-b border-gray-200 dark:border-gray-700">
            <span class="text-gray-600 dark:text-gray-400">Last backup</span>
            <span class="text-gray-900 dark:text-gray-100">{lastBackupDate}</span>
          </div>
        {/if}
        
        <div class="pt-2">
          <a 
            href="#backup-settings" 
            class="text-blue-600 dark:text-blue-400 hover:underline text-sm flex items-center"
          >
            <FontAwesomeIcon icon="cog" class="mr-1" />
            <span>Manage Backups</span>
          </a>
        </div>
      </div>
    </div>
  {/if}