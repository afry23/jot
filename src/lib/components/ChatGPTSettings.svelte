<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import {
    chatGPTConfig,
    loadChatGPTConfig,
    saveChatGPTConfig,
    REWRITE_STYLES,
  } from "$lib/stores/chatgptService";

  const OPENAI_ENDPOINT = "https://api.openai.com/v1/chat/completions";
  const OLLAMA_ENDPOINT = "http://localhost:11434/v1/chat/completions";

  let apiKey: string = "";
  let model: string = "gpt-4o-mini";
  let endpoint: string = OPENAI_ENDPOINT;
  let maxTokens: number = 1000;
  let temperature: number = 0.7;
  let provider: string = "openai";
  let defaultRewriteStyle: string = "formal_sie";

  let saveSuccess: boolean = false;
  let saveError: string | null = null;
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  const openaiModels = [
    { value: "gpt-4o-mini", label: "GPT-4o Mini (schnell, günstig)" },
    { value: "gpt-4o", label: "GPT-4o" },
    { value: "gpt-4-turbo", label: "GPT-4 Turbo" },
    { value: "gpt-3.5-turbo", label: "GPT-3.5 Turbo" },
  ];

  const ollamaModels = [
    { value: "llama3.2", label: "Llama 3.2" },
    { value: "llama3.1", label: "Llama 3.1" },
    { value: "llama3", label: "Llama 3" },
    { value: "mistral", label: "Mistral" },
    { value: "qwen2.5", label: "Qwen 2.5" },
    { value: "gemma2", label: "Gemma 2" },
  ];

  $: modelOptions = provider === "ollama" ? ollamaModels : openaiModels;
  $: showApiKey = provider !== "ollama";

  onMount(async () => {
    await loadChatGPTConfig();
    if ($chatGPTConfig) {
      model = $chatGPTConfig.model;
      endpoint = $chatGPTConfig.endpoint;
      maxTokens = $chatGPTConfig.max_tokens;
      temperature = $chatGPTConfig.temperature;
      provider = $chatGPTConfig.provider;
      defaultRewriteStyle = $chatGPTConfig.default_rewrite_style;
    }
  });

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
  });

  function selectProvider(p: string) {
    provider = p;
    if (p === "openai") {
      endpoint = OPENAI_ENDPOINT;
      model = "gpt-4o-mini";
    } else if (p === "ollama") {
      endpoint = OLLAMA_ENDPOINT;
      model = "llama3.2";
      apiKey = "";
    }
  }

  async function handleSaveConfig() {
    saveSuccess = false;
    saveError = null;

    try {
      const success = await saveChatGPTConfig({
        apiKey: apiKey || undefined,
        model,
        endpoint,
        maxTokens,
        temperature,
        provider,
        defaultRewriteStyle,
      });

      if (success) {
        apiKey = "";
        saveSuccess = true;
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
          saveSuccess = false;
        }, 3000);
      } else {
        saveError = "Einstellungen konnten nicht gespeichert werden";
      }
    } catch (error) {
      saveError = `Fehler: ${error}`;
    }
  }

  $: temperaturePercent = Math.round(temperature * 100);
  function updateTemperature(event: Event) {
    const input = event.target as HTMLInputElement;
    temperature = parseFloat(input.value) / 100;
  }

  const styleEntries = Object.entries(REWRITE_STYLES);
</script>

