import { describe, expect, it } from "vitest";
import { actionFor, type KeyInfo } from "./keys";

const k = (key: string, mods: Partial<KeyInfo> = {}): KeyInfo => ({
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("actionFor", () => {
  it("moves between changes", () => {
    expect(actionFor(k("F3"))).toBe("nextChange");
    expect(actionFor(k("F3", { shiftKey: true }))).toBe("previousChange");
    expect(actionFor(k("ArrowDown", { altKey: true }))).toBe("nextChange");
    expect(actionFor(k("ArrowUp", { altKey: true }))).toBe("previousChange");
  });

  it("opens find and go to line with Ctrl or Cmd", () => {
    expect(actionFor(k("f", { ctrlKey: true }))).toBe("find");
    expect(actionFor(k("G", { metaKey: true }))).toBe("goToLine");
  });

  it("switches the side with F6", () => {
    expect(actionFor(k("F6"))).toBe("switchSide");
  });

  it("never binds Tab, so there is no keyboard trap", () => {
    for (const mods of [{}, { shiftKey: true }, { ctrlKey: true }, { altKey: true }]) {
      expect(actionFor(k("Tab", mods))).toBeUndefined();
    }
  });

  it("ignores plain letters and unknown combinations", () => {
    expect(actionFor(k("f"))).toBeUndefined();
    expect(actionFor(k("ArrowDown"))).toBeUndefined();
    expect(actionFor(k("f", { ctrlKey: true, shiftKey: true }))).toBeUndefined();
  });
});
