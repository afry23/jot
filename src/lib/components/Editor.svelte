<script lang="ts">
  import { onMount, tick } from "svelte";
  import { activeTab } from "$lib/stores/tabs";
  import { notes, updateNote } from "$lib/stores/notes";
  import { theme, fontSize } from "$lib/stores/settings";
  import { viewMode } from "$lib/stores/viewMode";
  import { formatMarkdown } from "$lib/utils/textFormatting";
  import { tabColors, withOpacity } from "$lib/utils/colors";
  import {
    undoHistory,
    redoHistory,
    myundo,
    myredo,
  } from "$lib/stores/history";
  import { saveNote } from "$lib/utils/persistence";
  import {
    cursorPositions,
    scrollPositions,
  } from "$lib/stores/cursorPostionStore";

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

  // Initial cursor position - default to end of text
  let cursorPosition = 0;
  let scrollTop = 0;

  // Save cursor position when it changes
  function saveCursorPosition() {
    if (plainTextEditor && $activeTab !== undefined) {
      const position = plainTextEditor.selectionStart;
      cursorPositions.update((positions) => {
        positions[$activeTab] = position;
        return positions;
      });
    }
  }

  // Save scroll position when it changes
  function saveScrollPosition() {
    if ($viewMode === "edit" && plainTextEditor && $activeTab !== undefined) {
      scrollPositions.update((positions) => {
        positions[$activeTab] = plainTextEditor.scrollTop;
        return positions;
      });
    } else if (
      $viewMode === "preview" &&
      previewContainer &&
      $activeTab !== undefined
    ) {
      scrollPositions.update((positions) => {
        positions[$activeTab] = previewContainer.scrollTop;
        return positions;
      });
    }
  }

  async function insertTimestamp() {
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

    await tick();
    // Move cursor after the inserted timestamp
    plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
      start + formattedTimestamp.length;
    cursorPosition = plainTextEditor.selectionStart;
  }

  async function toggleBold() {
    const start = plainTextEditor.selectionStart;
    const end = plainTextEditor.selectionEnd;
    const currentContent = $notes[$activeTab] || "";

    if (start !== end) {
      // Text selected, wrap it in ** for bold
      const selectedText = currentContent.substring(start, end);
      const isBold =
        selectedText.startsWith("**") && selectedText.endsWith("**");

      const newContent =
        currentContent.substring(0, start) +
        (isBold ? selectedText.slice(2, -2) : `**${selectedText}**`) +
        currentContent.substring(end);

      updateNote($activeTab, newContent);

      await tick();
      // Adjust selection to include the ** markers
      plainTextEditor.selectionStart = start;
      plainTextEditor.selectionEnd =
        start + (isBold ? selectedText.length - 4 : selectedText.length + 4);
      cursorPosition = plainTextEditor.selectionStart;
    }
  }

  async function toggleItalic() {
    const start = plainTextEditor.selectionStart;
    const end = plainTextEditor.selectionEnd;
    const currentContent = $notes[$activeTab] || "";

    if (start !== end) {
      // Text selected, wrap it in * for italic
      const selectedText = currentContent.substring(start, end);
      const isItalic =
        selectedText.startsWith("*") && selectedText.endsWith("*");

      const newContent =
        currentContent.substring(0, start) +
        (isItalic ? selectedText.slice(1, -1) : `*${selectedText}*`) +
        currentContent.substring(end);

      updateNote($activeTab, newContent);

      // Adjust selection to include the * markers
      await tick();
      plainTextEditor.selectionStart = start;
      plainTextEditor.selectionEnd =
        start + (isItalic ? selectedText.length - 2 : selectedText.length + 2);
      cursorPosition = plainTextEditor.selectionStart;
    }
  }

  // Add a list shortcut function
  async function toggleList(ordered = false) {
    const start = plainTextEditor.selectionStart;
    const end = plainTextEditor.selectionEnd;
    const currentContent = $notes[$activeTab] || "";

    // Check if we have a multiline selection
    const selectedText = currentContent.substring(start, end);
    const isMultiLine = selectedText.includes("\n");

    if (isMultiLine) {
      // Handle multiline case
      const firstLineStart = currentContent.lastIndexOf("\n", start - 1) + 1;
      const lastLineEnd =
        end === currentContent.length ? end : currentContent.indexOf("\n", end);
      const lastLineEnd2 =
        lastLineEnd === -1 ? currentContent.length : lastLineEnd;

      // Get all the affected text (including complete first and last lines)
      const affectedText = currentContent.substring(
        firstLineStart,
        lastLineEnd2,
      );
      const lines = affectedText.split("\n");

      // Check if all lines are already lists of the same type
      const allAlreadyListed = lines.every((line) =>
        ordered ? /^\d+\.\s/.test(line) : /^-\s/.test(line),
      );

      let newLines;

      if (allAlreadyListed) {
        // Remove list formatting from all lines
        newLines = lines.map((line) =>
          line.replace(ordered ? /^\d+\.\s/ : /^-\s/, ""),
        );
      } else {
        // Add list formatting to all lines
        newLines = lines.map((line, index) => {
          // If line is already a list item of the type we're converting to, leave it alone
          if (ordered && /^\d+\.\s/.test(line)) return line;
          if (!ordered && /^-\s/.test(line)) return line;

          // If it's the other type of list, convert it
          if (ordered && /^-\s/.test(line))
            return line.replace(/^-\s/, `${index + 1}. `);
          if (!ordered && /^\d+\.\s/.test(line))
            return line.replace(/^\d+\.\s/, "- ");

          // Otherwise add new list formatting
          return ordered ? `${index + 1}. ${line}` : `- ${line}`;
        });
      }

      // Update the content
      const newContent =
        currentContent.substring(0, firstLineStart) +
        newLines.join("\n") +
        currentContent.substring(lastLineEnd2);

      updateNote($activeTab, newContent);

      // Update selection to encompass the modified text
      await tick();
      plainTextEditor.selectionStart = firstLineStart;
      plainTextEditor.selectionEnd =
        firstLineStart + newLines.join("\n").length;
      cursorPosition = plainTextEditor.selectionStart;
    } else {
      // Get the beginning of the current line
      const lineStart = currentContent.lastIndexOf("\n", start - 1) + 1;
      const linePrefix = currentContent.substring(lineStart, start);

      // Check if we're already in a list
      const isAlreadyList = ordered
        ? /^\d+\.\s/.test(linePrefix)
        : /^-\s/.test(linePrefix);

      let newContent;
      let listPrefix = "";

      if (isAlreadyList) {
        // Remove list formatting
        const newLinePrefix = linePrefix.replace(
          ordered ? /^\d+\.\s/ : /^-\s/,
          "",
        );
        newContent =
          currentContent.substring(0, lineStart) +
          newLinePrefix +
          currentContent.substring(start);
      } else {
        // Add list formatting
        listPrefix = ordered ? "1. " : "- ";
        newContent =
          currentContent.substring(0, lineStart) +
          listPrefix +
          currentContent.substring(lineStart);
      }

      updateNote($activeTab, newContent);

      // Adjust cursor position
      await tick();
      plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
        start + (isAlreadyList ? -listPrefix.length : listPrefix.length);
      cursorPosition = plainTextEditor.selectionStart;
    }
  }

  async function insertLink() {
    const start = plainTextEditor.selectionStart;
    const end = plainTextEditor.selectionEnd;
    const currentContent = $notes[$activeTab] || "";

    let linkText = "link text";
    let linkTemplate = "[link text](url)";

    if (start !== end) {
      // Use the selected text as the link text
      linkText = currentContent.substring(start, end);
      linkTemplate = `[${linkText}](url)`;
    }

    const newContent =
      currentContent.substring(0, start) +
      linkTemplate +
      currentContent.substring(end);

    updateNote($activeTab, newContent);

    // Position cursor in the URL part
    await tick();
    const urlStart = start + linkText.length + 3; // After "[linkText]("
    plainTextEditor.selectionStart = urlStart;
    plainTextEditor.selectionEnd = urlStart + 3; // Select "url"
    cursorPosition = plainTextEditor.selectionStart;
  }

  // Key shortcuts for formatting
  async function handleKeydown(event: KeyboardEvent) {
    // Update cursor position on any keydown
    if (plainTextEditor) {
      cursorPosition = plainTextEditor.selectionStart;
    }

    if (event.ctrlKey) {
      switch (event.key) {
        case "t":
          // CTRL+T for date/time shortcut
          event.preventDefault();
          insertTimestamp();
          break;
        case "b":
          event.preventDefault();
          toggleBold();
          break;
        case "i":
          event.preventDefault();
          toggleItalic();
          break;
        case "z":
          // CTRL+Z for undo
          event.preventDefault();
          handleUndo();
          break;
        case "y":
          // CTRL+Y for redo
          event.preventDefault();
          handleRedo();
          break;
        case "l":
          event.preventDefault();
          toggleList(false); // Unordered list
          break;
        case "o":
          event.preventDefault();
          toggleList(true); // Ordered list
          break;
        case "k":
          event.preventDefault();
          insertLink();
          break;
      }
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
  }

  function handleInput() {
    if (plainTextEditor) {
      const newContent = plainTextEditor.value;
      updateNote($activeTab, newContent);
      cursorPosition = plainTextEditor.selectionStart;
      saveCursorPosition();

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
      saveCursorPosition();
    }
  }

  // Track scroll position
  function handleScroll() {
    if ($viewMode === "edit" && plainTextEditor) {
      scrollTop = plainTextEditor.scrollTop;
      saveScrollPosition();
    } else if ($viewMode === "preview" && previewContainer) {
      scrollTop = previewContainer.scrollTop;
      saveScrollPosition();
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

  // Function to update editor content based on current tab
  function updateEditorContent() {
    if ($notes[$activeTab] !== undefined) {
      // Reset cursor position to end of text for new tab

      const storedCursorPosition = $cursorPositions[$activeTab];
      cursorPosition =
        storedCursorPosition !== undefined
          ? storedCursorPosition
          : $notes[$activeTab]
            ? $notes[$activeTab].length
            : 0;

      // Get stored scroll position for this tab or default to top
      const storedScrollPosition = $scrollPositions[$activeTab];
      scrollTop = storedScrollPosition !== undefined ? storedScrollPosition : 0;

      // Update the UI
      setTimeout(() => {
        if ($viewMode === "edit" && plainTextEditor) {
          plainTextEditor.value = $notes[$activeTab] || "";

          // Focus and place cursor at the end if tab changed
          plainTextEditor.focus();
          plainTextEditor.selectionStart = plainTextEditor.selectionEnd =
            cursorPosition;
        }
      }, 10);
    }
  }

  function handleUndo() {
    const previousContent = myundo($activeTab);
    if (previousContent !== null) {
      notes.update((state) => {
        state[$activeTab] = previousContent;
        return state;
      });
    }
  }

  function handleRedo() {
    const nextContent = myredo($activeTab);
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
      style="font-size: {fontSizes[$fontSize]?.editor}; line-height: {fontSizes[
        $fontSize
      ]?.lineHeight};"
    ></textarea>
  {:else}
    <!-- Preview mode - read-only view of rendered markdown -->
    <div
      bind:this={previewContainer}
      class="preview-container"
      on:scroll={handleScroll}
      aria-label="Preview"
      style="font-size: {fontSizes[$fontSize]?.editor}; line-height: {fontSizes[
        $fontSize
      ]?.lineHeight};"
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
