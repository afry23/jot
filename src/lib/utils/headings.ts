import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
} from "@codemirror/view";
import { EditorState, type Range, StateField } from "@codemirror/state";
import {
  syntaxTree,
  foldService,
  foldable,
  foldedRanges,
  foldEffect,
  unfoldEffect,
  codeFolding,
} from "@codemirror/language";

const HEADING_RE = /^ {0,3}(#{1,6})[ \t]*/;

// --- Heading-Erkennung ---

function headingLevel(name: string): number {
  const m = /^ATXHeading([1-6])$/.exec(name);
  return m ? Number(m[1]) : 0;
}

interface HeadingInfo {
  lineFrom: number;
  lineTo: number;
  level: number;
}

function collectHeadings(state: EditorState): HeadingInfo[] {
  const out: HeadingInfo[] = [];
  syntaxTree(state).iterate({
    enter(node) {
      const level = headingLevel(node.name);
      if (!level) return;
      const line = state.doc.lineAt(node.from);
      out.push({ lineFrom: line.from, lineTo: line.to, level });
      return false;
    },
  });
  return out;
}

// --- Folding: Abschnitt bis zum nächsten Heading gleicher/höherer Ebene ---

const headingFoldService = foldService.of((state, lineStart, lineEnd) => {
  let level = 0;
  syntaxTree(state).iterate({
    from: lineStart,
    to: lineEnd,
    enter(node) {
      if (level) return false;
      const l = headingLevel(node.name);
      if (l && state.doc.lineAt(node.from).from === lineStart) level = l;
      return !level;
    },
  });
  if (!level) return null;

  let end = state.doc.length;
  let found = false;
  syntaxTree(state).iterate({
    from: lineEnd,
    enter(node) {
      if (found) return false;
      const l = headingLevel(node.name);
      if (l) {
        if (l <= level) {
          end = state.doc.lineAt(node.from).from - 1;
          found = true;
        }
        return false;
      }
      return true;
    },
  });

  // trailing Leerzeilen nicht mit einklappen
  while (end > lineEnd && /^\s*$/.test(state.doc.lineAt(end).text)) {
    end = state.doc.lineAt(end).from - 1;
  }
  if (end <= lineEnd) return null;
  return { from: lineEnd, to: end };
});

function foldedRangeAt(state: EditorState, pos: number): { from: number; to: number } | null {
  let found: { from: number; to: number } | null = null;
  foldedRanges(state).between(pos, pos, (from, to) => {
    if (from === pos) found = { from, to };
  });
  return found;
}

// --- Widget: Fold-Pfeil + Level-Label + Popup ---

const ARROW_SVG =
  '<svg viewBox="0 0 10 10" width="9" height="9" fill="currentColor"><path d="M2 3l3 4 3-4z"/></svg>';

class HeadingToolsWidget extends WidgetType {
  constructor(
    readonly info: HeadingInfo,
    readonly foldableHere: boolean,
    readonly folded: boolean
  ) {
    super();
  }

  eq(o: HeadingToolsWidget) {
    return (
      this.info.lineFrom === o.info.lineFrom &&
      this.info.level === o.info.level &&
      this.foldableHere === o.foldableHere &&
      this.folded === o.folded
    );
  }

  toDOM(view: EditorView) {
    const { info } = this;
    const dom = document.createElement("span");
    dom.className = "cm-heading-tools" + (this.folded ? " cm-heading-tools-folded" : "");

    if (this.foldableHere) {
      const arrow = document.createElement("button");
      arrow.type = "button";
      arrow.className = "cm-heading-fold" + (this.folded ? " cm-heading-fold-closed" : "");
      arrow.title = this.folded ? "Ausklappen" : "Einklappen";
      arrow.innerHTML = ARROW_SVG;
      arrow.onmousedown = (e) => {
        e.preventDefault();
        const state = view.state;
        const line = state.doc.lineAt(info.lineFrom);
        const existing = foldedRangeAt(state, line.to);
        if (existing) {
          view.dispatch({ effects: unfoldEffect.of(existing) });
        } else {
          const range = foldable(state, line.from, line.to);
          if (range) view.dispatch({ effects: foldEffect.of(range) });
        }
      };
      dom.append(arrow);
    }

    const label = document.createElement("button");
    label.type = "button";
    label.className = "cm-heading-level";
    label.textContent = `H${info.level}`;
    label.title = "Überschrift-Ebene ändern";
    label.onmousedown = (e) => {
      e.preventDefault();
      e.stopPropagation();
      toggleMenu(view, dom, info);
    };
    dom.append(label);
    return dom;
  }

  ignoreEvent() {
    return true;
  }
}

function setLevel(view: EditorView, lineFrom: number, level: number) {
  const line = view.state.doc.lineAt(lineFrom);
  const m = HEADING_RE.exec(line.text);
  if (!m) return;
  const insert = level > 0 ? "#".repeat(level) + " " : "";
  view.dispatch({
    changes: { from: line.from, to: line.from + m[0].length, insert },
  });
  view.focus();
}

function toggleMenu(view: EditorView, host: HTMLElement, info: HeadingInfo) {
  const existing = host.querySelector(".cm-heading-menu");
  if (existing) {
    existing.remove();
    return;
  }
  const menu = document.createElement("div");
  menu.className = "cm-heading-menu";

  const close = () => {
    menu.remove();
    document.removeEventListener("mousedown", onOutside, true);
    document.removeEventListener("keydown", onKey, true);
  };
  const onOutside = (e: MouseEvent) => {
    if (!menu.contains(e.target as Node)) close();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") close();
  };

  const addItem = (text: string, level: number, cls = "") => {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "cm-heading-menu-item" + (level === info.level ? " cm-heading-menu-active" : "") + cls;
    b.textContent = text;
    b.onmousedown = (e) => {
      e.preventDefault();
      e.stopPropagation();
      close();
      setLevel(view, info.lineFrom, level);
    };
    menu.append(b);
  };
  for (let i = 1; i <= 6; i++) addItem(`H${i}`, i);
  addItem("Text", 0, " cm-heading-menu-text");

  host.append(menu);
  document.addEventListener("mousedown", onOutside, true);
  document.addEventListener("keydown", onKey, true);
}

// --- Decorations ---

function buildDecorations(state: EditorState): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const sel = state.selection.main;
  for (const h of collectHeadings(state)) {
    const folded = foldedRangeAt(state, h.lineTo) !== null;
    const canFold = folded || foldable(state, h.lineFrom, h.lineTo) !== null;
    const active = sel.from <= h.lineTo && sel.to >= h.lineFrom;
    ranges.push(
      Decoration.line({
        class: "cm-heading-line" + (active ? " cm-heading-line-active" : ""),
      }).range(h.lineFrom)
    );
    ranges.push(
      Decoration.widget({
        widget: new HeadingToolsWidget(h, canFold, folded),
        side: -1,
      }).range(h.lineFrom)
    );
  }
  return Decoration.set(ranges, true);
}

const headingDecorations = StateField.define<DecorationSet>({
  create: (state) => buildDecorations(state),
  update(deco, tr) {
    if (
      tr.docChanged ||
      tr.selection ||
      foldedRanges(tr.state) !== foldedRanges(tr.startState) ||
      syntaxTree(tr.state) !== syntaxTree(tr.startState)
    ) {
      return buildDecorations(tr.state);
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

export function headingsExtension() {
  return [
    headingFoldService,
    codeFolding({
      placeholderDOM(view, onclick) {
        const el = document.createElement("span");
        el.className = "cm-heading-fold-placeholder";
        el.textContent = "…";
        el.title = "Ausklappen";
        el.onclick = onclick;
        return el;
      },
    }),
    headingDecorations,
  ];
}
