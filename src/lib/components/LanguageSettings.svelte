<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import {
    languageConfig,
    loadLanguageServicesConfig,
    saveLanguageToolConfig,
  } from "$lib/stores/languageServices";
  import Button from "./Button.svelte";
  import FormField from "./FormField.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { SettingsCommands } from "$lib/utils/tauriCommands";

  // Local state
  let languagetoolApiKey: string = "";
  let languagetoolUsername: string = "";
  let languagetoolEndpoint: string = "";
  let hasStoredApiKey: boolean = false;

  // Success/error messages
  let ltSaveSuccess: boolean = false;
  let ltSaveError: string | null = null;
  let ltSaveTimeout: ReturnType<typeof setTimeout> | null = null;

  // Load current configuration
  onMount(async () => {
    await loadLanguageServicesConfig();

    if ($languageConfig) {
      languagetoolUsername = $languageConfig.languagetool_username || "";
      languagetoolEndpoint = $languageConfig.languagetool_endpoint;
    }

    await checkCredentialStatus();
  });

  // Clear timeouts on destroy
  onDestroy(() => {
    if (ltSaveTimeout) clearTimeout(ltSaveTimeout);
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

  async function checkCredentialStatus() {
    if (languagetoolUsername) {
      const hasCredential = await invoke<boolean>(SettingsCommands.HAS_LANGUAGETOOL_API_KEY, { username: languagetoolUsername });
      hasStoredApiKey = !!hasCredential;
      languagetoolApiKey = "";
    }
  }

  // Reset to default configuration
  function resetLTDefaults() {
    languagetoolEndpoint = "https://api.languagetoolplus.com/v2/check";
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
          placeholder={hasStoredApiKey ? "Neuen Key eingeben um zu ersetzen" : "LanguageTool API Key eingeben"}
          label="API Key"
        />
        {#if hasStoredApiKey}
          <p class="mt-1 text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
            <FontAwesomeIcon icon="check-circle" />
            API Key gespeichert – Feld leer lassen um beizubehalten
          </p>
        {:else}
          <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">Wird zusammen mit dem Benutzernamen für die Authentifizierung verwendet</p>
        {/if}
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
</div>