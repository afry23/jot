<script lang="ts">
  import { onMount, tick } from "svelte";
  import { activeTab } from "$lib/stores/tabs";
  import { notes, updateNote } from "$lib/stores/notes";
  import { theme, fontSize } from "$lib/stores/settings";
  import { viewMode } from "$lib/stores/viewMode";
  import { formatMarkdown } from "$lib/utils/textFormatting";
  import { tabColors, withOpacity } from "$lib/utils/colors";
  import { undoHistory, redoHistory, undo, redo } from "$lib/stores/history";
  import { saveNote } from "$lib/utils/persistence";

  // Reference to editor elements
  let plainTextEditor: HTMLTextAreaElement;
  let previewContainer: HTMLDivElement;

  // Get the appropriate color for the current tab
  $: currentTabColor = tabColors[$activeTab];
  $: mutedColor = withOpacity(currentTabColor, 0.5);
  $: isCurrentTabEmpty =
    !$notes[$activeTab] || $notes[$activeTab].trim() === "";
  $: displayColor = isCurrentTabEmpty ? mutedColor : currentTabColor;
  $: canUndo = $undoHistory[$activeTab] && $undoHistory[$activeTab].length > 0;
  $: canRedo = $redoHistory[$activeTab] && $redoHistory[$activeTab].length > 0;

  // Font size mappings
  const fontSizes = {
    small: {
      editor: "14px",
      h1: "1.6em",
      h2: "1.4em",
      h3: "1.2em",
      lineHeight: "1.4",
    },
    medium: {
      editor: "16px",
      h1: "1.8em",
      h2: "1.5em",
      h3: "1.3em",
      lineHeight: "1.5",
    },
    large: {
      editor: "18px",
      h1: "2em",
      h2: "1.7em",
      h3: "1.5em",
      lineHeight: "1.6",
    },
  };

  // Track editor state
  let previousTab = -1;
  let previousViewMode = $viewMode;
  let previousFontSize = $fontSize;

  // Initial cursor position - default to end of text
  let cursorPosition = $notes[$activeTab] ? $notes[$activeTab].length : 0;
  let scrollTop = 0;

  // Key shortcuts for formatting
  async function handleKeydown(event: KeyboardEvent) {
    // Update cursor position on any keydown
    if (plainTextEditor) {
      cursorPosition = plainTextEditor.selectionStart;
    }

    // Insert Date/Time shortcut
    if (event.ctrlKey && event.key === "t") {
      event.preventDefault();

      // Format current date and time
      const now = new Date();
      const formattedDate = now.toLocaleString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });

      const start = plainTextEditor.selectionStart;
      const end = plainTextEditor.selectionEnd;

      // Insert timestamp at cursor position
      const formattedTimestamp = `**${formattedDate}** `;
      const currentContent = $notes[$activeTab] || "";
      const newContent =
        currentContent.substring(0, start) +
        formattedTimestamp +
        currentContent.substring(end);

      updateNote($activeTab, newContent);

      // Move cursor after the inserted timestamp
      await tick();
      plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
        start + formattedTimestamp.length;
      cursorPosition = plainTextEditor.selectionStart;
    }

    // Tab key in plain text editor
    if (event.key === "Tab" && $viewMode === "edit") {
      event.preventDefault();
      const currentContent = $notes[$activeTab] || "";
      const start = plainTextEditor.selectionStart;
      const end = plainTextEditor.selectionEnd;

      // Handle selected text indentation
      if (start !== end) {
        const selectedText = currentContent.substring(start, end);

        if (event.shiftKey) {
          // Unindent selected lines
          const lines = selectedText.split("\n");
          const unindented = lines
            .map((line) => (line.startsWith("  ") ? line.substring(2) : line))
            .join("\n");

          const newContent =
            currentContent.substring(0, start) +
            unindented +
            currentContent.substring(end);

          updateNote($activeTab, newContent);

          // Schedule cursor position update after DOM updates
          await tick();
          plainTextEditor.value = newContent;
          plainTextEditor.selectionStart = start;
          plainTextEditor.selectionEnd = start + unindented.length;
        } else {
          // Indent selected lines
          const indented = selectedText
            .split("\n")
            .map((line) => "  " + line)
            .join("\n");

          const newContent =
            currentContent.substring(0, start) +
            indented +
            currentContent.substring(end);

          updateNote($activeTab, newContent);

          // Schedule cursor position update after DOM updates
          await tick();
          plainTextEditor.value = newContent;
          plainTextEditor.selectionStart = start;
          plainTextEditor.selectionEnd = start + indented.length;
        }
      } else {
        // No selection, just insert or remove tab at cursor position
        if (event.shiftKey) {
          // Check if there are spaces before the cursor that can be removed
          const lineStart = currentContent.lastIndexOf("\n", start - 1) + 1;
          const linePrefix = currentContent.substring(lineStart, start);

          // If there are at least 2 spaces at the beginning of the line, remove them
          if (linePrefix.startsWith("  ")) {
            const newContent =
              currentContent.substring(0, lineStart) +
              linePrefix.substring(2) +
              currentContent.substring(start);

            updateNote($activeTab, newContent);

            // Move cursor to new position
            await tick();
            plainTextEditor.value = newContent;
            plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
              Math.max(start - 2, lineStart);
          }
        } else {
          // Regular tab - insert spaces
          const newContent =
            currentContent.substring(0, start) +
            "  " +
            currentContent.substring(end);

          updateNote($activeTab, newContent);

          // Move cursor after the inserted tab
          await tick();
          plainTextEditor.value = newContent;
          plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
            start + 2;
        }
      }
    }

    // Add CTRL+B for bold formatting
    if (event.ctrlKey && event.key === "b") {
      event.preventDefault();

      const start = plainTextEditor.selectionStart;
      const end = plainTextEditor.selectionEnd;
      const currentContent = $notes[$activeTab] || "";

      if (start !== end) {
        // Text selected, wrap it in ** for bold
        const selectedText = currentContent.substring(start, end);
        const boldText = `**${selectedText}**`;

        const newContent =
          currentContent.substring(0, start) +
          boldText +
          currentContent.substring(end);
        updateNote($activeTab, newContent);

        // Adjust selection to include the ** markers
        setTimeout(() => {
          plainTextEditor.selectionStart = start;
          plainTextEditor.selectionEnd = start + boldText.length;
          cursorPosition = plainTextEditor.selectionStart;
        }, 0);
      } else {
        // No selection, just insert ** ** and place cursor in between
        const boldMarkers = "****";
        const newContent =
          currentContent.substring(0, start) +
          boldMarkers +
          currentContent.substring(end);
        updateNote($activeTab, newContent);

        // Place cursor between the markers
        await tick();
        plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
          start + 2;
        cursorPosition = plainTextEditor.selectionStart;
      }
    }

    // Add CTRL+I for italic formatting
    if (event.ctrlKey && event.key === "i") {
      event.preventDefault();

      const start = plainTextEditor.selectionStart;
      const end = plainTextEditor.selectionEnd;
      const currentContent = $notes[$activeTab] || "";

      if (start !== end) {
        // Text selected, wrap it in * for italic
        const selectedText = currentContent.substring(start, end);
        const italicText = `*${selectedText}*`;

        const newContent =
          currentContent.substring(0, start) +
          italicText +
          currentContent.substring(end);
        updateNote($activeTab, newContent);

        // Adjust selection to include the * markers
        await tick();
          plainTextEditor.selectionStart = start;
          plainTextEditor.selectionEnd = start + italicText.length;
          cursorPosition = plainTextEditor.selectionStart;
      } else {
        // No selection, just insert ** and place cursor in between
        const italicMarkers = "**";
        const newContent =
          currentContent.substring(0, start) +
          italicMarkers +
          currentContent.substring(end);
        updateNote($activeTab, newContent);

        // Place cursor between the markers
        await tick();
          plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
            start + 1;
          cursorPosition = plainTextEditor.selectionStart;
      }
    }
  }

  function handleInput() {
    if (plainTextEditor) {
      const newContent = plainTextEditor.value;
      updateNote($activeTab, newContent);
      cursorPosition = plainTextEditor.selectionStart;

      if (newContent === "") {
        saveNote($activeTab, newContent);
        cursorPosition = 0;
      }
    }
  }

  // Track cursor position
  function handleSelectionChange() {
    if (plainTextEditor) {
      cursorPosition = plainTextEditor.selectionStart;
    }
  }

  // Track scroll position
  function handleScroll() {
    if ($viewMode === "edit" && plainTextEditor) {
      scrollTop = plainTextEditor.scrollTop;
    } else if ($viewMode === "preview" && previewContainer) {
      scrollTop = previewContainer.scrollTop;
    }
  }

  // Apply font size to editor elements
  function applyFontSize(size = $fontSize) {
    if (!size || !fontSizes[size]) return;

    console.log(`Applying font size: ${size}`);

    // Apply to plain text editor
    if (plainTextEditor) {
      plainTextEditor.style.fontSize = fontSizes[size].editor;
      plainTextEditor.style.lineHeight = fontSizes[size].lineHeight;
    }

    // Apply to preview container
    if (previewContainer) {
      previewContainer.style.fontSize = fontSizes[size].editor;
      previewContainer.style.lineHeight = fontSizes[size].lineHeight;
    }
  }

  // Handle view mode transitions
  function handleViewModeChange() {
    console.log(`View mode changed from ${previousViewMode} to ${$viewMode}`);
    console.log(
      `Current cursor position: ${cursorPosition}, scroll: ${scrollTop}`,
    );

    if (previousViewMode === "edit" && $viewMode === "preview") {
      // Switching from edit to preview
      // Capture current position from edit mode
      if (plainTextEditor) {
        cursorPosition = plainTextEditor.selectionStart;
        scrollTop = plainTextEditor.scrollTop;
        console.log(
          `Saved from edit mode - cursor: ${cursorPosition}, scroll: ${scrollTop}`,
        );
      }

      // Apply to preview mode (with slight delay to ensure rendering)
      setTimeout(() => {
        if (previewContainer) {
          // Apply font size to preview
          applyFontSize($fontSize);
        }
      }, 50);
    } else if (previousViewMode === "preview" && $viewMode === "edit") {
      // Switching from preview to edit
      // Capture scroll position from preview
      if (previewContainer) {
        scrollTop = previewContainer.scrollTop;
        console.log(`Saved scroll from preview mode: ${scrollTop}`);
      }

      // Apply to edit mode
      setTimeout(() => {
        if (plainTextEditor) {
          // Apply the font size to edit mode
          applyFontSize($fontSize);

          // Set cursor position
          plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
            cursorPosition;
          // Set scroll position (approximately)
          plainTextEditor.scrollTop = scrollTop;
          // Focus the editor
          plainTextEditor.focus();
          console.log(
            `Applied to edit mode - cursor: ${cursorPosition}, scroll: ${scrollTop}`,
          );
        }
      }, 50);
    }

    previousViewMode = $viewMode;
  }

  // Update editor content when active tab changes
  $: if ($activeTab !== previousTab) {
    previousTab = $activeTab;
    updateEditorContent();
  }

  // Detect view mode changes and handle synchronization
  $: if ($viewMode !== previousViewMode) {
    handleViewModeChange();
  }

  // Detect font size changes and apply them
  $: if ($fontSize !== previousFontSize) {
    console.log(`Font size changed from ${previousFontSize} to ${$fontSize}`);
    previousFontSize = $fontSize;
    setTimeout(() => applyFontSize($fontSize), 10);
  }

  // Function to update editor content based on current tab
  function updateEditorContent() {
    if ($notes[$activeTab] !== undefined) {
      // Reset cursor position to end of text for new tab
      cursorPosition = $notes[$activeTab] ? $notes[$activeTab].length : 0;
      scrollTop = 0;

      // Update the UI
      setTimeout(() => {
        if ($viewMode === "edit" && plainTextEditor) {
          plainTextEditor.value = $notes[$activeTab] || "";

          // Focus and place cursor at the end if tab changed
          if (previousTab !== $activeTab) {
            plainTextEditor.focus();
            plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
              cursorPosition;
          }
        }

        // Apply font size
        applyFontSize($fontSize);
      }, 10);
    }
  }

  function handleUndo() {
    const previousContent = undo($activeTab);
    if (previousContent !== null) {
      notes.update((state) => {
        state[$activeTab] = previousContent;
        return state;
      });
    }
  }

  function handleRedo() {
    const nextContent = redo($activeTab);
    if (nextContent !== null) {
      notes.update((state) => {
        state[$activeTab] = nextContent;
        return state;
      });
    }
  }

  onMount(() => {
    // Set up listeners for cursor position tracking in edit mode
    if (plainTextEditor) {
      plainTextEditor.addEventListener("click", handleSelectionChange);
      plainTextEditor.addEventListener("keyup", handleSelectionChange);
      plainTextEditor.addEventListener("select", handleSelectionChange);
      plainTextEditor.addEventListener("scroll", handleScroll);
    }

    // Apply font size after component is mounted
    setTimeout(() => {
      updateEditorContent();
      applyFontSize($fontSize);
    }, 20);

    // Cleanup
    return () => {
      if (plainTextEditor) {
        plainTextEditor.removeEventListener("click", handleSelectionChange);
        plainTextEditor.removeEventListener("keyup", handleSelectionChange);
        plainTextEditor.removeEventListener("select", handleSelectionChange);
        plainTextEditor.removeEventListener("scroll", handleScroll);
      }
    };
  });
