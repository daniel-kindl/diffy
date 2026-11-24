#!/usr/bin/env node
// Check the body of a pull request: an issue link with a number, an
// "## Evidence" section and one "Observability:" line with a value.
// The rules are in the dkstack write-pr skill:
// https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/skills/write-pr/SKILL.md#evidence
// This is a project script, not a dkstack copy. dkstack v1.6.0 has no body
// check. Replace this script with the dkstack copy when a dkstack release has one.
// A pull request from a bot account is skipped.
// Usage: BODY="<body>" AUTHOR="<login>" node scripts/local/check-pr-body.mjs

import { pathToFileURL } from "node:url";

const ISSUE_LINK =
  /\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?):?\s+(?:(?:[\w.-]+\/[\w.-]+)?#\d+|https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/issues\/\d+)/i;
const EVIDENCE = /^## Evidence[ \t]*$/m;
const OBSERVABILITY = /^[ \t]*(?:[-*][ \t]+)?Observability:[ \t]*(.*)$/gm;
const PLACEHOLDER = /<[^>]*>/;

// Return the list of problems. An empty list means a valid body.
export function checkPrBody(rawBody, author = "") {
  if (author.endsWith("[bot]")) return [];
  // An HTML comment or a fenced code block can hold example lines. They are not part of the body.
  const body = rawBody
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/^[ \t]*(`{3,}|~{3,})[^\n]*\n[\s\S]*?^[ \t]*\1[ \t]*$/gm, "");
  const errors = [];
  if (!ISSUE_LINK.test(body)) {
    errors.push('the body must link an issue with a number, for example "Closes #12"');
  }
  if (!EVIDENCE.test(body)) errors.push('the body must have an "## Evidence" section');
  const values = [...body.matchAll(OBSERVABILITY)].map((m) => m[1].trim());
  if (values.length !== 1) {
    errors.push(`the body must have one "Observability:" line, and it has ${values.length}`);
  } else if (values[0] === "" || PLACEHOLDER.test(values[0])) {
    errors.push('the "Observability:" line must have a value, for example "none, docs only"');
  }
  return errors;
}

function main() {
  const errors = checkPrBody(process.env.BODY ?? "", process.env.AUTHOR ?? "");
  for (const error of errors) console.error(`PR body: ${error}`);
  if (errors.length > 0) return 1;
  console.log("The PR body links an issue and has the Evidence section and one Observability line.");
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
