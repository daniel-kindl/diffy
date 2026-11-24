#!/usr/bin/env node
// Check the body of a pull request: an issue link with a number, an
// "## Evidence" section and one "Observability:" line with a value in that
// section. The rules are in the dkstack write-pr skill:
// https://raw.githubusercontent.com/daniel-kindl/dkstack/v1/skills/write-pr/SKILL.md#evidence
// This is a project script, not a dkstack copy. dkstack v1.6.0 has no body
// check. Replace this script with the dkstack copy when a dkstack release has one.
// A pull request from a bot account is skipped. See the exception in AGENTS.md.
// The script prints only its diagnostics. It never prints the body or the author.
// Usage in CI: node scripts/local/check-pr-body.mjs
//   (reads the pull_request event from GITHUB_EVENT_PATH)
// Usage locally: node scripts/local/check-pr-body.mjs <body file>

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const KEYWORD = String.raw`\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?):?\s+`;
const ISSUE_URL = String.raw`https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/issues\/\d+`;
const ISSUE_REF = String.raw`(?:[\w.-]+\/[\w.-]+)?#\d+`;
// "Closes #12", "Closes owner/repo#12", "Closes <issue URL>", or the same
// reference as the text or the target of a Markdown link.
const ISSUE_LINK = new RegExp(
  `${KEYWORD}(?:${ISSUE_REF}|${ISSUE_URL}|\\[(?:${ISSUE_REF}|${ISSUE_URL})\\]\\([^)\\s]+\\)|\\[[^\\]]+\\]\\(${ISSUE_URL}\\))`,
  "i",
);
const EVIDENCE = /^## Evidence[ \t]*$/m;
const NEXT_SECTION = /^#{1,2}[ \t]/m;
const OBSERVABILITY = /^[ \t]*(?:[-*][ \t]+)?Observability:[ \t]*(.*)$/gm;
const PLACEHOLDER = /<[^>]*>/;

const observabilityValues = (text) => [...text.matchAll(OBSERVABILITY)].map((m) => m[1].trim());

// Return the text of the "## Evidence" section, or null when it is missing.
function evidenceSection(body) {
  const start = EVIDENCE.exec(body);
  if (!start) return null;
  const rest = body.slice(start.index + start[0].length);
  const end = NEXT_SECTION.exec(rest);
  return end ? rest.slice(0, end.index) : rest;
}

// Return the list of problems. An empty list means a valid body.
export function checkPrBody(rawBody, author = "") {
  if (author.endsWith("[bot]")) return [];
  // An HTML comment or a fenced code block can hold example lines. They are not part of the body.
  const body = (rawBody ?? "")
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/^[ \t]*(`{3,}|~{3,})[^\n]*\n[\s\S]*?^[ \t]*\1[ \t]*$/gm, "");
  const errors = [];
  if (!ISSUE_LINK.test(body)) {
    errors.push('the body must link an issue with a number, for example "Closes #12"');
  }
  const evidence = evidenceSection(body);
  if (evidence === null) {
    errors.push('the body must have an "## Evidence" section');
    return errors;
  }
  const all = observabilityValues(body);
  const inEvidence = observabilityValues(evidence);
  if (all.length !== 1 || inEvidence.length !== 1) {
    errors.push(
      `the body must have one "Observability:" line, in the Evidence section. It has ${all.length}, and ${inEvidence.length} in the Evidence section`,
    );
  } else if (inEvidence[0] === "" || PLACEHOLDER.test(inEvidence[0])) {
    errors.push('the "Observability:" line must have a value, for example "none, docs only"');
  }
  return errors;
}

// Return { body, author } from a body file, or from the pull_request event.
export function readInput(args, env) {
  if (args.length > 0) return { body: readFileSync(args[0], "utf8"), author: "" };
  if (!env.GITHUB_EVENT_PATH) throw new Error("give a body file, or set GITHUB_EVENT_PATH");
  const event = JSON.parse(readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  const pr = event.pull_request;
  if (!pr) throw new Error("the event has no pull_request");
  return { body: pr.body ?? "", author: pr.user?.login ?? "" };
}

function main() {
  let input;
  try {
    input = readInput(process.argv.slice(2), process.env);
  } catch (error) {
    console.error(`PR body: ${error.message}`);
    return 2;
  }
  if (input.author.endsWith("[bot]")) {
    console.log("A bot opened this pull request. The body check is skipped (see AGENTS.md).");
    return 0;
  }
  const errors = checkPrBody(input.body, input.author);
  for (const error of errors) console.error(`PR body: ${error}`);
  if (errors.length > 0) return 1;
  console.log("The PR body links an issue and has the Evidence section with one Observability line.");
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
