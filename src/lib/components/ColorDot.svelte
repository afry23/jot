<script lang="ts">
  import { emptyTabColor } from "$lib/utils/colors";

  export let color: string;
  export let active: boolean = false;
  export let isEmpty: boolean = false; // Prop to check if tab is empty
  export let onClick: () => void;
</script>

<button 
  class="w-6 h-6 rounded-full border-3 bg-transparent cursor-pointer transition-all duration-200 ease-in-out p-0 m-0"
  style="--dot-color: {color}; --empty-color: {emptyTabColor}; border-color: {isEmpty ? emptyTabColor : color};"
  class:active
  class:empty={isEmpty}
  on:click={onClick}
  aria-label="Tab selector"
>
</button>

<style>
  .active {
    background-color: var(--dot-color);
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
    transform: scale(1.05);
  }
  
  .empty.active {
    background-color: var(--empty-color);
  }

  button:hover:not(.active) {
    background-color: color-mix(in srgb, var(--dot-color) 20%, transparent);
    transform: scale(1.05);
  }
  
  .empty:hover:not(.active) {
    background-color: color-mix(in srgb, var(--empty-color) 20%, transparent);
  }
  
  /* Custom border-3 class (Tailwind doesn't have this by default) */
  .border-3 {
    border-width: 3px;
  }
</style>