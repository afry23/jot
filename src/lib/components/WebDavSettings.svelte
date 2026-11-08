<script lang="ts">
  import { onMount } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import "../fa-icons";
  import Button from "./Button.svelte";
  import FormField from "./FormField.svelte";
  import {
    webdavConfig,
    isSyncing,
    lastSync,
    lastSyncResult,
    syncError,
    loadWebdavConfig,
    hasWebdavPassword,
    saveWebdavConfig,
    testWebdavConnection,
    syncNow,
    type WebDavConfig,
  } from "#lib/stores/webdavStore";

  let enabled = false;
  let url = "";
  let username = "";
  let remoteDir = "jot";
  let password = "";
  let passwordStored = false;

  let testing = false;
  let testOk = false;
  let message: { kind: "success" | "error"; text: string } | null = null;

  onMount(async () => {
    const config = await loadWebdavConfig();
    enabled = config.enabled;
    url = config.url;
    username = config.username;
    remoteDir = config.remoteDir;
    passwordStored = await hasWebdavPassword(username);
  });

  function currentConfig(): WebDavConfig {
    return {
      enabled,
      url: url.trim(),
      username: username.trim(),
      remoteDir: remoteDir.trim(),
    };
  }

  $: insecure = url.trim().toLowerCase().startsWith("http://");

  async function handleTest() {
    testing = true;
    testOk = false;
    message = null;
    try {
      await testWebdavConnection(currentConfig(), password);
      testOk = true;
      message = { kind: "success", text: "Connection successful" };
    } catch (error) {
      message = { kind: "error", text: `${error}` };
    } finally {
      testing = false;
    }
  }

  async function handleSave() {
    message = null;
    try {
      await saveWebdavConfig(currentConfig(), password);
      passwordStored = await hasWebdavPassword(username.trim());
      password = "";
      message = { kind: "success", text: "Settings saved" };
      if (enabled) await syncNow();
    } catch (error) {
      message = { kind: "error", text: `${error}` };
    }
  }

  async function handleSync() {
    message = null;
    // Make sure the backend uses what is currently entered
    await saveWebdavConfig({ ...currentConfig(), enabled }, password);
    await syncNow();
  }
</script>

<div class="space-y-6">
  <section>
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">
      WebDAV Sync
    </h3>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
      Sync your notes with a WebDAV server (e.g. Nextcloud). Notes are synced at
      startup and shortly after you stop typing. If both sides changed, the
      newer version wins and the older one is kept on the server as
      <code>note_N.conflict-&lt;date&gt;.md</code>.
    </p>

    <label class="flex items-center gap-2 mb-4 text-sm">
      <input type="checkbox" bind:checked={enabled} />
      Enable WebDAV sync
    </label>

    <FormField
      label="Server URL"
      id="webdav-url"
      bind:value={url}
      placeholder="https://cloud.example.com/remote.php/dav/files/user"
    />
    {#if insecure}
      <p class="text-sm text-yellow-700 dark:text-yellow-400 mb-4">
        <FontAwesomeIcon icon="exclamation-triangle" class="mr-1" />
        Unencrypted HTTP exposes your password and notes, and is blocked on Android.
        Use HTTPS.
      </p>
    {/if}
    <FormField label="Username" id="webdav-user" bind:value={username} />
    <FormField
      label="Password"
      id="webdav-password"
      type="password"
      bind:value={password}
      placeholder={passwordStored ? "Stored – leave empty to keep" : ""}
    />
    <FormField
      label="Remote folder"
      id="webdav-dir"
      bind:value={remoteDir}
      hint="Created automatically if missing"
    />

    <div class="flex flex-wrap gap-2 mt-2">
      <Button
        variant="outline"
        on:click={handleTest}
        loading={testing}
        disabled={testing}
      >
        Test connection
      </Button>
      <Button variant="primary" on:click={handleSave}>Save</Button>
      <Button
        variant="default"
        on:click={handleSync}
        loading={$isSyncing}
        disabled={$isSyncing || !enabled}
      >
        <FontAwesomeIcon icon="cloud" class="mr-2" />
        Sync now
      </Button>
    </div>

    {#if message}
      <div
        class="mt-2 text-sm flex items-center {message.kind === 'success'
          ? 'text-green-600 dark:text-green-400'
          : 'text-red-600 dark:text-red-400'}"
      >
        <FontAwesomeIcon
          icon={message.kind === "success"
            ? "check-circle"
            : "exclamation-circle"}
          class="mr-1"
        />
        <span>{message.text}</span>
      </div>
    {/if}
  </section>

  <section class="pt-6 border-t border-gray-200 dark:border-gray-700">
    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-200 mb-2">
      Status
    </h3>
    {#if $syncError}
      <div class="text-sm text-red-600 dark:text-red-400 flex items-center">
        <FontAwesomeIcon icon="exclamation-circle" class="mr-1" />
        <span>Last sync failed: {$syncError}</span>
      </div>
    {:else if $lastSync}
      <div class="text-sm text-gray-700 dark:text-gray-300">
        Last sync: {$lastSync.toLocaleString()}
        {#if $lastSyncResult}
          – {$lastSyncResult.pushed.length} uploaded, {$lastSyncResult.pulled
            .length} downloaded{#if $lastSyncResult.conflicts.length > 0}, {$lastSyncResult
              .conflicts.length} conflict(s) (copy kept on server){/if}
        {/if}
      </div>
    {:else}
      <div class="text-sm text-gray-500 dark:text-gray-400">
        Not synced yet.
      </div>
    {/if}
  </section>
</div>
