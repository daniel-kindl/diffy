// Monaco themes whose syntax colors have enough contrast on every background
// that a line can have in a diff: the plain editor, and the added, removed,
// modified and collapsed lines. Monaco's own `vs` and `vs-dark` colors do not
// (a keyword on a modified line is 3.1:1 in `vs-dark`), so each color is moved
// towards black (light) or white (dark) until it passes WCAG AA, 4.5:1.
import { ensureContrast } from "./contrast";

export interface LineBackgrounds {
  plain: string;
  added: string;
  removed: string;
  modified: string;
  collapsed: string;
}

/** Monaco's base colors for the token names that its basic grammars use. */
const BASE_LIGHT: Record<string, string> = {
  comment: "#008000",
  string: "#a31515",
  "string.escape": "#ee0000",
  keyword: "#0000ff",
  number: "#098658",
  regexp: "#811f3f",
  type: "#267f99",
  tag: "#800000",
  "attribute.name": "#e50000",
  "attribute.value": "#0451a5",
  variable: "#001080",
  predefined: "#795e26",
  delimiter: "#000000",
  operator: "#000000",
};

const BASE_DARK: Record<string, string> = {
  comment: "#6a9955",
  string: "#ce9178",
  "string.escape": "#d7ba7d",
  keyword: "#569cd6",
  number: "#b5cea8",
  regexp: "#d16969",
  type: "#4ec9b0",
  tag: "#569cd6",
  "attribute.name": "#9cdcfe",
  "attribute.value": "#ce9178",
  variable: "#9cdcfe",
  predefined: "#dcdcaa",
  delimiter: "#d4d4d4",
  operator: "#d4d4d4",
};

export const MIN_CONTRAST = 4.5;

export interface ThemeData {
  base: "vs" | "vs-dark";
  inherit: true;
  rules: { token: string; foreground: string }[];
  colors: Record<string, string>;
}

const strip = (hex: string) => hex.replace("#", "");

export function buildTheme(dark: boolean, bg: LineBackgrounds): ThemeData {
  const all = [bg.plain, bg.added, bg.removed, bg.modified, bg.collapsed];
  const towards = dark ? "white" : "black";
  const fix = (c: string) => ensureContrast(c, all, MIN_CONTRAST, towards);
  const base = dark ? BASE_DARK : BASE_LIGHT;
  return {
    base: dark ? "vs-dark" : "vs",
    inherit: true,
    rules: Object.entries(base).map(([token, color]) => ({
      token,
      foreground: strip(fix(color)),
    })),
    colors: {
      "editor.background": bg.plain,
      "editor.foreground": fix(dark ? "#d4d4d4" : "#000000"),
      // The line numbers are text too. The gutter marks are drawn by our CSS.
      "editorLineNumber.foreground": fix(dark ? "#858585" : "#237893"),
      "editorLineNumber.activeForeground": fix(dark ? "#c6c6c6" : "#0b216f"),
    },
  };
}

/** A #rrggbb value, or undefined for anything else (transparent, a system color). */
export function asHex(value: string): string | undefined {
  const v = value.trim().toLowerCase();
  return /^#[0-9a-f]{6}$/.test(v) ? v : undefined;
}
