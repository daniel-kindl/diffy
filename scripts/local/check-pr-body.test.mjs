import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { checkPrBody, readInput } from "./check-pr-body.mjs";

const VALID = "Closes #19\n\n## Evidence\n\n- Observability: none, docs only\n";
const URL = "https://github.com/daniel-kindl/diffy/issues/19";

test("a valid body passes", () => {
  assert.deepEqual(checkPrBody(VALID), []);
});

test("an empty or missing body fails with each problem", () => {
  assert.equal(checkPrBody("").length, 2);
  assert.equal(checkPrBody(null).length, 2);
});

test("a body with no issue link fails", () => {
  assert.match(checkPrBody(VALID.replace("Closes #19", "See the issue"))[0], /link an issue/);
});

test("a keyword with no issue number fails", () => {
  assert.match(checkPrBody(VALID.replace("Closes #19", "Closes the issue"))[0], /link an issue/);
});

test("an issue URL and a cross-repository reference count as links", () => {
  assert.deepEqual(checkPrBody(VALID.replace("#19", URL)), []);
  assert.deepEqual(checkPrBody(VALID.replace("#19", "daniel-kindl/diffy#19")), []);
});

test("a Markdown link to an issue counts as a link", () => {
  assert.deepEqual(checkPrBody(VALID.replace("#19", `[#19](${URL})`)), []);
  assert.deepEqual(checkPrBody(VALID.replace("#19", `[the bootstrap issue](${URL})`)), []);
});

test("a Markdown link to a page that is not an issue fails", () => {
  const body = VALID.replace("#19", "[docs](https://github.com/daniel-kindl/diffy/blob/main/README.md)");
  assert.match(checkPrBody(body)[0], /link an issue/);
});

test("a Markdown link with no closing keyword fails, because the merge must close the issue", () => {
  const body = VALID.replace("Closes #19", `See [#19](${URL})`);
  assert.match(checkPrBody(body)[0], /closing keyword/);
});

test("a Markdown link whose destination is not an issue fails, even with an issue number as text", () => {
  assert.match(checkPrBody(VALID.replace("#19", "[#19](README.md)"))[0], /link an issue/);
  assert.match(checkPrBody(VALID.replace("#19", `[#19](${URL.replace("issues", "pull")})`))[0], /link an issue/);
});

test("a setext heading after Evidence ends the section", () => {
  const body = "Closes #19\n\n## Evidence\n\n- check: pass\n\nNotes\n-----\n\nObservability: none\n";
  assert.match(checkPrBody(body)[0], /0 in the Evidence section/);
  const top = "Closes #19\n\n## Evidence\n\n- check: pass\n\nNotes\n=====\n\nObservability: none\n";
  assert.match(checkPrBody(top)[0], /0 in the Evidence section/);
});

test("a setext Evidence heading starts the section", () => {
  const body = "Closes #19\n\nEvidence\n--------\n\nObservability: none, docs only\n";
  assert.deepEqual(checkPrBody(body), []);
});

test("a thematic break or a list item over a dash line is not a heading", () => {
  const rule = "Closes #19\n\n## Evidence\n\n- check: pass\n\n---\n\nObservability: none, docs only\n";
  assert.deepEqual(checkPrBody(rule), []);
  const list = "Closes #19\n\n## Evidence\n\n- check: pass\n---\nObservability: none, docs only\n";
  assert.deepEqual(checkPrBody(list), []);
});

test("a heading in a code block does not end the section", () => {
  const body = "Closes #19\n\n## Evidence\n\n```\n## Notes\n```\n\nObservability: none, docs only\n";
  assert.deepEqual(checkPrBody(body), []);
});

test("a closing reference in inline code does not count", () => {
  assert.match(checkPrBody(VALID.replace("Closes #19", "`Closes #19`"))[0], /link an issue/);
  assert.match(checkPrBody(VALID.replace("Closes #19", "``Closes #19``"))[0], /link an issue/);
});

test("inline code ends at a blank line, so a later reference counts", () => {
  assert.deepEqual(checkPrBody(VALID.replace("Closes #19", "A ` mark.\n\nCloses #19")), []);
});

test("a closing reference in the label of a link to another page does not count", () => {
  const body = VALID.replace("Closes #19", "[Closes #19 in Diffy](README.md)");
  assert.match(checkPrBody(body)[0], /link an issue/);
});

test("a two-line setext heading uses the whole paragraph as its text", () => {
  const body = "Closes #19\n\nSupporting\nEvidence\n--------\n\nObservability: none, docs only\n";
  assert.match(checkPrBody(body)[0], /Evidence/);
});

test("a body with no Evidence section fails", () => {
  assert.match(checkPrBody(VALID.replace("## Evidence", "## Checks"))[0], /Evidence/);
});

test("two Observability lines fail", () => {
  assert.match(checkPrBody(`${VALID}Observability: none\n`)[0], /It has 2/);
});

test("an Observability line outside the Evidence section fails", () => {
  const body = "Closes #19\n\nObservability: none, docs only\n\n## Evidence\n\n- check: pass\n";
  assert.match(checkPrBody(body)[0], /0 in the Evidence section/);
});

test("an Observability line in a later section fails", () => {
  const body = `Closes #19\n\n## Evidence\n\n- check: pass\n\n## Notes\n\nObservability: none\n`;
  assert.match(checkPrBody(body)[0], /0 in the Evidence section/);
});

test("a subsection of Evidence is part of the section", () => {
  const body = "Closes #19\n\n## Evidence\n\n### Logs\n\nObservability: none, docs only\n";
  assert.deepEqual(checkPrBody(body), []);
});

test("a placeholder or an empty Observability value fails", () => {
  assert.match(checkPrBody(VALID.replace("none, docs only", "<reason>"))[0], /value/);
  assert.match(checkPrBody(VALID.replace(" none, docs only", ""))[0], /value/);
});

test("example lines in comments and code blocks do not count", () => {
  const body = "<!-- Closes #1 -->\n```\n## Evidence\nObservability: x\n```\n";
  assert.equal(checkPrBody(body).length, 2);
});

test("a bot author is skipped", () => {
  assert.deepEqual(checkPrBody("", "renovate[bot]"), []);
});

test("the input comes from the pull_request event", () => {
  const dir = mkdtempSync(join(tmpdir(), "pr-body-"));
  const path = join(dir, "event.json");
  writeFileSync(path, JSON.stringify({ pull_request: { body: null, user: { login: "renovate[bot]" } } }));
  assert.deepEqual(readInput([], { GITHUB_EVENT_PATH: path }), { body: "", author: "renovate[bot]" });
});

test("the input comes from a body file", () => {
  const dir = mkdtempSync(join(tmpdir(), "pr-body-"));
  const path = join(dir, "body.md");
  writeFileSync(path, VALID);
  assert.deepEqual(readInput([path], {}), { body: VALID, author: "" });
});

test("no file and no event is an error", () => {
  assert.throws(() => readInput([], {}), /GITHUB_EVENT_PATH/);
  const dir = mkdtempSync(join(tmpdir(), "pr-body-"));
  const path = join(dir, "event.json");
  writeFileSync(path, JSON.stringify({ push: {} }));
  assert.throws(() => readInput([], { GITHUB_EVENT_PATH: path }), /no pull_request/);
});
