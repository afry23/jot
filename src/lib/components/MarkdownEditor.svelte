<script lang="ts">
    import { onMount, onDestroy, createEventDispatcher } from "svelte";
    import { EditorView } from "prosemirror-view";
    import { EditorState } from "prosemirror-state";
    import {
        schema,
        defaultMarkdownParser,
        defaultMarkdownSerializer,
    } from "prosemirror-markdown";
    import { keymap } from "prosemirror-keymap";
    import { baseKeymap } from "prosemirror-commands";
    import { history } from "prosemirror-history";
    import { dropCursor } from "prosemirror-dropcursor";
    import { gapCursor } from "prosemirror-gapcursor";
    import { theme, fontSize } from "$lib/stores/settings";
    import { activeTab } from "$lib/stores/tabs";
    import { notes, updateNote } from "$lib/stores/notes";
    import { tabColors, withOpacity } from "$lib/utils/colors";
    import { saveNote } from "$lib/utils/persistence";
    import { getHeadingColorWithOpacity } from "$lib/utils/uiColors";

    // Props
    export let initialContent: string =
        "# Hello World\n\nThis is some **bold** text and *italic* text.\n\n- List item 1\n- List item 2\n- List item 3\n\n> This is a blockquote";

    // Svelte event dispatcher
    const dispatch = createEventDispatcher();

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

    const heading1Color = getHeadingColorWithOpacity(tabColors[0], 1);
    const heading2Color = getHeadingColorWithOpacity(tabColors[1], 2);
    const heading3Color = getHeadingColorWithOpacity(tabColors[2], 3);
    const heading4Color = getHeadingColorWithOpacity(tabColors[3], 4);
    const heading5Color = getHeadingColorWithOpacity(tabColors[4], 5);
    const heading6Color = getHeadingColorWithOpacity(tabColors[5], 6);

    // Abstract interface for both views
    interface EditorInterface {
        get content(): string;
        focus(): void;
        destroy(): void;
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
        }

        get content(): string {
            return this.textarea.value;
        }

        focus(): void {
            this.textarea.focus();
        }

        destroy(): void {
            if (this.container) {
                this.container.innerHTML = "";
            }
        }
    }

    // ProseMirror-based view
    class ProseMirrorView implements EditorInterface {
        private view: EditorView;
        private container: HTMLElement;

        constructor(target: HTMLElement, content: string) {
            this.container = target;

            // Clear target completely
            this.container.innerHTML = "";

            // Choose plugins based on showMenuBar prop
            const plugins = [
                history(),
                keymap(baseKeymap),
                keymap({
                    "Mod-b": (state, dispatch) => {
                        const { from, to } = state.selection;
                        const strongType = schema.marks.strong;
                        if (dispatch) {
                            const hasMark = state.doc.rangeHasMark(
                                from,
                                to,
                                strongType,
                            );
                            if (hasMark) {
                                dispatch(
                                    state.tr.removeMark(from, to, strongType),
                                );
                            } else {
                                dispatch(
                                    state.tr.addMark(
                                        from,
                                        to,
                                        strongType.create(),
                                    ),
                                );
                            }
                        }
                        return true;
                    },
                    "Mod-i": (state, dispatch) => {
                        const { from, to } = state.selection;
                        const emType = schema.marks.em;
                        if (dispatch) {
                            const hasMark = state.doc.rangeHasMark(
                                from,
                                to,
                                emType,
                            );
                            if (hasMark) {
                                dispatch(state.tr.removeMark(from, to, emType));
                            } else {
                                dispatch(
                                    state.tr.addMark(from, to, emType.create()),
                                );
                            }
                        }
                        return true;
                    },
                    "Mod-`": (state, dispatch) => {
                        const { from, to } = state.selection;
                        const codeType = schema.marks.code;
                        if (dispatch) {
                            const hasMark = state.doc.rangeHasMark(
                                from,
                                to,
                                codeType,
                            );
                            if (hasMark) {
                                dispatch(
                                    state.tr.removeMark(from, to, codeType),
                                );
                            } else {
                                dispatch(
                                    state.tr.addMark(
                                        from,
                                        to,
                                        codeType.create(),
                                    ),
                                );
                            }
                        }
                        return true;
                    },
                    "Mod-z": (state, dispatch) => {
                        return require("prosemirror-history").undo(
                            state,
                            dispatch,
                        );
                    },
                    "Mod-y": (state, dispatch) => {
                        return require("prosemirror-history").redo(
                            state,
                            dispatch,
                        );
                    },
                    "Mod-Shift-z": (state, dispatch) => {
                        return require("prosemirror-history").redo(
                            state,
                            dispatch,
                        );
                    },
                }),
                dropCursor(),
                gapCursor(),
            ];

            this.view = new EditorView(this.container, {
                state: EditorState.create({
                    doc: defaultMarkdownParser.parse(content),
                    plugins: plugins,
                }),
            });
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
            const ViewClass =
                newMode === "markdown" ? MarkdownView : ProseMirrorView;
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
        switchView("prosemirror");
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

    // Export function to set content
    export function setContent(content: string): void {
        currentContent = content;
        if (currentView) {
            const currentMode = viewMode;
            currentView.destroy();
            currentView = null;

            setTimeout(() => {
                const ViewClass =
                    currentMode === "markdown" ? MarkdownView : ProseMirrorView;
                currentView = new ViewClass(editorContainer, content);

                setTimeout(() => {
                    if (currentView) {
                        currentView.focus();
                    }
                }, 10);
            }, 10);
        }
    }

    // Handle radio button changes
    function handleModeChange(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.checked) {
            switchView(target.value as "markdown" | "prosemirror");
        }
    }

    // Handle global keydown for view switching
    function handleGlobalKeyDown(event: KeyboardEvent) {
        if (
            (event.ctrlKey || event.metaKey) &&
            event.key.toLowerCase() === "e"
        ) {
            event.preventDefault();
            const newMode =
                viewMode === "prosemirror" ? "markdown" : "prosemirror";
            switchView(newMode);
        }
    }
</script>

<svelte:window on:keydown={handleGlobalKeyDown} />

<div class="editor-wrapper">
    <div
        class="editor-container"
        bind:this={editorContainer}
        class:dark-theme={$theme === "dark"}
        style="--tab-color: {displayColor}; font-size: {fontSizes[$fontSize]
            ?.editor}; line-height: {fontSizes[$fontSize]?.lineHeight};"
    ></div>
