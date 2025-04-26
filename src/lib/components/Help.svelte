<script lang="ts">
  import { onMount } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import { shortcuts } from "../utils/shortcutsHelp";
  import "../fa-icons";

  let isOpen = false;

  // Markdown syntax help
  const markdownHelp = [
    { syntax: "**bold text**", description: "Bold text" },
    { syntax: "*italic text*", description: "Italic text" },
    { syntax: "- item", description: "List item" },
    { syntax: "1. item", description: "Ordered list item" },
    { syntax: "[text](url)", description: "Link" },
  ];

  export function open() {
    isOpen = true;
  }

  export function close() {
    isOpen = false;
  }

  function handleOutsideClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (isOpen && !target.closest(".help-panel")) {
      close();
    }
  }

  onMount(() => {
    document.addEventListener("click", handleOutsideClick);
    return () => {
      document.removeEventListener("click", handleOutsideClick);
    };
  });
</script>

{#if isOpen}
  <div class="help-overlay">
    <div class="help-panel">
      <h2>
        <FontAwesomeIcon icon="info-circle" />
        <span>Help</span>
      </h2>

      <div class="help-section">
        <h3>Keyboard Shortcuts</h3>
        <div class="shortcuts-list">
          {#each shortcuts as shortcut}
            <div class="markdown-item">
              <kbd>{shortcut.key}</kbd>
              <span>{shortcut.action}</span>
            </div>
          {/each}
        </div>
      </div>

      <div class="help-section">
        <h3>Markdown Formatting</h3>
        <div class="markdown-help">
          {#each markdownHelp as help}
            <div class="markdown-item">
              <code>{help.syntax}</code>
              <span>{help.description}</span>
            </div>
          {/each}
        </div>
      </div>

      <div class="help-section">
        <h3>Features</h3>
        <ul
          class="list-disc pl-5 text-gray-600 dark:text-gray-400 space-y-1"
        >
          <li>Seven color-coded tabs for quick note organization</li>
          <li>Markdown support for basic formatting</li>
          <li>Dark and light themes</li>
          <li>Automatic saving</li>
          <li>Nextcloud sync for cross-device access</li>
          <li>Backup and restore capability</li>
          <li>AI-powered writing assistance</li>
          <li>Grammar checking and translation tools</li>
        </ul>
      </div>

      <div class="help-section">
        <h3>AI Features</h3>
        <p>
          Jot includes integration with ChatGPT for enhanced writing assistance:
        </p>
        <ul>
          <li>
            <strong>Chat with ChatGPT</strong>: Use selected text as a prompt or
            create a new prompt
          </li>
          <li>
            <strong>Summarize</strong>: Automatically summarize your notes using
            AI
          </li>
        </ul>
        <p>
          To use these features, you need to provide your OpenAI API key in the
          AI Settings.
        </p>
      </div>

      <div class="about-section">
        <h3>About Jot</h3>
        <p>
          A minimal note-taking app for quick thoughts and ideas. Organize your
          notes with color-coded tabs and simple markdown formatting.
        </p>
        <p>Version 1.0.0</p>
      </div>

      <div class="footer">
        <button class="close-btn" on:click={close}> Close </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .help-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease;
  }

  .help-panel {
    width: 500px;
    max-width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    background-color: var(--bg-color);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    padding: 24px;
    animation: slideIn 0.3s ease;
  }

  h2 {
    margin-top: 0;
    margin-bottom: 20px;
    color: var(--text-color);
    font-size: 22px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  h3 {
    font-size: 18px;
    margin-bottom: 12px;
    color: var(--text-color);
  }

  .help-section {
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  :global(.app.dark) .help-section {
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  }

  .about-section {
    margin-bottom: 24px;
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow-y: auto;
  }

  kbd {
    display: inline-block;
    padding: 4px 8px;
    background-color: rgba(0, 0, 0, 0.07);
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    font-family: monospace;
    font-size: 14px;
    min-width: 80px;
    text-align: center;
  }

  :global(.app.dark) kbd {
    background-color: rgba(255, 255, 255, 0.15);
    border: 1px solid rgba(255, 255, 255, 0.25);
    color: #e0e0e0;
  }

  .markdown-help {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 300px;
    overflow-y: auto;
  }

  .markdown-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.02);
  }

  :global(.app.dark) .markdown-item {
    background-color: rgba(255, 255, 255, 0.07);
  }

  code {
    font-family: monospace;
    background-color: rgba(0, 0, 0, 0.05);
    padding: 4px 6px;
    border-radius: 3px;
    font-size: 14px;
  }

  :global(.app.dark) code {
    background-color: rgba(255, 255, 255, 0.15);
    color: #e0e0e0;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    margin-top: 20px;
  }

  button {
    background-color: rgba(0, 0, 0, 0.05);
    border: none;
    border-radius: 4px;
    padding: 8px 16px;
    cursor: pointer;
    color: var(--text-color);
    font-weight: 500;
    transition: all 0.2s ease;
  }

  button:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  :global(.app.dark) button {
    background-color: rgba(255, 255, 255, 0.15);
    color: #e0e0e0;
  }

  :global(.app.dark) button:hover {
    background-color: rgba(255, 255, 255, 0.25);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideIn {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style>
