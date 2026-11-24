import { test } from "node:test";
import assert from "node:assert/strict";
import { titlesToCheck } from "./check-titles.mjs";

const SHA = "a".repeat(40);
const BAD = "fix: a title that is much longer than the fifty character limit";

test("a valid PR title passes", () => {
  assert.deepEqual(titlesToCheck({ pull_request: { title: "chore: bootstrap" } }), [{ label: "PR title", errors: [] }]);
});

test("a problem does not repeat the title", () => {
  const [{ errors }] = titlesToCheck({ pull_request: { title: BAD } });
  assert.equal(errors.length, 1);
  assert.ok(!errors[0].includes(BAD));
  assert.ok(!errors[0].includes("longer than"));
});

test("a push checks each commit title and names the commit by its short SHA", () => {
  const calls = [];
  const log = (...args) => {
    calls.push(args);
    return `1234567\0docs: one\n89abcde\0${BAD}\n`;
  };
  const results = titlesToCheck({ before: "0".repeat(40), after: SHA }, log);
  assert.deepEqual(calls[0].at(-1), `${SHA}~1..${SHA}`);
  assert.deepEqual(results.map((r) => [r.label, r.errors.length]), [
    ["commit 1234567 title", 0],
    ["commit 89abcde title", 1],
  ]);
});

test("an event with no pull request and no push head is an error", () => {
  assert.throws(() => titlesToCheck({}), /no push head/);
});