<div class="space-y-5">
  <!-- Provider -->
  <div>
    <p class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
      Anbieter
    </p>
    <div class="provider-tabs">
      {#each [["openai", "OpenAI"], ["ollama", "Ollama (lokal)"], ["custom", "Benutzerdefiniert"]] as [key, label]}
        <button
          class="provider-tab"
          class:active={provider === key}
          on:click={() => selectProvider(key)}
          type="button"
        >
          {label}
        </button>
      {/each}
    </div>
    {#if provider === "ollama"}
      <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
        Kein API-Key erforderlich — Ollama muss lokal laufen.
      </p>
    {/if}
  </div>

  <!-- API Key -->
  {#if showApiKey}
    <div>
      <label for="api-key" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        API-Key {$chatGPTConfig.api_key_stored ? "(gespeichert ✓)" : ""}
      </label>
      <input
        id="api-key"
        type="password"
        bind:value={apiKey}
        placeholder={$chatGPTConfig.api_key_stored ? "Neuen Key eingeben zum Ersetzen" : "API-Key eingeben"}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-400"
      />
    </div>
  {/if}

  <!-- Model -->
  <div>
    <label for="model" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
      Modell
    </label>
    {#if provider === "custom"}
      <input
        id="model"
        type="text"
        bind:value={model}
        placeholder="Modellname, z.B. gpt-4o-mini"
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-400"
      />
    {:else}
      <select
        id="model"
        bind:value={model}
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-400"
      >
        {#each modelOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    {/if}
  </div>

  <!-- Endpoint -->
  <div>
    <label for="endpoint" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
      API-Endpunkt
    </label>
    <div class="flex gap-2">
      <input
        id="endpoint"
        type="text"
        bind:value={endpoint}
        placeholder="https://api.openai.com/v1/chat/completions"
        class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-400"
      />
      {#if provider !== "custom"}
        <button
          class="p-2 bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-200 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors"
          on:click={() => selectProvider(provider)}
          title="Endpunkt zurücksetzen"
          type="button"
        >
          <FontAwesomeIcon icon="undo" />
        </button>
      {/if}
    </div>
  </div>

  <!-- Max Tokens -->
  <div>
    <label for="max-tokens" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
      Maximale Tokens
    </label>
    <input
      type="number"
      id="max-tokens"
      bind:value={maxTokens}
      min="50"
      max="4000"
      step="50"
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-400"
    />
    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
      Maximale Länge der Antwort (höher = längere Ausgaben)
    </p>
  </div>

  <!-- Temperature -->
  <div>
    <label for="temperature" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
      Kreativität: {temperaturePercent}%
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
    <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
      <span>Präzise</span>
      <span>Kreativ</span>
    </div>
  </div>

  <!-- Default rewrite style -->
  <div>
    <label for="default-style" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
      Standard-Stil (Umformulieren)
    </label>
    <select
      id="default-style"
      bind:value={defaultRewriteStyle}
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-orange-400"
    >
      {#each styleEntries as [key, style]}
        <option value={key}>{style.label}</option>
      {/each}
    </select>
    <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
      Vorausgewählter Stil beim Öffnen des KI-Modals (Mod+Shift+R)
    </p>
  </div>

  <!-- Save button -->
  <div class="flex items-center gap-3 mt-4">
    <button
      class="px-4 py-2 bg-[#f0a05a] text-white font-medium rounded-md hover:bg-[#e09050] transition-colors flex items-center gap-2"
      on:click={handleSaveConfig}
      type="button"
    >
      <FontAwesomeIcon icon="save" />
      <span>Speichern</span>
    </button>

    {#if saveSuccess}
      <div class="text-green-600 dark:text-green-400 flex items-center gap-1">
        <FontAwesomeIcon icon="check-circle" />
        <span>Gespeichert</span>
      </div>
    {/if}

    {#if saveError}
      <div class="text-red-600 dark:text-red-400 flex items-center gap-1">
        <FontAwesomeIcon icon="exclamation-circle" />
        <span>{saveError}</span>
      </div>
    {/if}
  </div>

  <!-- Info box -->
  <div class="bg-blue-50 dark:bg-blue-900/20 rounded-md p-4 border border-blue-100 dark:border-blue-800">
    <h3 class="text-sm font-medium text-blue-800 dark:text-blue-300 mb-1">
      Tastenkürzel
    </h3>
    <p class="text-sm text-blue-700 dark:text-blue-400">
      <kbd class="kbd">Mod+Shift+R</kbd> — Text markieren und KI-Aktionen öffnen (Umformulieren, Erweitern, Übersetzen)
    </p>
  </div>
</div>

<style>
  .provider-tabs {
    display: flex;
    gap: 0.4em;
  }

  .provider-tab {
    flex: 1;
    padding: 0.45em 0.75em;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    background: transparent;
    color: inherit;
    font-size: 0.82em;
    cursor: pointer;
    transition: all 0.15s;
    opacity: 0.7;
  }

  :global(.dark) .provider-tab {
    border-color: rgba(255, 255, 255, 0.15);
  }

  .provider-tab:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.04);
  }

  :global(.dark) .provider-tab:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .provider-tab.active {
    background: rgba(240, 160, 90, 0.12);
    border-color: #f0a05a;
    color: #f0a05a;
    opacity: 1;
    font-weight: 600;
  }

  kbd.kbd {
    display: inline-block;
    padding: 0.1em 0.4em;
    background: rgba(0, 0, 0, 0.07);
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9em;
  }

  :global(.dark) kbd.kbd {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
