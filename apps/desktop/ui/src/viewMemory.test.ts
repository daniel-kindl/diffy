import { describe, expect, it } from "vitest";
import { scrollToRestore, viewKey } from "./viewMemory";

describe("viewMemory", () => {
  it("builds a key from the sources and the mode", () => {
    expect(viewKey(["a", "b"], false)).toBe("split|a|b");
    expect(viewKey(["a", undefined], true)).toBe("unified|a|");
    expect(viewKey(["a", "b"], true)).not.toBe(viewKey(["a", "b"], false));
  });

  it("restores the scroll for the same place", () => {
    const key = viewKey(["a", "b"], false);
    expect(scrollToRestore({ key, scrollTop: 420 }, key)).toBe(420);
  });

  it("starts at the top for another place, or with nothing remembered", () => {
    const key = viewKey(["a", "b"], false);
    expect(scrollToRestore({ key: viewKey(["a", "c"], false), scrollTop: 420 }, key)).toBe(0);
    expect(scrollToRestore({ key: viewKey(["a", "b"], true), scrollTop: 420 }, key)).toBe(0);
    expect(scrollToRestore(undefined, key)).toBe(0);
    expect(scrollToRestore({ key, scrollTop: 0 }, key)).toBe(0);
  });
});
