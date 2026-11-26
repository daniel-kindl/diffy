// Option B (ADR 0004): two read-only Monaco editors that show the canonical
// hunks from Rust. Only the six features of Q58 are imported: find, tab focus,
// go to line, clipboard, context menu and the read-only message. There is no
// editing feature, folding, hover or link feature.
import * as monaco from "monaco-editor/editor/editor.api.js";
import "monaco-editor/editor/contrib/find/browser/findController.js";
import "monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode.js";
import "monaco-editor/editor/standalone/browser/quickAccess/standaloneGotoLineQuickAccess.js";
import "monaco-editor/editor/contrib/clipboard/browser/clipboard.js";
import "monaco-editor/editor/contrib/contextmenu/browser/contextmenu.js";
import "monaco-editor/editor/contrib/readOnlyMessage/browser/contribution.js";
import { decorations, type Hunk, type LineDecoration } from "./hunks";

export const FONT_FAMILY = '"Cascadia Mono", Consolas, monospace';

function create(el: HTMLElement, text: string, label: string): monaco.editor.IStandaloneCodeEditor {
  const editor = monaco.editor.create(el, {
    value: text,
    readOnly: true,
    readOnlyMessage: { value: "This comparison is read-only." },
    fontFamily: FONT_FAMILY,
    automaticLayout: true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    renderLineHighlight: "none",
    accessibilitySupport: "on",
    ariaLabel: label,
    // No feature outside Q58 is wanted, also if a user setting turns it on.
    folding: false,
    links: false,
    hover: { enabled: "off" },
    contextmenu: true,
  });
  // The textarea keeps the readonly attribute for assistive technology (Q85).
  editor.getDomNode()?.querySelector("textarea")?.setAttribute("readonly", "readonly");
  return editor;
}

function toMonaco(ds: LineDecoration[]): monaco.editor.IModelDeltaDecoration[] {
  return ds.map((d) => ({
    range: new monaco.Range(d.startLine, 1, d.endLine, 1),
    options: { isWholeLine: true, className: `diffy-line-${d.kind}` },
  }));
}

export interface ComparisonView {
  left: monaco.editor.IStandaloneCodeEditor;
  right: monaco.editor.IStandaloneCodeEditor;
  dispose(): void;
}

/** Show a comparison: raw text of each side and the canonical hunks. */
export function showComparison(
  leftEl: HTMLElement,
  rightEl: HTMLElement,
  base: string,
  head: string,
  hunks: readonly Hunk[],
): ComparisonView {
  const left = create(leftEl, base, "Base");
  const right = create(rightEl, head, "Head");
  const d = decorations(hunks);
  const l = left.createDecorationsCollection(toMonaco(d.left));
  const r = right.createDecorationsCollection(toMonaco(d.right));
  return {
    left,
    right,
    dispose() {
      l.clear();
      r.clear();
      left.dispose();
      right.dispose();
    },
  };
}
