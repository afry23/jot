<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher, tick } from "svelte";
  import { EditorView } from "prosemirror-view";
  import { EditorState } from "prosemirror-state";
  import {
    schema,
    defaultMarkdownParser,
    defaultMarkdownSerializer,
  } from "prosemirror-markdown";
  import { buildInputRules, buildKeymap } from "prosemirror-example-setup";
  import { liftListItem, sinkListItem } from "prosemirror-schema-list";
  import { keymap } from "prosemirror-keymap";
  import { baseKeymap } from "prosemirror-commands";
  import { dropCursor } from "prosemirror-dropcursor";
  import { gapCursor } from "prosemirror-gapcursor";
  import { TextSelection } from "prosemirror-state";
  import { theme, fontSize } from "$lib/stores/settings";
  import { activeTab, setActiveTab } from "$lib/stores/tabs";
  import { notes, updateNote } from "$lib/stores/notes";
  import { tabColors, withOpacity } from "$lib/utils/colors";
  import { saveNote } from "$lib/utils/persistence";
  import {
    undoHistory,
    redoHistory,
    myundo,
    myredo,
  } from "$lib/stores/history";
  import { getHeadingColorWithOpacity, uiColors } from "$lib/utils/uiColors";
  import { history } from "prosemirror-history";
  import { undo, redo } from "prosemirror-history";
  import { getCodeBackground, getCodeBorder } from "$lib/utils/textFormatting";
  import {
    cursorPositions,
    scrollPositions,
  } from "$lib/stores/cursorPostionStore";
  import { open } from "@tauri-apps/plugin-shell";
  import { logger } from "$lib/utils/logger";
  import LinkModal from "./LinkModal.svelte";

  let showLinkModal = false;
  let linkModalInitialHref = "";
  let linkModalInitialText = "";
  let linkModalSelection: { from: number; to: number } | null = null;
  let proseMirrorViewRef: ProseMirrorView | null = null;

  // Props
  export let initialContent: string =
    "# Hello World\n\nThis is some **bold** text and *italic* text.\n\n- List item 1\n- List item 2\n- List item 3\n\n> This is a blockquote";

  // Svelte event dispatcher
  const dispatch = createEventDispatcher<Record<string, any>>();

  // Component state
  let editorContainer: HTMLElement;
  let viewMode: "markdown" | "prosemirror" = "prosemirror";
  let currentView: EditorInterface | null = null;
  let currentContent = initialContent;

  // Get the appropriate color for the current tab
  $: currentTabColor = tabColors[$activeTab];
  $: mutedColor = withOpacity(currentTabColor, 0.5);
  $: isCurrentTabEmpty = currentContent?.trim() === "";
  $: displayColor = isCurrentTabEmpty ? mutedColor : currentTabColor;
  $: shadowColor = withOpacity(currentTabColor, 0.25);

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

  const heading1Color = getHeadingColorWithOpacity(tabColors[$activeTab], 1);
  const heading2Color = getHeadingColorWithOpacity(tabColors[$activeTab], 2);
  const heading3Color = getHeadingColorWithOpacity(tabColors[$activeTab], 3);
  const heading4Color = getHeadingColorWithOpacity(tabColors[$activeTab], 4);
  const heading5Color = getHeadingColorWithOpacity(tabColors[$activeTab], 5);
  const heading6Color = getHeadingColorWithOpacity(tabColors[$activeTab], 6);
  const codeBackgroundColor = getCodeBackground(tabColors[$activeTab]);
  const codeBorderColor = getCodeBorder(tabColors[$activeTab]);

  // Add this near other component state variables
  let previousTab = -1;
  let lastSavedContent = "";
  let editorReady = false;
  let updateVersion = 0;

  // Track tab changes and update editor content accordingly
  $: if ($activeTab !== previousTab && editorReady) {
    // Save current content before switching tabs
    if (currentView && previousTab >= 0) {
      const currentContent = currentView.content;
      if (currentContent !== $notes[previousTab]) {
        updateNote(previousTab, currentContent);
      }
    }

    previousTab = $activeTab;
    updateEditorContent();
  }

  async function openUrl(url: string) {
    try {
      await open(url);
    } catch (error) {
      console.error("Failed to open URL:", error);
    }
  }

  export function reloadContentFromStore() {
    updateEditorContent();
  }

  function handleLinkModalSubmit(
    event: CustomEvent<{ href: string; text: string }>
  ) {
    showLinkModal = false;
    if (!linkModalSelection || !proseMirrorViewRef) return;
    const { href, text } = event.detail;
    const { from, to } = linkModalSelection;
    const view = proseMirrorViewRef.view;
    const linkType = schema.marks.link;

    let tr = view.state.tr;
    // If selection is empty, insert the text
    if (from === to) {
      tr = tr.insertText(text, from, to);
      tr = tr.addMark(from, from + text.length, linkType.create({ href }));
      tr = tr.setSelection(
        TextSelection.create(tr.doc, from, from + text.length)
      );
    } else {
      // Replace selected text with new text and link
      tr = tr.insertText(text, from, to);
      tr = tr.addMark(from, from + text.length, linkType.create({ href }));
      tr = tr.setSelection(
        TextSelection.create(tr.doc, from, from + text.length)
      );
    }
    view.dispatch(tr);
    proseMirrorViewRef = null;
    linkModalSelection = null;
  }
  function handleLinkModalCancel() {
    showLinkModal = false;
    proseMirrorViewRef = null;
    linkModalSelection = null;
  }

  // Update content when active tab changes
  async function updateEditorContent() {
    updateVersion += 1;
    const thisVersion = updateVersion;
    const tabIndex = $activeTab;

    if ($notes[tabIndex] !== undefined) {
      currentContent = $notes[tabIndex] || "";
      lastSavedContent = currentContent;

      // Get stored cursor position for this tab or default to end of text
      const storedCursorPosition = $cursorPositions[$activeTab];
      const newPosition =
        storedCursorPosition !== undefined
          ? storedCursorPosition
          : currentContent.length;

      // Get stored scroll position
      const storedScrollPosition = $scrollPositions[$activeTab];
      const newScrollTop =
        storedScrollPosition !== undefined ? storedScrollPosition : 0;

      // Destroy the current view before creating a new one
      if (currentView) {
        currentView.destroy();
        currentView = null;
      }

      await tick(); // Ensure DOM is updated before creating new view

      if (thisVersion !== updateVersion) return;
      if (tabIndex !== $activeTab) return;

      // Create a new view with the latest content
      const ViewClass =
        viewMode === "markdown" ? MarkdownView : ProseMirrorView;
      currentView = new ViewClass(editorContainer, currentContent);

      // Apply cursor and scroll positions after content is set
      setTimeout(() => {
        if (thisVersion !== updateVersion) return;
        if (tabIndex !== $activeTab) return;
        if (currentView) {
          // Your editor might need specific handling for cursor/scroll
          // This is a placeholder - implement based on your editor's API
          applyCursorAndScrollPosition(newPosition, newScrollTop);
          currentView.focus(); // Ensure the view is focused after setting positions
        }
      }, 50);
    }
  }

  // Implement cursor and scroll position application
  function applyCursorAndScrollPosition(position: number, scrollTop: number) {
    // Implementation will depend on your editor's API
    // For example, with ProseMirror you might need to:
    // - Set selection to the position
    // - Set scroll position of the container
    if (
      currentView &&
      viewMode === "prosemirror" &&
      currentView instanceof ProseMirrorView
    ) {
      currentView.setCursorPosition(position);
      currentView.setScrollPosition(scrollTop);
    } else if (
      currentView &&
      viewMode === "markdown" &&
      currentView instanceof MarkdownView
    ) {
      currentView.setCursorPosition(position);
      currentView.setScrollPosition(scrollTop);
    }
  }

  // Handle content changes and save to store
  function handleContentChange() {
    if (!currentView || !editorReady) return;

    const newContent = currentView.content;

    // Only update if content actually changed and we're still on the same tab
    if (newContent !== $notes[$activeTab] && $activeTab === previousTab) {
      // Update the notes store
      updateNote($activeTab, newContent);

      // Save note to persistent storage if needed
      if (newContent !== lastSavedContent) {
        saveNote($activeTab, newContent);
        lastSavedContent = newContent;
      }
    }
  }

  function handleUndo() {
    console.log("Handling undo for tab:", $activeTab);
    const previousContent = myundo($activeTab);
    if (previousContent !== null) {
      notes.update((state) => {
        state[$activeTab] = previousContent;
        return state;
      });
      if (viewMode === "markdown" && currentView instanceof MarkdownView) {
        currentView.setContent(previousContent);
      }
    }
  }

  function handleRedo() {
    console.log("Handling redo for tab:", $activeTab);
    const nextContent = myredo($activeTab);
    if (nextContent !== null) {
      notes.update((state) => {
        state[$activeTab] = nextContent;
        return state;
      });
      if (viewMode === "markdown" && currentView instanceof MarkdownView) {
        currentView.setContent(nextContent);
      }
    }
  }

  // Abstract interface for both views
  interface EditorInterface {
    get content(): string;
    focus(): void;
    destroy(): void;
    setCursorPosition(position: number): void;
    setScrollPosition(scrollTop: number): void;
    saveCursorPosition(): void;
    saveScrollPosition(): void;
  }

  // Textarea-based markdown view
  class MarkdownView implements EditorInterface {
    private textarea: HTMLTextAreaElement;
    private container: HTMLElement;

    constructor(target: HTMLElement, content: string) {
      this.container = target;
      this.textarea = document.createElement("textarea");
      this.textarea.value = content;
      this.textarea.className = "markdown-textarea";

      // Clear target and add textarea
      this.container.innerHTML = "";
      this.container.appendChild(this.textarea);

      // Add event listeners for cursor and scroll position tracking
      this.textarea.addEventListener("click", () => this.saveCursorPosition());
      this.textarea.addEventListener("keyup", () => this.saveCursorPosition());
      this.textarea.addEventListener("select", () => this.saveCursorPosition());
      this.textarea.addEventListener("scroll", () => this.saveScrollPosition());

      // Add input event listener for content changes
      this.textarea.addEventListener("input", () => handleContentChange());

      this.textarea.addEventListener("keydown", this.handleKeyDown.bind(this));
    }

    get content(): string {
      return this.textarea.value;
    }

    setContent(content: string): void {
      this.textarea.value = content;
    }

    focus(): void {
      this.textarea.focus();
    }

    destroy(): void {
      if (this.container) {
        this.container.innerHTML = "";
      }
    }

    setCursorPosition(position: number): void {
      if (this.textarea) {
        this.textarea.selectionStart = this.textarea.selectionEnd = position;
      }
    }

    setScrollPosition(scrollTop: number): void {
      if (this.textarea) {
        this.textarea.scrollTop = scrollTop;
      }
    }

    saveCursorPosition(): void {
      if (this.textarea && $activeTab !== undefined) {
        const position = this.textarea.selectionStart;
        cursorPositions.update((positions) => {
          positions[$activeTab] = position;
          return positions;
        });
      }
    }

    saveScrollPosition(): void {
      if (this.textarea && $activeTab !== undefined) {
        scrollPositions.update((positions) => {
          positions[$activeTab] = this.textarea.scrollTop;
          return positions;
        });
      }
    }

    handleKeyDown(event: {
      metaKey: any;
      ctrlKey: any;
      key: string;
      preventDefault: () => void;
      shiftKey: any;
    }) {
      // Check for Mod key (Cmd on Mac, Ctrl on Windows)
      const isMod = event.metaKey || event.ctrlKey;

      if (isMod && event.key === "b") {
        // Bold
        event.preventDefault();
        this.wrapSelection("**", "**");
        return true;
      } else if (isMod && event.key === "i") {
        // Italic
        event.preventDefault();
        this.wrapSelection("*", "*");
        return true;
      } else if (isMod && event.key === "`") {
        // Code
        event.preventDefault();
        this.wrapSelection("`", "`");
        return true;
      } else if (isMod && event.key === "t") {
        // Timestamp
        event.preventDefault();
        this.insertTimestamp();
        return true;
      } else if (isMod && event.key === "z") {
        // Undo
        event.preventDefault();
        handleUndo();
        return true;
      } else if (isMod && event.key === "y") {
        // Redo
        event.preventDefault();
        handleRedo();
        return true;
      } else if (isMod && event.key === "k") {
        // Link
        event.preventDefault();
        // Add a template for link insertion in the format [text](url)
        this.insertLinkTemplate();
        return true;
      } else if (event.key === "Tab") {
        console.log("Tab key pressed in MarkdownView");
        event.preventDefault();

        if (event.shiftKey) {
          // Try to lift (unindent) list item
          return this.liftListItem();
        } else {
          // Try to sink (indent) list item
          return this.sinkListItem();
        }
      }
      return false;
    }

    sinkListItem(): boolean {
      const { selectionStart, value } = this.textarea;
      const lines = value.split("\n");

      // Find the line containing the cursor
      let currentPos = 0;
      let currentLineIndex = -1;

      for (let i = 0; i < lines.length; i++) {
        const lineEnd = currentPos + lines[i].length;
        if (selectionStart >= currentPos && selectionStart <= lineEnd) {
          currentLineIndex = i;
          break;
        }
        currentPos = lineEnd + 1;
      }

      if (currentLineIndex === -1) return false;

      const currentLine = lines[currentLineIndex];

      // Check if current line is a list item
      const listItemRegex = /^(\s*)(-|\d+\.)\s(.*)$/;
      const match = currentLine.match(listItemRegex);

      if (match) {
        console.log("Found list item, sinking (indenting)");

        // Add 4 spaces before the existing content
        lines[currentLineIndex] = "    " + currentLine;

        const newValue = lines.join("\n");
        this.textarea.value = newValue;

        // Adjust cursor position (add 4 spaces)
        const newCursorPos = selectionStart + 4;
        this.textarea.selectionStart = this.textarea.selectionEnd =
          newCursorPos;

        handleContentChange();
        this.saveCursorPosition();
        return true;
      } else {
        // Not a list item, insert 4 spaces at cursor
        console.log("Not a list item, inserting spaces at cursor");
        const before = value.substring(0, selectionStart);
        const after = value.substring(selectionStart);
        const newValue = before + "    " + after;

        this.textarea.value = newValue;
        this.textarea.selectionStart = this.textarea.selectionEnd =
          selectionStart + 4;

        handleContentChange();
        this.saveCursorPosition();
        return true;
      }
    }

    liftListItem(): boolean {
      const { selectionStart, value } = this.textarea;
      const lines = value.split("\n");

      // Find the line containing the cursor
      let currentPos = 0;
      let currentLineIndex = -1;

      for (let i = 0; i < lines.length; i++) {
        const lineEnd = currentPos + lines[i].length;
        if (selectionStart >= currentPos && selectionStart <= lineEnd) {
          currentLineIndex = i;
          break;
        }
        currentPos = lineEnd + 1;
      }

      if (currentLineIndex === -1) return false;

      const currentLine = lines[currentLineIndex];

      // Check if current line is a list item with indentation
      const listItemRegex = /^(\s*)(-|\d+\.)\s(.*)$/;
      const match = currentLine.match(listItemRegex);

      if (match && match[1].length >= 4) {
        console.log("Found indented list item, lifting (unindenting)");

        // Remove 4 spaces from the beginning
        lines[currentLineIndex] = currentLine.substring(4);

        const newValue = lines.join("\n");
        this.textarea.value = newValue;

        // Adjust cursor position (subtract 4 spaces)
        const newCursorPos = Math.max(selectionStart - 4, currentPos);
        this.textarea.selectionStart = this.textarea.selectionEnd =
          newCursorPos;

        handleContentChange();
        this.saveCursorPosition();
        return true;
      } else if (!match) {
        // Not a list item, try to remove 4 spaces before cursor
        console.log("Not a list item, trying to remove spaces before cursor");
        const before = value.substring(0, selectionStart);
        if (before.endsWith("    ")) {
          const newBefore = before.substring(0, before.length - 4);
          const after = value.substring(selectionStart);
          const newValue = newBefore + after;

          this.textarea.value = newValue;
          this.textarea.selectionStart = this.textarea.selectionEnd =
            selectionStart - 4;

          handleContentChange();
          this.saveCursorPosition();
          return true;
        }
        return false;
      } else {
        // List item with no indentation to remove
        console.log("List item has no indentation to remove");
        return false;
      }
    }

    insertLinkTemplate() {
      const { selectionStart, selectionEnd, value } = this.textarea;
      const selectedText =
        value.substring(selectionStart, selectionEnd) || "link text";
      const template = `[${selectedText}](url)`;

      // Replace the selected text with the link template
      this.textarea.setRangeText(template, selectionStart, selectionEnd, "end");

      // Place cursor inside the url part for convenience
      const urlStart = selectionStart + template.indexOf("](url)") + 2;
      const urlEnd = urlStart + 3; // length of "url"
      this.textarea.selectionStart = urlStart;
      this.textarea.selectionEnd = urlEnd;

      // Trigger content change and update cursor position
      handleContentChange();
      this.saveCursorPosition();
    }

    wrapSelection(before: string | any[], after: string | any[]) {
      const { selectionStart, selectionEnd, value } = this.textarea;
      let selectedText = value.substring(selectionStart, selectionEnd);

      // Check if selection is already wrapped
      const hasBefore =
        value.substring(selectionStart - before.length, selectionStart) ===
        before;
      const hasAfter =
        value.substring(selectionEnd, selectionEnd + after.length) === after;

      if (hasBefore && hasAfter) {
        // Remove formatting
        const newStart = selectionStart - before.length;
        const newEnd = selectionEnd - before.length;
        const newValue =
          value.substring(0, newStart) +
          selectedText +
          value.substring(selectionEnd + after.length);

        this.textarea.value = newValue;
        this.textarea.selectionStart = newStart;
        this.textarea.selectionEnd = newEnd;
      } else {
        // Add formatting
        const newText = before + selectedText + after;
        this.textarea.setRangeText(
          newText,
          selectionStart,
          selectionEnd,
          "end"
        );

        if (selectionStart === selectionEnd) {
          // No selection: place cursor between markers
          this.textarea.selectionStart = this.textarea.selectionEnd =
            selectionStart + before.length;
        } else {
          // Selection: select the wrapped text
          this.textarea.selectionStart = selectionStart + before.length;
          this.textarea.selectionEnd = selectionEnd + before.length;
        }
      }

      // Trigger content change
      handleContentChange();

      // Update cursor position in store
      this.saveCursorPosition();
    }

    insertTimestamp() {
      const now = new Date();
      const formattedDate = now.toLocaleString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });

      const start = this.textarea.selectionStart;
      const end = this.textarea.selectionEnd;

      // Insert timestamp at cursor position
      const formattedTimestamp = `**${formattedDate}** `;
      const currentContent = $notes[$activeTab] || "";
      const newContent =
        currentContent.substring(0, start) +
        formattedTimestamp +
        currentContent.substring(end);

      this.textarea.value = newContent;

      // Move cursor after the inserted timestamp
      this.textarea.selectionStart = this.textarea.selectionEnd =
        start + formattedTimestamp.length;
    }
  }

  // ProseMirror-based view
  class ProseMirrorView implements EditorInterface {
    private view: EditorView;
    private container: HTMLElement;

    constructor(target: HTMLElement, content: string) {
      console.log("Creating ProseMirrorView with content");
      this.container = target;

      // Clear target completely
      this.container.innerHTML = "";

      // Choose plugins based on showMenuBar prop
      const plugins = [
        buildInputRules(schema),
        history(),
        keymap({
          "Mod-b": (state, dispatch) => {
            const { from, to } = state.selection;
            const strongType = schema.marks.strong;
            if (dispatch) {
              const hasMark = state.doc.rangeHasMark(from, to, strongType);
              if (hasMark) {
                dispatch(state.tr.removeMark(from, to, strongType));
              } else {
                dispatch(state.tr.addMark(from, to, strongType.create()));
              }
            }
            return true;
          },
          "Mod-i": (state, dispatch) => {
            const { from, to } = state.selection;
            const emType = schema.marks.em;
            if (dispatch) {
              const hasMark = state.doc.rangeHasMark(from, to, emType);
              if (hasMark) {
                dispatch(state.tr.removeMark(from, to, emType));
              } else {
                dispatch(state.tr.addMark(from, to, emType.create()));
              }
            }
            return true;
          },
          "Mod-`": (state, dispatch) => {
            const { from, to } = state.selection;
            const codeType = schema.marks.code;
            if (dispatch) {
              const hasMark = state.doc.rangeHasMark(from, to, codeType);
              if (hasMark) {
                dispatch(state.tr.removeMark(from, to, codeType));
              } else {
                dispatch(state.tr.addMark(from, to, codeType.create()));
              }
            }
            return true;
          },
          "Mod-k": (state, dispatch, view) => {
            if (!view) return false;
            const { from, to, empty } = state.selection;
            let initialText = "";
            if (!empty) {
              initialText = state.doc.textBetween(from, to, " ");
            }
            // Save selection and show modal
            linkModalSelection = { from, to };
            linkModalInitialHref = "";
            linkModalInitialText = initialText || "link text";
            if (currentView instanceof ProseMirrorView) {
              proseMirrorViewRef = currentView;
            } else {
              proseMirrorViewRef = null;
            }
            showLinkModal = true;
            return true;
          },
          "Mod-t": (state, dispatch) => {
            if (!dispatch) return false;
            const now = new Date();
            const formattedDate = now.toLocaleString(undefined, {
              year: "numeric",
              month: "short",
              day: "numeric",
              hour: "2-digit",
              minute: "2-digit",
              second: "2-digit",
            });
            // Insert as bold markdown (like MarkdownView)
            const timestamp = `${formattedDate} `;
            const { from, to } = state.selection;
            // Insert the text and apply bold mark
            const tr = state.tr
              .insertText(timestamp, from, to)
              .addMark(
                from,
                from + timestamp.length,
                schema.marks.strong.create()
              );
            dispatch(tr);
            return true;
          },
          Tab: (state, dispatch) => {
            console.log("Tab key pressed in ProseMirrorView");
            // Try to sink (indent) list item first
            if (sinkListItem(schema.nodes.list_item)(state, dispatch)) {
              return true;
            }
            // If not in a list item, fall back to inserting spaces
            if (dispatch) {
              const tr = state.tr.insertText("    ", state.selection.from);
              dispatch(tr);
            }
            return true;
          },
          "Shift-Tab": (state, dispatch) => {
            console.log("Shift-Tab key pressed in ProseMirrorView");
            // Try to lift (unindent) list item
            return liftListItem(schema.nodes.list_item)(state, dispatch);
          },
          "Mod-z": (state, dispatch) => {
            return undo(state, dispatch);
          },
          "Mod-y": (state, dispatch) => {
            return redo(state, dispatch);
          },
          "Mod-Shift-z": (state, dispatch) => {
            return require("prosemirror-history").redo(state, dispatch);
          },
        }),
        keymap(buildKeymap(schema)),
        keymap(baseKeymap),
        dropCursor(),
        gapCursor(),
      ];

      this.view = new EditorView(this.container, {
        state: EditorState.create({
          doc: defaultMarkdownParser.parse(content),
          plugins: plugins,
        }),
        handleClick: (view, pos, event) => {
          const { node } = view.state.doc.resolve(pos);

          // If it's a text node, check if it has a link mark
          if (node && node.isText) {
            const marks = node.marks;
            const linkMark = marks.find(
              (mark) => mark.type === schema.marks.link
            );

            if (linkMark) {
              event.preventDefault();
              const href = linkMark.attrs.href;

              // Check if we need to use Tauri's API for opening links
              // or just use the browser's window.open
              try {
                // For Tauri apps
                // Import this at the top: import { open } from '@tauri-apps/api/shell';
                open(href);

                // For browser/non-Tauri environments:
                window.open(href, "_blank", "noopener,noreferrer");
              } catch (e) {
                console.error("Error opening link:", e);
              }

              return true;
            }
          }
          return false;
        },
      });

      this.container.addEventListener(
        "click",
        this.handleContainerClick.bind(this)
      );

      // Add DOM event listener for content changes
      this.view.dom.addEventListener("blur", () => handleContentChange());

      // Add transaction handler to detect content changes
      this.view.setProps({
        dispatchTransaction: (transaction) => {
          // Apply the transaction to the current state
          const newState = this.view.state.apply(transaction);

          // Update the editor view
          this.view.updateState(newState);

          // If the document changed, handle content update
          if (transaction.docChanged) {
            handleContentChange();
          }

          // Save cursor position when it changes
          if (transaction.selectionSet) {
            this.saveCursorPosition();
          }
        },
      });

      // Add scroll listener to the container
      this.container.addEventListener("scroll", () =>
        this.saveScrollPosition()
      );
    }

    private handleContainerClick(e: MouseEvent): void {
      // Walk up the DOM tree to find if a link was clicked
      let target = e.target as HTMLElement;
      while (target && target !== this.container) {
        if (target.tagName === "A") {
          e.preventDefault();
          const href = target.getAttribute("href");
          if (href) {
            try {
              // For Tauri apps
              // Import this at the top: import { open } from '@tauri-apps/api/shell';
              openUrl(href);

              // For browser/non-Tauri environments:
              window.open(href, "_blank", "noopener,noreferrer");
            } catch (e) {
              console.error("Error opening link:", e);
            }
          }
          return;
        }
        target = target.parentElement;
      }
    }

    get content(): string {
      return defaultMarkdownSerializer.serialize(this.view.state.doc);
    }

    focus(): void {
      this.view.focus();
    }

    destroy(): void {
      if (this.view) {
        this.view.destroy();
      }
      if (this.container) {
        this.container.innerHTML = "";
      }
    }

    setCursorPosition(position: number): void {
      // Convert flat position to ProseMirror document position
      // This is an approximation - ProseMirror positions work differently
      try {
        const resolvedPos = Math.min(
          position,
          this.view.state.doc.content.size
        );
        const selection = TextSelection.create(
          this.view.state.doc,
          resolvedPos
        );
        const transaction = this.view.state.tr.setSelection(selection);
        this.view.dispatch(transaction);
      } catch (e) {
        console.error("Error setting cursor position:", e);
      }
    }

    setScrollPosition(scrollTop: number): void {
      if (this.container) {
        this.container.scrollTop = scrollTop;
      }
    }

    saveCursorPosition(): void {
      if (this.view && $activeTab !== undefined) {
        // Get the current selection's head position (cursor)
        const position = this.view.state.selection.head;
        cursorPositions.update((positions) => {
          positions[$activeTab] = position;
          return positions;
        });
      }
    }

    saveScrollPosition(): void {
      if (this.container && $activeTab !== undefined) {
        scrollPositions.update((positions) => {
          positions[$activeTab] = this.container.scrollTop;
          return positions;
        });
      }
    }
  }

  // Switch between view modes
  function switchView(newMode: "markdown" | "prosemirror") {
    if (viewMode === newMode && currentView) return;

    // Save current content
    if (currentView) {
      currentContent = currentView.content;
      currentView.destroy();
      currentView = null;
    }

    // Wait a tick for cleanup, then create new view
    setTimeout(() => {
      const ViewClass = newMode === "markdown" ? MarkdownView : ProseMirrorView;
      currentView = new ViewClass(editorContainer, currentContent);
      viewMode = newMode;

      // Focus after another tick
      setTimeout(() => {
        if (currentView) {
          currentView.focus();
        }
      }, 10);
    }, 10);
  }

  // Initialize the editor when component mounts
  onMount(() => {
    // Set up initial view
    switchView("prosemirror");

    // Set editor as ready
    editorReady = true;

    // Listen for safe tab switching events
    const handleSaveContentEvent = async (event: Event) => {
      const customEvent = event as CustomEvent<{ newTabIndex: number }>;
      const { newTabIndex } = customEvent.detail;

      // Save current content first
      if (currentView) {
        const currentContent = currentView.content;
        const currentStoreContent = $notes[$activeTab] || "";

        if (currentContent !== currentStoreContent) {
          updateNote($activeTab, currentContent);
          await saveNote($activeTab, currentContent);
        }
      }

      // Now it's safe to switch tabs
      await setActiveTab(newTabIndex);
    };

    window.addEventListener("save-current-content", handleSaveContentEvent);

    // Return cleanup function
    return () => {
      window.removeEventListener(
        "save-current-content",
        handleSaveContentEvent
      );
      if (currentView) {
        currentView.destroy();
        currentView = null;
      }
    };
  });

  // Cleanup when component is destroyed
  onDestroy(() => {
    if (currentView) {
      currentView.destroy();
      currentView = null;
    }
  });

  // Export function to get current content
  export function getContent(): string {
    return currentView ? currentView.content : currentContent;
  }

  // Handle global keydown for view switching
  function handleGlobalKeyDown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "e") {
      event.preventDefault();
      const newMode = viewMode === "prosemirror" ? "markdown" : "prosemirror";
      switchView(newMode);
    }
  }
