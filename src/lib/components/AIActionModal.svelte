<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import type { IconProp } from "@fortawesome/fontawesome-svg-core";
  import {
    REWRITE_STYLES,
    TRANSLATE_LANGUAGES,
    processText,
    chatGPTConfig,
  } from "$lib/stores/chatgptService";

  export let selectedText: string = "";
  export let defaultStyle: string = "formal_sie";

  type Action = "rewrite" | "expand" | "translate";

  const dispatch = createEventDispatcher<{ submit: { result: string }; cancel: void }>();

  let action: Action = "rewrite";
  let selectedStyle: string = defaultStyle;
  let customPrompt: string = "";
  let targetLang: string = "en";
  let isLoading = false;
  let error: string | null = null;

  const actions: { key: Action; label: string; icon: IconProp }[] = [
    { key: "rewrite", label: "Umformulieren", icon: "exchange-alt" },
    { key: "expand", label: "Erweitern", icon: "edit" },
    { key: "translate", label: "Übersetzen", icon: "language" },
  ];

  onMount(() => {
    selectedStyle = $chatGPTConfig.default_rewrite_style ?? defaultStyle;
  });

  async function handleSubmit() {
    if (isLoading) return;
    isLoading = true;
    error = null;

    try {
      const result = await processText(selectedText, action, {
        styleKey: selectedStyle,
        customPrompt: customPrompt || undefined,
        targetLang,
      });
      dispatch("submit", { result });
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  function handleCancel() {
    dispatch("cancel");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") handleCancel();
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) handleSubmit();
  }

  const styleKeys = Object.keys(REWRITE_STYLES);
  const langKeys = Object.keys(TRANSLATE_LANGUAGES);
</script>

<div
  class="modal-backdrop"
  role="button"
  tabindex="-1"
  on:click={handleCancel}
  on:keydown={handleKeydown}
></div>

<div class="modal" role="dialog" aria-modal="true">
  <div class="modal-header">
    <FontAwesomeIcon icon="magic" />
    <span>KI-Textbearbeitung</span>
  </div>

  <div class="preview">
    <span class="preview-label">Ausgewählter Text</span>
    <p class="preview-text">{selectedText.length > 120 ? selectedText.slice(0, 120) + "…" : selectedText}</p>
  </div>

  <div class="action-tabs">
    {#each actions as a}
      <button
        class="action-tab"
        class:active={action === a.key}
        on:click={() => { action = a.key; error = null; }}
        type="button"
      >
        <FontAwesomeIcon icon={a.icon} />
        <span>{a.label}</span>
      </button>
    {/each}
  </div>

  {#if action === "rewrite"}
    <div class="style-grid">
      {#each styleKeys as key}
        <button
          class="style-pill"
          class:selected={selectedStyle === key}
          on:click={() => selectedStyle = key}
          type="button"
        >
          {REWRITE_STYLES[key].label}
        </button>
      {/each}
    </div>
    {#if selectedStyle === "custom"}
      <textarea
        class="custom-prompt"
        bind:value={customPrompt}
        placeholder="Anweisung eingeben, z.B. 'Schreibe sachlicher und kürzer'"
        rows="3"
      ></textarea>
    {/if}
  {:else if action === "translate"}
    <div class="lang-row">
      <label for="target-lang" class="lang-label">Zielsprache</label>
      <select id="target-lang" class="lang-select" bind:value={targetLang}>
        {#each langKeys as key}
          <option value={key}>{TRANSLATE_LANGUAGES[key]}</option>
        {/each}
      </select>
    </div>
  {:else}
    <p class="expand-hint">Der Text wird mit mehr Details und Kontext ausgebaut.</p>
  {/if}

  {#if error}
    <div class="error-box">
      <FontAwesomeIcon icon="exclamation-triangle" />
      <span>{error}</span>
    </div>
  {/if}

  <div class="modal-actions">
    <button class="btn-cancel" on:click={handleCancel} type="button">Abbrechen</button>
    <button
      class="btn-submit"
      on:click={handleSubmit}
      type="button"
      disabled={isLoading || (action === "rewrite" && selectedStyle === "custom" && !customPrompt.trim())}
    >
      {#if isLoading}
        <FontAwesomeIcon icon="spinner" spin />
        <span>Wird bearbeitet…</span>
      {:else}
        <FontAwesomeIcon icon="magic" />
        <span>Ausführen</span>
      {/if}
    </button>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    z-index: 1000;
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--background, #fff);
    color: var(--foreground, #222);
    padding: 1.5em;
    border-radius: 10px;
    z-index: 1001;
    width: 400px;
    max-width: 92vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.22);
    display: flex;
    flex-direction: column;
    gap: 1em;
  }

  :global(.dark) .modal {
    background: #1e1e1e;
    color: #e0e0e0;
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: 0.5em;
    font-weight: 600;
    font-size: 1rem;
    color: #f0a05a;
  }

  .preview {
    background: rgba(0, 0, 0, 0.04);
    border-radius: 6px;
    padding: 0.6em 0.8em;
  }

  :global(.dark) .preview {
    background: rgba(255, 255, 255, 0.06);
  }

  .preview-label {
    font-size: 0.72em;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.55;
    display: block;
    margin-bottom: 0.3em;
  }

  .preview-text {
    font-size: 0.88em;
    margin: 0;
    opacity: 0.85;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .action-tabs {
    display: flex;
    gap: 0.4em;
  }

  .action-tab {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4em;
    padding: 0.5em 0.75em;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.12);
    background: transparent;
    color: inherit;
    font-size: 0.82em;
    cursor: pointer;
    transition: all 0.15s;
    opacity: 0.7;
  }

  :global(.dark) .action-tab {
    border-color: rgba(255, 255, 255, 0.12);
  }

  .action-tab:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.05);
  }

  :global(.dark) .action-tab:hover {
    background: rgba(255, 255, 255, 0.07);
  }

  .action-tab.active {
    background: rgba(240, 160, 90, 0.12);
    border-color: #f0a05a;
    color: #f0a05a;
    opacity: 1;
    font-weight: 600;
  }

  .style-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4em;
  }

  .style-pill {
    padding: 0.35em 0.75em;
    border-radius: 999px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    background: transparent;
    color: inherit;
    font-size: 0.82em;
    cursor: pointer;
    transition: all 0.15s;
  }

  :global(.dark) .style-pill {
    border-color: rgba(255, 255, 255, 0.15);
  }

  .style-pill:hover {
    background: rgba(0, 0, 0, 0.06);
  }

  :global(.dark) .style-pill:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .style-pill.selected {
    background: #f0a05a;
    border-color: #f0a05a;
    color: #fff;
    font-weight: 600;
  }

  .custom-prompt {
    width: 100%;
    padding: 0.5em 0.75em;
    border: 1px solid rgba(0, 0, 0, 0.15);
    border-radius: 6px;
    background: transparent;
    color: inherit;
    font-size: 0.88em;
    resize: vertical;
    box-sizing: border-box;
    font-family: inherit;
  }

  :global(.dark) .custom-prompt {
    border-color: rgba(255, 255, 255, 0.15);
  }

  .custom-prompt:focus {
    outline: none;
    border-color: #f0a05a;
  }

  .lang-row {
    display: flex;
    align-items: center;
    gap: 0.75em;
  }

  .lang-label {
    font-size: 0.88em;
    opacity: 0.75;
    white-space: nowrap;
  }

  .lang-select {
    flex: 1;
    padding: 0.45em 0.75em;
    border: 1px solid rgba(0, 0, 0, 0.15);
    border-radius: 6px;
    background: transparent;
    color: inherit;
    font-size: 0.88em;
    cursor: pointer;
  }

  :global(.dark) .lang-select {
    border-color: rgba(255, 255, 255, 0.15);
    background: #2a2a2a;
  }

  .lang-select:focus {
    outline: none;
    border-color: #f0a05a;
  }

  .expand-hint {
    font-size: 0.85em;
    opacity: 0.65;
    margin: 0;
    font-style: italic;
  }

  .error-box {
    display: flex;
    align-items: flex-start;
    gap: 0.5em;
    background: rgba(220, 38, 38, 0.08);
    border: 1px solid rgba(220, 38, 38, 0.25);
    border-radius: 6px;
    padding: 0.6em 0.8em;
    font-size: 0.85em;
    color: #dc2626;
  }

  :global(.dark) .error-box {
    background: rgba(220, 38, 38, 0.12);
    color: #f87171;
  }

  .modal-actions {
    display: flex;
    gap: 0.6em;
    justify-content: flex-end;
    margin-top: 0.25em;
  }

  .btn-cancel {
    padding: 0.5em 1em;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    background: transparent;
    color: inherit;
    font-size: 0.88em;
    cursor: pointer;
    transition: background 0.15s;
  }

  :global(.dark) .btn-cancel {
    border-color: rgba(255, 255, 255, 0.15);
  }

  .btn-cancel:hover {
    background: rgba(0, 0, 0, 0.06);
  }

  :global(.dark) .btn-cancel:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .btn-submit {
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.5em 1.1em;
    border-radius: 6px;
    border: none;
    background: #f0a05a;
    color: #fff;
    font-size: 0.88em;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-submit:hover:not(:disabled) {
    background: #e09050;
  }

  .btn-submit:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
