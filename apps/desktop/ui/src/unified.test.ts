import { describe, expect, it } from "vitest";
import type { Hunk } from "./hunks";
import { changeStarts, lineLabel, nextStart, splitLines, unifiedLines } from "./unified";

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
