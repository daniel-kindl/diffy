// The unified view (C-05): one editor with the removed lines before the added
// lines of each hunk. It comes from the same canonical hunks as the
// side-by-side view, so the views never disagree.
import type { Hunk } from "./hunks";

export type UnifiedKind = "context" | "removed" | "added" | "collapsed";

export interface UnifiedLine {
  kind: UnifiedKind;
  text: string;
  /** 1-based line numbers in the base and in the head. */
  base: number | null;
  head: number | null;
}

/** Split text into lines. A final newline does not make an empty last line. */
export function splitLines(text: string): string[] {
  if (text === "") return [];
  const lines = text.split("\n");
  if (lines[lines.length - 1] === "") lines.pop();
  return lines;
}

export function unifiedLines(base: string, head: string, hunks: readonly Hunk[]): UnifiedLine[] {
  const a = splitLines(base);
  const b = splitLines(head);
  const out: UnifiedLine[] = [];
  let i = 0;
  let j = 0;
  for (const h of hunks) {
    while (i < h.base.start) {
      out.push({ kind: "context", text: a[i] ?? "", base: i + 1, head: j + 1 });
      i++;
      j++;
    }
    for (; i < h.base.end; i++) {
      out.push({ kind: "removed", text: a[i] ?? "", base: i + 1, head: null });
    }
    for (; j < h.head.end; j++) {
      out.push({ kind: "added", text: b[j] ?? "", base: null, head: j + 1 });
    }
  }
  while (i < a.length) {
    out.push({ kind: "context", text: a[i] ?? "", base: i + 1, head: j + 1 });
    i++;
    j++;
  }
  return out;
}

/** The label of the line number column: the base and the head numbers. */
export function lineLabel(line: UnifiedLine | undefined): string {
  if (!line) return "";
  return `${line.base?.toString() ?? ""} ${line.head?.toString() ?? ""}`.trim().replace(" ", " | ");
}

/** The 1-based line numbers where a run of changed lines starts. */
export function changeStarts(lines: readonly UnifiedLine[]): number[] {
  const starts: number[] = [];
  lines.forEach((l, i) => {
    const changed = (k: UnifiedKind | undefined) => k === "removed" || k === "added";
    if (changed(l.kind) && (i === 0 || !changed(lines[i - 1]?.kind))) starts.push(i + 1);
  });
  return starts;
}

/** The next or previous change start, wrapping around. */
export function nextStart(
  starts: readonly number[],
  current: number,
  direction: 1 | -1,
): number | undefined {
  if (starts.length === 0) return undefined;
  if (direction === 1) return starts.find((s) => s > current) ?? starts[0];
  return [...starts].reverse().find((s) => s < current) ?? starts[starts.length - 1];
}

/** The most unchanged lines that stay visible around a change. Q82 default. */
export const DEFAULT_CONTEXT = 3;

/**
 * Collapse each run of unchanged lines that is more than `2 * context` long into
 * one marker line, and keep `context` lines next to each change. A change is
 * never hidden. With no change at all, the whole text is one collapsed run.
 */
export function collapseUnchanged(
  lines: readonly UnifiedLine[],
  context = DEFAULT_CONTEXT,
): UnifiedLine[] {
  const out: UnifiedLine[] = [];
  let i = 0;
  while (i < lines.length) {
    if (lines[i]?.kind !== "context") {
      out.push(lines[i] as UnifiedLine);
      i++;
      continue;
    }
    let end = i;
    while (end < lines.length && lines[end]?.kind === "context") end++;
    const before = i === 0 ? 0 : context;
    const after = end === lines.length ? 0 : context;
    const hidden = end - i - before - after;
    if (hidden > 1) {
      out.push(...lines.slice(i, i + before));
      out.push({
        kind: "collapsed",
        text: `${String(hidden)} unchanged lines`,
        base: null,
        head: null,
      });
      out.push(...lines.slice(end - after, end));
    } else {
      out.push(...lines.slice(i, end));
    }
    i = end;
  }
  return out;
}
