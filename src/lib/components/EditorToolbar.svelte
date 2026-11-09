<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
  import type { IconName } from "@fortawesome/fontawesome-svg-core";

  export let hasSelection = false;

  const STORAGE_KEY = "jot-toolbar-collapsed";

  // Vom Nutzer gewählter Zustand (persistiert)
  let collapsed =
    typeof localStorage !== "undefined" &&
    localStorage.getItem(STORAGE_KEY) === "true";
  // Bei Selektion automatisch eingeblendet, solange die Selektion besteht
  let autoShown = false;
  let prevHasSelection = false;

  $: {
    if (hasSelection && !prevHasSelection && collapsed) autoShown = true;
    if (!hasSelection) autoShown = false;
    prevHasSelection = hasSelection;
  }

  $: visible = !collapsed || autoShown;

  function toggle() {
    if (visible) {
      collapsed = true;
      autoShown = false;
    } else {
      collapsed = false;
    }
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(STORAGE_KEY, String(collapsed));
    }
  }

  const dispatch = createEventDispatcher<{ action: string }>();

  type Item = {
    action: string;
    icon: IconName;
    label: string;
    needsSelection?: boolean;
  };
  type Group = Item[];

  const groups: Group[] = [
    [
      {
        action: "bold",
        icon: "bold",
        label: "Bold (Ctrl+B)",
        needsSelection: true,
      },
      {
        action: "italic",
        icon: "italic",
        label: "Italic (Ctrl+I)",
        needsSelection: true,
      },
      {
        action: "strike",
        icon: "strikethrough",
        label: "Strikethrough (Ctrl+Shift+X)",
        needsSelection: true,
      },
      { action: "code", icon: "code", label: "Inline code (Ctrl+`)" },
    ],
    [
      { action: "link", icon: "link", label: "Link (Ctrl+K)" },
      { action: "timestamp", icon: "clock", label: "Timestamp (Ctrl+T)" },
    ],
    [
      { action: "outdent", icon: "outdent", label: "Outdent (Shift+Tab)" },
      { action: "indent", icon: "indent", label: "Indent (Tab)" },
    ],
    [
      { action: "undo", icon: "rotate-left", label: "Undo (Ctrl+Z)" },
      { action: "redo", icon: "rotate-right", label: "Redo (Ctrl+Y)" },
    ],
  ];

  // preventDefault verhindert Fokusverlust im Editor (Selektion + Android-Tastatur bleiben erhalten)
  function keepFocus(event: Event) {
    event.preventDefault();
  }
</script>

<div
  class="toolbar"
  class:collapsed={!visible}
  role="toolbar"
  aria-label="Formatting"
>
  <button
    type="button"
    class="tool toggle"
    title={visible ? "Hide toolbar" : "Show toolbar"}
    aria-label={visible ? "Hide toolbar" : "Show toolbar"}
    aria-expanded={visible}
    on:pointerdown={keepFocus}
    on:mousedown={keepFocus}
    on:click={toggle}
  >
    <!-- {#key}: FontAwesomeIcon aktualisiert das Icon bei Prop-Änderung nicht -->
    {#key visible}
      <FontAwesomeIcon icon={visible ? "times" : "ellipsis"} />
    {/key}
  </button>
  {#each groups as group, i}
    {#if visible}
      <span class="separator" aria-hidden="true"></span>
    {/if}
    {#each group as item}
      {#if visible}
        <button
          type="button"
          class="tool"
          disabled={item.needsSelection && !hasSelection}
          title={item.label}
          aria-label={item.label}
          on:pointerdown={keepFocus}
          on:mousedown={keepFocus}
          on:click={() => dispatch("action", item.action)}
        >
          <FontAwesomeIcon icon={item.icon} />
        </button>
      {/if}
    {/each}
  {/each}
</div>

<style>
  .toolbar {
    position: absolute;
    left: 50%;
    bottom: calc(12px + env(safe-area-inset-bottom, 0px));
    transform: translateX(-50%);
    max-width: calc(100% - 16px);
    display: flex;
    align-items: center;
    gap: 0;
    padding: 3px 6px;
    overflow-x: auto;
    scrollbar-width: none;
    border-radius: 999px;
    background: rgba(250, 250, 250, 0.82);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(0, 0, 0, 0.08);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.12);
    color: #444;
  }

  /* Eingeklappt: nur Toggle-Button, unten rechts */
  .toolbar.collapsed {
    left: auto;
    right: calc(12px + env(safe-area-inset-right, 0px));
    transform: none;
  }

  :global(body.dark) .toolbar {
    background: rgba(40, 40, 40, 0.82);
    border-color: rgba(255, 255, 255, 0.1);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.5);
    color: #ddd;
  }

  .toolbar::-webkit-scrollbar {
    display: none;
  }

  .tool {
    flex: none;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    color: inherit;
    background: transparent;
    border: none;
    border-radius: 999px;
    opacity: 0.7;
    cursor: pointer;
    transition:
      background 0.12s,
      opacity 0.12s;
  }

  .tool:hover:not(:disabled),
  .tool:active:not(:disabled) {
    opacity: 1;
    background: rgba(128, 128, 128, 0.2);
  }

  .tool:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .separator {
    flex: none;
    width: 1px;
    height: 16px;
    margin: 0 4px;
    background: rgba(128, 128, 128, 0.3);
  }

  /* Touch: größere Trefferflächen */
  @media (pointer: coarse) {
    .tool {
      width: 44px;
      height: 44px;
      font-size: 15px;
    }
  }
</style>
