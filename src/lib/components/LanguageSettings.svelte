<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import {
    languageConfig,
    loadLanguageServicesConfig,
    saveLanguageToolConfig,
    saveDeepLConfig,
  } from "$lib/stores/languageServices";
  import Button from "./Button.svelte";
  import FormField from "./FormField.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { SettingsCommands } from "$lib/utils/tauriCommands";

  // Local state
  let languagetoolApiKey: string = "";
  let languagetoolUsername: string = "";
  let languagetoolEndpoint: string = "";
  let deeplApiKey: string = "";
  let deeplEndpoint: string = "";
  let languageToolStatus: boolean = false;
  let deeplStatus: boolean = false;

  // Success/error messages
  let ltSaveSuccess: boolean = false;
  let ltSaveError: string | null = null;
  let deeplSaveSuccess: boolean = false;
  let deeplSaveError: string | null = null;
  let ltSaveTimeout: number | null = null;
  let deeplSaveTimeout: number | null = null;

  // Load current configuration
  onMount(async () => {
    await loadLanguageServicesConfig();

    if ($languageConfig) {
      languagetoolApiKey = $languageConfig.languagetool_api_key || "";
      languagetoolUsername = $languageConfig.languagetool_username || "";
      languagetoolEndpoint = $languageConfig.languagetool_endpoint;
      deeplApiKey = $languageConfig.deepl_api_key || "";
      deeplEndpoint = $languageConfig.deepl_endpoint;
    }

    await checkCredentialStatus();
  });

  // Clear timeouts on destroy
  onDestroy(() => {
    if (ltSaveTimeout) clearTimeout(ltSaveTimeout);
    if (deeplSaveTimeout) clearTimeout(deeplSaveTimeout);
  });

  // Save LanguageTool configuration
  async function handleSaveLTConfig() {
    ltSaveSuccess = false;
    ltSaveError = null;

    try {
      const success = await saveLanguageToolConfig(
        languagetoolApiKey || undefined,
        languagetoolUsername || undefined,
        languagetoolEndpoint
      );

      if (success) {
        ltSaveSuccess = true;
        if (ltSaveTimeout) clearTimeout(ltSaveTimeout);
        ltSaveTimeout = setTimeout(() => {
          ltSaveSuccess = false;
        }, 3000);
      } else {
        ltSaveError = "Failed to save settings";
      }
    } catch (error) {
      ltSaveError = `Error: ${error}`;
    }
  }

  // Save DeepL configuration
  async function handleSaveDeepLConfig() {
    deeplSaveSuccess = false;
    deeplSaveError = null;

    try {
      const success = await saveDeepLConfig(
        deeplApiKey || undefined,
        deeplEndpoint
      );

      if (success) {
        deeplSaveSuccess = true;
        if (deeplSaveTimeout) clearTimeout(deeplSaveTimeout);
        deeplSaveTimeout = setTimeout(() => {
          deeplSaveSuccess = false;
        }, 3000);
      } else {
        deeplSaveError = "Failed to save settings";
      }
    } catch (error) {
      deeplSaveError = `Error: ${error}`;
    }
  }

  async function checkCredentialStatus() {
  // For LanguageTool
  if (languagetoolUsername) {
    const hasCredential = await invoke(SettingsCommands.HAS_LANGUAGETOOL_API_KEY, { username: languagetoolUsername });
    console.log("LanguageTool credential status:", hasCredential);
    languagetoolApiKey = hasCredential ? 'default' : '';
  }
  
  // For DeepL
  const hasDeeplCredential = await invoke('has_deepl_credential');
  console.log("DeepL credential status:", hasDeeplCredential);
  deeplApiKey = hasDeeplCredential ? 'default' : '';
}

  // Reset to default configuration
  function resetLTDefaults() {
    languagetoolEndpoint = "https://api.languagetoolplus.com/v2/check";
  }

  function resetDeepLDefaults() {
    deeplEndpoint = "https://api-free.deepl.com/v2/translate";
  }
</script>

