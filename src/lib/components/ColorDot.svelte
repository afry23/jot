<script lang="ts">
    export let color: string;
    export let active: boolean = false;
    export let onClick: () => void;
  </script>
  
  <button 
    class="dot" 
    class:active 
    style="--dot-color: {color};"
    on:click={onClick}
  >
  </button>
  
  <style>
    .dot {
      width: 24px;
      height: 24px;
      border-radius: 50%;
      border: 2px solid var(--dot-color);
      background-color: transparent;
      cursor: pointer;
      transition: all 0.2s ease;
      padding: 0;
      margin: 0;
    }
  
    .dot.active {
      background-color: var(--dot-color);
      box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
      transform: scale(1.05);
    }
  
    .dot:hover:not(.active) {
      background-color: rgba(var(--dot-color), 0.2);
      transform: scale(1.05);
    }
  </style>