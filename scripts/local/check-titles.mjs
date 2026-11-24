#!/usr/bin/env node
// Check the titles of a CI run with the dkstack check-title rules, and print
// no text from a title. scripts/check-title.mjs is a dkstack copy. It prints
// the title, and some of its diagnostics contain parts of the title, for
// example the type. This project script imports its rules and prints only
// fixed diagnostics.
// - pull_request event: the PR title, from GITHUB_EVENT_PATH.
// - push event: each commit title of the push. A problem names the commit
//   by its short SHA.
// Usage in CI: node scripts/local/check-titles.mjs

import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { checkTitle, TYPES } from "../check-title.mjs";
import { FixedError, readEvent, safeMessage } from "./event.mjs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });

// The dkstack diagnostics that contain no title text. Each other diagnostic
// becomes a fixed message.
const FIXED_DIAGNOSTICS = [
  /^the title is empty$/,
  /^the title must be one line$/,
  /^the title must not have an (?:em|en) dash \(U\+201[34]\)\. Use a comma or a new sentence$/,
  /^the title must start with a type, as in "fix: \.\.\."$/,
  /^the scope must use only a-z, 0-9 and single hyphens$/,
  /^put one space after the colon, then a description$/,
  /^the title must not end with a space$/,
  /^the title has \d+ characters\. Use \d+ or fewer, without the \(#NN\) suffix$/,
];
const TYPE_DIAGNOSTIC = /^the type ".*" is not allowed/s;

// Return a diagnostic with no text from the title.
export function fixedDiagnostic(error) {
  if (FIXED_DIAGNOSTICS.some((pattern) => pattern.test(error))) return error;
  if (TYPE_DIAGNOSTIC.test(error)) return `the type is not allowed. Use one of: ${TYPES.join(", ")}`;
  return "the title is not a conventional commit";
}

const check = (title) => checkTitle(title).map(fixedDiagnostic);

// Return the base of a push: the tip before the push, or the parent of the
// head when that commit is not in the clone (a new branch or a force-push).
function pushBase(event, log) {
  const before = event.before ?? "";
  if (/^[0-9a-f]{40}$/.test(before) && !/^0+$/.test(before)) {
    try {
      log("cat-file", "-e", `${before}^{commit}`);
      return before;
    } catch {
      // Use the parent of the head.
    }
  }
  return `${event.after}~1`;
}

// Return [{ label, errors }] for each title of the event.
export function titlesToCheck(event, log = git) {
  if (event.pull_request) {
    return [{ label: "PR title", errors: check(event.pull_request.title ?? "") }];
  }
  if (typeof event.after !== "string" || !/^[0-9a-f]{40}$/.test(event.after)) {
    throw new FixedError("the event has no pull_request and no push head");
  }
  let output;
  try {
    output = log("log", "--format=%h%x00%s", `${pushBase(event, log)}..${event.after}`);
  } catch {
    throw new FixedError("git log failed for the commits of the push");
  }
  return output
    .split("\n")
    .filter((line) => line !== "")
    .map((line) => {
      const [sha, title] = line.split("\0");
      return { label: `commit ${sha} title`, errors: check(title ?? "") };
    });
}

function main() {
  let results;
  try {
    results = titlesToCheck(readEvent(process.env));
  } catch (error) {
    console.error(`titles: ${safeMessage(error, "cannot check the titles")}`);
    return 2;
  }
  let failed = 0;
  for (const { label, errors } of results) {
    for (const error of errors) console.error(`${label}: ${error}`);
    if (errors.length > 0) failed++;
  }
  if (failed > 0) return 1;
  console.log(`${results.length} title(s) are conventional commits.`);
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
