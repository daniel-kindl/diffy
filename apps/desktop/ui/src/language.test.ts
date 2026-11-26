import { describe, expect, it } from "vitest";
import { languageOf, PLAIN_TEXT } from "./language";

describe("languageOf", () => {
  it("maps known extensions, case-insensitively", () => {
    expect(languageOf("src/main.rs")).toBe("rust");
    expect(languageOf("a/b/App.TSX")).toBe("typescript");
    expect(languageOf("C:\\x\\y.json")).toBe("json");
  });

  it("maps known file names", () => {
    expect(languageOf("docker/Dockerfile")).toBe("dockerfile");
  });

  it("shows an unknown language, a dotfile and no extension as plain text", () => {
    expect(languageOf("notes.xyz")).toBe(PLAIN_TEXT);
    expect(languageOf(".gitignore")).toBe(PLAIN_TEXT);
    expect(languageOf("LICENSE")).toBe(PLAIN_TEXT);
    expect(languageOf("")).toBe(PLAIN_TEXT);
  });
});
