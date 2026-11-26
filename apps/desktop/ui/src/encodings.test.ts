import { describe, expect, it } from "vitest";
import { LEGACY_ENCODINGS } from "./encodings";

describe("LEGACY_ENCODINGS", () => {
  it("starts with the default and has unique labels", () => {
    expect(LEGACY_ENCODINGS[0]?.label).toBe("");
    const labels = LEGACY_ENCODINGS.map((e) => e.label);
    expect(new Set(labels).size).toBe(labels.length);
  });

  it("has no utf-16 or utf-8 entry, because the BOM and the default cover them", () => {
    expect(LEGACY_ENCODINGS.some((e) => /^utf/i.test(e.label))).toBe(false);
  });
});
