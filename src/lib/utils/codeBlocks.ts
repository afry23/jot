import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
} from "@codemirror/view";
import {
  EditorState,
  type Range,
  StateEffect,
  StateField,
} from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { languages } from "@codemirror/language-data";
import type { CompletionContext, CompletionResult } from "@codemirror/autocomplete";
import { logger } from "./logger";

interface BlockInfo {
  from: number;
  to: number;
  lang: string;
  infoFrom: number | null;
  infoTo: number | null;
  markTo: number;
  codeFrom: number;
  codeTo: number;
}

// --- Zeilennummern-State (pro Block, nur UI) ---

const toggleLineNumbers = StateEffect.define<number>();

const numberedBlocks = StateField.define<number[]>({
  create: () => [],
  update(value, tr) {
    let next = tr.docChanged ? value.map((p) => tr.changes.mapPos(p, -1)) : value;
    for (const e of tr.effects) {
      if (e.is(toggleLineNumbers)) {
        next = next.includes(e.value)
          ? next.filter((p) => p !== e.value)
          : [...next, e.value];
      }
    }
    return next;
  },
});

function collectBlocks(state: EditorState): BlockInfo[] {
  const blocks: BlockInfo[] = [];
  syntaxTree(state).iterate({
    enter(node) {
      if (node.name !== "FencedCode") return;
      let infoFrom: number | null = null;
      let infoTo: number | null = null;
      let markTo = node.from;
      let codeFrom = -1;
      let codeTo = -1;
      let seenMark = false;
      for (let c = node.node.firstChild; c; c = c.nextSibling) {
        if (c.name === "CodeMark" && !seenMark) {
          markTo = c.to;
          seenMark = true;
        } else if (c.name === "CodeInfo") {
          infoFrom = c.from;
          infoTo = c.to;
        } else if (c.name === "CodeText") {
          if (codeFrom < 0) codeFrom = c.from;
          codeTo = c.to;
        }
      }
      const lang =
        infoFrom !== null
          ? state.sliceDoc(infoFrom, infoTo!).trim().split(/\s+/)[0]
          : "";
      if (codeFrom < 0) {
        // leerer Block: Code-Bereich = Zeile nach Fence (leer)
        codeFrom = codeTo = Math.min(node.to, state.doc.lineAt(node.from).to);
      }
      blocks.push({
        from: node.from,
        to: node.to,
        lang,
        infoFrom,
        infoTo,
        markTo,
        codeFrom,
        codeTo,
      });
      return false;
    },
  });
  return blocks;
}

const COPY_SVG =
  '<svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.4"><rect x="5.5" y="5.5" width="8" height="8" rx="1.5"/><path d="M10.5 3.5v-1a1 1 0 0 0-1-1h-7a1 1 0 0 0-1 1v7a1 1 0 0 0 1 1h1"/></svg>';
const CHECK_SVG =
  '<svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 8.5l3.2 3.2L13 4.8"/></svg>';

class CodeToolsWidget extends WidgetType {
  constructor(
    readonly block: BlockInfo,
    readonly numbered: boolean
  ) {
    super();
  }

  eq(o: CodeToolsWidget) {
    return (
      this.block.from === o.block.from &&
      this.block.codeFrom === o.block.codeFrom &&
      this.block.codeTo === o.block.codeTo &&
      this.numbered === o.numbered
    );
  }

  toDOM(view: EditorView) {
    const { block } = this;
    const dom = document.createElement("span");
    dom.className = "cm-codeblock-tools";

    const copyBtn = document.createElement("button");
    copyBtn.type = "button";
    copyBtn.className = "cm-codeblock-btn cm-codeblock-copy";
    copyBtn.title = "Code kopieren";
    copyBtn.innerHTML = COPY_SVG;
    copyBtn.onmousedown = (e) => {
      e.preventDefault();
      const code = view.state.sliceDoc(block.codeFrom, block.codeTo);
      navigator.clipboard
        .writeText(code)
        .then(() => {
          copyBtn.innerHTML = CHECK_SVG;
          setTimeout(() => (copyBtn.innerHTML = COPY_SVG), 1200);
        })
        .catch((err) => logger.error("Copy code failed", String(err)));
    };

    const lnBtn = document.createElement("button");
    lnBtn.type = "button";
    lnBtn.className = "cm-codeblock-btn cm-codeblock-ln" + (this.numbered ? " cm-codeblock-btn-on" : "");
    lnBtn.textContent = "#";
    lnBtn.title = "Zeilennummern";
    lnBtn.onmousedown = (e) => {
      e.preventDefault();
      view.dispatch({ effects: toggleLineNumbers.of(block.from) });
    };

    dom.append(copyBtn, lnBtn);
    return dom;
  }

  ignoreEvent() {
    return true;
  }
}

function buildDecorations(state: EditorState): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const sel = state.selection.main;
  const numbered = state.field(numberedBlocks);
  for (const block of collectBlocks(state)) {
    const isNumbered = numbered.includes(block.from);
    const first = state.doc.lineAt(block.from).number;
    const last = state.doc.lineAt(block.to).number;
    ranges.push(
      Decoration.widget({
        widget: new CodeToolsWidget(block, isNumbered),
        side: -1,
      }).range(block.from)
    );
    for (let n = first; n <= last; n++) {
      const line = state.doc.line(n);
      const isFence = n === first || (n === last && /^\s*(`{3,}|~{3,})\s*$/.test(line.text));
      const attrs: Record<string, string> = {};
      let cls = "cm-codeblock-line";
      const onFenceWithCursor = sel.from <= line.to && sel.to >= line.from;
      if (isFence && !onFenceWithCursor) cls += " cm-codeblock-fence-hidden";
      if (isNumbered && !isFence) {
        cls += " cm-codeblock-numbered";
        attrs["data-ln"] = String(n - first);
      }
      ranges.push(Decoration.line({ class: cls, attributes: attrs }).range(line.from));
    }
  }
  return Decoration.set(ranges, true);
}

const codeBlockDecorations = StateField.define<DecorationSet>({
  create: (state) => buildDecorations(state),
  update(deco, tr) {
    if (tr.docChanged || tr.selection || tr.effects.some((e) => e.is(toggleLineNumbers)) || syntaxTree(tr.state) !== syntaxTree(tr.startState)) {
      return buildDecorations(tr.state);
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

export function codeBlocksExtension() {
  return [numberedBlocks, codeBlockDecorations];
}

// --- Sprach-Autocomplete nach ``` ---

const languageOptions = (() => {
  const seen = new Set<string>();
  const out: { label: string; detail: string }[] = [];
  for (const l of languages) {
    for (const label of [l.name.toLowerCase(), ...l.alias]) {
      if (/\s/.test(label) || seen.has(label)) continue;
      seen.add(label);
      out.push({ label, detail: l.name });
    }
  }
  return out;
})();

export function languageCompletion(ctx: CompletionContext): CompletionResult | null {
  const line = ctx.state.doc.lineAt(ctx.pos);
  const before = line.text.slice(0, ctx.pos - line.from);
  const m = /^\s*(?:`{3,}|~{3,})([\w+#.-]*)$/.exec(before);
  if (!m) return null;
  // nur öffnende Fence (nicht schließende): vorheriger Text darf keine offene Fence sein
  const node = syntaxTree(ctx.state).resolveInner(line.from + before.length - m[1].length, -1);
  let n: typeof node | null = node;
  while (n && n.name !== "FencedCode") n = n.parent;
  if (n && n.from < line.from) return null;
  return {
    from: ctx.pos - m[1].length,
    options: languageOptions,
    validFor: /^[\w+#.-]*$/,
  };
}
