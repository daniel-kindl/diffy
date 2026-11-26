import { describe, expect, it } from "vitest";
import { decorations, kindOf, nextHunkLine, type Hunk } from "./hunks";

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
