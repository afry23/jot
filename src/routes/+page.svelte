<script lang="ts">
  import { onMount } from 'svelte';
  import Header from '$lib/components/Header.svelte';
  import EditorContainer from '$lib/components/EditorContainer.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import { activeTab } from '$lib/stores/tabs';
  import { notes } from '$lib/stores/notes';
  import { theme, loadSettings } from '$lib/stores/settings';
  import { loadNotes, saveNote } from '$lib/utils/persistence';
  import '../app.css';

  // Load notes and settings on mount
  onMount(async () => {
    await loadSettings();
    await loadNotes();
  });

  // Auto-save when content changes
  $: if ($notes[$activeTab]) {
    saveNote($activeTab, $notes[$activeTab]);
  }
</script>

<svelte:head>
  <title>Jot</title>
</svelte:head>

<div class="app" class:dark={$theme === 'dark'} class:light={$theme === 'light'}>
  <Header />
  <main>
    <EditorContainer />
  </main>
  <StatusBar />
</div>

<style>
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
    transition: background-color 0.3s, color 0.3s;
  }

  .app.dark {
    --bg-color: #222;
    --text-color: #e0e0e0;
  }

  main {
    flex: 1;
    overflow: hidden;
    padding: 0;
    margin: 0;
  }
</style>