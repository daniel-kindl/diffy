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
import { decorations, nextHunkLine, type Hunk, type LineDecoration } from "./hunks";
import { actionFor } from "./keys";
import { languageOf } from "./language";

export const FONT_FAMILY = '"Cascadia Mono", Consolas, monospace';

function create(
  el: HTMLElement,
  text: string,
  label: string,
  path: string,
): monaco.editor.IStandaloneCodeEditor {
  const editor = monaco.editor.create(el, {
    value: text,
    language: languageOf(path),
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
    lineDecorationsWidth: 18,
  });
  // The textarea keeps the readonly attribute for assistive technology (Q85).
  editor.getDomNode()?.querySelector("textarea")?.setAttribute("readonly", "readonly");
  return editor;
}

function toMonaco(ds: LineDecoration[]): monaco.editor.IModelDeltaDecoration[] {
  return ds.map((d) => ({
    range: new monaco.Range(d.startLine, 1, d.endLine, 1),
    options: {
      isWholeLine: true,
      className: `diffy-line-${d.kind}`,
      // A mark in the gutter, so no change relies on color alone.
      linesDecorationsClassName: `diffy-mark-${d.kind}`,
    } satisfies monaco.editor.IModelDecorationOptions,
  }));
}

/** Follow the app theme: Monaco has its own light and dark themes. */
export function setEditorTheme(dark: boolean): void {
  monaco.editor.setTheme(dark ? "vs-dark" : "vs");
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
  path = "",
): ComparisonView {
  const left = create(leftEl, base, "Base", path);
  const right = create(rightEl, head, "Head", path);
  const d = decorations(hunks);
  const l = left.createDecorationsCollection(toMonaco(d.left));
  const r = right.createDecorationsCollection(toMonaco(d.right));
  let active = right;
  left.onDidFocusEditorText(() => (active = left));
  right.onDidFocusEditorText(() => (active = right));
  const onKey = (e: KeyboardEvent) => {
    const action = actionFor(e);
    if (!action) return;
    e.preventDefault();
    switch (action) {
      case "nextChange":
      case "previousChange": {
        const from = right.getPosition()?.lineNumber ?? 1;
        const line = nextHunkLine(hunks, from, action === "nextChange" ? 1 : -1);
        if (line !== undefined) {
          right.revealLineInCenter(line);
          right.setPosition({ lineNumber: line, column: 1 });
          right.focus();
        }
        break;
      }
      case "find":
        void active.getAction("actions.find")?.run();
        break;
      case "goToLine":
        void active.getAction("editor.action.gotoLine")?.run();
        break;
      case "switchSide":
        (active === left ? right : left).focus();
        break;
    }
  };
  document.addEventListener("keydown", onKey);
  return {
    left,
    right,
    dispose() {
      document.removeEventListener("keydown", onKey);
      l.clear();
      r.clear();
      left.dispose();
      right.dispose();
    },
  };
}
