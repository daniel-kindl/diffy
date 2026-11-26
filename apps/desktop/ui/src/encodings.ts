// The legacy encodings that a user can select for a side (Q26). There is no
// heuristic detection: the default is UTF-8, or UTF-16 and UTF-8 by the BOM.
// The labels are WHATWG labels, which Rust checks again.
export const LEGACY_ENCODINGS: readonly { label: string; name: string }[] = [
  { label: "", name: "Default (BOM or UTF-8)" },
  { label: "windows-1250", name: "Windows-1250 (Central European)" },
  { label: "windows-1252", name: "Windows-1252 (Western European)" },
  { label: "iso-8859-2", name: "ISO-8859-2 (Central European)" },
  { label: "iso-8859-15", name: "ISO-8859-15 (Western European)" },
  { label: "windows-1251", name: "Windows-1251 (Cyrillic)" },
  { label: "koi8-r", name: "KOI8-R (Cyrillic)" },
  { label: "shift_jis", name: "Shift JIS" },
  { label: "gbk", name: "GBK" },
  { label: "big5", name: "Big5" },
  { label: "euc-kr", name: "EUC-KR" },
];
