<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { get } from "svelte/store";
  import {
    EditorView,
    ViewPlugin,
    keymap,
    Decoration,
    hoverTooltip,
    type DecorationSet,
    type ViewUpdate,
  } from "@codemirror/view";
  import { EditorState, Compartment, StateEffect, StateField, RangeSetBuilder } from "@codemirror/state";
  import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
  import {
    HighlightStyle,
    syntaxHighlighting,
    syntaxTree,
  } from "@codemirror/language";
  import {
    history,
    defaultKeymap,
    historyKeymap,
  } from "@codemirror/commands";
  import { tags } from "@lezer/highlight";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-shell";
  import { theme, fontSize } from "$lib/stores/settings";
  import { spellCheckEnabled, type GrammarError } from "$lib/stores/languageServices";
  import { activeTab } from "$lib/stores/tabs";
  import { notes, updateNote } from "$lib/stores/notes";
  import { tabColors } from "$lib/utils/colors";
  import { saveNote } from "$lib/utils/persistence";
  import { cursorPositions, scrollPositions } from "$lib/stores/cursorPostionStore";
  import { getHeadingColorWithOpacity } from "$lib/utils/uiColors";
  import { getCodeBackground } from "$lib/utils/textFormatting";
  import { logger } from "$lib/utils/logger";
  import LinkModal from "./LinkModal.svelte";

  // --- Hide-Marks Plugin: Syntax nur auf Cursor-Zeile sichtbar ---
  const HIDE_MARKS = new Set([
    "HeaderMark",
    "EmphasisMark",
    "CodeMark",
    "QuoteMark",
    "StrikethroughMark",
    "LinkMark",
    "URL",
  ]);

  function buildHideMarksDecos(v: EditorView): DecorationSet {
    const { state } = v;
    const sel = state.selection.main;
    const fromLine = state.doc.lineAt(sel.from).number;
    const toLine = state.doc.lineAt(sel.to).number;
    const builder = new RangeSetBuilder<Decoration>();
    for (const { from, to } of v.visibleRanges) {
      syntaxTree(state).iterate({
        from,
        to,
        enter(node) {
          if (HIDE_MARKS.has(node.name)) {
            const markLine = state.doc.lineAt(node.from).number;
            if (markLine < fromLine || markLine > toLine) {
              builder.add(node.from, node.to, Decoration.replace({}));
            }
          }
        },
      });
    }
    return builder.finish();
  }

  const hideMarksPlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = buildHideMarksDecos(view);
      }
      update(update: ViewUpdate) {
        if (update.docChanged || update.selectionSet || update.viewportChanged) {
          this.decorations = buildHideMarksDecos(update.view);
        }
      }
    },
    { decorations: (v) => v.decorations }
  );

  // --- Spell-Check via LanguageTool ---

  interface SpellError {
    from: number;
    to: number;
    error: GrammarError;
  }

  interface SpellState {
    decorations: DecorationSet;
    errors: SpellError[];
  }

  const setSpellErrors = StateEffect.define<SpellError[]>();

  const spellErrorField = StateField.define<SpellState>({
    create: () => ({ decorations: Decoration.none, errors: [] }),
    update(state, tr) {
      let decorations = state.decorations.map(tr.changes);
      let errors = state.errors
        .map((e) => ({
          ...e,
          from: tr.changes.mapPos(e.from),
          to: tr.changes.mapPos(e.to, 1),
        }))
        .filter((e) => e.from < e.to);

      for (const effect of tr.effects) {
        if (effect.is(setSpellErrors)) {
          const newErrors = effect.value;
          const builder = new RangeSetBuilder<Decoration>();
          for (const { from, to } of newErrors.sort((a, b) => a.from - b.from)) {
            if (from < to) {
              builder.add(from, to, Decoration.mark({ class: "cm-spell-error" }));
            }
          }
          decorations = builder.finish();
          errors = newErrors;
        }
      }

      return { decorations, errors };
    },
    provide: (f) => EditorView.decorations.from(f, (s) => s.decorations),
  });

  const spellCheckPlugin = ViewPlugin.fromClass(
    class {
      private timer: ReturnType<typeof setTimeout> | null = null;
      private view: EditorView;

      constructor(view: EditorView) {
        this.view = view;
        this.scheduleCheck();
      }

      update(update: ViewUpdate) {
        if (update.docChanged) this.scheduleCheck();
      }

      scheduleCheck() {
        if (this.timer) clearTimeout(this.timer);
        this.timer = setTimeout(() => this.runCheck(), 1500);
      }

      async runCheck() {
        const text = this.view.state.doc.toString();
        if (!text.trim()) {
          this.view.dispatch({ effects: setSpellErrors.of([]) });
          return;
        }
        try {
          const result = await invoke<{ matches: GrammarError[] }>("check_grammar", {
            text,
            language: "auto",
          });
          if (!this.view.dom.isConnected) return;
          const errors: SpellError[] = result.matches.map((m) => ({
            from: m.offset,
            to: m.offset + m.length,
            error: m,
          }));
          this.view.dispatch({ effects: setSpellErrors.of(errors) });
        } catch (e) {
          logger.error("Spell check failed", String(e));
        }
      }

      destroy() {
        if (this.timer) clearTimeout(this.timer);
      }
    }
  );

  const spellHoverTooltip = hoverTooltip(
    (view, pos) => {
    if (!view.state.field(spellErrorField, false)) return null;
    const { errors } = view.state.field(spellErrorField);
    const match = errors.find((e) => pos >= e.from && pos <= e.to);
    if (!match) return null;

    return {
      pos: match.from,
      end: match.to,
      above: true,
      create() {
        const isDark = !!document.querySelector(".dark");
        const dom = document.createElement("div");
        dom.className = "cm-spell-tooltip";
        dom.style.cssText = isDark
          ? "background:#2d2d2d;color:#e0e0e0;border-color:rgba(255,255,255,0.12)"
          : "background:#fff;color:#1a1a1a;border-color:rgba(0,0,0,0.12)";

        if (match.error.replacements.length === 0) {
          const none = document.createElement("span");
          none.className = "cm-spell-tooltip-empty";
          none.textContent = match.error.message;
          dom.appendChild(none);
        } else {
          match.error.replacements.slice(0, 5).forEach((r) => {
            const btn = document.createElement("button");
            btn.className = "cm-spell-suggestion";
            btn.style.cssText = isDark
              ? "border-color:rgba(255,255,255,0.18);color:#e0e0e0"
              : "border-color:rgba(0,0,0,0.15);color:#1a1a1a";
            btn.textContent = r.value;
            btn.onmousedown = (e) => {
              e.preventDefault();
              const { errors } = view.state.field(spellErrorField);
              const changes = view.state.changes({ from: match.from, to: match.to, insert: r.value });
              const remaining = errors
                .filter((err) => err.from !== match.from || err.to !== match.to)
                .map((err) => ({
                  ...err,
                  from: changes.mapPos(err.from),
                  to: changes.mapPos(err.to, 1),
                }))
                .filter((err) => err.from < err.to);
              view.dispatch({
                changes: { from: match.from, to: match.to, insert: r.value },
                effects: setSpellErrors.of(remaining),
              });
            };
            dom.appendChild(btn);
          });
        }

        return { dom };
      },
    };
  },
  {
    hideOn: (tr) => tr.effects.some((e) => e.is(setSpellErrors)),
  }
  );

  function buildSpellCheckExtensions() {
    return [spellErrorField, spellCheckPlugin, spellHoverTooltip];
  }


  // --- Zeilen-Hervorhebung nach Tab-Wechsel ---
  const setHighlightLine = StateEffect.define<number | null>();

  const highlightLineField = StateField.define<DecorationSet>({
    create: () => Decoration.none,
    update(deco, tr) {
      deco = deco.map(tr.changes);
      for (const e of tr.effects) {
        if (e.is(setHighlightLine)) {
          if (e.value === null) {
            deco = Decoration.none;
          } else {
            const line = tr.state.doc.lineAt(e.value);
            deco = Decoration.set([
              Decoration.line({ class: "cm-activated-line" }).range(line.from),
            ]);
          }
        }
      }
      return deco;
    },
    provide: (f) => EditorView.decorations.from(f),
  });

  let highlightTimeout: ReturnType<typeof setTimeout> | null = null;

  export let initialContent: string = "";

  const dispatch = createEventDispatcher<{ change: string }>();

  let editorContainer: HTMLElement;
  let view: EditorView | null = null;
  let showLinkModal = false;
  let linkModalInitialText = "";

  const fontSizeMap: Record<string, string> = {
    small: "14px",
    medium: "16px",
    large: "18px",
  };

  const highlightCompartment = new Compartment();
  const editorThemeCompartment = new Compartment();
  const spellCheckCompartment = new Compartment();

  function buildHighlightStyle() {
    const isDark = $theme === "dark";
    const color = tabColors[$activeTab];
    const h1Color = getHeadingColorWithOpacity(color, 1);
    const h2Color = getHeadingColorWithOpacity(color, 2);
    const h3Color = getHeadingColorWithOpacity(color, 3);
    const h4Color = getHeadingColorWithOpacity(color, 4);
    const h5Color = getHeadingColorWithOpacity(color, 5);
    const h6Color = getHeadingColorWithOpacity(color, 6);
    const codeBg = getCodeBackground(color);
    const syntaxMarkColor = isDark
      ? "rgba(200,200,200,0.38)"
      : "rgba(0,0,0,0.32)";

    return HighlightStyle.define([
      // Headings — larger font, tab-color-based hue
      {
        tag: tags.heading1,
        fontSize: "1.8em",
        fontWeight: "bold",
        color: h1Color,
        lineHeight: "1.3",
      },
      {
        tag: tags.heading2,
        fontSize: "1.5em",
        fontWeight: "bold",
        color: h2Color,
        lineHeight: "1.3",
      },
      {
        tag: tags.heading3,
        fontSize: "1.25em",
        fontWeight: "600",
        color: h3Color,
        lineHeight: "1.3",
      },
      { tag: tags.heading4, fontWeight: "600", color: h4Color },
      { tag: tags.heading5, fontWeight: "600", color: h5Color },
      { tag: tags.heading6, fontWeight: "600", color: h6Color },
      // Bold & italic
      { tag: tags.strong, fontWeight: "bold" },
      { tag: tags.emphasis, fontStyle: "italic" },
      { tag: tags.strikethrough, textDecoration: "line-through" },
      // Links
      {
        tag: tags.link,
        color: color,
        textDecoration: "underline",
        textDecorationStyle: "dotted",
        cursor: "pointer",
      },
      { tag: tags.url, color: color, opacity: "0.7" },
      // Inline code
      {
        tag: tags.monospace,
        fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
        fontSize: "0.88em",
        background: codeBg,
        borderRadius: "3px",
        padding: "0.1em 0.3em",
      },
      // Blockquote
      {
        tag: tags.quote,
        color: isDark ? "#BBBBBB" : "#666666",
        fontStyle: "italic",
      },
      // Syntax marks (**, *, #, `, >, -) — dimmed so they fade into background
      {
        tag: tags.processingInstruction,
        color: syntaxMarkColor,
        fontWeight: "normal",
        fontStyle: "normal",
      },
    ]);
  }

  function buildEditorTheme() {
    const isDark = $theme === "dark";
    const fs = fontSizeMap[$fontSize] ?? "16px";
    const caretColor = tabColors[$activeTab];
    const textColor = isDark ? "#E0E0E0" : "#1a1a1a";
    const selBg = isDark ? "rgba(255,255,255,0.13)" : "rgba(0,0,0,0.09)";
    const activeLineBg = isDark ? "rgba(255,255,255,0.03)" : "rgba(0,0,0,0.025)";

    return EditorView.theme(
      {
        "&": {
          height: "100%",
          fontSize: fs,
          backgroundColor: "transparent",
        },
        ".cm-content": {
          caretColor,
          fontFamily:
            "'Segoe UI', system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
          padding: "20px 28px 200px",
          lineHeight: "1.7",
          color: textColor,
          maxWidth: "800px",
        },
        ".cm-cursor, .cm-dropCursor": {
          borderLeftColor: caretColor,
          borderLeftWidth: "2px",
        },
        "&.cm-focused .cm-cursor": { borderLeftColor: caretColor },
        ".cm-selectionBackground": { backgroundColor: selBg },
        ".cm-focused .cm-selectionBackground": { backgroundColor: selBg },
        "::selection": { backgroundColor: selBg },
        ".cm-activeLine": { backgroundColor: activeLineBg },
        ".cm-gutters": { display: "none" },
        ".cm-scroller": { overflow: "auto", height: "100%" },
        ".cm-line": { padding: "0 4px" },
        "&.cm-focused": { outline: "none" },
      },
      { dark: isDark }
    );
  }

  function reconfigureEditor() {
    if (!view) return;
    view.dispatch({
      effects: [
        highlightCompartment.reconfigure(
          syntaxHighlighting(buildHighlightStyle())
        ),
        editorThemeCompartment.reconfigure(buildEditorTheme()),
      ],
    });
  }

  // Reconfigure when theme or font size changes
  $: {
    const _t = $theme;
    const _f = $fontSize;
    if (view && (_t || _f)) reconfigureEditor();
  }

  // Toggle spell check on/off
  $: if (view) {
    view.dispatch({
      effects: spellCheckCompartment.reconfigure(
        $spellCheckEnabled ? buildSpellCheckExtensions() : []
      ),
    });
  }

  // --- Markdown formatting helpers ---

  function toggleWrap(marker: string): boolean {
    if (!view) return false;
    const { state } = view;
    const { from, to } = state.selection.main;
    const selected = state.sliceDoc(from, to);
    const mLen = marker.length;
    const before = state.sliceDoc(Math.max(0, from - mLen), from);
    const after = state.sliceDoc(to, Math.min(state.doc.length, to + mLen));

    if (before === marker && after === marker) {
      view.dispatch({
        changes: [
          { from: from - mLen, to: from, insert: "" },
          { from: to, to: to + mLen, insert: "" },
        ],
        selection: { anchor: from - mLen, head: to - mLen },
      });
    } else {
      view.dispatch({
        changes: { from, to, insert: `${marker}${selected}${marker}` },
        selection: { anchor: from + mLen, head: to + mLen },
      });
    }
    return true;
  }

  function insertTimestamp(): boolean {
    if (!view) return false;
    const now = new Date();
    const formatted = now.toLocaleString(undefined, {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
    const { from } = view.state.selection.main;
    const charBefore = from > 0 ? view.state.sliceDoc(from - 1, from) : "";
    const charAfter = view.state.sliceDoc(from, from + 1);
    const pre = charBefore && !/[\s]/.test(charBefore) ? " " : "";
    const post = charAfter && !/[\s]/.test(charAfter) ? " " : "";
    const stamp = `${pre}**${formatted}**${post}`;
    view.dispatch({
      changes: { from, to: from, insert: stamp },
      selection: { anchor: from + stamp.length },
    });
    return true;
  }

  function openLinkModal(): boolean {
    if (!view) return false;
    const { from, to } = view.state.selection.main;
    linkModalInitialText = view.state.sliceDoc(from, to);
    showLinkModal = true;
    return true;
  }

  function handleLinkSubmit(
    event: CustomEvent<{ href: string; text: string }>
  ) {
    if (!view) return;
    const { href, text } = event.detail;
    const { from, to } = view.state.selection.main;
    const linkText = `[${text}](${href})`;
    view.dispatch({
      changes: { from, to, insert: linkText },
      selection: { anchor: from + linkText.length },
    });
    showLinkModal = false;
    view.focus();
  }

  function handleLinkCancel() {
    showLinkModal = false;
    view?.focus();
  }

  function indentLine(): boolean {
    if (!view) return false;
    const { state } = view;
    const { from } = state.selection.main;
    const line = state.doc.lineAt(from);
    view.dispatch({
      changes: { from: line.from, to: line.from, insert: "    " },
      selection: { anchor: from + 4 },
    });
    return true;
  }

  function unindentLine(): boolean {
    if (!view) return false;
    const { state } = view;
    const { from } = state.selection.main;
    const line = state.doc.lineAt(from);
    const match = line.text.match(/^( {1,4}|\t)/);
    if (!match) return false;
    const n = match[0].length;
    view.dispatch({
      changes: { from: line.from, to: line.from + n, insert: "" },
      selection: { anchor: Math.max(line.from, from - n) },
    });
    return true;
  }

  // --- Persistence ---

  function handleUpdate(update: ViewUpdate) {
    if (!update.docChanged) return;
    const content = update.state.doc.toString();
    updateNote($activeTab, content);
    dispatch("change", content);
    saveNote($activeTab, content);
  }

  function saveCursorAndScroll() {
    if (!view) return;
    const pos = view.state.selection.main.head;
    const scrollTop = view.scrollDOM.scrollTop;
    cursorPositions.update((cp) => ({ ...cp, [$activeTab]: pos }));
    scrollPositions.update((sp) => ({ ...sp, [$activeTab]: scrollTop }));
  }

  function restorePositions() {
    if (!view) return;
    const pos = $cursorPositions[$activeTab];
    const safePos =
      typeof pos === "number"
        ? Math.min(pos, view.state.doc.length)
        : 0;

    // Cursor setzen, zur Position scrollen und Zeile kurz hervorheben
    view.dispatch({
      selection: { anchor: safePos },
      scrollIntoView: true,
      effects: setHighlightLine.of(safePos),
    });

    // Hervorhebung nach 900 ms wieder entfernen
    if (highlightTimeout) clearTimeout(highlightTimeout);
    highlightTimeout = setTimeout(() => {
      view?.dispatch({ effects: setHighlightLine.of(null) });
    }, 900);

    view.focus();
  }

  onMount(() => {
    const content = initialContent ?? "";

    view = new EditorView({
      state: EditorState.create({
        doc: content,
        extensions: [
          markdown({ base: markdownLanguage }),
          hideMarksPlugin,
          highlightLineField,
          highlightCompartment.of(syntaxHighlighting(buildHighlightStyle())),
          editorThemeCompartment.of(buildEditorTheme()),
          spellCheckCompartment.of([]),
          history(),
          EditorView.lineWrapping,
          EditorView.updateListener.of(handleUpdate),
          EditorView.domEventHandlers({
            click(event, editorView) {
              const pos = editorView.posAtCoords({
                x: event.clientX,
                y: event.clientY,
              });
              if (pos === null) return false;
              const tree = syntaxTree(editorView.state);
              let node = tree.resolve(pos, 1);
              while (node) {
                if (node.name === "URL") {
                  const href = editorView.state.sliceDoc(node.from, node.to);
                  if (/^https?:\/\/|^mailto:/i.test(href)) {
                    event.preventDefault();
                    open(href).catch((e) =>
                      logger.error("Failed to open URL", e)
                    );
                    return true;
                  }
                  break;
                }
                if (node.name === "Link" || node.name === "Image") {
                  let child = node.firstChild;
                  while (child) {
                    if (child.name === "URL") {
                      const href = editorView.state.sliceDoc(child.from, child.to);
                      if (/^https?:\/\/|^mailto:/i.test(href)) {
                        event.preventDefault();
                        open(href).catch((e) =>
                          logger.error("Failed to open URL", e)
                        );
                        return true;
                      }
                    }
                    child = child.nextSibling;
                  }
                  break;
                }
                if (!node.parent) break;
                node = node.parent;
              }
              return false;
            },
          }),
          keymap.of([
            { key: "Mod-b", run: () => toggleWrap("**") },
            { key: "Mod-i", run: () => toggleWrap("*") },
            { key: "Mod-`", run: () => toggleWrap("`") },
            { key: "Mod-k", run: () => openLinkModal() },
            { key: "Mod-t", run: () => insertTimestamp() },
            { key: "Tab", run: () => indentLine() },
            { key: "Shift-Tab", run: () => unindentLine() },
            ...historyKeymap,
            ...defaultKeymap,
          ]),
        ],
      }),
      parent: editorContainer,
    });

    // Initialize spell check if it was previously enabled
    if (get(spellCheckEnabled)) {
      view.dispatch({
        effects: spellCheckCompartment.reconfigure(buildSpellCheckExtensions()),
      });
    }

    requestAnimationFrame(() => restorePositions());
  });

  onDestroy(() => {
    if (highlightTimeout) clearTimeout(highlightTimeout);
    if (view) {
      saveCursorAndScroll();
      const content = view.state.doc.toString();
      updateNote($activeTab, content);
      saveNote($activeTab, content);
      view.destroy();
      view = null;
    }
  });
</script>

<div class="editor-wrapper" bind:this={editorContainer}></div>

{#if showLinkModal}
  <LinkModal
    initialText={linkModalInitialText}
    on:submit={handleLinkSubmit}
    on:cancel={handleLinkCancel}
  />
{/if}

<style>
  .editor-wrapper {
    height: 100%;
    width: 100%;
    overflow: hidden;
  }

  :global(.cm-editor) {
    height: 100%;
    outline: none !important;
  }

  :global(.cm-scroller) {
    height: 100%;
    overflow: auto !important;
  }

  :global(.cm-activated-line) {
    animation: cm-line-flash 0.9s ease-out forwards;
  }

  @keyframes -global-cm-line-flash {
    0%   { background-color: rgba(255, 200, 80, 0.28); }
    40%  { background-color: rgba(255, 200, 80, 0.18); }
    100% { background-color: transparent; }
  }

  :global(.cm-spell-error) {
    text-decoration: underline wavy red;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
  }

  :global(.cm-spell-tooltip) {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 6px 8px;
    max-width: 280px;
    background: var(--cm-tooltip-bg, #fff);
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 6px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  }

  :global(.cm-spell-suggestion) {
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    background: transparent;
    cursor: pointer;
    font-size: 0.88em;
    color: inherit;
    transition: background 0.1s;
  }

  :global(.cm-spell-suggestion:hover) {
    background: rgba(0, 0, 0, 0.06);
  }

  :global(.cm-spell-tooltip-empty) {
    font-size: 0.85em;
    opacity: 0.7;
    font-style: italic;
  }
</style>
