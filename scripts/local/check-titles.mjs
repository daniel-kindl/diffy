#!/usr/bin/env node
// Check the titles of a CI run with the dkstack check-title rules, and print
// no title. scripts/check-title.mjs is a dkstack copy, and it prints the
// title. This project script imports its rules and prints only the problems.
// - pull_request event: the PR title, from GITHUB_EVENT_PATH.
// - push event: each commit title of the push. A problem names the commit
//   by its short SHA.
// Usage in CI: node scripts/local/check-titles.mjs

import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { checkTitle } from "../check-title.mjs";
import { readEvent } from "./event.mjs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });

// Return the base of a push: the tip before the push, or the parent of the
// head when that commit is not in the clone (a new branch or a force-push).
function pushBase(event) {
  const before = event.before ?? "";
  if (/^[0-9a-f]{40}$/.test(before) && !/^0+$/.test(before)) {
    try {
      git("cat-file", "-e", `${before}^{commit}`);
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
    return [{ label: "PR title", errors: checkTitle(event.pull_request.title ?? "") }];
  }
  if (!event.after) throw new Error("the event has no pull_request and no push head");
  const output = log("log", "--format=%h%x00%s", `${pushBase(event)}..${event.after}`);
  return output
    .split("\n")
    .filter((line) => line !== "")
    .map((line) => {
      const [sha, title] = line.split("\0");
      return { label: `commit ${sha} title`, errors: checkTitle(title) };
    });
}

function main() {
  let results;
  try {
    results = titlesToCheck(readEvent(process.env));
  } catch (error) {
    console.error(`titles: ${error.message}`);
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
