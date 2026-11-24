import { test } from "node:test";
import assert from "node:assert/strict";
import { checkPrBody } from "./check-pr-body.mjs";

const VALID = "Closes #19\n\n## Evidence\n\n- Observability: none, docs only\n";

test("a valid body passes", () => {
  assert.deepEqual(checkPrBody(VALID), []);
});

test("an empty body fails with each problem", () => {
  assert.equal(checkPrBody("").length, 3);
});

test("a body with no issue link fails", () => {
  assert.match(checkPrBody(VALID.replace("Closes #19", "See the issue"))[0], /link an issue/);
});

test("an issue URL counts as a link", () => {
  const body = VALID.replace("#19", "https://github.com/daniel-kindl/diffy/issues/19");
  assert.deepEqual(checkPrBody(body), []);
});

test("a body with no Evidence section fails", () => {
  assert.match(checkPrBody(VALID.replace("## Evidence", "## Checks"))[0], /Evidence/);
});

test("two Observability lines fail", () => {
  assert.match(checkPrBody(`${VALID}Observability: none\n`)[0], /it has 2/);
});

test("a placeholder or an empty Observability value fails", () => {
  assert.match(checkPrBody(VALID.replace("none, docs only", "<reason>"))[0], /value/);
  assert.match(checkPrBody(VALID.replace(" none, docs only", ""))[0], /value/);
});

test("example lines in comments and code blocks do not count", () => {
  const body = "<!-- Closes #1 -->\n```\n## Evidence\nObservability: x\n```\n";
  assert.equal(checkPrBody(body).length, 3);
});

test("a bot author is skipped", () => {
  assert.deepEqual(checkPrBody("", "renovate[bot]"), []);
});
