import { describe, expect, it } from "vitest";
import { decorations, gaps, kindOf, nextHunkLine, type Hunk } from "./hunks";

const hunks: Hunk[] = [
  { base: { start: 1, end: 1 }, head: { start: 1, end: 3 } },
  { base: { start: 5, end: 6 }, head: { start: 7, end: 7 } },
  { base: { start: 9, end: 10 }, head: { start: 10, end: 12 } },
];

describe("hunks", () => {
  it("names the change kind", () => {
    expect(hunks.map(kindOf)).toEqual(["added", "removed", "modified"]);
  });

  it("gives 1-based decorations and skips an empty side", () => {
    const d = decorations(hunks);
    expect(d.left).toEqual([
      { startLine: 6, endLine: 6, kind: "removed" },
      { startLine: 10, endLine: 10, kind: "modified" },
    ]);
    expect(d.right).toEqual([
      { startLine: 2, endLine: 3, kind: "added" },
      { startLine: 11, endLine: 12, kind: "modified" },
    ]);
  });

  it("moves to the next and previous hunk and wraps", () => {
    expect(nextHunkLine(hunks, 1, 1)).toBe(2);
    expect(nextHunkLine(hunks, 11, 1)).toBe(2);
    expect(nextHunkLine(hunks, 11, -1)).toBe(8);
    expect(nextHunkLine(hunks, 2, -1)).toBe(11);
    expect(nextHunkLine([], 1, 1)).toBeUndefined();
  });
});

describe("gaps", () => {
  it("gives the shorter side blank space after the hunk", () => {
    expect(
      gaps([
        { base: { start: 1, end: 3 }, head: { start: 1, end: 6 } },
        { base: { start: 10, end: 14 }, head: { start: 13, end: 14 } },
      ]),
    ).toEqual([
      { side: "base", afterLine: 3, lines: 3 },
      { side: "head", afterLine: 14, lines: 3 },
    ]);
  });

  it("puts a pure insertion after the base line before it", () => {
    expect(gaps([{ base: { start: 4, end: 4 }, head: { start: 4, end: 6 } }])).toEqual([
      { side: "base", afterLine: 4, lines: 2 },
    ]);
  });

  it("has no gap for an equal replacement or no change", () => {
    expect(gaps([{ base: { start: 1, end: 2 }, head: { start: 1, end: 2 } }])).toEqual([]);
    expect(gaps([])).toEqual([]);
  });

  it("keeps the two sides at the same height after each hunk", () => {
    const hunks: Hunk[] = [
      { base: { start: 2, end: 3 }, head: { start: 2, end: 5 } },
      { base: { start: 8, end: 12 }, head: { start: 10, end: 11 } },
    ];
    let offsetBase = 0;
    let offsetHead = 0;
    for (const g of gaps(hunks)) {
      if (g.side === "base") offsetBase += g.lines;
      else offsetHead += g.lines;
    }
    const last = hunks[hunks.length - 1] as Hunk;
    // After the last hunk, the same line is at the same row on both sides.
    expect(last.base.end + offsetBase).toBe(last.head.end + offsetHead);
  });
});