</script>

<div
  class="editor-container"
  class:dark-theme={$theme === "dark"}
  style="--tab-color: {displayColor};"
>
  {#if $viewMode === "edit"}
    <!-- Plain text editor - always the source of truth -->
    <textarea
      class="plain-text-editor"
      bind:this={plainTextEditor}
      value={$notes[$activeTab] || ""}
      on:input={handleInput}
      on:keydown={handleKeydown}
      on:scroll={handleScroll}
      spellcheck="true"
      aria-label="Plain text editor"
    ></textarea>
  {:else}
    <!-- Preview mode - read-only view of rendered markdown -->
    <div
      bind:this={previewContainer}
      class="preview-container"
      on:scroll={handleScroll}
      aria-label="Preview"
    >
      {#if !$notes[$activeTab] || !$notes[$activeTab].trim()}
        <div class="placeholder">Start writing...</div>
      {:else}
        {@html formatMarkdown($notes[$activeTab] || "", tabColors[$activeTab])}
      {/if}
    </div>
  {/if}
</div>

<style>
  .editor-container {
    height: 100%;
    overflow: hidden;
    background-color: transparent;
    transition:
      background-color 0.3s,
      color 0.3s;
  }

  .dark-theme {
    color: #e0e0e0;
  }

  .plain-text-editor {
    width: 100%;
    height: 100%;
    outline: none;
    font-family: "SF Mono", SFMono-Regular, Consolas, "Liberation Mono", Menlo,
      monospace;
    color: inherit;
    background-color: transparent;
    border: none;
    resize: none;
    padding: 20px;
    box-sizing: border-box;
    caret-color: var(--tab-color);
  }

  .preview-container {
    width: 100%;
    height: 100%;
    padding: 20px;
    outline: none;
    overflow-y: auto;
    color: inherit;
    white-space: pre-wrap;
    font-family:
      system-ui,
      -apple-system,
      BlinkMacSystemFont,
      sans-serif;
  }

  .placeholder {
    color: #aaa;
    font-style: italic;
    pointer-events: none;
  }

  .dark-theme .placeholder {
    color: #666;
  }

  .dark-theme .preview-container :global(.cursor-marker) {
    box-shadow: 0 0 3px rgba(255, 255, 255, 0.2);
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 0.7;
    }
    50% {
      opacity: 0;
    }
  }
</style>
