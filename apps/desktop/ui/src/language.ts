// The language of a file for syntax highlighting (C-07). It depends on the
// file name only. It never reads content and never changes the diff: the
// canonical hunks come from Rust, and the language only colors the text.

const BY_EXTENSION: Record<string, string> = {
  rs: "rust",
  ts: "typescript",
  tsx: "typescript",
  js: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  jsx: "javascript",
  json: "json",
  md: "markdown",
  html: "html",
  css: "css",
  py: "python",
  go: "go",
  java: "java",
  c: "c",
  h: "c",
  cpp: "cpp",
  cc: "cpp",
  hpp: "cpp",
  cs: "csharp",
  sh: "shell",
  yml: "yaml",
  yaml: "yaml",
  toml: "ini",
  sql: "sql",
  xml: "xml",
};

const BY_FILE_NAME: Record<string, string> = {
  Dockerfile: "dockerfile",
  Makefile: "makefile",
};

export const PLAIN_TEXT = "plaintext";

/** The Monaco language id for a path. An unknown language is plain text. */
export function languageOf(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? "";
  const byName = BY_FILE_NAME[name];
  if (byName) return byName;
  const dot = name.lastIndexOf(".");
  if (dot <= 0) return PLAIN_TEXT;
  return BY_EXTENSION[name.slice(dot + 1).toLowerCase()] ?? PLAIN_TEXT;
}
