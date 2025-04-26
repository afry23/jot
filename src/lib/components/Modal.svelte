<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from "svelte";
  import { theme } from "$lib/stores/settings";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import Button from "./Button.svelte";

  // Props
  export let title: string = "";
  export let icon: string | null = null;
  export let width: string = "500px";
  export let closeOnClickOutside: boolean = false;
  export let showFooter: boolean = true;

  // Internal state
  let isOpen = false;
  let dialogElement: HTMLDivElement;
  const dispatch = createEventDispatcher();

  // Public methods for controlling the modal
  export function open() {
    isOpen = true;
  }

  export function close() {
    isOpen = false;
    dispatch("close");
  }

  function handleClickOutside(event: MouseEvent) {
    if (
      isOpen &&
      closeOnClickOutside &&
      dialogElement &&
      !dialogElement.contains(event.target as Node)
    ) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (isOpen && event.key === "Escape") {
      close();
    }
  }

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickOutside);
    document.removeEventListener("keydown", handleKeydown);
  });
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div
      bind:this={dialogElement}
      class="modal-dialog bg-white dark:bg-gray-800 rounded-lg shadow-xl flex flex-col max-h-[90vh] w-full max-w-[90vw] overflow-hidden"
      style="width: {width};"
    >
      <div
        class="flex justify-between items-center p-4 border-b border-gray-200 dark:border-gray-700"
      >
        <h2
          class="text-lg font-medium text-gray-900 dark:text-gray-100 flex items-center gap-2"
        >
          {#if icon}
            <FontAwesomeIcon {icon} />
          {/if}
          {title}
        </h2>

        <Button
          variant="ghost"
          size="sm"
          on:click={close}
          class="w-8 h-8 p-0 rounded-full"
        >
          <FontAwesomeIcon icon="times" />
        </Button>
      </div>

      <div class="p-4 overflow-auto flex-1 modal-content">
        <slot />
      </div>

      {#if showFooter}
        <div
          class="p-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-2"
        >
          <slot name="footer">
            <Button on:click={close}>Close</Button>
          </slot>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  /* Custom animations */
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideIn {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  div[class*="fixed"] {
    animation: fadeIn 0.2s ease;
  }

  div[class*="fixed"] > div {
    animation: slideIn 0.3s ease;
  }

  /* Fix for nested scrollable elements */
  .modal-content {
    scrollbar-width: thin;
  }

  .modal-content::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }

  .modal-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .modal-content::-webkit-scrollbar-thumb {
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }

  .modal-content::-webkit-scrollbar-thumb:hover {
    background-color: rgba(0, 0, 0, 0.3);
  }

  :global(.dark) .modal-content::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.2);
  }

  :global(.dark) .modal-content::-webkit-scrollbar-thumb:hover {
    background-color: rgba(255, 255, 255, 0.3);
  }
</style>
