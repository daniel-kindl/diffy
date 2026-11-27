import { describe, expect, it } from "vitest";
import { contrast, ensureContrast, mix } from "./contrast";
import { asHex, buildTheme, MIN_CONTRAST, type LineBackgrounds } from "./monacoTheme";
import { themes } from "./testing/tokensCss";

function backgrounds(name: "light" | "dark"): LineBackgrounds {
  const r = themes[name];
  return {
    plain: r("s-surface"),
    added: r("c-line-added-bg"),
    removed: r("c-line-removed-bg"),
    modified: r("c-line-modified-bg"),
    collapsed: r("c-line-collapsed-bg"),
  };
}

describe("ensureContrast", () => {
  it("keeps a color that already passes", () => {
    expect(ensureContrast("#000000", ["#ffffff"], 4.5, "black")).toBe("#000000");
  });

  it("moves a failing color on the way to black until it passes, and no further than needed", () => {
    const fixed = ensureContrast("#09885a", ["#d1fae5", "#fee2e2"], 4.5, "black");
    expect(contrast(fixed, "#d1fae5")).toBeGreaterThanOrEqual(4.5);
    expect(contrast(fixed, "#fee2e2")).toBeGreaterThanOrEqual(4.5);
    // Replay the steps: the color before the returned one fails, so it is the nearest.
    const backgrounds = ["#d1fae5", "#fee2e2"];
    let previous = "#09885a";
    let current = previous;
    while (current !== fixed) {
      previous = current;
      current = mix(current, "#000000", 0.05);
    }
    expect(previous).not.toBe(fixed);
    expect(backgrounds.every((b) => contrast(previous, b) >= 4.5)).toBe(false);
  });

  it("gives white as the last resort on dark backgrounds that nothing else passes", () => {
    expect(ensureContrast("#222222", ["#111111"], 21, "white")).toBe("#ffffff");
  });

  it("mixes channels", () => {
    expect(mix("#000000", "#ffffff", 0.5)).toBe("#808080");
    expect(mix("#102030", "#102030", 0.7)).toBe("#102030");
  });
});

describe.each(["light", "dark"] as const)("the %s Monaco theme", (name) => {
  const bg = backgrounds(name);
  const all = Object.values(bg);
  const theme = buildTheme(name === "dark", bg);

  it("uses the app surface as the editor background and inherits the Monaco base", () => {
    expect(theme.colors["editor.background"]).toBe(bg.plain);
    expect(theme.base).toBe(name === "dark" ? "vs-dark" : "vs");
    expect(theme.inherit).toBe(true);
  });

  it("gives every syntax color at least 4.5:1 on each line background", () => {
    expect(theme.rules.length).toBeGreaterThan(10);
    for (const rule of theme.rules) {
      for (const b of all) {
        expect(contrast(`#${rule.foreground}`, b), `${rule.token} on ${b}`).toBeGreaterThanOrEqual(
          MIN_CONTRAST,
        );
      }
    }
  });

  it("gives the text and the line numbers at least 4.5:1 too", () => {
    for (const key of [
      "editor.foreground",
      "editorLineNumber.foreground",
      "editorLineNumber.activeForeground",
    ]) {
      for (const b of all) {
        expect(contrast(theme.colors[key] ?? "", b), `${key} on ${b}`).toBeGreaterThanOrEqual(
          MIN_CONTRAST,
        );
      }
    }
  });
});

describe("why the Monaco colors are not used as they are", () => {
  it("a Monaco vs-dark keyword fails on the modified line background", () => {
    expect(contrast("#569cd6", backgrounds("dark").modified)).toBeLessThan(4.5);
    expect(contrast("#098658", backgrounds("light").removed)).toBeLessThan(4.5);
  });
});

describe("asHex", () => {
  it("accepts #rrggbb only", () => {
    expect(asHex(" #D1FAE5 ")).toBe("#d1fae5");
    expect(asHex("transparent")).toBeUndefined();
    expect(asHex("Canvas")).toBeUndefined();
    expect(asHex("#fff")).toBeUndefined();
  });
});
