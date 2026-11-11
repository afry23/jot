import { Decoration, EditorView, type DecorationSet } from "@codemirror/view";
import {
  EditorState,
  RangeSet,
  StateField,
  type Range,
} from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";

// Inline-Elemente: Marks werden versteckt, solange die Selektion das Element nicht berührt.
const INLINE_PARENTS = new Set([
  "StrongEmphasis",
  "Emphasis",
  "Strikethrough",
  "InlineCode",
  "Link",
]);
const INLINE_MARKS = new Set([
  "EmphasisMark",
  "StrikethroughMark",
  "CodeMark",
  "LinkMark",
  "URL",
  "LinkTitle",
]);
// Block-Marks: versteckt, solange die Selektion nicht in derselben Zeile liegt.
const BLOCK_MARKS = new Set(["HeaderMark", "QuoteMark"]);

// replace ohne Widget: Text verschwindet, Cursor-Navigation bleibt korrekt (atomicRanges)
const hidden = Decoration.replace({});

function touches(state: EditorState, from: number, to: number): boolean {
  return state.selection.ranges.some((r) => r.from <= to && r.to >= from);
}

function buildDecorations(state: EditorState): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const tree = syntaxTree(state);

  {
    tree.iterate({
      enter(node) {
        const name = node.name;

        if (BLOCK_MARKS.has(name)) {
          const line = state.doc.lineAt(node.from);
          // Nur führende Marks (kein schließendes "##")
          if (
            name === "HeaderMark" &&
            state.sliceDoc(line.from, node.from).trim() !== ""
          )
            return false;
          if (touches(state, line.from, line.to)) return false;
          let end = node.to;
          if (end < line.to && state.sliceDoc(end, end + 1) === " ") end++;
          ranges.push(hidden.range(node.from, end));
          return false;
        }

        if (INLINE_MARKS.has(name)) {
          const parent = node.node.parent;
          if (!parent || !INLINE_PARENTS.has(parent.name)) return;
          if (parent.name === "Link") {
            // Referenz-Links ([a][b]) unangetastet lassen
            let hasUrl = false;
            for (let c = parent.firstChild; c; c = c.nextSibling) {
              if (c.name === "URL") hasUrl = true;
            }
            if (!hasUrl) return false;
            // Nur Klammern/URL/Titel; Link-Text bleibt
          }
          // Elementweise aufdecken: nur das Element unter/neben dem Cursor
          // zeigt seine Syntax, Rest der Zeile bleibt gerendert
          if (touches(state, parent.from, parent.to)) return false;
          if (node.to > node.from)
            ranges.push(hidden.range(node.from, node.to));
          return false;
        }
      },
    });
  }

  return Decoration.set(ranges, true);
}

const liveMarkdownField = StateField.define<DecorationSet>({
  create: (state) => buildDecorations(state),
  update(deco, tr) {
    if (
      tr.docChanged ||
      tr.selection ||
      syntaxTree(tr.state) !== syntaxTree(tr.startState)
    ) {
      // Gleiches Ergebnis -> altes (gemapptes) Set behalten, damit CodeMirror
      // keine Decoration-Änderung erkennt und das DOM der Zeile nicht anfasst.
      const mapped = deco.map(tr.changes);
      const next = buildDecorations(tr.state);
      return RangeSet.eq([mapped], [next]) ? mapped : next;
    }
    return deco;
  },
  provide: (f) => [
    EditorView.decorations.from(f),
    EditorView.atomicRanges.of((view) => view.state.field(f)),
  ],
});

export function liveMarkdownExtension() {
  return [liveMarkdownField];
}
