// Reads tokens.css and resolves a token for each theme. Used by the tests only.
import { readFileSync } from "node:fs";

export const css = readFileSync(new URL("../tokens.css", import.meta.url), "utf8");
export const style = readFileSync(new URL("../style.css", import.meta.url), "utf8");

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
export const light = new Map<string, string>();
for (const b of blocks(/(?:^|\n):root\s*\{([^}]*)\}/g)) {
  for (const [k, v] of declarations(b)) light.set(k, v);
}
export const darkMedia = declarations(
  blocks(
    /@media \(prefers-color-scheme: dark\)\s*\{\s*:root:not\(\[data-theme="light"\]\)\s*\{([^}]*)\}/g,
  )[0] ?? "",
);
export const darkAttr = declarations(blocks(/:root\[data-theme="dark"\]\s*\{([^}]*)\}/g)[0] ?? "");

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

export const themes = { light: resolver(new Map()), dark: resolver(darkMedia) };
