<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { theme } from "$lib/stores/settings";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import {
    chatGPTConfig,
    loadChatGPTConfig,
    saveChatGPTConfig,
  } from "$lib/stores/chatgptService";
    import FormField from "./FormField.svelte";

  // Local state
  let apiKey: string = "";
  let model: string = "gpt-3.5-turbo";
  let endpoint: string = "https://api.openai.com/v1/chat/completions";
  let maxTokens: number = 500;
  let temperature: number = 0.7;

  // Success/error messages
  let saveSuccess: boolean = false;
  let saveError: string | null = null;
  let saveTimeout: number | null = null;

  // Model options
  const modelOptions = [
    { value: "gpt-3.5-turbo", label: "GPT-3.5 Turbo" },
    { value: "gpt-4", label: "GPT-4" },
    { value: "gpt-4-turbo", label: "GPT-4 Turbo" },
    { value: "gpt-3.5-turbo-16k", label: "GPT-3.5 Turbo (16k)" },
  ];

  // Load current configuration
  onMount(async () => {
    await loadChatGPTConfig();

    if ($chatGPTConfig) {
      apiKey = $chatGPTConfig.api_key || "";
      model = $chatGPTConfig.model;
      endpoint = $chatGPTConfig.endpoint;
      maxTokens = $chatGPTConfig.max_tokens;
      temperature = $chatGPTConfig.temperature;
    }
  });

  // Clear timeouts on destroy
  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
  });

  // Save ChatGPT configuration
  async function handleSaveConfig() {
    saveSuccess = false;
    saveError = null;

    try {
      const success = await saveChatGPTConfig(
        apiKey || undefined,
        model,
        endpoint,
        maxTokens,
        temperature,
      );

      if (success) {
        saveSuccess = true;
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
          saveSuccess = false;
        }, 3000);
      } else {
        saveError = "Failed to save settings";
      }
    } catch (error) {
      saveError = `Error: ${error}`;
    }
  }

  // Reset to default configuration
  function resetDefaults() {
    model = "gpt-3.5-turbo";
    endpoint = "https://api.openai.com/v1/chat/completions";
    maxTokens = 500;
    temperature = 0.7;
  }

  // Format temperature as percentage for UI
  $: temperaturePercent = Math.round(temperature * 100);
  function updateTemperature(event: Event) {
    const input = event.target as HTMLInputElement;
    temperature = parseFloat(input.value) / 100;
  }
</script>

<div class="space-y-6">
  <div>
    <FormField id="api-key" label="API Key" type="password" bind:value={apiKey} placeholder="Enter your OpenAI API key"/>
    <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
      Required to use ChatGPT features
    </div>
  </div>
  <div>
    <label
      for="model"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
      >Model</label
    >
    <select
      id="model"
      bind:value={model}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      {#each modelOptions as option}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
    <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
      Different models have different capabilities and pricing
    </div>
  </div>

  <div>
    <label
      for="max-tokens"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
      >Max Tokens</label
    >
    <input
      type="number"
      id="max-tokens"
      bind:value={maxTokens}
      min="50"
      max="4000"
      step="50"
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
    <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
      Maximum length of generated response (higher = longer responses)
    </div>
  </div>

  <div>
    <label
      for="temperature"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
    >
      Temperature: {temperaturePercent}%
    </label>
    <input
      type="range"
      id="temperature"
      min="0"
      max="100"
      value={temperaturePercent}
      on:input={updateTemperature}
      class="w-full h-2 bg-gray-200 dark:bg-gray-600 rounded-lg appearance-none cursor-pointer"
    />
    <div
      class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1"
    >
      <span>Precise</span>
      <span>Creative</span>
    </div>
    <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
      Controls randomness (lower = more deterministic)
    </div>
  </div>

  <div>
    <label
      for="endpoint"
      class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
      >API Endpoint</label
    >
    <div class="flex gap-2">
      <input
        type="text"
        id="endpoint"
        bind:value={endpoint}
        placeholder="https://api.openai.com/v1/chat/completions"
        class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <button
        class="p-2 bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-200 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors"
        on:click={resetDefaults}
        title="Reset to default"
      >
        <FontAwesomeIcon icon="undo" />
      </button>
    <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
      Only change if you're using a proxy or custom endpoint
    </div>
    </div>
  </div>

  <div class="flex items-center gap-3 mt-6">
    <button
      class="px-4 py-2 bg-[#f0a05a] text-white font-medium rounded-md hover:bg-[#e09050] transition-colors flex items-center gap-2"
      on:click={handleSaveConfig}
    >
      <FontAwesomeIcon icon="save" />
      <span>Save Settings</span>
    </button>

    {#if saveSuccess}
      <div class="text-green-600 dark:text-green-400 flex items-center">
        <FontAwesomeIcon icon="check-circle" class="mr-1" />
        <span>Settings saved</span>
      </div>
    {/if}

    {#if saveError}
      <div class="text-red-600 dark:text-red-400 flex items-center">
        <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
        <span>{saveError}</span>
      </div>
    {/if}
  </div>

  <div
    class="bg-blue-50 dark:bg-blue-900/20 rounded-md p-4 border border-blue-100 dark:border-blue-800 mt-6"
  >
    <h3 class="text-sm font-medium text-blue-800 dark:text-blue-300 mb-2">
      About OpenAI API
    </h3>
    <p class="text-sm text-blue-700 dark:text-blue-400 mb-2">
      To use the ChatGPT features, you need an OpenAI API key. You can get one
      by:
    </p>
    <ol
      class="list-decimal pl-5 text-sm text-blue-700 dark:text-blue-400 space-y-1 mb-2"
    >
      <li>
        Creating an account at <a
          href="https://platform.openai.com"
          target="_blank"
          rel="noopener noreferrer"
          class="text-blue-600 dark:text-blue-300 underline hover:no-underline"
          >platform.openai.com</a
        >
      </li>
      <li>Navigating to API Keys section</li>
      <li>Creating a new secret key</li>
    </ol>
    <p class="text-sm font-medium text-blue-700 dark:text-blue-400">
      Note: Using the API incurs charges based on your usage. Check
      <a
        href="https://openai.com/pricing"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-600 dark:text-blue-300 underline hover:no-underline"
        >OpenAI's pricing</a
      > for details.
    </p>
  </div>
</div>