<div class="space-y-6">
  <!-- LanguageTool Configuration Section -->
  <section class="pb-5 border-b border-gray-200 dark:border-gray-700">
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">LanguageTool Configuration</h3>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
      Configure LanguageTool API for grammar and spell checking.
      <a
        href="https://languagetool.org/proofreading-api"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-600 dark:text-blue-400 hover:underline"
      >
        Get API key
      </a>
    </p>

    <div class="space-y-4">
      <div>
        <FormField
          id="lt-username"
          type="text"
          bind:value={languagetoolUsername}
          placeholder="Enter your LanguageTool username"
          label="Username"
        />
        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">Required when using an API key</p>
      </div>

      <div>
        <FormField
          id="lt-api-key"
          type="password"
          bind:value={languagetoolApiKey}
          placeholder="Enter your LanguageTool API key"
          label="API Key"
        />
        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">Used together with username for authentication</p>
      </div>

      <div>
        <label for="lt-endpoint" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">API Endpoint</label>
        <div class="flex gap-2">
          <input
            type="text"
            id="lt-endpoint"
            bind:value={languagetoolEndpoint}
            placeholder="https://api.languagetool.org/v2/check"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            class="p-2 bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-200 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors"
            on:click={resetLTDefaults}
            title="Reset to default"
          >
            <FontAwesomeIcon icon="undo" />
          </button>
        </div>
      </div>

      <div class="flex items-center gap-3 mt-4">
        <Button 
          variant="primary" 
          icon="save" 
          on:click={handleSaveLTConfig}
        >
          Save LanguageTool Settings
        </Button>

        {#if ltSaveSuccess}
          <div class="text-green-600 dark:text-green-400 flex items-center">
            <FontAwesomeIcon icon="check-circle" class="mr-1" />
            <span class="text-sm">Settings saved</span>
          </div>
        {/if}

        {#if ltSaveError}
          <div class="text-red-600 dark:text-red-400 flex items-center">
            <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
            <span class="text-sm">{ltSaveError}</span>
          </div>
        {/if}
      </div>

      <div class="bg-blue-50 dark:bg-blue-900/20 rounded-md p-3 border border-blue-100 dark:border-blue-800 mt-4">
        <p class="text-sm text-blue-800 dark:text-blue-300">
          <strong>Note:</strong> The public LanguageTool API can be used without
          authentication, but has strict usage limits. For better service:
        </p>
        <ul class="list-disc pl-5 text-sm text-blue-700 dark:text-blue-400 mt-2 space-y-1">
          <li>Leave both username and API key empty to use the public API</li>
          <li>Enter both username and API key for premium access</li>
          <li>Your username is usually your registered email</li>
        </ul>
      </div>
    </div>
  </section>

  <!-- DeepL Configuration Section -->
  <section class="pt-2">
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">DeepL Configuration</h3>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
      Configure DeepL API for translation services.
      <a
        href="https://www.deepl.com/pro#developer"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-600 dark:text-blue-400 hover:underline"
      >
        Get API key
      </a>
    </p>

    <div class="space-y-4">
      <div>
        <FormField
          id="deepl-api-key"
          type="password"
          bind:value={deeplApiKey}
          placeholder="Enter your DeepL API key (required)"
          label="API Key"
        />
        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
          API key is required for translation functionality
        </p>
      </div>

      <div>
        <label for="deepl-endpoint" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">API Endpoint</label>
        <div class="flex gap-2">
          <input
            type="text"
            id="deepl-endpoint"
            bind:value={deeplEndpoint}
            placeholder="https://api-free.deepl.com/v2/translate"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            class="p-2 bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-200 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors"
            on:click={resetDeepLDefaults}
            title="Reset to default"
          >
            <FontAwesomeIcon icon="undo" />
          </button>
        </div>
      </div>

      <div class="flex items-center gap-3 mt-4">
        <Button 
          variant="primary" 
          icon="save" 
          on:click={handleSaveDeepLConfig}
        >
          Save DeepL Settings
        </Button>

        {#if deeplSaveSuccess}
          <div class="text-green-600 dark:text-green-400 flex items-center">
            <FontAwesomeIcon icon="check-circle" class="mr-1" />
            <span class="text-sm">Settings saved</span>
          </div>
        {/if}

        {#if deeplSaveError}
          <div class="text-red-600 dark:text-red-400 flex items-center">
            <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
            <span class="text-sm">{deeplSaveError}</span>
          </div>
        {/if}
      </div>
    </div>
  </section>
</div>