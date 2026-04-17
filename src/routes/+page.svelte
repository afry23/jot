<script lang="ts">
  import { onMount } from "svelte";
  import Header from "$lib/components/Header.svelte";
  import EditorContainer from "$lib/components/EditorContainer.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import { activeTab } from "$lib/stores/tabs";
  import { notes, updateNote } from "$lib/stores/notes";
  import { theme, fontSize, loadSettings } from "$lib/stores/settings";
  import { loadNotes, saveNote } from "$lib/utils/persistence";
  import "../app.css";
  import { setupKeyboardShortcuts } from "$lib/stores/keyboardShortcuts";
  import { initializeHistory } from "$lib/stores/history";
  import { logger } from "$lib/utils/logger";
  import { Window } from "@tauri-apps/api/window";

  let loading = true;

  // Load notes and settings on mount
  onMount(() => {
    // Load application settings and notes asynchronously
    const loadAppData = async () => {
      logger.info("Loading application data");

      try {
        // Load settings first so we get the active tab
        logger.info("Loading settings");
        await loadSettings();
        // Then load notes content
        logger.info("Loading notes");
        await loadNotes();
        initializeHistory($notes);
      } catch (error) {
        logger.error("Failed to load application data:", error);
      } finally {
        loading = false;
        logger.info("Application data loaded successfully");
      }
    };
    loadAppData();

    // Setup global keyboard shortcuts
    let cleanupShortcuts: (() => void) | undefined;
    setupKeyboardShortcuts().then((cleanup) => {
      cleanupShortcuts = cleanup;
    });

    Window.getCurrent().onFocusChanged(async (event) => {
      if (event.event) {
        // Focus the CodeMirror editor when the window becomes visible
        setTimeout(() => {
          const editor = document.querySelector(".cm-content") as HTMLElement;
          if (editor) editor.focus();
        }, 100);
      }
    });

    // Cleanup function
    return () => {
      if (cleanupShortcuts) cleanupShortcuts();
    };
  });

  // Auto-save when content changes
  $: if ($notes[$activeTab]) {
    saveNote($activeTab, $notes[$activeTab]);
  }
</script>

<svelte:head>
  <title>Jot</title>
</svelte:head>

<div
  class="app"
  class:dark={$theme === "dark"}
  class:light={$theme === "light"}
  class:font-size-small={$fontSize === "small"}
  class:font-size-medium={$fontSize === "medium"}
  class:font-size-large={$fontSize === "large"}
>
  <Header />
  <main>
    {#if loading}
      <div class="loading-screen">
        <p>Loading...</p>
      </div>
    {:else}
      <EditorContainer />
    {/if}
  </main>
  <StatusBar />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
    background-color: #f7f7f7;
    color: #333;
    transition:
      background-color 0.3s,
      color 0.3s;
  }

  :global(body.dark) {
    background-color: #121212;
    color: #e0e0e0;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    --bg-color: #f5f5f5;
    --text-color: #333;
    --accent-color: #f0a05a;
    --tab-inactive: #888;
    background-color: var(--bg-color);
    color: var(--text-color);
    transition:
      background-color 0.3s,
      color 0.3s;
  }

  .app.dark {
    --bg-color: #222;
    --text-color: #e0e0e0;
  }

  /* Font size styles for global application */
  .app.font-size-small {
    --base-font-size: 14px;
  }

  .app.font-size-medium {
    --base-font-size: 16px;
  }

  .app.font-size-large {
    --base-font-size: 18px;
  }

  main {
    flex: 1;
    overflow: hidden;
    padding: 0;
    margin: 0;
  }

  .loading-screen {
    text-align: center;
    margin-top: 2rem;
    font-size: 1.2em;
    color: var(--text-color);
  }
</style>
