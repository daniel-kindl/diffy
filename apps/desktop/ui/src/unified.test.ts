import { describe, expect, it } from "vitest";
import type { Hunk } from "./hunks";
import {
  changeStarts,
  collapseUnchanged,
  lineLabel,
  nextStart,
  splitLines,
  unifiedLines,
} from "./unified";

describe("splitLines", () => {
  it("ignores the final newline and keeps empty lines", () => {
    expect(splitLines("a\nb\n")).toEqual(["a", "b"]);
    expect(splitLines("a\n\nb")).toEqual(["a", "", "b"]);
    expect(splitLines("")).toEqual([]);
  });
});

describe("unifiedLines", () => {
  const base = "one\ntwo\nthree\nfour\n";
  const head = "one\nTWO\nTWO2\nthree\nfour\nfive\n";
  const hunks: Hunk[] = [
    { base: { start: 1, end: 2 }, head: { start: 1, end: 3 } },
    { base: { start: 4, end: 4 }, head: { start: 5, end: 6 } },
  ];

  it("puts the removed lines before the added lines of a hunk", () => {
    const u = unifiedLines(base, head, hunks);
    expect(u.map((l) => `${l.kind[0] ?? "?"}:${l.text}`)).toEqual([
      "c:one",
      "r:two",
      "a:TWO",
      "a:TWO2",
      "c:three",
      "c:four",
      "a:five",
    ]);
  });

  it("numbers each line in its own file and gives context both numbers", () => {
    const u = unifiedLines(base, head, hunks);
    expect(u[1]).toMatchObject({ base: 2, head: null });
    expect(u[2]).toMatchObject({ base: null, head: 2 });
    expect(u[4]).toMatchObject({ base: 3, head: 4 });
    expect(u[6]).toMatchObject({ base: null, head: 6 });
  });

  it("has every base line once and every head line once", () => {
    const u = unifiedLines(base, head, hunks);
    expect(u.filter((l) => l.kind !== "added")).toHaveLength(4);
    expect(u.filter((l) => l.kind !== "removed")).toHaveLength(6);
  });

  it("is the text itself when there is no change", () => {
    const u = unifiedLines("a\nb\n", "a\nb\n", []);
    expect(u.map((l) => l.kind)).toEqual(["context", "context"]);
  });

  it("labels the number column", () => {
    expect(lineLabel({ kind: "context", text: "", base: 3, head: 4 })).toBe("3 | 4");
    expect(lineLabel({ kind: "removed", text: "", base: 3, head: null })).toBe("3");
    expect(lineLabel({ kind: "added", text: "", base: null, head: 4 })).toBe("4");
    expect(lineLabel(undefined)).toBe("");
  });
});

describe("change navigation in the unified view", () => {
  const lines = unifiedLines("a\nb\nc\nd\n", "a\nB\nc\nd\ne\n", [
    { base: { start: 1, end: 2 }, head: { start: 1, end: 2 } },
    { base: { start: 4, end: 4 }, head: { start: 4, end: 5 } },
  ]);

  it("finds where each run of changes starts", () => {
    expect(changeStarts(lines)).toEqual([2, 6]);
    expect(changeStarts([])).toEqual([]);
  });

  it("moves to the next and previous start and wraps", () => {
    expect(nextStart([2, 6], 1, 1)).toBe(2);
    expect(nextStart([2, 6], 6, 1)).toBe(2);
    expect(nextStart([2, 6], 6, -1)).toBe(2);
    expect(nextStart([2, 6], 2, -1)).toBe(6);
    expect(nextStart([], 1, 1)).toBeUndefined();
  });
});

describe("collapseUnchanged", () => {
  const numbered = (n: number) =>
    Array.from({ length: n }, (_, i) => `line ${String(i + 1)}`).join("\n") + "\n";
  const edited = (n: number, at: number[]) =>
    Array.from({ length: n }, (_, i) =>
      at.includes(i) ? `CHANGED ${String(i + 1)}` : `line ${String(i + 1)}`,
    ).join("\n") + "\n";

  it("keeps the context around a change and collapses the rest", () => {
    const hunks = [{ base: { start: 19, end: 20 }, head: { start: 19, end: 20 } }];
    const u = collapseUnchanged(unifiedLines(numbered(40), edited(40, [19]), hunks), 3);
    expect(u.map((l) => l.kind)).toEqual([
      "collapsed",
      "context",
      "context",
      "context",
      "removed",
      "added",
      "context",
      "context",
      "context",
      "collapsed",
    ]);
    expect(u[0]?.text).toBe("16 unchanged lines");
    expect(u[9]?.text).toBe("17 unchanged lines");
  });

  it("never hides a change and keeps each changed line", () => {
    const hunks = [
      { base: { start: 2, end: 3 }, head: { start: 2, end: 3 } },
      { base: { start: 30, end: 31 }, head: { start: 30, end: 31 } },
    ];
    const all = unifiedLines(numbered(40), edited(40, [2, 30]), hunks);
    const u = collapseUnchanged(all, 3);
    expect(u.filter((l) => l.kind === "removed" || l.kind === "added")).toEqual(
      all.filter((l) => l.kind === "removed" || l.kind === "added"),
    );
  });

  it("does not collapse a run that is too short to save lines", () => {
    // Two lines hidden would be one marker: 1 line saved. One hidden line is kept.
    const hunks = [
      { base: { start: 0, end: 1 }, head: { start: 0, end: 1 } },
      { base: { start: 8, end: 9 }, head: { start: 8, end: 9 } },
    ];
    const all = unifiedLines(numbered(9), edited(9, [0, 8]), hunks);
    // 7 unchanged lines between the changes, 3 + 3 context, 1 hidden: kept as is.
    expect(collapseUnchanged(all, 3)).toEqual(all);
  });

  it("collapses a whole file with no change into one marker", () => {
    const u = collapseUnchanged(unifiedLines(numbered(30), numbered(30), []), 3);
    expect(u).toHaveLength(1);
    expect(u[0]).toMatchObject({ kind: "collapsed", text: "30 unchanged lines" });
  });

  it("does not count a collapsed marker as a change for the keys", () => {
    const hunks = [{ base: { start: 19, end: 20 }, head: { start: 19, end: 20 } }];
    const u = collapseUnchanged(unifiedLines(numbered(40), edited(40, [19]), hunks), 3);
    expect(changeStarts(u)).toEqual([5]);
  });

  it("handles an empty file", () => {
    expect(collapseUnchanged([], 3)).toEqual([]);
  });
});
