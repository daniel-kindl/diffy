// WCAG 2.x contrast ratio between two sRGB colors given as #rrggbb.

function channel(v: number): number {
  const s = v / 255;
  return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

export function luminance(hex: string): number {
  const m = /^#([0-9a-f]{6})$/i.exec(hex);
  if (!m?.[1]) throw new Error(`not a #rrggbb color: ${hex}`);
  const n = parseInt(m[1], 16);
  return (
    0.2126 * channel((n >> 16) & 255) + 0.7152 * channel((n >> 8) & 255) + 0.0722 * channel(n & 255)
  );
}

export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

function toHex(n: number): string {
  return Math.round(Math.min(255, Math.max(0, n)))
    .toString(16)
    .padStart(2, "0");
}

/** Move a #rrggbb color a fraction of the way towards another color. */
export function mix(from: string, to: string, t: number): string {
  const a = parseInt(from.slice(1), 16);
  const b = parseInt(to.slice(1), 16);
  const ch = (shift: number) => toHex(((a >> shift) & 255) * (1 - t) + ((b >> shift) & 255) * t);
  return `#${ch(16)}${ch(8)}${ch(0)}`;
}

/**
 * The nearest color to `fg`, on the way to black or to white, that has at least
 * `min` contrast on each of the backgrounds. A color that already passes is
 * returned as it is. Black or white is the last resort.
 */
export function ensureContrast(
  fg: string,
  backgrounds: readonly string[],
  min: number,
  towards: "black" | "white",
): string {
  const end = towards === "black" ? "#000000" : "#ffffff";
  let color = fg;
  for (let i = 0; i < 40; i++) {
    if (backgrounds.every((bg) => contrast(color, bg) >= min)) return color;
    color = mix(color, end, 0.05);
  }
  return end;
}
