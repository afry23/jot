<script lang="ts">
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import Button from "./Button.svelte";
    
    export let onConfirm: () => void;
    export let onCancel: () => void;
    export let backupDate: string;
  </script>
  
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-medium text-gray-900 dark:text-gray-100 flex items-center gap-2">
          <FontAwesomeIcon icon="exclamation-triangle" class="text-yellow-500" />
          Confirm Restore
        </h2>
      </div>
      
      <div class="mb-6">
        <p class="text-gray-700 dark:text-gray-300 mb-4">
          Are you sure you want to restore from the backup created on <strong>{backupDate}</strong>?
        </p>
        <p class="text-gray-700 dark:text-gray-300 mb-4">
          This will replace all your current notes with the ones from this backup.
        </p>
        <div class="p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded border border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-300 text-sm">
          <FontAwesomeIcon icon="exclamation-triangle" class="mr-2" />
          Any changes since this backup was created will be lost. Consider creating a new backup of your current state first.
        </div>
      </div>
      
      <div class="flex justify-end gap-3">
        <Button variant="outline" on:click={onCancel}>
          Cancel
        </Button>
        <Button variant="danger" on:click={onConfirm}>
          <FontAwesomeIcon icon="undo" class="mr-2" />
          Restore Backup
        </Button>
      </div>
    </div>
  </div>
