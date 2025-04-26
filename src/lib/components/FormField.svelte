<script lang="ts">
    export let label: string;
    export let id: string;
    export let type: string = "text";
    export let value: string;
    export let placeholder: string = "";
    export let hint: string = "";
    export let required: boolean = false;
    export let readonly: boolean = false;
    
    function handleInput(event: Event) {
      const target = event.target as HTMLInputElement;
      value = type === "checkbox" ? target.checked.toString() : target.value;
    }
  </script>
  
  <div class="mb-4">
    <label for={id} class="block mb-1 text-sm font-medium">{label}</label>
    
    {#if type === "textarea"}
      <textarea 
        {id}
        bind:value
        {placeholder}
        {required}
        {readonly}
        on:input={handleInput}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
      ></textarea>
    {:else}
      <input 
        {type}
        {id}
        bind:value
        {placeholder}
        {required}
        {readonly}
        on:input={handleInput}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
      />
    {/if}
    
    {#if hint}
      <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">{hint}</p>
    {/if}
  </div>