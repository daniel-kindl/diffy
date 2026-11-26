import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { contrast } from "./contrast";

const css = readFileSync(new URL("./tokens.css", import.meta.url), "utf8");
const style = readFileSync(new URL("./style.css", import.meta.url), "utf8");

function declarations(block: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const m of block.matchAll(/--([\w-]+):\s*([^;]+);/g))
    out.set(m[1] ?? "", (m[2] ?? "").trim());
  return out;
}

function blocks(re: RegExp): string[] {
  return [...css.matchAll(re)].map((m) => m[1] ?? "");
}

// Top-level :root blocks (not indented), the dark blocks and nothing else.
const light = new Map<string, string>();
for (const b of blocks(/(?:^|\n):root\s*\{([^}]*)\}/g)) {
  for (const [k, v] of declarations(b)) light.set(k, v);
}
const darkMedia = declarations(
  blocks(
    /@media \(prefers-color-scheme: dark\)\s*\{\s*:root:not\(\[data-theme="light"\]\)\s*\{([^}]*)\}/g,
  )[0] ?? "",
);
const darkAttr = declarations(blocks(/:root\[data-theme="dark"\]\s*\{([^}]*)\}/g)[0] ?? "");

function resolver(overrides: Map<string, string>) {
  const all = new Map([...light, ...overrides]);
  const resolve = (name: string, depth = 0): string => {
    if (depth > 10) throw new Error(`token cycle at ${name}`);
    const value = all.get(name);
    if (value === undefined) throw new Error(`unknown token ${name}`);
    const ref = /^var\(--([\w-]+)\)$/.exec(value);
    return ref?.[1] ? resolve(ref[1], depth + 1) : value;
  };
  return resolve;
}

const themes = { light: resolver(new Map()), dark: resolver(darkMedia) };

// [foreground, background, minimum ratio]
const TEXT: [string, string, number][] = [
  ["s-text", "s-surface", 4.5],
  ["s-text", "s-surface-raised", 4.5],
  ["s-text-muted", "s-surface", 4.5],
  ["s-text-muted", "s-surface-raised", 4.5],
  ["s-added-fg", "s-added-bg", 4.5],
  ["s-removed-fg", "s-removed-bg", 4.5],
  ["s-modified-fg", "s-modified-bg", 4.5],
  ["c-bar-fg", "c-bar-bg", 4.5],
  ["c-explorer-fg", "c-explorer-bg", 4.5],
  ["c-explorer-muted", "c-explorer-bg", 4.5],
  ["c-mark-added", "c-line-added-bg", 4.5],
  ["c-mark-removed", "c-line-removed-bg", 4.5],
  ["c-mark-modified", "c-line-modified-bg", 4.5],
];
// Essential boundaries and the focus ring need 3:1 (WCAG 2.2 non-text contrast).
const NON_TEXT: [string, string, number][] = [
  ["s-border", "s-surface", 3],
  ["s-border", "s-surface-raised", 3],
  ["s-focus", "s-surface", 3],
  ["s-focus", "s-surface-raised", 3],
  ["c-bar-border", "c-bar-bg", 3],
  ["c-focus-ring", "c-explorer-bg", 3],
];

describe.each(Object.entries(themes))("%s theme", (_name, resolve) => {
  it.each(TEXT)("text %s on %s is at least %f:1", (fg, bg, min) => {
    expect(contrast(resolve(fg), resolve(bg))).toBeGreaterThanOrEqual(min);
  });
  it.each(NON_TEXT)("boundary %s on %s is at least %f:1", (fg, bg, min) => {
    expect(contrast(resolve(fg), resolve(bg))).toBeGreaterThanOrEqual(min);
  });
});

describe("contrast", () => {
  it("measures the known extremes and rejects a low pair", () => {
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("#777777", "#888888")).toBeLessThan(3);
  });
});

describe("token structure", () => {
  it("the dark theme is the same for the system rule and the manual override", () => {
    expect([...darkAttr].sort()).toEqual([...darkMedia].sort());
    expect(darkMedia.size).toBeGreaterThan(5);
  });

  it("components use only semantic tokens, and semantic tokens only primitives or values", () => {
    for (const [name, value] of light) {
      const ref = /^var\(--([\w-]+)\)$/.exec(value)?.[1];
      if (name.startsWith("c-") && ref) expect(ref.startsWith("s-")).toBe(true);
      if (name.startsWith("s-") && ref) expect(ref.startsWith("p-")).toBe(true);
    }
  });

  it("forced colors drop the line backgrounds, so the gutter marks carry the meaning", () => {
    const forced = css.slice(css.indexOf("@media (forced-colors: active)"));
    expect(forced).toContain("--c-line-added-bg: transparent");
    for (const mark of ['content: "+"', 'content: "-"', 'content: "~"']) {
      expect(style).toContain(mark);
    }
  });
});
