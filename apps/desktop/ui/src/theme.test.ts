import { describe, expect, it } from "vitest";
import { nextPreference, parsePreference, resolveTheme } from "./theme";

describe("theme", () => {
  it("follows the system by default and lets the user override it", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });

  it("treats an unknown stored value as the system theme", () => {
    expect(parsePreference(null)).toBe("system");
    expect(parsePreference("blue")).toBe("system");
    expect(parsePreference("dark")).toBe("dark");
  });

  it("cycles system, light, dark", () => {
    expect(nextPreference("system")).toBe("light");
    expect(nextPreference("light")).toBe("dark");
    expect(nextPreference("dark")).toBe("system");
  });
});
