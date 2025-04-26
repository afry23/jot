<script lang="ts">
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  
  // Props
  export let type: "button" | "submit" | "reset" = "button";
  export let variant: "default" | "primary" | "success" | "danger" | "ghost" | "outline" = "default";
  export let size: "sm" | "md" | "lg" = "md";
  export let icon: string | null = null;
  export let iconPosition: "left" | "right" = "left";
  export let disabled: boolean = false;
  export let fullWidth: boolean = false;
  export let loading: boolean = false;
  
  // Compute text color based on variant
  $: textColorClass = 
    variant === "default" ? "text-gray-900 dark:text-gray-100" :
    (variant === "primary" || variant === "success" || variant === "danger") ? "text-white" :
    "";
    
  // Compute hover style based on variant
  $: hoverClass = !disabled ? (
    variant === "default" ? "hover:bg-gray-200 dark:hover:bg-gray-600" :
    variant === "primary" ? "hover:bg-[#e09050]" :
    variant === "success" ? "hover:bg-green-600" :
    variant === "danger" ? "hover:bg-red-600" :
    (variant === "ghost" || variant === "outline") ? "hover:bg-gray-100 dark:hover:bg-gray-800" :
    ""
  ) : "";
</script>

<button
  {type}
  class="inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background {textColorClass} {hoverClass}"
  class:w-full={fullWidth}
  
  class:text-xs={size === "sm"}
  class:h-8={size === "sm"}
  class:px-3={size === "sm"}
  
  class:text-sm={size === "md"}
  class:h-10={size === "md"}
  class:px-4={size === "md"}
  
  class:text-base={size === "lg"}
  class:h-12={size === "lg"}
  class:px-6={size === "lg"}
  
  class:bg-gray-100={variant === "default"}
  class:dark:bg-gray-700={variant === "default"}
  
  class:bg-[#f0a05a]={variant === "primary"}
  
  class:bg-green-500={variant === "success"}
  
  class:bg-red-500={variant === "danger"}
  
  class:bg-transparent={variant === "ghost" || variant === "outline"}
  
  class:border={variant === "outline"}
  class:border-gray-300={variant === "outline"}
  class:dark:border-gray-600={variant === "outline"}
  
  {disabled}
  on:click
  {...$$restProps}
>
  {#if loading}
    <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
  {/if}
  
  {#if icon && iconPosition === "left" && !loading}
    <FontAwesomeIcon {icon} class="mr-2 -ml-0.5 h-4 w-4" />
  {/if}
  
  <slot />
  
  {#if icon && iconPosition === "right"}
    <FontAwesomeIcon {icon} class="ml-2 -mr-0.5 h-4 w-4" />
  {/if}
</button>