</script>

<svelte:window on:keydown={handleGlobalKeyDown} />
{#if showLinkModal}
  <LinkModal
    initialHref={linkModalInitialHref}
    initialText={linkModalInitialText}
    on:submit={handleLinkModalSubmit}
    on:cancel={handleLinkModalCancel}
  />
{/if}
<div class="editor-wrapper">
  <div
    class="editor-container"
    bind:this={editorContainer}
    class:dark-theme={$theme === "dark"}
    style="--tab-color: {displayColor}; 
               --muted-color: {mutedColor};
               --h1-color: {heading1Color};
               --h2-color: {heading2Color};
               --h3-color: {heading3Color};
               --h4-color: {heading4Color};
               --h5-color: {heading5Color};
               --h6-color: {heading6Color};
               --borderLight: {$uiColors.borderLight};
               --link-color: {currentTabColor};
               --borderMedium: {$uiColors.borderMedium};
               --code-background: {codeBackgroundColor};
               --code-border: {codeBorderColor};
            font-size: {fontSizes[$fontSize]?.editor}; line-height: {fontSizes[
      $fontSize
    ]?.lineHeight};"
  ></div>
</div>

<style>
  .editor-wrapper {
    width: 100%;
    margin: 0 auto;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      sans-serif;
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: transparent;
  }

  .editor-container {
    position: relative;
    flex: 1;
    display: flex;
    overflow: hidden;
    background-color: transparent;
    transform:
      background-color 0.3s,
      color 0.3s;
    outline: none;
    border: none;
    resize: none;
    padding: 2px;
    box-sizing: border-box;
    caret-color: var(--tab-color);
  }

  .dark-theme {
    color: #e0e0e0;
  }
  /* Markdown Textarea Styling */
  :global(.markdown-textarea) {
    width: 100%;
    height: 100%;
    padding: 20px;
    border: none;
    font-family: "SF Mono", SFMono-Regular, Consolas, "Liberation Mono", Menlo,
      monospace;
    /*background: #f8fff9;*/
    background-color: transparent;
    color: inherit;
    resize: none;
    outline: none;
    box-sizing: border-box;
    caret-color: var(--tab-color);
  }

  :global(.markdown-textarea::placeholder) {
    color: #6c757d;
    font-style: italic;
  }

  /* ProseMirror Editor Styling */
  :global(.ProseMirror) {
    padding: 20px;
    height: 100%;
    width: 100%;
    overflow-y: auto;
    outline: none;
    background-color: transparent;
    color: inherit;
    border: none;
    resize: none;
    box-sizing: border-box;
  }

  :global(.ProseMirror h1) {
    font-size: 1.8em;
    font-weight: bold;
    margin: 20px 0 12px 0;
    color: var(--h1-color);
    border-bottom: 2px solid var(--tab-color, #e9ecef);
    padding-bottom: 8px;
  }

  :global(.ProseMirror h2) {
    font-size: 1.5em;
    font-weight: bold;
    margin: 18px 0 10px 0;
    color: var(--h2-color);
  }

  :global(.ProseMirror h3) {
    font-size: 1.3em;
    font-weight: bold;
    color: var(--h3-color);
    margin: 16px 0 8px 0;
  }

  :global(.ProseMirror h4) {
    font-size: 1.1em;
    font-weight: bold;
    color: var(--h4-color);
    margin: 16px 0 8px 0;
  }

  :global(.ProseMirror h5) {
    font-size: 1em;
    font-weight: bold;
    color: var(--h5-color);
    margin: 16px 0 8px 0;
  }

  :global(.ProseMirror h6) {
    font-size: 0.9em;
    font-weight: bold;
    color: var(--h6-color);
    margin: 16px 0 8px 0;
  }

  :global(.ProseMirror p) {
    margin: 10px 0;
    color: inherit;
  }

  :global(.ProseMirror ul, .ProseMirror ol) {
    margin: 10px 0;
    padding-left: 28px;
  }

  :global(.ProseMirror li) {
    margin: 4px 0;
  }

  :global(.ProseMirror ul) {
    list-style-type: disc;
    margin: 10px 0;
    padding-left: 28px;
  }

  :global(.ProseMirror ol) {
    list-style-type: decimal;
    margin: 10px 0;
    padding-left: 28px;
  }

  :global(.ProseMirror ol li) {
    padding-left: 4px; /* Add a bit of padding for the list marker */
    display: list-item;
  }

  :global(.ProseMirror blockquote) {
    border-left: 4px solid var(--tab-color, #007acc);
    padding-left: 16px;
    margin: 16px 0;
    font-style: italic;
    color: inherit;
    background: #f8f9fa;
    padding: 12px 16px;
    border-radius: 0 4px 4px 0;
  }

  :global(.ProseMirror code) {
    background-color: var(--code-background);
    padding: 0.1em 0.2em;
    border-radius: 2px;
    font-family: "Monaco", "Menlo", "Ubuntu Mono", monospace;
    font-size: 0.9em;
    color: var(--tab-color);
    border-left: 2px solid var(--code-border);
  }

  :global(.ProseMirror pre) {
    background-color: var(--code-background);
    padding: 16px;
    border-radius: 2px;
    overflow-x: auto;
    border-left: 2px solid var(--code-border);
    margin: 16px 0;
  }

  :global(.ProseMirror pre code) {
    background-color: var(--code-background);
    background: none;
    padding: 0;
    border: none;
    color: var(--tab-color);
  }

  :global(.ProseMirror a) {
    color: var(--link-color);
    text-decoration: none;
    border-bottom: 5px dotted var(--borderLight);
  }

  :global(.ProseMirror a:hover) {
    opacity: 0.8;
    border-bottom: 5px dotted var(--borderMedium);
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