</div>

<style>
    .editor-wrapper {
        max-width: 100%;
        margin: 0 auto;
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
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
        padding: 20px;
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
        font-family: "Monaco", "Menlo", "Ubuntu Mono", monospace;
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
        font-size: 28px;
        font-weight: bold;
        margin: 20px 0 12px 0;
        color: inherit;
        border-bottom: 2px solid var(--tab-color, #e9ecef);
        padding-bottom: 8px;
    }

    :global(.ProseMirror h2) {
        font-size: 22px;
        font-weight: bold;
        margin: 18px 0 10px 0;
        color: inherit;
    }

    :global(.ProseMirror h3) {
        font-size: 18px;
        font-weight: bold;
        margin: 16px 0 8px 0;
        color: inherit;
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
        background-color: #f1f3f4;
        padding: 3px 6px;
        border-radius: 4px;
        font-family: "Monaco", "Menlo", "Ubuntu Mono", monospace;
        font-size: 13px;
        color: inherit;
        border: 1px solid #e9ecef;
    }

    :global(.ProseMirror pre) {
        background-color: #f8f9fa;
        padding: 16px;
        border-radius: 6px;
        overflow-x: auto;
        margin: 16px 0;
        border: 1px solid #e9ecef;
    }

    :global(.ProseMirror pre code) {
        background: none;
        padding: 0;
        border: none;
        color: inherit;
    }

    /* Dropdown menu styling */
    :global(.ProseMirror-menu-dropdown) {
        position: relative;
        display: inline-block;
    }

    :global(.ProseMirror-menu-dropdown-menu) {
        position: absolute;
        top: 100%;
        left: 0;
        background: white;
        border: 1px solid #dee2e6;
        border-radius: 4px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        z-index: 1000;
        min-width: 140px;
    }

    :global(.ProseMirror-menu-dropdown-item) {
        display: block;
        padding: 8px 12px;
        cursor: pointer;
        border: none;
        background: none;
        text-align: left;
        width: 100%;
        font-size: 14px;
        color: #495057;
    }

    :global(.ProseMirror-menu-dropdown-item:hover) {
        background: #f8f9fa;
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
