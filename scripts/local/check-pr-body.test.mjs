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
