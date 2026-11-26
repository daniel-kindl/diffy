/** A canonical hunk from Rust. Lines are 0-based and the end is exclusive. */
export interface Hunk {
  base: { start: number; end: number };
  head: { start: number; end: number };
}

export type ChangeKind = "added" | "removed" | "modified";

export interface LineDecoration {
  /** 1-based, as Monaco numbers lines. */
  startLine: number;
  endLine: number;
  kind: ChangeKind;
}

export function kindOf(h: Hunk): ChangeKind {
  const removed = h.base.end - h.base.start;
  const added = h.head.end - h.head.start;
  if (removed === 0) return "added";
  if (added === 0) return "removed";
  return "modified";
}

/** The decorations for the two editors. A pure insertion has no base line. */
export function decorations(hunks: readonly Hunk[]): {
  left: LineDecoration[];
  right: LineDecoration[];
} {
  const left: LineDecoration[] = [];
  const right: LineDecoration[] = [];
  for (const h of hunks) {
    const kind = kindOf(h);
    if (h.base.end > h.base.start) {
      left.push({ startLine: h.base.start + 1, endLine: h.base.end, kind });
    }
    if (h.head.end > h.head.start) {
      right.push({ startLine: h.head.start + 1, endLine: h.head.end, kind });
    }
  }
  return { left, right };
}

/** The line to scroll to for the next or previous hunk, wrapping around. */
export function nextHunkLine(
  hunks: readonly Hunk[],
  currentHeadLine: number,
  direction: 1 | -1,
): number | undefined {
  if (hunks.length === 0) return undefined;
  const starts = hunks.map((h) => h.head.start + 1);
  if (direction === 1) {
    return starts.find((s) => s > currentHeadLine) ?? starts[0];
  }
  return [...starts].reverse().find((s) => s < currentHeadLine) ?? starts[starts.length - 1];
}
