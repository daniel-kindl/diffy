// Run the project scripts as CI runs them, and check that stderr and stdout
// contain no text from the title, the body or the event file.
import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync, execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const SCRIPTS = resolve(import.meta.dirname);
const MARKER = "zzsecretmarker";

function run(script, event, cwd = SCRIPTS) {
  const dir = mkdtempSync(join(tmpdir(), "cli-"));
  const path = join(dir, "event.json");
  writeFileSync(path, typeof event === "string" ? event : JSON.stringify(event));
  const env = { ...process.env, GITHUB_EVENT_PATH: path };
  return spawnSync(process.execPath, [join(SCRIPTS, script)], { cwd, env, encoding: "utf8" });
}

function assertNoMarker(result) {
  assert.ok(!result.stdout.includes(MARKER), "stdout has text from the input");
  assert.ok(!result.stderr.includes(MARKER), "stderr has text from the input");
}

test("an invalid PR type is not in the output", () => {
  const result = run("check-titles.mjs", { pull_request: { title: `${MARKER}: add a thing` } });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /the type is not allowed/);
  assertNoMarker(result);
});

test("each invalid PR title diagnostic has no title text", () => {
  for (const title of [`fix(${MARKER} x): a`, `fix:${MARKER}`, `fix: ${MARKER} `, `fix: ${MARKER}—x`, `${MARKER}`, `fix: ${MARKER.repeat(4)}`]) {
    const result = run("check-titles.mjs", { pull_request: { title } });
    assert.equal(result.status, 1, title);
    assertNoMarker(result);
  }
});

test("an invalid commit title in a push is not in the output", () => {
  const repo = mkdtempSync(join(tmpdir(), "repo-"));
  const git = (...args) => execFileSync("git", ["-c", "user.name=t", "-c", "user.email=t@t", ...args], { cwd: repo, encoding: "utf8" });
  git("init", "-q");
  git("commit", "-q", "--allow-empty", "-m", "chore: start");
  const before = git("rev-parse", "HEAD").trim();
  git("commit", "-q", "--allow-empty", "-m", "docs: one");
  git("commit", "-q", "--allow-empty", "-m", `${MARKER}: two`);
  const after = git("rev-parse", "HEAD").trim();
  const result = run("check-titles.mjs", { before, after }, repo);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /^commit [0-9a-f]+ title: the type is not allowed/m);
  assert.equal(result.stderr.trim().split("\n").length, 1);
  assertNoMarker(result);
});

test("a malformed event file is not in the output", () => {
  for (const script of ["check-titles.mjs", "check-pr-body.mjs"]) {
    const result = run(script, `{"pull_request": {"title": "${MARKER}`);
    assert.equal(result.status, 2, script);
    assert.match(result.stderr, /not valid JSON/);
    assertNoMarker(result);
  }
});

test("an invalid PR body is not in the output", () => {
  const result = run("check-pr-body.mjs", { pull_request: { body: `${MARKER} Closes`, user: { login: MARKER } } });
  assert.equal(result.status, 1);
  assertNoMarker(result);
});
